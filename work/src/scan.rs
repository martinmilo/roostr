use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Read, Write};
use std::os::fd::OwnedFd;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use crate::entrypoints::Inventory;
use crate::model::{Place, Problem};
use crate::repository::{ENTRYPOINTS_FILE, Unread, read_regular_text};

pub(crate) const SCAN_FILE: &str = ".roostr/scan";

pub(crate) struct Error {
    pub(crate) code: u8,
    pub(crate) message: String,
}

/// Why the scan script gave no inventory. The messages are the reasons `roostr scan` gives.
pub(crate) enum Failure {
    Missing(io::Error),
    NotExecutable,
    /// Executable, and the system could not start it.
    CannotStart {
        error: io::Error,
        /// What the shebang line names, as written after `#!`, a carriage return included;
        /// `None` when the first line is not a shebang line.
        interpreter: Option<String>,
    },
    Exited {
        reason: String,
        /// The exit code; for a script a signal ended, 128 plus its number, as a shell gives it.
        code: i32,
        stderr: Vec<u8>,
    },
    Malformed {
        /// Each names the line of the output it is on.
        problems: Vec<Problem>,
        stderr: Vec<u8>,
    },
}

impl Failure {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::Missing(error) => {
                format!("Cannot read {SCAN_FILE}: {error}. Add an executable file at {SCAN_FILE}.")
            }
            Self::NotExecutable => {
                format!("{SCAN_FILE} must be an executable file; run chmod +x {SCAN_FILE}.")
            }
            Self::CannotStart {
                error,
                interpreter: None,
            } => format!(
                "Cannot run {SCAN_FILE}: {error}. Its first line is not a shebang line, such as \
                 #!/bin/sh, naming the program that runs it."
            ),
            Self::CannotStart {
                error,
                interpreter: Some(interpreter),
            } => {
                let program = interpreter.trim_end_matches('\r').trim();
                let mut message = format!(
                    "{SCAN_FILE} cannot be started: its shebang line, #!{program}, names an \
                     interpreter that cannot be started: {error}."
                );
                if interpreter.ends_with('\r') {
                    message.push_str(&format!(
                        " The file has Windows line endings, so the system looks for a program \
                         called {program} followed by a carriage return; save {SCAN_FILE} with \
                         Unix line endings."
                    ));
                }
                message
            }
            Self::Exited { reason, .. } => format!("{SCAN_FILE} {reason}; scan refused."),
            Self::Malformed { problems, .. } => problems
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    // A script that could not be found or started is a setup problem for `roostr scan`;
    // one that ran and failed is a refusal.
    fn code(&self) -> u8 {
        match self {
            Self::Missing(_) | Self::NotExecutable | Self::CannotStart { .. } => 2,
            Self::Exited { .. } | Self::Malformed { .. } => 1,
        }
    }
}

impl From<Failure> for Error {
    fn from(failure: Failure) -> Self {
        Self {
            code: failure.code(),
            message: failure.message(),
        }
    }
}

/// Where the script's stderr goes: on to roostr's own stderr as it arrives, or kept.
#[derive(Clone, Copy)]
pub(crate) enum Stderr {
    Relay,
    Keep,
}

/// The scan script, if it is an executable file.
pub(crate) fn script(root: &Path) -> Result<PathBuf, Failure> {
    let script = root.join(SCAN_FILE);
    let metadata = fs::metadata(&script).map_err(Failure::Missing)?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(Failure::NotExecutable);
    }
    Ok(script)
}

/// Runs the script in the project folder and judges its output, as `roostr scan` does.
/// What the script wrote, taken once it has exited and closed its stdout.
struct Output {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

// Runs the script: its answer is complete once it has exited and its stdout is closed. A
// helper it leaves running with stderr open (a preloader) holds nothing: stderr is a socket
// read until it is empty after the script exited.
fn run_script(root: &Path, script: &Path, stderr: Stderr) -> io::Result<Output> {
    let (ours, theirs) = match stderr {
        Stderr::Relay => (None, Stdio::inherit()),
        Stderr::Keep => {
            let (ours, theirs) = UnixStream::pair()?;
            ours.set_read_timeout(Some(Duration::from_millis(10)))?;
            (Some(ours), Stdio::from(OwnedFd::from(theirs)))
        }
    };
    // Execute the file itself, without a shell.
    let mut command = Command::new(script);
    command
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(theirs);
    let mut child = command.spawn()?;
    // Our copy of the script's end of stderr goes with the command.
    drop(command);
    let exited = Arc::new(AtomicBool::new(false));
    let reader = ours.map(|mut socket| {
        let exited = Arc::clone(&exited);
        thread::spawn(move || {
            let mut written = Vec::new();
            let mut chunk = [0; 8192];
            loop {
                let after_exit = exited.load(Ordering::Acquire);
                match socket.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(size) => written.extend_from_slice(&chunk[..size]),
                    Err(error)
                        if matches!(
                            error.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                        ) =>
                    {
                        // Empty, and the script had exited before this read began.
                        if after_exit {
                            break;
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                    Err(_) => break,
                }
            }
            written
        })
    });
    let mut stdout = Vec::new();
    let read = child
        .stdout
        .take()
        .map_or(Ok(0), |mut pipe| pipe.read_to_end(&mut stdout));
    let status = child.wait();
    exited.store(true, Ordering::Release);
    let stderr = reader
        .map(|reader| reader.join().unwrap_or_default())
        .unwrap_or_default();
    read?;
    Ok(Output {
        status: status?,
        stdout,
        stderr,
    })
}

pub(crate) fn execute(root: &Path, script: &Path, stderr: Stderr) -> Result<Inventory, Failure> {
    let output = run_script(root, script, stderr).map_err(|error| Failure::CannotStart {
        error,
        interpreter: interpreter(script),
    })?;
    if !output.status.success() {
        let (reason, code) = match (output.status.code(), output.status.signal()) {
            (Some(code), _) => (format!("exited with code {code}"), code),
            (None, signal) => (
                format!("terminated by a signal ({})", output.status),
                128 + signal.unwrap_or_default(),
            ),
        };
        return Err(Failure::Exited {
            reason,
            code,
            stderr: output.stderr,
        });
    }
    let inventory = match std::str::from_utf8(&output.stdout) {
        Ok(text) => Inventory::parse_scan(text),
        Err(error) => {
            let line = output.stdout[..error.valid_up_to()]
                .iter()
                .filter(|&&byte| byte == b'\n')
                .count()
                + 1;
            return Err(Failure::Malformed {
                problems: vec![Problem::new(
                    Place::Line(line),
                    format!("scan output must be UTF-8: {error}"),
                )],
                stderr: output.stderr,
            });
        }
    };
    if !inventory.problems.is_empty() {
        return Err(Failure::Malformed {
            problems: inventory.problems,
            stderr: output.stderr,
        });
    }
    Ok(inventory)
}

pub(crate) fn run(root: &Path) -> Result<String, Error> {
    let script = script(root)?;
    let destination = root.join(ENTRYPOINTS_FILE);
    // A named pipe or a device in its place is never opened: reading it may wait forever.
    let previous = match read_regular_text(&destination) {
        Ok(text) => Some(text),
        Err(unread) if unread.is_missing() => None,
        Err(Unread::Failed(error)) => {
            return Err(Error {
                code: 2,
                message: format!("Cannot read {ENTRYPOINTS_FILE}: {error}"),
            });
        }
        Err(Unread::NotRegular) => {
            return Err(Error {
                code: 2,
                message: format!(
                    "Cannot read {ENTRYPOINTS_FILE}: not a regular file; roostr reads only \
                     regular files."
                ),
            });
        }
    };
    let inventory = execute(root, &script, Stderr::Relay)?;
    let canonical = inventory.to_tsv();
    if previous.as_deref() != Some(canonical.as_str()) {
        replace_inventory(&destination, &canonical).map_err(|error| Error {
            code: 2,
            message: format!("Cannot write {ENTRYPOINTS_FILE}: {error}"),
        })?;
    }
    Ok(report(&inventory, previous.as_deref()))
}

fn report(inventory: &Inventory, previous: Option<&str>) -> String {
    let count = inventory.entries.len();
    let Some(previous) = previous else {
        return format!("{count} entrypoints, first scan\n");
    };
    let previous = Inventory::parse(previous);
    let added = inventory.missing_from(&previous);
    let removed = previous.missing_from(inventory);
    let added: Vec<_> = added.into_iter().map(ToString::to_string).collect();
    let removed: Vec<_> = removed.into_iter().map(ToString::to_string).collect();
    let mut lines = Vec::new();
    lines.extend(added.iter().map(|reference| format!("+ {reference}")));
    lines.extend(removed.iter().map(|reference| format!("- {reference}")));
    if !lines.is_empty() {
        lines.push(String::new());
    }
    lines.push(format!(
        "{count} entrypoints, {} added, {} removed",
        added.len(),
        removed.len()
    ));
    format!("{}\n", lines.join("\n"))
}

struct TemporaryFile(PathBuf);

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        #[expect(
            clippy::let_underscore_must_use,
            reason = "best-effort cleanup in Drop; a leftover temp file is harmless"
        )]
        let _ = fs::remove_file(&self.0);
    }
}

// A failed write cannot truncate the previous scan. Rename only a complete file,
// in the same directory so the replacement stays on the same filesystem.
fn replace_inventory(destination: &Path, text: &str) -> io::Result<()> {
    static NEXT_FILE: AtomicU64 = AtomicU64::new(0);
    let (temporary, mut file) = loop {
        let sequence = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
        let path = destination.with_file_name(format!(
            ".entrypoints.tsv.{}.{}.tmp",
            std::process::id(),
            sequence
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => break (TemporaryFile(path), file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };
    file.write_all(text.as_bytes())?;
    if let Ok(metadata) = fs::metadata(destination) {
        file.set_permissions(metadata.permissions())?;
    }
    file.sync_all()?;
    fs::rename(&temporary.0, destination)
}

// What the script's shebang line names, as written after `#!` up to the end of the line.
fn interpreter(script: &Path) -> Option<String> {
    let mut first = Vec::new();
    io::BufReader::new(fs::File::open(script).ok()?.take(4096))
        .read_until(b'\n', &mut first)
        .ok()?;
    let line = first.strip_suffix(b"\n").unwrap_or(&first);
    line.strip_prefix(b"#!")
        .map(|program| String::from_utf8_lossy(program).into_owned())
}
