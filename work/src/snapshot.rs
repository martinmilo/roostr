//! What was already changed in the work tree when a session started: each file `git status`
//! listed, with its content as `git hash-object --no-filters` names it, so that `check
//! --session` can tell the user's unfinished work from the session's.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crate::git::{self, Git};

/// The largest file hashed; a larger one is noted by its size and modification time.
const HASHED_AT_MOST: u64 = 8 * 1024 * 1024;

/// How long the snapshot may take, from when it begins.
const BUDGET: Duration = Duration::from_secs(3);

/// A file as noted.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum State {
    Deleted,
    /// Its content's blob id.
    Blob(String),
    /// A file not hashed: too large, not a regular file, or past the budget.
    Stat {
        size: u64,
        seconds: i64,
        nanos: i64,
    },
    /// A submodule or a nested repository, by the commit its `HEAD` is at.
    Commit(Option<String>),
    /// A symbolic link, by where it points.
    Link(Vec<u8>),
}

/// The files that differed from the start commit when the session started.
pub(crate) struct Snapshot {
    /// The Git folder of the work tree it was taken in.
    pub(crate) worktree: Vec<u8>,
    pub(crate) files: HashMap<Vec<u8>, State>,
}

/// A snapshot under way: `git status` started, its output on its way.
pub(crate) struct Taking {
    child: Child,
    output: Receiver<io::Result<Vec<u8>>>,
    deadline: Instant,
}

/// Starts the snapshot of the whole work tree: `git status` runs while the record is written.
pub(crate) fn begin(git: &Git) -> Option<Taking> {
    let deadline = Instant::now() + BUDGET;
    let mut child = git
        .process()
        .current_dir(git.top())
        .args([
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignore-submodules=none",
            "--no-renames",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let (sender, output) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        sender
            .send(stdout.read_to_end(&mut bytes).map(|_| bytes))
            .ok();
    });
    Some(Taking {
        child,
        output,
        deadline,
    })
}

/// A `git status` not waited for, its snapshot not wanted or too late, is stopped.
impl Drop for Taking {
    fn drop(&mut self) {
        // Once waited for, the child is not signalled again.
        self.child.kill().ok();
        self.child.wait().ok();
    }
}

impl Taking {
    /// The snapshot, unless `git status` failed or did not answer within the budget: then it
    /// is stopped, and there is none. Files not hashed by the end of the budget are noted by
    /// their size and modification time.
    pub(crate) fn finish(mut self, git: &Git) -> Option<Snapshot> {
        let output = self
            .output
            .recv_timeout(self.deadline.saturating_duration_since(Instant::now()));
        // Not answered in time: `Drop` stops it.
        let Ok(Ok(output)) = output else {
            return None;
        };
        if !self.child.wait().ok()?.success() {
            return None;
        }
        let status = git::parse_status(&output).ok()?;
        let mut paths: Vec<&[u8]> = status.paths().collect();
        paths.sort_unstable();
        paths.dedup();
        let states = states(git, &paths, None, Some(self.deadline));
        Some(Snapshot {
            worktree: git.git_dir().as_os_str().as_bytes().to_vec(),
            files: paths.into_iter().map(<[u8]>::to_vec).zip(states).collect(),
        })
    }
}

impl Snapshot {
    /// The paths among `paths` whose file is still as the snapshot noted it.
    pub(crate) fn unchanged<'a>(&self, git: &Git, paths: &[&'a [u8]]) -> Vec<&'a [u8]> {
        let noted: Vec<(&[u8], &State)> = paths
            .iter()
            .filter_map(|path| Some((*path, self.files.get(*path)?)))
            .collect();
        let (paths, notes): (Vec<&[u8]>, Vec<&State>) = noted.into_iter().unzip();
        let now = states(git, &paths, Some(&notes), None);
        paths
            .into_iter()
            .zip(notes)
            .zip(now)
            .filter(|((_, noted), now)| *noted == now)
            .map(|((path, _), _)| path)
            .collect()
    }

    /// The snapshot as the record holds it: the work tree, then each path and its state, every
    /// field ended by a NUL, and `end` last, so that a snapshot cut short is not read.
    pub(crate) fn encode(&self) -> Vec<u8> {
        let mut bytes = b"snapshot\n".to_vec();
        let mut field = |field: &[u8]| {
            bytes.extend_from_slice(field);
            bytes.push(0);
        };
        field(&self.worktree);
        let mut files: Vec<_> = self.files.iter().collect();
        files.sort_by(|a, b| a.0.cmp(b.0));
        for (path, state) in files {
            field(path);
            field(&match state {
                State::Deleted => b"deleted".to_vec(),
                State::Blob(id) => format!("blob {id}").into_bytes(),
                State::Stat {
                    size,
                    seconds,
                    nanos,
                } => format!("stat {size} {seconds} {nanos}").into_bytes(),
                State::Commit(Some(id)) => format!("commit {id}").into_bytes(),
                State::Commit(None) => b"commit none".to_vec(),
                State::Link(target) => [b"link ".as_slice(), target].concat(),
            });
        }
        field(b"end");
        bytes
    }

    pub(crate) fn decode(bytes: &[u8]) -> Option<Self> {
        let mut fields = bytes.strip_prefix(b"snapshot\n")?.split(|byte| *byte == 0);
        let worktree = fields.next()?.to_vec();
        let mut files = HashMap::new();
        loop {
            let path = fields.next()?;
            if path == b"end" {
                return Some(Self { worktree, files });
            }
            let state = fields.next()?;
            let text = || std::str::from_utf8(state).ok();
            let state = if state == b"deleted" {
                State::Deleted
            } else if let Some(target) = state.strip_prefix(b"link ") {
                State::Link(target.to_vec())
            } else if let Some(id) = text()?.strip_prefix("blob ") {
                State::Blob(id.to_owned())
            } else if let Some(id) = text()?.strip_prefix("commit ") {
                State::Commit((id != "none").then(|| id.to_owned()))
            } else {
                let mut numbers = text()?.strip_prefix("stat ")?.split(' ');
                State::Stat {
                    size: numbers.next()?.parse().ok()?,
                    seconds: numbers.next()?.parse().ok()?,
                    nanos: numbers.next()?.parse().ok()?,
                }
            };
            files.insert(path.to_vec(), state);
        }
    }
}

/// How a file is to be looked at.
enum Look {
    Done(State),
    Hash,
    Nested,
}

/// The state of each of `paths` now, Git paths from the top of the work tree. With `notes`, each
/// is looked at as its note was taken, so that a file noted by its size is not hashed now.
/// Past `deadline`, files are not hashed, and repositories are noted by their size too.
pub(crate) fn states(
    git: &Git,
    paths: &[&[u8]],
    notes: Option<&[&State]>,
    deadline: Option<Instant>,
) -> Vec<State> {
    let top = git.top();
    let on_disk =
        |path: &[u8]| top.join(OsStr::from_bytes(path.strip_suffix(b"/").unwrap_or(path)));
    let mut looks: Vec<Look> = paths
        .iter()
        .enumerate()
        .map(|(index, path)| look(&on_disk(path), notes.map(|notes| notes[index])))
        .collect();
    let to_hash: Vec<usize> = (0..looks.len())
        .filter(|&index| matches!(looks[index], Look::Hash))
        .collect();
    let ids = hash(
        git,
        &top,
        &to_hash
            .iter()
            .map(|&index| paths[index])
            .collect::<Vec<_>>(),
        deadline,
    );
    for (index, id) in to_hash.into_iter().zip(ids) {
        looks[index] = Look::Done(match id {
            Some(id) => State::Blob(id),
            None => stat(&on_disk(paths[index])),
        });
    }
    looks
        .into_iter()
        .zip(paths)
        .map(|(look, path)| match look {
            Look::Done(state) => state,
            Look::Hash => stat(&on_disk(path)),
            Look::Nested => {
                if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                    stat(&on_disk(path))
                } else {
                    nested_commit(git, &on_disk(path))
                }
            }
        })
        .collect()
}

fn look(path: &Path, noted: Option<&State>) -> Look {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Look::Done(State::Deleted);
    };
    let kind = metadata.file_type();
    if kind.is_symlink() {
        return Look::Done(match fs::read_link(path) {
            Ok(target) => State::Link(target.into_os_string().into_vec()),
            Err(_) => stat_of(&metadata),
        });
    }
    if kind.is_dir() && noted.is_none_or(|noted| matches!(noted, State::Commit(_))) {
        return Look::Nested;
    }
    if kind.is_file()
        && metadata.len() <= HASHED_AT_MOST
        && noted.is_none_or(|noted| matches!(noted, State::Blob(_)))
    {
        return Look::Hash;
    }
    Look::Done(stat_of(&metadata))
}

fn stat(path: &Path) -> State {
    match fs::symlink_metadata(path) {
        Ok(metadata) => stat_of(&metadata),
        Err(_) => State::Deleted,
    }
}

fn stat_of(metadata: &fs::Metadata) -> State {
    State::Stat {
        size: metadata.len(),
        seconds: metadata.mtime(),
        nanos: metadata.mtime_nsec(),
    }
}

// The commit the repository in `folder` has `HEAD` at: a submodule or a nested repository.
// A folder that is no repository of its own has none.
fn nested_commit(git: &Git, folder: &Path) -> State {
    if !folder.join(".git").exists() {
        return State::Commit(None);
    }
    let output = git
        .process()
        .current_dir(folder)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .args(["rev-parse", "--verify", "--quiet", "HEAD^{commit}"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output();
    State::Commit(
        output
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|id| id.trim().to_owned())
            .filter(|id| !id.is_empty()),
    )
}

// The blob id of each file, as `git hash-object --no-filters` computes it: no object is
// written, no clean filter runs. One Git call for all; a file it cannot read, and those
// after it, and every file past `deadline`, have none.
fn hash(
    git: &Git,
    top: &PathBuf,
    paths: &[&[u8]],
    deadline: Option<Instant>,
) -> Vec<Option<String>> {
    let mut ids = vec![None; paths.len()];
    if paths.is_empty() || deadline.is_some_and(|deadline| Instant::now() >= deadline) {
        return ids;
    }
    let Ok(mut child) = git
        .process()
        .current_dir(top)
        .args(["hash-object", "--no-filters", "--stdin-paths"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return ids;
    };
    let (Some(mut stdin), Some(mut stdout)) = (child.stdin.take(), child.stdout.take()) else {
        return ids;
    };
    let mut input = Vec::new();
    for path in paths {
        input.extend(stdin_path(path));
    }
    // Written and read beside each other, so that neither pipe fills while the other waits.
    thread::spawn(move || {
        stdin.write_all(&input).ok();
    });
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).ok();
        sender.send(bytes).ok();
    });
    let output = match deadline {
        Some(deadline) => receiver
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .ok(),
        None => receiver.recv().ok(),
    };
    let Some(output) = output else {
        child.kill().ok();
        child.wait().ok();
        return ids;
    };
    child.wait().ok();
    for (id, line) in ids.iter_mut().zip(output.split(|byte| *byte == b'\n')) {
        let line = String::from_utf8_lossy(line);
        if !line.is_empty() && line.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            *id = Some(line.into_owned());
        }
    }
    ids
}

// A path as `--stdin-paths` reads it, one a line: quoted as C quotes it when it holds a
// newline or starts with a quote.
fn stdin_path(path: &[u8]) -> Vec<u8> {
    let mut line = Vec::with_capacity(path.len() + 3);
    if path.contains(&b'\n') || path.starts_with(b"\"") {
        line.push(b'"');
        for &byte in path {
            match byte {
                b'"' | b'\\' => line.extend([b'\\', byte]),
                b'\n' => line.extend(b"\\n"),
                _ => line.push(byte),
            }
        }
        line.push(b'"');
    } else {
        line.extend_from_slice(path);
    }
    line.push(b'\n');
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_snapshot_reads_back_as_it_was_written() {
        let snapshot = Snapshot {
            worktree: b"/work/.git".to_vec(),
            files: HashMap::from([
                (b"app/a.rb".to_vec(), State::Blob("ab12".into())),
                (b"gone\nline".to_vec(), State::Deleted),
                (
                    b"big.bin".to_vec(),
                    State::Stat {
                        size: 9_000_000,
                        seconds: 1_700_000_000,
                        nanos: 5,
                    },
                ),
                (b"vendor/lib/".to_vec(), State::Commit(Some("cd34".into()))),
                (b"empty/".to_vec(), State::Commit(None)),
                (b"link".to_vec(), State::Link(b"../target".to_vec())),
            ]),
        };
        let read = Snapshot::decode(&snapshot.encode()).unwrap();
        assert_eq!(read.worktree, snapshot.worktree);
        assert_eq!(read.files, snapshot.files);
        let cut = snapshot.encode();
        assert!(Snapshot::decode(&cut[..cut.len() - 4]).is_none());
    }

    #[test]
    fn a_path_with_a_newline_is_quoted_for_stdin_paths() {
        assert_eq!(stdin_path(b"app/a.rb"), b"app/a.rb\n");
        assert_eq!(stdin_path(b"a\n\"b\\"), b"\"a\\n\\\"b\\\\\"\n");
        assert_eq!(stdin_path(b"\"q"), b"\"\\\"q\"\n");
    }
}
