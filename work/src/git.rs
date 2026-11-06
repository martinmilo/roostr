use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::{fs, io};

use crate::dates::Period;
use crate::entrypoints::Inventory;
use crate::model::{Parsed, Process};
use crate::repository::{ENTRYPOINTS_FILE, PROCESSES_FOLDER, Repository};

// Every option below exists in Git 2.25, the oldest Git roostr supports. Configuration that
// a newer Git reads and an older one does not know is set with `-c`, which an older Git
// ignores.

/// The oldest Git roostr runs with.
const OLDEST: (u32, u32) = (2, 25);

/// Why Git could not be read: `git` is not on the `PATH`, it is older than roostr needs, or
/// what Git said.
#[derive(Debug)]
pub(crate) struct Unreadable(pub(crate) String);

impl From<String> for Unreadable {
    fn from(reason: String) -> Self {
        Self(reason)
    }
}

impl From<&str> for Unreadable {
    fn from(reason: &str) -> Self {
        Self(reason.to_owned())
    }
}

// What a command that cannot do its work without Git says.
impl std::fmt::Display for Unreadable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Git could not be read: {}", self.0)
    }
}

impl From<Unreadable> for String {
    fn from(error: Unreadable) -> Self {
        error.to_string()
    }
}

fn not_started(error: &io::Error) -> Unreadable {
    if error.kind() == io::ErrorKind::NotFound {
        Unreadable("git was not found on the PATH".into())
    } else {
        Unreadable(format!("git could not be started: {error}"))
    }
}

// The version `git --version` gives, when it is older than roostr needs. Asked only after
// a Git call failed, so that a working Git costs no extra call.
fn too_old() -> Option<String> {
    let output = Command::new("git")
        .arg("--version")
        .env("LC_ALL", "C")
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let version = text.trim().strip_prefix("git version ")?;
    (parse_version(version)? < OLDEST).then(|| version.to_owned())
}

// `2.20.0`, `2.39.3 (Apple Git-145)`, `2.45.1.windows.1`: the first two numbers.
fn parse_version(version: &str) -> Option<(u32, u32)> {
    let mut numbers = version.split(['.', ' ']);
    let major = numbers.next()?.parse().ok()?;
    let minor = numbers.next()?.parse().ok()?;
    Some((major, minor))
}

// Why a Git call that should have succeeded failed.
fn failed(output: &Output) -> Unreadable {
    if let Some(version) = too_old() {
        return Unreadable(format!(
            "roostr needs Git {}.{} or later; the git on the PATH is {version}",
            OLDEST.0, OLDEST.1
        ));
    }
    let said = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if said.is_empty() {
        Unreadable(format!("git failed and said nothing ({})", output.status))
    } else {
        Unreadable(said)
    }
}

#[derive(Debug)]
pub(crate) struct Commit {
    id: String,
    timestamp: i64,
    subject: String,
    /// The files in which it differs from its first parent; every file of a root commit.
    first_parent: HashSet<Vec<u8>>,
    /// The files in which it differs from every one of its parents: what it touched.
    touched: Vec<Vec<u8>>,
}

/// The commits reachable from `HEAD`, from one walk of the history: children before their
/// parents, and which commits touched which files.
pub(crate) struct History {
    commits: Vec<Commit>,
    parents: Vec<Vec<usize>>,
    index: HashMap<String, usize>,
    by_file: HashMap<Vec<u8>, Vec<usize>>,
}

/// A commit as `commits_after` gives it.
pub(crate) struct Committed {
    pub(crate) id: String,
    /// The committer date, in seconds.
    pub(crate) timestamp: i64,
    pub(crate) committer: String,
    /// The files in the project folder it touched.
    pub(crate) touched: Vec<Vec<u8>>,
}

/// `entrypoints.tsv` and the records at a commit.
#[derive(Default)]
pub(crate) struct RecordsAt {
    pub(crate) inventory: Option<Inventory>,
    pub(crate) processes: Vec<(String, Parsed<Process>)>,
}

/// The Git work tree the project is in. Paths Git prints are relative to the top of the
/// work tree; `prefix` is the project folder's path from there.
pub(crate) struct Git {
    root: PathBuf,
    prefix: Vec<u8>,
    head: Option<String>,
    /// In a shallow clone, the commits at its edge, kept without their parents.
    shallow: Vec<String>,
    /// While a merge is in progress, the commits being merged (`MERGE_HEAD`).
    merging: Vec<String>,
    /// The Git folder of this work tree: `.git`, or `.git/worktrees/<name>` in a linked one.
    git_dir: PathBuf,
    /// The Git folder every work tree of the repository shares: `.git`.
    common_dir: PathBuf,
}

/// Why a shallow clone's history cannot be read as the history.
pub(crate) const SHALLOW: &str = "the history is shallow; git fetch --unshallow fetches all of it";

pub(crate) struct EntrypointChanges {
    pub(crate) added: Vec<String>,
    pub(crate) removed: Vec<String>,
}

pub(crate) struct Changes {
    pub(crate) first_day: String,
    pub(crate) entrypoints: Option<EntrypointChanges>,
    pub(crate) processes: Vec<(String, Vec<String>)>,
}

impl Changes {
    pub(crate) fn summary(&self) -> String {
        let entrypoints = match &self.entrypoints {
            Some(changes) => format!(
                "{} added, {} removed",
                changes.added.len(),
                changes.removed.len()
            ),
            None => "not compared".to_owned(),
        };
        format!(
            "entrypoints: {entrypoints}   processes: {} changed",
            self.processes.len()
        )
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.processes.is_empty()
            && self
                .entrypoints
                .as_ref()
                .is_none_or(|changes| changes.added.is_empty() && changes.removed.is_empty())
    }
}

/// A process whose record may be behind the files it lists.
struct Candidate<'a> {
    id: &'a str,
    record: Vec<u8>,
    /// Each file as the record writes it, and as Git spells it.
    files: Vec<(&'a str, Vec<u8>)>,
}

impl Git {
    /// The work tree `root` is in, or `None` outside one. One Git call: it also finds the
    /// project folder's place in the work tree and the commit `HEAD` names, if any.
    pub(crate) fn open(root: &Path) -> Result<Option<Self>, Unreadable> {
        let mut git = Self {
            root: root.to_owned(),
            prefix: Vec::new(),
            head: None,
            shallow: Vec::new(),
            merging: Vec::new(),
            git_dir: PathBuf::new(),
            common_dir: PathBuf::new(),
        };
        let output = match git
            .process()
            .args([
                "rev-parse",
                "--is-inside-work-tree",
                "--show-prefix",
                "--absolute-git-dir",
                // Relative to the folder Git runs in, or absolute.
                "--git-common-dir",
                // Where Git lists a shallow clone's edge, read here without another call.
                "--git-path",
                "shallow",
                "--git-path",
                "MERGE_HEAD",
                "--verify",
                "--quiet",
                "HEAD^{commit}",
            ])
            .output()
        {
            Ok(output) => output,
            // Without git, a project with no `.git` in or above its folder is outside a
            // repository, as Git would have said.
            Err(error)
                if error.kind() == io::ErrorKind::NotFound
                    && !root.ancestors().any(|folder| folder.join(".git").exists()) =>
            {
                return Ok(None);
            }
            Err(error) => return Err(not_started(&error)),
        };
        // Exit 1 with the first two answers: a repository with no commits yet.
        if !matches!(output.status.code(), Some(0 | 1)) {
            if String::from_utf8_lossy(&output.stderr).contains("not a git repository") {
                return Ok(None);
            }
            return Err(failed(&output));
        }
        let mut lines = output.stdout.split(|byte| *byte == b'\n');
        if lines.next() != Some(b"true") {
            return Ok(None);
        }
        git.prefix = lines
            .next()
            .ok_or("Unexpected Git rev-parse output")?
            .to_vec();
        let git_dir = lines.next().ok_or("Unexpected Git rev-parse output")?;
        git.git_dir = PathBuf::from(OsStr::from_bytes(git_dir));
        let common_dir = lines.next().ok_or("Unexpected Git rev-parse output")?;
        git.common_dir = root.join(OsStr::from_bytes(common_dir));
        let shallow = lines.next().ok_or("Unexpected Git rev-parse output")?;
        git.shallow = read_commits(&root.join(OsStr::from_bytes(shallow)))?;
        let merging = lines.next().ok_or("Unexpected Git rev-parse output")?;
        git.merging = read_commits(&root.join(OsStr::from_bytes(merging)))?;
        if output.status.success() {
            git.head = Some(object_id(lines.next())?);
        }
        Ok(Some(git))
    }

    /// The commit `HEAD` names; `None` before the first commit.
    pub(crate) fn head(&self) -> Option<&str> {
        self.head.as_deref()
    }

    /// A `git` command run in the project folder, with the options every call of roostr's
    /// has.
    pub(crate) fn process(&self) -> Command {
        let mut command = Command::new("git");
        command.current_dir(&self.root).args([
            "--no-pager",
            "--no-optional-locks",
            "--literal-pathspecs",
            // Empty is "no fsmonitor" both to a Git that reads a hook path here and to
            // one that reads a boolean.
            "-c",
            "core.fsmonitor=",
            "-c",
            "color.ui=false",
        ]);
        // Git reads must not refresh the index, run a repository helper, or fetch
        // missing objects from a partial clone.
        command
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_NO_LAZY_FETCH", "1")
            .env("LC_ALL", "C");
        command
    }

    fn command(&self, args: &[&str]) -> Result<Output, Unreadable> {
        self.process()
            .args(args)
            .output()
            .map_err(|error| not_started(&error))
    }

    fn checked(&self, args: &[&str]) -> Result<Vec<u8>, Unreadable> {
        let output = self.command(args)?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(failed(&output))
        }
    }

    // The path of a repository file as Git spells it. Only plain relative paths map to
    // one; a path that climbs out or starts at the root names nothing in the repository.
    pub(crate) fn path(&self, relative: &str) -> Result<Vec<u8>, String> {
        let mut path = self.prefix.clone();
        for component in Path::new(relative).components() {
            match component {
                Component::CurDir => continue,
                Component::Normal(name) => {
                    if !path.is_empty() && path.last() != Some(&b'/') {
                        path.push(b'/');
                    }
                    path.extend_from_slice(name.as_encoded_bytes());
                }
                _ => return Err(format!("{relative:?} is not a path inside the repository")),
            }
        }
        Ok(path)
    }

    /// A Git path as a path from the project folder, when it is inside the project.
    pub(crate) fn in_project<'a>(&self, path: &'a [u8]) -> Option<&'a [u8]> {
        path.strip_prefix(self.prefix.as_slice())
    }

    /// The commit `revision` names, or `None` when Git cannot resolve it to one.
    pub(crate) fn resolve(&self, revision: &str) -> Result<Option<String>, Unreadable> {
        // Git would read a leading dash as an option; no commit is named that way.
        if revision.starts_with('-') {
            return Ok(None);
        }
        let output = self.command(&[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{revision}^{{commit}}"),
        ])?;
        match output.status.code() {
            Some(0) => Ok(Some(object_id(
                output.stdout.split(|byte| *byte == b'\n').next(),
            )?)),
            Some(1) => Ok(None),
            _ => Err(failed(&output)),
        }
    }

    /// The best common ancestors of two commits; none when they share no commit.
    /// The merge bases of `base` and `HEAD`; during a merge, of `base` and the merge commit
    /// it will make.
    pub(crate) fn merge_bases(&self, base: &str, head: &str) -> Result<Vec<String>, Unreadable> {
        let mut args = vec!["merge-base", "--all", base, head];
        args.extend(self.merging.iter().map(String::as_str));
        let output = self.command(&args)?;
        match output.status.code() {
            Some(0) => output
                .stdout
                .split(|byte| *byte == b'\n')
                .filter(|line| !line.is_empty())
                .map(|line| object_id(Some(line)).map_err(Unreadable))
                .collect(),
            Some(1) if output.stdout.is_empty() => Ok(Vec::new()),
            _ => Err(failed(&output)),
        }
    }

    /// The files in the project folder that differ from `HEAD` in the index or the work
    /// tree, untracked files included and ignored files left out.
    /// What `git status` says of the files in the project folder: staged and not.
    pub(crate) fn status(&self) -> Result<Status, Unreadable> {
        Ok(parse_status(&self.checked(&[
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignore-submodules=none",
            "--no-renames",
            "--",
            ".",
        ])?)?)
    }

    /// The files in the project folder with uncommitted changes.
    pub(crate) fn uncommitted(&self) -> Result<HashSet<Vec<u8>>, Unreadable> {
        self.changed(&self.status()?)
    }

    /// The files `status` shows changed: staged, edited, deleted or not tracked. While a
    /// merge is in progress, only those the merge commit will change: where the working tree,
    /// or what is staged, differs from `HEAD` and from every commit being merged. A file taken
    /// whole from one side is not a change of the merge.
    pub(crate) fn changed(&self, status: &Status) -> Result<HashSet<Vec<u8>>, Unreadable> {
        if self.merging.is_empty() || status.entries.is_empty() {
            return Ok(status
                .entries
                .iter()
                .map(|entry| entry.path.clone())
                .collect());
        }
        let untracked: HashSet<&[u8]> = status
            .entries
            .iter()
            .filter(|entry| entry.untracked())
            .map(|entry| entry.path.as_slice())
            .collect();
        let mut worktree = self.differing(&["HEAD"])?;
        worktree.extend(untracked.iter().map(|path| path.to_vec()));
        // An unmerged file has nothing staged yet; its working tree says what it will be.
        let mut staged: HashSet<Vec<u8>> = status
            .entries
            .iter()
            .filter(|entry| !matches!(entry.index, b' ' | b'?') && !entry.unmerged())
            .map(|entry| entry.path.clone())
            .collect();
        for commit in &self.merging {
            let differs = self.differing(&[commit])?;
            worktree.retain(|path| differs.contains(path) || untracked.contains(path.as_slice()));
            if !staged.is_empty() {
                let differs = self.differing(&["--cached", commit])?;
                staged.retain(|path| differs.contains(path));
            }
        }
        worktree.extend(staged);
        Ok(worktree)
    }

    // The files in the project folder in which the working tree (or with `--cached`, the
    // index) differs from a commit.
    fn differing(&self, against: &[&str]) -> Result<HashSet<Vec<u8>>, Unreadable> {
        let mut args = vec![
            "-c",
            "diff.relative=false",
            "diff",
            "--name-only",
            "-z",
            "--no-renames",
            "--no-ext-diff",
            "--no-textconv",
            "--ignore-submodules=none",
        ];
        args.extend(against);
        args.extend(["--", "."]);
        Ok(self
            .checked(&args)?
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .map(<[u8]>::to_vec)
            .collect())
    }

    /// Whether the repository is a shallow clone.
    pub(crate) fn is_shallow(&self) -> bool {
        !self.shallow.is_empty()
    }

    pub(crate) fn is_merging(&self) -> bool {
        !self.merging.is_empty()
    }

    /// The Git folder of this work tree.
    pub(crate) fn git_dir(&self) -> &Path {
        &self.git_dir
    }

    /// The Git folder every work tree of the repository shares.
    pub(crate) fn common_dir(&self) -> &Path {
        &self.common_dir
    }

    /// The top of the work tree, which the paths Git prints are relative to: reached from the
    /// project folder with `..`, as the system resolves it, so that a folder reached through
    /// a link still leads there.
    pub(crate) fn top(&self) -> PathBuf {
        let mut top = self.root.clone();
        for _ in self.prefix.iter().filter(|byte| **byte == b'/') {
            top.push("..");
        }
        top
    }

    /// The email of the committer Git would write into a commit made now; `None` when Git
    /// has no identity to commit with.
    pub(crate) fn committer_email(&self) -> Option<String> {
        let output = self.command(&["var", "GIT_COMMITTER_IDENT"]).ok()?;
        if !output.status.success() {
            return None;
        }
        // `Name <email> 1700000000 +0000`
        let ident = String::from_utf8(output.stdout).ok()?;
        let (_, rest) = ident.rsplit_once('<')?;
        let (email, _) = rest.split_once('>')?;
        Some(email.to_owned())
    }

    /// The commits `HEAD` reaches and `start` does not, while a merge is in progress those
    /// the commits being merged reach as well, leaving out each that is patch-identical to
    /// one `start` reaches: a replay of it. Without `start`, every commit. Each with what it
    /// touched, its committer's email and the committer date.
    pub(crate) fn commits_after(&self, start: Option<&str>) -> Result<Vec<Committed>, Unreadable> {
        let Some(head) = self.head() else {
            return Ok(Vec::new());
        };
        let tips: Vec<&str> = std::iter::once(head)
            .chain(self.merging.iter().map(String::as_str))
            .collect();
        let mut args: Vec<String> = [
            "-c",
            "log.diffMerges=separate",
            "-c",
            "diff.relative=false",
            "log",
            "-m",
            "--root",
            "--format=%x01%H %T %ct %P%x00%ce",
            "-z",
            "--name-only",
            "--no-renames",
            "--no-ext-diff",
            "--no-textconv",
            "--no-notes",
            "--no-decorate",
            "--no-show-signature",
        ]
        .iter()
        .map(|arg| (*arg).to_owned())
        .collect();
        match start {
            Some(start) => {
                args.extend(["--cherry-pick".to_owned(), "--right-only".to_owned()]);
                args.extend(tips.iter().map(|tip| format!("{start}...{tip}")));
            }
            None => args.extend(tips.iter().map(|tip| (*tip).to_owned())),
        }
        args.push("--".to_owned());
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let walk = self.checked(&args)?;
        Ok(commits(parse_walk(&walk)?)
            .into_iter()
            .map(|(commit, _)| Committed {
                id: commit.id,
                timestamp: commit.timestamp,
                committer: commit.subject,
                touched: commit
                    .touched
                    .into_iter()
                    .filter(|path| self.in_project(path).is_some())
                    .collect(),
            })
            .collect())
    }

    /// The commits being merged, while a merge is in progress.
    pub(crate) fn merging(&self) -> &[String] {
        &self.merging
    }

    /// One walk of the whole history reachable from `HEAD`, listing for every commit the
    /// files it differs in from each of its parents (`-m`), from which both what a merge
    /// changed against its first parent and what it touched are worked out.
    /// While a merge is in progress, the commits being merged come into the walk as well: the
    /// history as it will be once the merge is committed.
    pub(crate) fn history(&self) -> Result<History, Unreadable> {
        self.walk(&self.merging)
    }

    /// The history reachable from `HEAD` alone, also during a merge.
    pub(crate) fn head_history(&self) -> Result<History, Unreadable> {
        self.walk(&[])
    }

    fn walk(&self, merging: &[String]) -> Result<History, Unreadable> {
        let Some(head) = self.head() else {
            return Ok(History::new(Vec::new()));
        };
        let mut args = vec![
            // On Git 2.31 and later `-m` follows log.diffMerges.
            "-c",
            "log.diffMerges=separate",
            // Paths from the top of the work tree, whatever folder Git runs in.
            "-c",
            "diff.relative=false",
            "log",
            "-m",
            "--topo-order",
            "--root",
            "--format=%x01%H %T %ct %P%x00%s",
            "--encoding=UTF-8",
            "-z",
            "--name-only",
            "--no-renames",
            "--no-ext-diff",
            "--no-textconv",
            "--no-notes",
            "--no-decorate",
            "--no-show-signature",
            head,
        ];
        args.extend(merging.iter().map(String::as_str));
        args.push("--");
        let walk = self.checked(&args)?;
        Ok(History::new(commits(parse_walk(&walk)?)))
    }

    /// Whether the history is all there: not in a shallow clone, whose oldest commits look as
    /// if they added every file at once.
    pub(crate) fn whole_history(&self) -> Result<(), Unreadable> {
        if self.shallow.is_empty() {
            Ok(())
        } else {
            Err(SHALLOW.into())
        }
    }

    /// Whether all the history after the base, given as its merge bases with `HEAD`, is in
    /// the clone: each commit at a shallow clone's edge that `HEAD` reaches is reached by
    /// the base too.
    pub(crate) fn whole_after(&self, history: &History, merge_bases: &[String]) -> bool {
        if self.shallow.is_empty() {
            return true;
        }
        let reachable = history.ancestors(
            merge_bases
                .iter()
                .filter_map(|id| history.index.get(id).copied()),
        );
        self.shallow.iter().all(|edge| {
            history
                .index
                .get(edge)
                .is_none_or(|&index| reachable[index])
        })
    }

    /// The files in the project folder touched by a commit that is reachable from `HEAD`
    /// and not from the base, given as its merge bases with `HEAD`.
    pub(crate) fn touched_after(
        &self,
        history: &History,
        merge_bases: &[String],
    ) -> HashSet<Vec<u8>> {
        let reachable = history.ancestors(
            merge_bases
                .iter()
                .filter_map(|id| history.index.get(id).copied()),
        );
        history
            .commits
            .iter()
            .zip(reachable)
            .filter(|(_, reachable)| !reachable)
            .flat_map(|(commit, _)| commit.touched.iter())
            .filter(|path| self.in_project(path).is_some())
            .cloned()
            .collect()
    }

    /// For each readable record, the listed files that are newer than the record or have
    /// uncommitted changes, in the order of its `files`. With `only`, only those files are
    /// considered. A record with uncommitted changes counts as updated; one that has never
    /// been committed is skipped. `history` is walked here if it is needed and not yet.
    pub(crate) fn attention(
        &self,
        repository: &Repository,
        uncommitted: &HashSet<Vec<u8>>,
        only: Option<&HashSet<Vec<u8>>>,
        history: &mut Option<History>,
    ) -> Result<Vec<(String, Vec<String>)>, Unreadable> {
        let Some(head) = self.head() else {
            return Ok(Vec::new());
        };
        let mut candidates = Vec::new();
        for record in &repository.processes {
            let Some(process) = record.parsed.readable() else {
                continue;
            };
            let record_path = self.path(&record.file)?;
            if uncommitted.contains(&record_path) {
                continue;
            }
            let mut files: Vec<(&str, Vec<u8>)> = Vec::new();
            for item in &process.files {
                let Ok(path) = self.path(&item.value.path) else {
                    continue;
                };
                if only.is_none_or(|only| only.contains(&path))
                    && !files.iter().any(|(_, seen)| *seen == path)
                {
                    files.push((&item.value.path, path));
                }
            }
            if !files.is_empty() {
                candidates.push(Candidate {
                    id: &record.id,
                    record: record_path,
                    files,
                });
            }
        }
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        let all_uncommitted = candidates.iter().all(|candidate| {
            candidate
                .files
                .iter()
                .all(|(_, path)| uncommitted.contains(path))
        });
        if all_uncommitted && self.merging.is_empty() {
            // An uncommitted change is newer than every commit, so the history is not
            // needed: only whether the record was ever committed. Unchanged and present,
            // it is in HEAD unless Git ignores it.
            let committed = self.in_tree(head, candidates.iter().map(|c| &c.record))?;
            return Ok(candidates
                .into_iter()
                .filter(|candidate| committed.contains(&candidate.record))
                .map(|candidate| {
                    let files = candidate.files.iter().map(|(file, _)| (*file).to_owned());
                    (candidate.id.to_owned(), files.collect())
                })
                .collect());
        }
        if history.is_none() {
            *history = Some(self.history()?);
        }
        let history = history.as_ref().expect("the history was walked just above");
        let mut attention = Vec::new();
        for candidate in candidates {
            let Some(updates) = history.by_file.get(&candidate.record) else {
                continue;
            };
            // A record touched on both sides of a merge has seen what either side built on.
            let seen = history.ancestors(updates.iter().copied());
            let files: Vec<String> = candidate
                .files
                .into_iter()
                .filter(|(_, path)| {
                    uncommitted.contains(path)
                        || history
                            .by_file
                            .get(path)
                            .is_some_and(|commits| commits.iter().any(|&commit| !seen[commit]))
                })
                .map(|(file, _)| file.to_owned())
                .collect();
            if !files.is_empty() {
                attention.push((candidate.id.to_owned(), files));
            }
        }
        Ok(attention)
    }

    // Which of the paths are files in the commit's tree.
    fn in_tree<'a>(
        &self,
        commit: &str,
        paths: impl Iterator<Item = &'a Vec<u8>>,
    ) -> Result<HashSet<Vec<u8>>, Unreadable> {
        let output = self
            .process()
            .args(["ls-tree", "-z", "--full-tree", "--name-only", commit, "--"])
            .args(paths.map(|path| OsString::from_vec(path.clone())))
            .output()
            .map_err(|error| not_started(&error))?;
        if !output.status.success() {
            return Err(failed(&output));
        }
        Ok(output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .map(<[u8]>::to_vec)
            .collect())
    }

    /// `entrypoints.tsv` as it was in `commit`, when it was there and well formed.
    /// `entrypoints.tsv` and the process records as `commit` holds them: the inventory when it is
    /// there and has no problem, and each record `<id>.yml` directly in `.roostr/processes/`,
    /// by its id.
    pub(crate) fn records_at(&self, commit: &str) -> Result<RecordsAt, Unreadable> {
        let inventory_path = self.path(ENTRYPOINTS_FILE)?;
        let folder = self.path(PROCESSES_FOLDER)?;
        let tree = self
            .process()
            .args(["ls-tree", "-r", "-z", "--full-tree", commit, "--"])
            .arg(OsString::from_vec(inventory_path.clone()))
            .arg(OsString::from_vec(folder.clone()))
            .output()
            .map_err(|error| not_started(&error))?;
        if !tree.status.success() {
            return Err(failed(&tree));
        }
        let mut wanted: Vec<(Option<String>, String)> = Vec::new();
        for entry in tree
            .stdout
            .split(|byte| *byte == 0)
            .filter(|entry| !entry.is_empty())
        {
            // `<mode> <type> <object>\t<path>`
            let Some(separator) = entry.iter().position(|byte| *byte == b'\t') else {
                return Err(Unreadable(format!(
                    "Unexpected Git ls-tree output: {:?}",
                    String::from_utf8_lossy(entry)
                )));
            };
            let fields: Vec<_> = entry[..separator].split(|byte| *byte == b' ').collect();
            if fields.len() != 3 || fields[1] != b"blob" {
                continue;
            }
            let path = &entry[separator + 1..];
            let id = if path == inventory_path.as_slice() {
                None
            } else {
                let Some(name) = path
                    .strip_prefix(folder.as_slice())
                    .and_then(|rest| rest.strip_prefix(b"/"))
                    .filter(|name| !name.contains(&b'/') && !name.starts_with(b"."))
                    .and_then(|name| name.strip_suffix(b".yml"))
                    .and_then(|id| std::str::from_utf8(id).ok())
                else {
                    continue;
                };
                Some(name.to_owned())
            };
            let object = std::str::from_utf8(fields[2]).map_err(|_| "Invalid Git object id")?;
            wanted.push((id, object.to_owned()));
        }
        let contents = self.blobs(wanted.iter().map(|(_, object)| object.as_str()))?;
        let mut records = RecordsAt::default();
        for ((id, _), bytes) in wanted.into_iter().zip(contents) {
            let text = String::from_utf8_lossy(&bytes);
            match id {
                None => {
                    let inventory = Inventory::parse(&text);
                    records.inventory = inventory.problems.is_empty().then_some(inventory);
                }
                Some(id) => records.processes.push((id, crate::parse::process(&text))),
            }
        }
        Ok(records)
    }

    // The contents of Git objects, in one `cat-file --batch`.
    fn blobs<'a>(
        &self,
        objects: impl Iterator<Item = &'a str>,
    ) -> Result<Vec<Vec<u8>>, Unreadable> {
        let input: String = objects.map(|object| format!("{object}\n")).collect();
        if input.is_empty() {
            return Ok(Vec::new());
        }
        let mut child = self
            .process()
            .args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| not_started(&error))?;
        // Written beside the reading, so that neither side waits for the other.
        let writer = child
            .stdin
            .take()
            .map(|mut stdin| std::thread::spawn(move || stdin.write_all(input.as_bytes())));
        let output = child
            .wait_with_output()
            .map_err(|error| not_started(&error))?;
        if let Some(writer) = writer {
            writer
                .join()
                .map_err(|_| "Git cat-file input could not be written")?
                .map_err(|error| Unreadable(format!("Git cat-file input: {error}")))?;
        }
        if !output.status.success() {
            return Err(failed(&output));
        }
        let mut rest = output.stdout.as_slice();
        let mut contents = Vec::new();
        while !rest.is_empty() {
            let malformed = || Unreadable("Unexpected Git cat-file output".into());
            let end = rest
                .iter()
                .position(|byte| *byte == b'\n')
                .ok_or_else(malformed)?;
            let header = std::str::from_utf8(&rest[..end]).map_err(|_| malformed())?;
            let size: usize = header
                .rsplit(' ')
                .next()
                .and_then(|size| size.parse().ok())
                .ok_or_else(malformed)?;
            let body = rest.get(end + 1..end + 1 + size).ok_or_else(malformed)?;
            contents.push(body.to_vec());
            rest = rest.get(end + 2 + size..).unwrap_or_default();
        }
        Ok(contents)
    }

    pub(crate) fn inventory_at(&self, commit: &str) -> Result<Option<Inventory>, Unreadable> {
        let path = OsString::from_vec(self.path(ENTRYPOINTS_FILE)?);
        let tree = self
            .process()
            .args(["ls-tree", "-z", "--full-tree", commit, "--"])
            .arg(path)
            .output()
            .map_err(|error| not_started(&error))?;
        if !tree.status.success() {
            return Err(failed(&tree));
        }
        if tree.stdout.is_empty() {
            return Ok(None);
        }
        // ls-tree -z writes `<mode> <type> <object>\t<path>\0`; without the tab it is not
        // an entry, and taking the whole line as the header would misread it silently.
        let Some(separator) = tree.stdout.iter().position(|byte| *byte == b'\t') else {
            return Err(Unreadable(format!(
                "Unexpected Git ls-tree output: {:?}",
                String::from_utf8_lossy(&tree.stdout)
            )));
        };
        let header = &tree.stdout[..separator];
        let fields: Vec<_> = header.split(|byte| *byte == b' ').collect();
        if fields.len() != 3 || fields[1] != b"blob" {
            return Ok(None);
        }
        let id = std::str::from_utf8(fields[2]).map_err(|_| "Invalid Git object id")?;
        let bytes = self.checked(&["cat-file", "blob", id])?;
        let text = String::from_utf8(bytes)
            .map_err(|_| format!("{ENTRYPOINTS_FILE} in Git is not UTF-8"))?;
        let inventory = Inventory::parse(&text);
        Ok(inventory.problems.is_empty().then_some(inventory))
    }

    pub(crate) fn changes(
        &self,
        history: &History,
        repository: &Repository,
        period: &Period,
    ) -> Result<Changes, Unreadable> {
        let commits = &history.commits;
        // Do not stop a history walk at an old timestamp: descendants and branch
        // commits can have dates that run backwards.
        let baseline = commits
            .iter()
            .enumerate()
            .filter(|(_, commit)| commit.timestamp < period.start)
            // Equal dates keep the first (newest in history) snapshot.
            .max_by_key(|(index, commit)| (commit.timestamp, std::cmp::Reverse(*index)))
            .map(|(_, commit)| commit);
        let entrypoints = match (baseline, repository.inventory.as_ref()) {
            (Some(baseline), Some(now)) if now.problems.is_empty() => {
                self.inventory_at(&baseline.id)?.map(|then| {
                    let then = then.references();
                    let now = now.references();
                    EntrypointChanges {
                        added: now.difference(&then).cloned().collect(),
                        removed: then.difference(&now).cloned().collect(),
                    }
                })
            }
            _ => None,
        };
        let mut recent: Vec<_> = commits
            .iter()
            .filter(|commit| period.contains(commit.timestamp))
            .collect();
        // Stable sorting preserves descendant-before-ancestor order for equal dates.
        recent.sort_by_key(|commit| std::cmp::Reverse(commit.timestamp));
        let mut processes = Vec::new();
        for record in &repository.processes {
            let Some(process) = record.parsed.readable() else {
                continue;
            };
            let paths: HashSet<_> = std::iter::once(record.file.as_str())
                .chain(process.files.iter().map(|item| item.value.path.as_str()))
                .filter_map(|path| self.path(path).ok())
                .collect();
            let subjects: Vec<_> = recent
                .iter()
                .filter(|commit| !commit.first_parent.is_disjoint(&paths))
                .map(|commit| commit.subject.clone())
                .collect();
            if !subjects.is_empty() {
                processes.push((record.id.clone(), subjects));
            }
        }
        Ok(Changes {
            first_day: period.first_day.to_string(),
            entrypoints,
            processes,
        })
    }
}

impl History {
    fn new(commits: Vec<(Commit, Vec<String>)>) -> Self {
        let index: HashMap<String, usize> = commits
            .iter()
            .enumerate()
            .map(|(position, (commit, _))| (commit.id.clone(), position))
            .collect();
        let mut parents = Vec::with_capacity(commits.len());
        let mut by_file: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
        let commits = commits
            .into_iter()
            .enumerate()
            .map(|(position, (commit, parent_ids))| {
                // A parent outside the walk lies past a shallow clone's boundary.
                parents.push(
                    parent_ids
                        .iter()
                        .filter_map(|parent| index.get(parent).copied())
                        .collect(),
                );
                for file in &commit.touched {
                    by_file.entry(file.clone()).or_default().push(position);
                }
                commit
            })
            .collect();
        Self {
            commits,
            parents,
            index,
            by_file,
        }
    }

    /// Marks the given commits and every commit they reach.
    fn ancestors(&self, starts: impl IntoIterator<Item = usize>) -> Vec<bool> {
        let mut seen = vec![false; self.commits.len()];
        let mut pending: Vec<usize> = starts.into_iter().collect();
        while let Some(position) = pending.pop() {
            if !std::mem::replace(&mut seen[position], true) {
                pending.extend(&self.parents[position]);
            }
        }
        seen
    }
}

/// A commit as the walk prints it: once per parent it differs from, each time with the
/// files of that difference, in the order of its parents. A commit that differs from no
/// parent is printed once, with no files.
#[derive(Debug, PartialEq)]
struct Printed {
    id: String,
    tree: String,
    timestamp: i64,
    parents: Vec<String>,
    subject: String,
    lists: Vec<Vec<Vec<u8>>>,
}

// The header `--format=%x01%H %T %ct %P` writes: the marker, ids and the date.
fn parse_header(field: &[u8]) -> Option<(String, String, i64, Vec<String>)> {
    let text = std::str::from_utf8(field.strip_prefix(b"\x01")?).ok()?;
    let mut words = text.split(' ').filter(|word| !word.is_empty());
    let id = words.next()?;
    let tree = words.next()?;
    let timestamp = words.next()?.parse().ok()?;
    let parents: Vec<&str> = words.collect();
    let hex = |id: &&str| id.bytes().all(|byte| byte.is_ascii_hexdigit());
    (hex(&id) && hex(&tree) && parents.iter().all(hex)).then(|| {
        (
            id.to_owned(),
            tree.to_owned(),
            timestamp,
            parents.into_iter().map(str::to_owned).collect(),
        )
    })
}

// `log -z --name-only` output: the header, the subject, then the files, each its own
// field; the first file has a newline in front.
fn parse_walk(bytes: &[u8]) -> Result<Vec<Printed>, String> {
    let header = |field: &[u8]| parse_header(field.strip_prefix(b"\n").unwrap_or(field));
    let mut printed: Vec<Printed> = Vec::new();
    let mut fields = bytes.split(|byte| *byte == 0).peekable();
    while let Some(field) = fields.next() {
        if field.is_empty() || field == b"\n" {
            continue;
        }
        let (id, tree, timestamp, parents) =
            header(field).ok_or("Unexpected Git log output: expected a commit")?;
        let subject = fields.next().ok_or("Incomplete Git log output")?;
        let mut files = Vec::new();
        let mut first = true;
        while let Some(field) = fields.next_if(|field| header(field).is_none()) {
            let file = if std::mem::take(&mut first) {
                field.strip_prefix(b"\n").unwrap_or(field)
            } else {
                field
            };
            if !file.is_empty() {
                files.push(file.to_vec());
            }
        }
        match printed.last_mut() {
            Some(last) if last.id == id => last.lists.push(files),
            _ => printed.push(Printed {
                id,
                tree,
                timestamp,
                parents,
                subject: String::from_utf8_lossy(subject).into_owned(),
                lists: vec![files],
            }),
        }
    }
    Ok(printed)
}

// Works out, for each commit, what it changed against its first parent and what it
// touched. The lists of a merge follow its parents in order, leaving out a parent it does
// not differ from; that parent has the merge's own tree.
fn commits(printed: Vec<Printed>) -> Vec<(Commit, Vec<String>)> {
    let trees: HashMap<String, String> = printed
        .iter()
        .map(|commit| (commit.id.clone(), commit.tree.clone()))
        .collect();
    printed
        .into_iter()
        .map(|printed| {
            let mut lists = printed.lists.into_iter();
            let (first_parent, touched): (HashSet<_>, Vec<_>) = if printed.parents.len() < 2 {
                let files: Vec<_> = lists.flatten().collect();
                (files.iter().cloned().collect(), files)
            } else {
                let differs_from_first =
                    printed.parents.first().and_then(|id| trees.get(id)) != Some(&printed.tree);
                let complete = lists.len() == printed.parents.len();
                let first: Vec<_> = lists.next().unwrap_or_default();
                let rest: Vec<HashSet<_>> = lists.map(|list| list.into_iter().collect()).collect();
                let touched = if complete {
                    first
                        .iter()
                        .filter(|file| rest.iter().all(|list| list.contains(*file)))
                        .cloned()
                        .collect()
                } else {
                    Vec::new()
                };
                let first_parent = if differs_from_first {
                    first.into_iter().collect()
                } else {
                    HashSet::new()
                };
                (first_parent, touched)
            };
            (
                Commit {
                    id: printed.id,
                    timestamp: printed.timestamp,
                    subject: printed.subject,
                    first_parent,
                    touched,
                },
                printed.parents,
            )
        })
        .collect()
}

fn object_id(line: Option<&[u8]>) -> Result<String, String> {
    line.filter(|id| !id.is_empty() && id.iter().all(u8::is_ascii_hexdigit))
        .and_then(|id| String::from_utf8(id.to_vec()).ok())
        .ok_or_else(|| "Unexpected Git output: expected a commit id".to_owned())
}

/// What `git status --porcelain=v1` says: for each file, its state in the index against `HEAD`
/// and in the working tree against the index.
pub(crate) struct Status {
    entries: Vec<StatusEntry>,
}

struct StatusEntry {
    index: u8,
    worktree: u8,
    path: Vec<u8>,
}

impl StatusEntry {
    fn untracked(&self) -> bool {
        self.index == b'?'
    }

    fn unmerged(&self) -> bool {
        matches!(
            [self.index, self.worktree],
            [b'U', _] | [_, b'U'] | [b'D', b'D'] | [b'A', b'A']
        )
    }
}

impl Status {
    /// Every file the status lists, as Git spells it from the top of the work tree.
    pub(crate) fn paths(&self) -> impl Iterator<Item = &[u8]> {
        self.entries.iter().map(|entry| entry.path.as_slice())
    }

    /// The files whose working-tree state is not what is staged: an edit or a deletion not
    /// staged, a file Git does not track, a conflict not resolved.
    pub(crate) fn not_staged(&self) -> impl Iterator<Item = &[u8]> {
        self.entries
            .iter()
            .filter(|entry| entry.worktree != b' ' || entry.unmerged())
            .map(|entry| entry.path.as_slice())
    }
}

pub(crate) fn parse_status(bytes: &[u8]) -> Result<Status, String> {
    let mut fields = bytes.split(|byte| *byte == 0);
    let mut entries = Vec::new();
    while let Some(field) = fields.next().filter(|field| !field.is_empty()) {
        if field.len() < 4 || field[2] != b' ' {
            return Err("Invalid Git status output".into());
        }
        let (index, worktree) = (field[0], field[1]);
        entries.push(StatusEntry {
            index,
            worktree,
            path: field[3..].to_vec(),
        });
        if matches!(index, b'R' | b'C') || matches!(worktree, b'R' | b'C') {
            let source = fields.next().ok_or("Incomplete Git rename")?;
            entries.push(StatusEntry {
                index,
                worktree,
                path: source.to_vec(),
            });
        }
    }
    Ok(Status { entries })
}

// The commits a file in the Git folder lists, one a line: a shallow clone's edge, the commits
// being merged. None when the file is not there.
fn read_commits(path: &Path) -> Result<Vec<String>, Unreadable> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text.split_whitespace().map(str::to_owned).collect()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(Unreadable(format!(
            "cannot read {}: {error}",
            path.display()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A commit for `walk`: id, tree, parents, and the file list printed for each parent
    /// it differs from.
    type Shown<'a> = (&'a str, &'a str, &'a [&'a str], &'a [&'a [&'a str]]);

    fn walk(entries: &[Shown<'_>]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (id, tree, parents, lists) in entries {
            for files in *lists {
                bytes.extend(
                    format!(
                        "\x01{id} {tree} 1700000000 {}\0subject\0",
                        parents.join(" ")
                    )
                    .bytes(),
                );
                for (position, file) in files.iter().enumerate() {
                    if position == 0 {
                        bytes.push(b'\n');
                    }
                    bytes.extend(file.bytes());
                    bytes.push(0);
                }
            }
        }
        bytes
    }

    fn files(set: impl IntoIterator<Item = Vec<u8>>) -> Vec<String> {
        let mut files: Vec<_> = set
            .into_iter()
            .map(|file| String::from_utf8(file).unwrap())
            .collect();
        files.sort();
        files
    }

    #[test]
    fn a_merge_prints_one_list_per_parent_it_differs_from() {
        // m: a clean merge of b into a; n: merges main into a branch and equals main;
        // o: an "ours" merge that equals both parents; e: an empty commit.
        let bytes = walk(&[
            ("0f", "71", &["0e", "0d"], &[&[]]),
            ("0e", "71", &["0d"], &[&[]]),
            ("0d", "71", &["0c", "0a"], &[&["main.rb"]]),
            ("0a", "71", &["0b", "0c"], &[&["side.rb"], &["main.rb"]]),
            ("0b", "72", &["01"], &[&["main.rb"]]),
            ("0c", "73", &["01"], &[&["side.rb", "\n odd.rb"]]),
            ("01", "74", &[], &[&["main.rb", "side.rb"]]),
        ]);
        let printed = parse_walk(&bytes).unwrap();
        assert_eq!(printed.len(), 7);
        assert_eq!(printed[3].lists.len(), 2);
        let commits: Vec<_> = commits(printed)
            .into_iter()
            .map(|(commit, _)| commit)
            .collect();
        let [o, e, n, m, b, c, root] = &commits[..] else {
            panic!("{commits:?}");
        };
        assert_eq!(files(e.first_parent.clone()), Vec::<String>::new());
        assert!(o.first_parent.is_empty() && o.touched.is_empty());
        // n equals its second parent (main): it changed main.rb against the branch only.
        assert_eq!(files(n.first_parent.clone()), ["main.rb"]);
        assert!(n.touched.is_empty());
        assert_eq!(files(m.first_parent.clone()), ["side.rb"]);
        assert!(m.touched.is_empty());
        assert_eq!(files(b.touched.clone()), ["main.rb"]);
        assert_eq!(files(c.touched.clone()), ["\n odd.rb", "side.rb"]);
        assert_eq!(files(root.touched.clone()), ["main.rb", "side.rb"]);
    }

    #[test]
    fn a_merge_that_differs_from_every_parent_touches_the_files_in_all_its_lists() {
        let bytes = walk(&[
            (
                "0d",
                "75",
                &["0b", "0c"],
                &[&["both.rb", "side.rb"], &["both.rb", "main.rb"]],
            ),
            ("0b", "72", &["01"], &[&["both.rb", "main.rb"]]),
            ("0c", "73", &["01"], &[&["both.rb", "side.rb"]]),
            ("01", "74", &[], &[&["both.rb"]]),
        ]);
        let commits: Vec<_> = commits(parse_walk(&bytes).unwrap())
            .into_iter()
            .map(|(commit, _)| commit)
            .collect();
        assert_eq!(files(commits[0].touched.clone()), ["both.rb"]);
        assert_eq!(
            files(commits[0].first_parent.clone()),
            ["both.rb", "side.rb"]
        );
    }

    #[test]
    fn a_record_touched_on_both_sides_of_a_merge_has_seen_what_either_side_built_on() {
        let bytes = walk(&[
            ("0f", "70", &["0a", "0b"], &[&["record.yml"], &["main.rb"]]),
            ("0a", "71", &["01"], &[&["record.yml", "main.rb"]]),
            ("0b", "72", &["0c"], &[&["record.yml"]]),
            ("0c", "73", &["01"], &[&["branch.rb"]]),
            ("01", "74", &[], &[&["record.yml", "main.rb", "branch.rb"]]),
        ]);
        let history = History::new(commits(parse_walk(&bytes).unwrap()));
        let seen = history.ancestors(history.by_file[b"record.yml".as_slice()].iter().copied());
        for file in [b"main.rb".as_slice(), b"branch.rb"] {
            assert!(history.by_file[file].iter().all(|&commit| seen[commit]));
        }
        // The clean merge itself touched nothing.
        assert!(!seen[0] && history.commits[0].touched.is_empty());
    }
}
