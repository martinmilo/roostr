//! An agent's session: `roostr session start` notes the commit it started at and what was
//! already changed then, in the repository's Git folder, so that `roostr check --session` can
//! judge the session's own work at its end.

use std::fs::{self, OpenOptions};
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::dates;
use crate::git::Git;
use crate::snapshot::{self, Snapshot};

/// The most of a hook's input kept, and how long it is waited for. A hook runner closes stdin
/// once it has written the input; one that never does must not hold the agent. What comes past
/// the limit is read and dropped, so that the runner never writes into a closed pipe.
const INPUT_KEPT: usize = 16 * 1024 * 1024;
const INPUT_WAIT: Duration = Duration::from_secs(2);

/// A record that started longer ago than this is removed by the next `session start`.
const KEPT_SECONDS: i64 = 30 * 24 * 60 * 60;

/// How long before the session started a commit can still be its own: clocks differ.
pub(crate) const COMMIT_LEEWAY: i64 = 300;

/// Where a session started.
#[derive(Debug, PartialEq)]
pub(crate) enum Start {
    Commit(String),
    /// Before the repository's first commit.
    BeforeFirstCommit,
}

/// What `session start` noted.
pub(crate) struct Record {
    pub(crate) start: Start,
    /// When the session started, in seconds.
    pub(crate) started: i64,
    /// The email Git would have committed with then; `None` when it had none.
    pub(crate) committer: Option<String>,
    /// What was already changed then; `None` when it was not taken.
    pub(crate) snapshot: Option<Snapshot>,
}

/// What a hook passed on stdin.
pub(crate) struct Hook {
    input: Option<Input>,
    /// The input was over 16 MiB: no input.
    pub(crate) over_limit: bool,
}

impl Hook {
    /// The input on stdin, when it is one whole JSON object. Stdin is not read when it is a
    /// terminal, nor waited for past its time: what was kept by then counts.
    pub(crate) fn read() -> Self {
        match read_stdin() {
            Stdin::Terminal => Self {
                input: None,
                over_limit: false,
            },
            Stdin::Over => Self {
                input: None,
                over_limit: true,
            },
            Stdin::Kept(input) => Self {
                input: match object(&input) {
                    Some(Value::Object(fields)) => Some(Input { fields }),
                    _ => None,
                },
                over_limit: false,
            },
        }
    }

    /// The `session_id`, when roostr can use it as a file name.
    pub(crate) fn session_id(&self) -> Option<&str> {
        self.input
            .as_ref()?
            .string("session_id")
            .filter(|id| usable(id))
    }

    /// Whether the agent is given the project map: not when the session goes on with the
    /// history that holds the one it was given.
    pub(crate) fn wants_map(&self) -> bool {
        self.input
            .as_ref()
            .is_none_or(|input| !matches!(input.string("source"), Some("resume" | "fork")))
    }

    /// Whether the agent is in plan mode, where it cannot change files.
    pub(crate) fn in_plan_mode(&self) -> bool {
        self.input
            .as_ref()
            .is_some_and(|input| input.string("permission_mode") == Some("plan"))
    }
}

/// The hook's input: the JSON object on stdin.
struct Input {
    fields: Vec<(String, Value)>,
}

impl Input {
    // The value of `key`; of a key written twice, the last, as in most JSON readers.
    fn get(&self, key: &str) -> Option<&Value> {
        self.fields
            .iter()
            .rev()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    fn string(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            Value::String(text) => Some(text),
            _ => None,
        }
    }
}

enum Stdin {
    Terminal,
    Over,
    Kept(Vec<u8>),
}

// What stdin holds once it ends or 2 seconds have passed, at most 16 MiB of it.
fn read_stdin() -> Stdin {
    if io::stdin().is_terminal() {
        return Stdin::Terminal;
    }
    let deadline = Instant::now() + INPUT_WAIT;
    // `None` once more than the limit came.
    let (sender, receiver) = mpsc::channel::<Option<Vec<u8>>>();
    // The reader reads to the end, dropping what is past the limit; one left waiting on a
    // stdin that is never closed ends with the process. The input ends when it drops its
    // sender.
    thread::spawn(move || {
        let mut stdin = io::stdin().lock();
        let mut buffer = vec![0; 64 * 1024];
        let mut kept = 0;
        let mut over = false;
        loop {
            match stdin.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) if !over => {
                    let chunk = &buffer[..read];
                    if kept + read > INPUT_KEPT {
                        over = true;
                        sender.send(None).ok();
                    } else {
                        kept += read;
                        sender.send(Some(chunk.to_vec())).ok();
                    }
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(_) => break,
            }
        }
    });
    let mut input = Vec::new();
    while let Ok(chunk) = receiver.recv_timeout(deadline.saturating_duration_since(Instant::now()))
    {
        match chunk {
            Some(chunk) => input.extend(chunk),
            None => return Stdin::Over,
        }
    }
    Stdin::Kept(input)
}

/// 1 to 128 ASCII letters, digits, `-`, `_` and `.`, not starting with `.`: a file name and
/// never a path.
fn usable(id: &str) -> bool {
    (1..=128).contains(&id.len())
        && !id.starts_with('.')
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

/// Where the records are: in the Git folder every work tree of the repository shares.
pub(crate) fn folder(git: &Git) -> PathBuf {
    git.common_dir().join("roostr").join("sessions")
}

/// Records the session `id` started in the folder `cwd`. An error says why nothing was
/// recorded.
pub(crate) fn start(cwd: &Path, id: Option<&str>) -> Result<(), String> {
    let Some(id) = id else {
        return Err(
            "roostr session start: the input on stdin has no session_id it can use; \
             nothing recorded"
                .into(),
        );
    };
    let cannot = |reason: String| format!("roostr session start: {reason}; nothing recorded");
    let git = match Git::open(cwd) {
        Ok(Some(git)) => git,
        Ok(None) => return Err(cannot("not in a Git repository".into())),
        // Git's reason on one line, the command it suggests among it.
        Err(error) => {
            let reason = error.to_string();
            return Err(cannot(
                reason.split_whitespace().collect::<Vec<_>>().join(" "),
            ));
        }
    };
    let now = dates::now_seconds().map_err(cannot)?;
    let folder = folder(&git);
    prune(&folder, now);
    // A session resumed, compacted or cleared under the same id keeps its start.
    if find(&folder, id).is_some() {
        return Ok(());
    }
    // `git status` is the slow part of the snapshot: it runs while the record is written.
    let taking = snapshot::begin(&git);
    let record = Record {
        start: match git.head() {
            Some(commit) => Start::Commit(commit.to_owned()),
            None => Start::BeforeFirstCommit,
        },
        started: now,
        committer: git.committer_email(),
        snapshot: None,
    };
    let written = write(&folder, id, &record)
        .map_err(|error| cannot(format!("cannot write in {}: {error}", folder.display())))?;
    if let (Some(mut file), Some(taking)) = (written, taking) {
        if let Some(snapshot) = taking.finish(&git) {
            // A snapshot that cannot be added leaves a record without one, which counts every
            // uncommitted change as the session's.
            file.write_all(&snapshot.encode()).ok();
        }
    }
    Ok(())
}

// Writes what the record holds first, unless there is a record: the file, to add the snapshot
// to. Made with `create_new`, so that two hooks starting at once keep one record.
fn write(folder: &Path, id: &str, record: &Record) -> io::Result<Option<fs::File>> {
    fs::create_dir_all(folder)?;
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(folder.join(id))
    {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => return Ok(None),
        Err(error) => return Err(error),
    };
    let commit = match &record.start {
        Start::Commit(commit) => commit.as_str(),
        Start::BeforeFirstCommit => "none",
    };
    let committer = match &record.committer {
        Some(email) => format!("committer {email}"),
        None => "no committer".to_owned(),
    };
    file.write_all(
        format!(
            "roostr session\ncommit {commit}\nstarted {}\n{committer}\n",
            record.started
        )
        .as_bytes(),
    )?;
    Ok(Some(file))
}

// The path of the record of `id`, found in the folder's listing by its exact name: on a file
// system that ignores case, opening `Session-1` would find `session-1`.
fn find(folder: &Path, id: &str) -> Option<PathBuf> {
    fs::read_dir(folder)
        .ok()?
        .filter_map(Result::ok)
        .find(|entry| entry.file_name().to_str() == Some(id))
        .map(|entry| entry.path())
}

fn read(path: &Path) -> Option<Record> {
    let bytes = fs::read(path).ok()?;
    let mut rest = bytes.as_slice();
    let mut line = || {
        let end = rest.iter().position(|byte| *byte == b'\n')?;
        let line = std::str::from_utf8(&rest[..end]).ok()?;
        rest = &rest[end + 1..];
        Some(line)
    };
    if line()? != "roostr session" {
        return None;
    }
    let start = match line()?.strip_prefix("commit ")? {
        "none" => Start::BeforeFirstCommit,
        commit if commit.len() >= 40 && commit.bytes().all(|byte| byte.is_ascii_hexdigit()) => {
            Start::Commit(commit.to_owned())
        }
        _ => return None,
    };
    let started = line()?.strip_prefix("started ")?.parse().ok()?;
    let committer = match line()? {
        "no committer" => None,
        committer => Some(committer.strip_prefix("committer ")?.to_owned()),
    };
    Some(Record {
        start,
        started,
        committer,
        snapshot: Snapshot::decode(rest),
    })
}

/// The record `session start` made for the session `id`.
pub(crate) fn record(git: &Git, id: &str) -> Option<Record> {
    read(&find(&folder(git), id)?)
}

// Removes the records of sessions that started more than 30 days before `now`, and the count
// of holds kept beside each. A record that cannot be read, or that another process removed
// first, is left to be.
fn prune(folder: &Path, now: i64) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let Some(id) = entry
            .file_name()
            .to_str()
            .filter(|id| usable(id))
            .map(str::to_owned)
        else {
            continue;
        };
        if read(&entry.path()).is_some_and(|record| record.started < now - KEPT_SECONDS) {
            fs::remove_file(entry.path()).ok();
            fs::remove_file(holds_file(folder, &id)).ok();
        }
    }
}

// Where `check --session` counts its holds of the session `id`: beside its record, under a
// name no id has.
fn holds_file(folder: &Path, id: &str) -> PathBuf {
    folder.join(format!(".{id}.holds"))
}

/// Counts a hold of the session `id` with what it found, `seen`: how many holds in a row
/// found the same, this one included. `None` when the count cannot be kept.
pub(crate) fn count_hold(git: &Git, id: &str, seen: &str) -> Option<u32> {
    let path = holds_file(&folder(git), id);
    let count = fs::read_to_string(&path)
        .ok()
        .and_then(|text| {
            let (count, last) = text.split_once('\n')?;
            (last == seen).then(|| count.parse::<u32>().ok())?
        })
        .unwrap_or(0)
        .saturating_add(1);
    fs::write(&path, format!("{count}\n{seen}")).ok()?;
    Some(count)
}

/// Starts the count of holds of the session `id` again: a run that did not hold.
pub(crate) fn clear_holds(git: &Git, id: &str) {
    fs::remove_file(holds_file(&folder(git), id)).ok();
}

/// The JSON value `input` holds, when it holds one and nothing else.
fn object(input: &[u8]) -> Option<Value> {
    let text = std::str::from_utf8(input).ok()?;
    let mut parser = Parser {
        bytes: text.as_bytes(),
        at: 0,
    };
    let value = parser.value(0)?;
    parser.space();
    (parser.at == parser.bytes.len()).then_some(value)
}

enum Value {
    String(String),
    Object(Vec<(String, Self)>),
    Other,
}

/// A strict reader of JSON (RFC 8259), enough to find one string in an object.
struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Parser<'_> {
    /// Nesting deeper than this is refused rather than followed.
    const DEPTH: usize = 128;

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn eat(&mut self, byte: u8) -> bool {
        let found = self.peek() == Some(byte);
        if found {
            self.at += 1;
        }
        found
    }

    fn space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }

    fn value(&mut self, depth: usize) -> Option<Value> {
        if depth > Self::DEPTH {
            return None;
        }
        self.space();
        match self.peek()? {
            b'{' => {
                self.at += 1;
                let mut fields = Vec::new();
                self.space();
                if self.eat(b'}') {
                    return Some(Value::Object(fields));
                }
                loop {
                    self.space();
                    let key = self.string()?;
                    self.space();
                    if !self.eat(b':') {
                        return None;
                    }
                    let value = self.value(depth + 1)?;
                    fields.push((key, value));
                    self.space();
                    if self.eat(b'}') {
                        return Some(Value::Object(fields));
                    }
                    if !self.eat(b',') {
                        return None;
                    }
                }
            }
            b'[' => {
                self.at += 1;
                self.space();
                if self.eat(b']') {
                    return Some(Value::Other);
                }
                loop {
                    self.value(depth + 1)?;
                    self.space();
                    if self.eat(b']') {
                        return Some(Value::Other);
                    }
                    if !self.eat(b',') {
                        return None;
                    }
                }
            }
            b'"' => self.string().map(Value::String),
            b't' => self.word(b"true"),
            b'f' => self.word(b"false"),
            b'n' => self.word(b"null"),
            b'-' | b'0'..=b'9' => self.number(),
            _ => None,
        }
    }

    fn word(&mut self, word: &[u8]) -> Option<Value> {
        let found = self.bytes.get(self.at..)?.starts_with(word);
        found.then(|| {
            self.at += word.len();
            Value::Other
        })
    }

    fn digits(&mut self) -> Option<()> {
        let start = self.at;
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.at += 1;
        }
        (self.at > start).then_some(())
    }

    fn number(&mut self) -> Option<Value> {
        self.eat(b'-');
        if !self.eat(b'0') {
            self.digits()?;
        }
        if self.eat(b'.') {
            self.digits()?;
        }
        if self.eat(b'e') || self.eat(b'E') {
            if !self.eat(b'+') {
                self.eat(b'-');
            }
            self.digits()?;
        }
        Some(Value::Other)
    }

    fn hex4(&mut self) -> Option<u32> {
        let digits = std::str::from_utf8(self.bytes.get(self.at..self.at + 4)?).ok()?;
        if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        self.at += 4;
        u32::from_str_radix(digits, 16).ok()
    }

    fn string(&mut self) -> Option<String> {
        if !self.eat(b'"') {
            return None;
        }
        let mut text = String::new();
        let mut start = self.at;
        loop {
            let byte = self.peek()?;
            match byte {
                b'"' => {
                    text.push_str(std::str::from_utf8(&self.bytes[start..self.at]).ok()?);
                    self.at += 1;
                    return Some(text);
                }
                b'\\' => {
                    text.push_str(std::str::from_utf8(&self.bytes[start..self.at]).ok()?);
                    self.at += 1;
                    let escaped = self.peek()?;
                    self.at += 1;
                    text.push(match escaped {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let unit = self.hex4()?;
                            let code = if (0xD800..0xDC00).contains(&unit) {
                                if !(self.eat(b'\\') && self.eat(b'u')) {
                                    return None;
                                }
                                let low = self.hex4()?;
                                if !(0xDC00..0xE000).contains(&low) {
                                    return None;
                                }
                                0x10000 + ((unit - 0xD800) << 10) + (low - 0xDC00)
                            } else {
                                unit
                            };
                            char::from_u32(code)?
                        }
                        _ => return None,
                    });
                    start = self.at;
                }
                0x00..=0x1F => return None,
                _ => self.at += 1,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_session_id_is_a_string_in_one_json_object() {
        let input = |text: &str| match object(text.as_bytes()) {
            Some(Value::Object(fields)) => Some(Input { fields }),
            _ => None,
        };
        let id = |text: &str| input(text)?.string("session_id").map(str::to_owned);
        assert_eq!(
            id(r#"{"session_id":"5f0c8e2a-9b1d","cwd":"/x","n":[1,-2.5e3,true,null,{}]}"#),
            Some("5f0c8e2a-9b1d".into())
        );
        assert_eq!(
            id(" {\"session_id\" : \"a\\u0062\\/c\"} \n"),
            Some("ab/c".into())
        );
        assert_eq!(id(r#"{"session_id":"😀"}"#), Some("\u{1f600}".into()));
        assert_eq!(
            id(r#"{"session_id":"a","session_id":"b"}"#),
            Some("b".into())
        );
        for input in [
            "",
            "session one",
            "[\"session_id\"]",
            r#"{"session_id":5}"#,
            r#"{"other":"a"}"#,
            r#"{"session_id":"a"} trailing"#,
            r#"{"session_id":"a",}"#,
            "{\"session_id\":\"a\tb\"}",
            r#"{"session_id":"\ud83d"}"#,
            r#"{"session_id":01}"#,
            &"[".repeat(10_000),
        ] {
            assert_eq!(id(input), None, "{input:?}");
        }
        for (text, map) in [
            (r#"{"source":"startup"}"#, true),
            (r#"{"source":"resume"}"#, false),
            (r#"{"source":"fork"}"#, false),
            (r#"{"source":"Resume"}"#, true),
            (r#"{"source":42}"#, true),
            (r#"{"source":"resume","source":"clear"}"#, true),
            ("{}", true),
        ] {
            let hook = Hook {
                input: input(text),
                over_limit: false,
            };
            assert_eq!(hook.wants_map(), map, "{text}");
        }
    }

    #[test]
    fn an_id_names_a_file_and_never_a_path() {
        for id in ["5f0c8e2a-9b1d-4c7e", "Session_2026.11.06", &"s".repeat(128)] {
            assert!(usable(id), "{id}");
        }
        for id in [
            "",
            ".session",
            "../../hooks/pre-commit",
            "session one",
            "sessions/one",
            &"s".repeat(129),
            "café",
        ] {
            assert!(!usable(id), "{id}");
        }
    }
}
