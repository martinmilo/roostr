use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::entrypoints::Inventory;
use crate::repository::ENTRYPOINTS_FILE;

const SCAN_FILE: &str = ".roostr/scan";

pub(crate) struct Error {
    pub(crate) code: u8,
    pub(crate) message: String,
}

impl Error {
    fn cannot_run(message: impl Into<String>) -> Self {
        Self {
            code: 2,
            message: message.into(),
        }
    }

    fn refused(message: impl Into<String>) -> Self {
        Self {
            code: 1,
            message: message.into(),
        }
    }
}

pub(crate) fn run(root: &Path) -> Result<String, Error> {
    let script = root.join(SCAN_FILE);
    let metadata = fs::metadata(&script).map_err(|error| {
        Error::cannot_run(format!(
            "Cannot read {SCAN_FILE}: {error}. Add an executable file at {SCAN_FILE}."
        ))
    })?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(Error::cannot_run(format!(
            "{SCAN_FILE} must be an executable file; run chmod +x {SCAN_FILE}."
        )));
    }
    let destination = root.join(ENTRYPOINTS_FILE);
    let previous = match fs::read_to_string(&destination) {
        Ok(text) => Some(text),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(Error::cannot_run(format!(
                "Cannot read {ENTRYPOINTS_FILE}: {error}"
            )));
        }
    };

    // Execute the file itself, without a shell, and relay stderr unchanged as it arrives.
    let output = Command::new(&script)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|error| Error::cannot_run(format!("Cannot run {SCAN_FILE}: {error}")))?;
    if !output.status.success() {
        let reason = match output.status.code() {
            Some(code) => format!("exited with code {code}"),
            None => format!("terminated by a signal ({})", output.status),
        };
        return Err(Error::refused(format!(
            "{SCAN_FILE} {reason}; scan refused."
        )));
    }
    let text = std::str::from_utf8(&output.stdout).map_err(|error| {
        let line = output.stdout[..error.valid_up_to()]
            .iter()
            .filter(|&&byte| byte == b'\n')
            .count()
            + 1;
        Error::refused(format!("line {line}: scan output must be UTF-8: {error}"))
    })?;
    let inventory = Inventory::parse_scan(text);
    if !inventory.problems.is_empty() {
        return Err(Error::refused(inventory.problems.join("\n")));
    }
    let canonical = inventory.to_tsv();
    if previous.as_deref() != Some(canonical.as_str()) {
        replace_inventory(&destination, &canonical).map_err(|error| {
            Error::cannot_run(format!("Cannot write {ENTRYPOINTS_FILE}: {error}"))
        })?;
    }
    Ok(report(&inventory, previous.as_deref()))
}

fn report(inventory: &Inventory, previous: Option<&str>) -> String {
    let count = inventory.entries.len();
    let Some(previous) = previous else {
        return format!("{count} entrypoints, first scan\n");
    };
    let previous = Inventory::parse(previous).references();
    let current = inventory.references();
    let added: Vec<_> = current.difference(&previous).collect();
    let removed: Vec<_> = previous.difference(&current).collect();
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
