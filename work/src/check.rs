//! `roostr check`: whether `.roostr/` is current with the code.

use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::dates;
use crate::entrypoints::Inventory;
use crate::git::{Git, History, Status};
use crate::json;
use crate::model::{EntrypointRef, Finding};
use crate::repository::{ENTRYPOINTS_FILE, PROJECT_FILE, Repository, Unread, read_regular};
use crate::scan::{self, Failure, SCAN_FILE, Stderr};
use crate::session::{self, Hook, Record, Start};
use crate::snapshot;

/// What check judges.
#[derive(Clone, Copy)]
pub(crate) enum Scope<'a> {
    /// The whole project.
    Whole,
    /// The work after a commit, as the user wrote it.
    Base(&'a str),
    /// The commit being made: the work after `HEAD`, and what of `.roostr/` is not staged.
    Staged,
    /// What an agent did in its session, by what its hook passed on stdin.
    Session(Option<&'a Hook>),
}

/// Where the work to judge starts.
enum From<'a> {
    Head,
    /// A base as the user wrote it.
    Named(&'a str),
}

/// What check found wrong with `entrypoints.tsv`.
pub(crate) enum Entrypoints {
    /// The scan script finds other entrypoints, or the same ones written differently.
    Behind {
        added: Vec<String>,
        removed: Vec<String>,
    },
    NeverScanned,
    Failed(Failure),
}

#[derive(Default)]
pub(crate) struct Report {
    /// The files under `.roostr/` whose working-tree state is not what is staged.
    pub(crate) not_staged: Vec<String>,
    pub(crate) validate: Vec<Finding>,
    pub(crate) entrypoints: Option<Entrypoints>,
    pub(crate) processes: Vec<(String, Vec<String>)>,
    pub(crate) not_described: Vec<String>,
    /// The base the work was judged from: as the user wrote it, `HEAD`, or the commit a
    /// session started at. None without one, or for a session that started before the
    /// first commit.
    pub(crate) base: Option<String>,
    /// The value to suggest for `checked`; set when there are processes to report.
    pub(crate) checked: String,
    /// The session judged, when `--session` judged one by its record.
    pub(crate) session: Option<Judged>,
    /// Why `--session` judged as `--base HEAD`, when it says so.
    pub(crate) warning: Option<String>,
}

/// A session `--session` judged, and its work: what the count of holds compares.
pub(crate) struct Judged {
    git: Git,
    id: String,
    commits: Vec<String>,
    uncommitted: Vec<Vec<u8>>,
}

impl Report {
    pub(crate) fn is_empty(&self) -> bool {
        self.not_staged.is_empty()
            && self.validate.is_empty()
            && self.entrypoints.is_none()
            && self.processes.is_empty()
            && self.not_described.is_empty()
    }
}

/// The work after a base commit.
struct Work {
    /// Files touched after the base or uncommitted, as Git paths.
    changed: HashSet<Vec<u8>>,
    uncommitted: HashSet<Vec<u8>>,
    /// The commits whose `entrypoints.tsv` and records NOT DESCRIBED compares with: where the
    /// work left the base. None when nothing is committed yet.
    then: Vec<String>,
    /// The history, when finding the work needed it.
    history: Option<History>,
    /// With `--session`, the files the user left changed before the session started and the
    /// session did not touch.
    left_by_user: HashSet<Vec<u8>>,
}

/// Judges the project in `root` within `scope`. An error means check could not run.
pub(crate) fn run(root: &Path, scope: Scope<'_>) -> Result<Report, String> {
    let git = Git::open(root)?;
    let mut base = None;
    let mut not_staged = Vec::new();
    let mut warning = None;
    let mut judged = None;
    let mut work = match (scope, &git) {
        (Scope::Whole, Some(git)) => {
            if let Err(error) = git.whole_history() {
                return Err(format!(
                    "{error}\nroostr check --base <commit> can run in a shallow clone when the \
                     history after the commit is all in it."
                ));
            }
            None
        }
        (Scope::Whole, None) => None,
        (scope, None) => {
            let option = match scope {
                Scope::Staged => "--staged",
                Scope::Session(_) => "--session",
                _ => "--base",
            };
            return Err(format!(
                "This is not a Git repository; roostr check {option} needs Git history."
            ));
        }
        (Scope::Base(name), Some(git)) => {
            base = Some(name.to_owned());
            Some(work_after(git, From::Named(name), &git.status()?)?)
        }
        (Scope::Staged, Some(git)) => {
            base = Some("HEAD".to_owned());
            let status = git.status()?;
            not_staged = unstaged(git, &status);
            Some(work_after(git, From::Head, &status)?)
        }
        (Scope::Session(hook), Some(git)) => {
            let status = git.status()?;
            match session_record(git, hook, &mut warning)? {
                Some((id, record)) => {
                    base = match &record.start {
                        Start::Commit(commit) => Some(commit.clone()),
                        Start::BeforeFirstCommit => None,
                    };
                    let (work, commits) = session_work(git, &record, &status)?;
                    let mut uncommitted: Vec<_> = work.uncommitted.iter().cloned().collect();
                    uncommitted.sort();
                    judged = Some((id.to_owned(), commits, uncommitted));
                    Some(work)
                }
                None => {
                    base = Some("HEAD".to_owned());
                    Some(work_after(git, From::Head, &status)?)
                }
            }
        }
    };
    let mut report = Report {
        base,
        warning,
        ..Report::default()
    };
    if work
        .as_ref()
        .is_some_and(|work| work.changed.is_empty() && not_staged.is_empty())
    {
        report.session = judged
            .zip(git)
            .map(|((id, commits, uncommitted), git)| Judged {
                git,
                id,
                commits,
                uncommitted,
            });
        return Ok(report);
    }
    let repository = Repository::load(root.to_owned());
    // What the user left changed before the session, and the session did not touch, is not
    // the session's to fix.
    let left_by_user = |file: &str| {
        work.as_ref().is_some_and(|work| {
            !work.left_by_user.is_empty()
                && git
                    .as_ref()
                    .and_then(|git| git.path(file).ok())
                    .is_some_and(|path| work.left_by_user.contains(&path))
        })
    };
    report.not_staged = not_staged;
    report.validate = repository
        .validate()
        .into_iter()
        .filter(|finding| !left_by_user(&finding.file))
        .collect();
    let scan_needed = match (&work, &git) {
        (Some(work), Some(git)) => !work
            .changed
            .iter()
            .all(|path| git.in_project(path).is_some_and(is_record)),
        _ => true,
    };
    if scan_needed {
        report.entrypoints = entrypoints(root, &left_by_user)?;
    }
    if let Some(git) = &git {
        report.processes = match &mut work {
            Some(work) => git.attention(
                &repository,
                &work.uncommitted,
                Some(&work.changed),
                &mut work.history,
            )?,
            None => git.attention(&repository, &git.uncommitted()?, None, &mut None)?,
        };
        if !report.processes.is_empty() {
            let listed: HashSet<&str> =
                report.processes.iter().map(|(id, _)| id.as_str()).collect();
            let held = repository
                .readable_processes()
                .filter(|(id, _)| listed.contains(id))
                .filter_map(|(_, process)| process.checked.as_deref());
            report.checked = dates::checked_value(dates::now()?, held);
        }
        if let Some(work) = &work {
            report.not_described = not_described(git, &repository, work)?;
        }
    }
    report.session = judged
        .zip(git)
        .map(|((id, commits, uncommitted), git)| Judged {
            git,
            id,
            commits,
            uncommitted,
        });
    Ok(report)
}

// The session the hook names and its record, when there is one to judge by. Otherwise check
// judges as `--base HEAD`: without a word when the input names no session it can use, and
// saying why when the input was too large or there is no usable record.
fn session_record<'a>(
    git: &Git,
    hook: Option<&'a Hook>,
    warning: &mut Option<String>,
) -> Result<Option<(&'a str, Record)>, String> {
    let Some(hook) = hook else {
        return Ok(None);
    };
    if hook.over_limit {
        *warning = Some(
            "roostr check --session: the input on stdin is over 16 MiB; judged as --base HEAD"
                .into(),
        );
        return Ok(None);
    }
    let Some(id) = hook.session_id() else {
        return Ok(None);
    };
    let record = session::record(git, id).filter(|record| match &record.start {
        // The commit may be gone: reset, then removed by `git gc`.
        Start::Commit(commit) if Some(commit.as_str()) != git.head() => {
            git.resolve(commit).ok().flatten().is_some()
        }
        _ => true,
    });
    if record.is_none() {
        *warning = Some(format!(
            "roostr check --session: no usable record of session {id}; judged as --base HEAD"
        ));
    }
    Ok(record.map(|record| (id, record)))
}

// The session's work: the files its commits touched, and those changed and not as the
// snapshot noted them. A commit is the session's when it is reachable from `HEAD` and not
// from the start, is not a replay of one the start reaches, has the session's committer,
// and was made at most 5 minutes before the session started. Also the commits, for the count
// of holds.
fn session_work(
    git: &Git,
    record: &Record,
    status: &Status,
) -> Result<(Work, Vec<String>), String> {
    let all_uncommitted = git.changed(status)?;
    let (commits, then, history) = match &record.start {
        Start::Commit(start) if Some(start.as_str()) == git.head() && !git.is_merging() => {
            (Vec::new(), vec![start.clone()], None)
        }
        Start::Commit(start) => {
            let shares_none = || {
                format!(
                    "Cannot judge the session: HEAD shares no commit with the commit it \
                     started at, {}.",
                    start.get(..7).unwrap_or(start)
                )
            };
            let head = git.head().ok_or_else(shares_none)?;
            let merge_bases = git.merge_bases(start, head)?;
            if merge_bases.is_empty() {
                return Err(shares_none());
            }
            let history = whole_after(git, &merge_bases)?;
            (
                git.commits_after(Some(start))?,
                merge_bases.into_iter().take(1).collect(),
                history,
            )
        }
        Start::BeforeFirstCommit => {
            let history = whole_after(git, &[])?;
            (git.commits_after(None)?, Vec::new(), history)
        }
    };
    let earliest = record.started - session::COMMIT_LEEWAY;
    let commits: Vec<_> = commits
        .into_iter()
        .filter(|commit| {
            commit.timestamp >= earliest
                && record
                    .committer
                    .as_ref()
                    .is_none_or(|email| *email == commit.committer)
        })
        .collect();
    let committed: HashSet<Vec<u8>> = commits
        .iter()
        .flat_map(|commit| commit.touched.iter().cloned())
        .collect();
    // In another work tree than the one it was taken in, the snapshot says nothing.
    let as_noted: HashSet<Vec<u8>> = match record
        .snapshot
        .as_ref()
        .filter(|snapshot| snapshot.worktree == git.git_dir().as_os_str().as_bytes())
    {
        Some(snapshot) => {
            let paths: Vec<&[u8]> = all_uncommitted.iter().map(Vec::as_slice).collect();
            snapshot
                .unchanged(git, &paths)
                .into_iter()
                .map(<[u8]>::to_vec)
                .collect()
        }
        None => HashSet::new(),
    };
    let uncommitted: HashSet<Vec<u8>> = all_uncommitted.difference(&as_noted).cloned().collect();
    let left_by_user = as_noted.difference(&committed).cloned().collect();
    let mut changed = committed;
    changed.extend(uncommitted.iter().cloned());
    Ok((
        Work {
            changed,
            uncommitted,
            then,
            history,
            left_by_user,
        },
        commits.into_iter().map(|commit| commit.id).collect(),
    ))
}

// In a shallow clone, the history, once it is known to hold every commit after the start,
// given as its merge bases with `HEAD`; outside one, nothing needs walking yet.
fn whole_after(git: &Git, merge_bases: &[String]) -> Result<Option<History>, String> {
    if !git.is_shallow() {
        return Ok(None);
    }
    let history = git.history()?;
    if !git.whole_after(&history, merge_bases) {
        return Err(
            "Cannot judge the session: the history after the commit it started at is \
                    not all in this shallow clone. git fetch --unshallow fetches it."
                .into(),
        );
    }
    Ok(Some(history))
}

/// The line `--session` prints in place of the report when it lets the agent through.
pub(crate) const NOT_HELD_AGAIN: &str = "Not held again: 3 holds in a row found the same \
     problems and no change to the session's work. The problems remain; the pre-commit hook \
     and CI will report them.";

/// Whether `--session` lets the agent through rather than hold it, by the count of holds in
/// a row with the same problems and no change to the session's work. The count starts again
/// when the report holds nothing, and a count that cannot be kept lets nothing through.
pub(crate) fn let_through(report: &Report, scope: Scope<'_>) -> bool {
    let Some(judged) = &report.session else {
        return false;
    };
    if report.is_empty() {
        session::clear_holds(&judged.git, &judged.id);
        return false;
    }
    let paths: Vec<&[u8]> = judged.uncommitted.iter().map(Vec::as_slice).collect();
    let mut seen = DefaultHasher::new();
    json::check(scope, report).hash(&mut seen);
    judged.commits.hash(&mut seen);
    judged.uncommitted.hash(&mut seen);
    snapshot::states(&judged.git, &paths, None, None).hash(&mut seen);
    session::count_hold(&judged.git, &judged.id, &format!("{:016x}", seen.finish()))
        .is_some_and(|count| count > 3)
}

/// Starts the count of holds again for a session that was not judged.
pub(crate) fn not_judged(root: &Path, hook: &Hook) {
    if let (Some(id), Ok(Some(git))) = (hook.session_id(), Git::open(root)) {
        session::clear_holds(&git, id);
    }
}

fn work_after(git: &Git, from: From<'_>, status: &Status) -> Result<Work, String> {
    let label = match &from {
        From::Named(name) => (*name).to_owned(),
        From::Head => "HEAD".to_owned(),
    };
    let not_a_commit = || {
        format!(
            "Cannot use --base {label}: it is not a commit in this repository. If it exists \
             elsewhere, fetching more history may help."
        )
    };
    let not_all_here = || {
        format!(
            "Cannot use --base {label}: the history after it is not all in this shallow \
             clone. git fetch --unshallow fetches it."
        )
    };
    let uncommitted = git.changed(status)?;
    let mut changed = uncommitted.clone();
    let mut history = None;
    let then = match (git.head(), from) {
        (None, From::Named(base)) if base != "HEAD" => return Err(not_a_commit()),
        // Before the first commit, everything in the working tree is the work.
        (None, _) => Vec::new(),
        (Some(head), from) => {
            let commit = match from {
                From::Named(base) if base != "HEAD" => {
                    git.resolve(base)?.ok_or_else(not_a_commit)?
                }
                _ => head.to_owned(),
            };
            if commit == head {
                // During a merge, the commits being merged are part of this base too: only
                // what the merge itself changes is the work.
                std::iter::once(commit)
                    .chain(git.merging().iter().cloned())
                    .collect()
            } else {
                let merge_bases = git.merge_bases(&commit, head)?;
                if merge_bases.is_empty() {
                    return Err(format!(
                        "Cannot use --base {label}: it shares no commit with HEAD. In a shallow \
                         clone, fetch more history (git fetch --unshallow)."
                    ));
                }
                // Work committed after the base: what HEAD (and a merge in progress) reaches
                // and the base does not.
                if git.is_merging() || merge_bases.iter().all(|merge_base| merge_base != head) {
                    let walked = git.history()?;
                    if !git.whole_after(&walked, &merge_bases) {
                        return Err(not_all_here());
                    }
                    changed.extend(git.touched_after(&walked, &merge_bases));
                    history = Some(walked);
                }
                merge_bases.into_iter().take(1).collect()
            }
        }
    };
    Ok(Work {
        changed,
        uncommitted,
        then,
        history,
        left_by_user: HashSet::new(),
    })
}

// The files under `.roostr/` that roostr reads and whose working-tree state is not what is
// staged, by their paths in the project, in path order.
fn unstaged(git: &Git, status: &Status) -> Vec<String> {
    let mut files: Vec<String> = status
        .not_staged()
        .filter_map(|path| git.in_project(path))
        .filter(|path| read_by_roostr(path))
        .map(|path| String::from_utf8_lossy(path).into_owned())
        .collect();
    files.sort();
    files.dedup();
    files
}

// `project.yml`, a record `processes/<id>.yml`, the scan and `entrypoints.tsv`: the files
// under `.roostr/` that roostr reads.
fn read_by_roostr(path: &[u8]) -> bool {
    [PROJECT_FILE, SCAN_FILE, ENTRYPOINTS_FILE]
        .iter()
        .any(|file| path == file.as_bytes())
        || path
            .strip_prefix(b".roostr/processes/")
            .is_some_and(|name| {
                !name.contains(&b'/') && !name.starts_with(b".") && name.ends_with(b".yml")
            })
}

// A file whose change cannot change what the scan script finds.
fn is_record(path: &[u8]) -> bool {
    path == PROJECT_FILE.as_bytes() || path.starts_with(b".roostr/processes/")
}

// Whether `entrypoints.tsv` is current: what the scan finds compared with it byte for byte,
// without the rows, on either side, whose file the user left changed (see `--session`).
fn entrypoints(
    root: &Path,
    left_by_user: &dyn Fn(&str) -> bool,
) -> Result<Option<Entrypoints>, String> {
    let script = match scan::script(root) {
        Ok(script) => script,
        Err(Failure::Missing(error)) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(failure) => return Ok(Some(Entrypoints::Failed(failure))),
    };
    let recorded = match read_regular(&root.join(ENTRYPOINTS_FILE)) {
        Ok(bytes) => without_rows(&bytes, left_by_user),
        Err(unread) if unread.is_missing() => return Ok(Some(Entrypoints::NeverScanned)),
        Err(Unread::Failed(error)) => {
            return Err(format!("Cannot read {ENTRYPOINTS_FILE}: {error}"));
        }
        Err(Unread::NotRegular) => {
            return Err(format!(
                "Cannot read {ENTRYPOINTS_FILE}: not a regular file; roostr reads only regular \
                 files."
            ));
        }
    };
    let mut found = match scan::execute(root, &script, Stderr::Keep) {
        Ok(found) => found,
        Err(failure) => return Ok(Some(Entrypoints::Failed(failure))),
    };
    found.entries.retain(|entry| !left_by_user(&entry.file));
    if found.to_tsv().as_bytes() == recorded {
        return Ok(None);
    }
    let recorded = Inventory::parse(&String::from_utf8_lossy(&recorded));
    let names = |references: Vec<_>| references.into_iter().map(ToString::to_string).collect();
    Ok(Some(Entrypoints::Behind {
        added: names(found.missing_from(&recorded)),
        removed: names(recorded.missing_from(&found)),
    }))
}

// The rows of `entrypoints.tsv` whose file is not one of `left`, as written; the first line,
// `# covers:`, stays.
fn without_rows(text: &[u8], left: &dyn Fn(&str) -> bool) -> Vec<u8> {
    let mut kept = Vec::with_capacity(text.len());
    for (index, line) in text.split_inclusive(|byte| *byte == b'\n').enumerate() {
        let row = line.strip_suffix(b"\n").unwrap_or(line);
        let file = row.split(|byte| *byte == b'\t').nth(2);
        if index > 0 && file.is_some_and(|file| left(&String::from_utf8_lossy(file))) {
            continue;
        }
        kept.extend_from_slice(line);
    }
    kept
}

// The entrypoints in `entrypoints.tsv` that no process lists, unless at the base the file
// already had them and no process listed them there: that is old debt. A new entrypoint is
// reported, and so is one that lost its description.
fn not_described(git: &Git, repository: &Repository, work: &Work) -> Result<Vec<String>, String> {
    let Some(now) = &repository.inventory else {
        return Ok(Vec::new());
    };
    // With the file and the records as they were at the base, nothing is new.
    let touched = work.changed.iter().any(|path| {
        git.in_project(path).is_some_and(|path| {
            path == ENTRYPOINTS_FILE.as_bytes() || path.starts_with(b".roostr/processes/")
        })
    });
    if !touched {
        return Ok(Vec::new());
    }
    let mut compared = false;
    let mut debt: HashSet<EntrypointRef> = HashSet::new();
    let mut listed_then: HashMap<String, Vec<EntrypointRef>> = HashMap::new();
    for commit in &work.then {
        let at = git.records_at(commit)?;
        let mut listed = HashSet::new();
        for (id, parsed) in &at.processes {
            // A record roostr cannot read lists nothing.
            let Some(process) = parsed.readable() else {
                continue;
            };
            let references = process.entrypoints.iter().map(|item| item.value.clone());
            listed.extend(references.clone());
            listed_then
                .entry(id.clone())
                .or_default()
                .extend(references);
        }
        if let Some(inventory) = at.inventory {
            compared = true;
            debt.extend(
                inventory
                    .entries
                    .into_iter()
                    .map(|entry| entry.reference)
                    .filter(|reference| !listed.contains(reference)),
            );
        }
    }
    if !compared {
        return Ok(Vec::new());
    }
    let mut listed: HashSet<&EntrypointRef> = HashSet::new();
    for record in &repository.processes {
        match record.parsed.readable() {
            Some(process) => listed.extend(process.entrypoints.iter().map(|item| &item.value)),
            // A record that is not readable now lists what it listed at the base.
            None => listed.extend(listed_then.get(&record.id).into_iter().flatten()),
        }
    }
    let mut reported = HashSet::new();
    Ok(now
        .entries
        .iter()
        .map(|entry| &entry.reference)
        .filter(|reference| !listed.contains(reference) && !debt.contains(*reference))
        .filter(|reference| reported.insert(*reference))
        .map(ToString::to_string)
        .collect())
}
