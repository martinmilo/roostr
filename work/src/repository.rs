use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::characters;
use crate::entrypoints::Inventory;
use crate::model::{Finding, Parsed, Place, Problem, Process, Project};
use crate::parse;

pub(crate) const PROJECT_FILE: &str = ".roostr/project.yml";
pub(crate) const ENTRYPOINTS_FILE: &str = ".roostr/entrypoints.tsv";
pub(crate) const PROCESSES_FOLDER: &str = ".roostr/processes";

/// Why a file roostr reads was not read.
pub(crate) enum Unread {
    /// A named pipe, a socket, a device, or a link to one: never opened, since reading it may
    /// wait or run on forever.
    NotRegular,
    Failed(io::Error),
}

impl Unread {
    pub(crate) fn is_missing(&self) -> bool {
        matches!(self, Self::Failed(error) if error.kind() == io::ErrorKind::NotFound)
    }
}

/// What `validate` says of the file.
impl std::fmt::Display for Unread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRegular => write!(f, "not a regular file; roostr reads only regular files"),
            Self::Failed(error) => write!(f, "cannot read: {error}"),
        }
    }
}

/// Whether `path` may be opened: a regular file or a link to one. A folder may be too, and
/// fails to read as one.
fn regular(path: &Path) -> Result<(), Unread> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() || metadata.is_dir() => Ok(()),
        Ok(_) => Err(Unread::NotRegular),
        Err(error) => Err(Unread::Failed(error)),
    }
}

/// The bytes of a file roostr reads, which is opened only when it is a regular file.
pub(crate) fn read_regular(path: &Path) -> Result<Vec<u8>, Unread> {
    regular(path)?;
    fs::read(path).map_err(Unread::Failed)
}

/// The text of a file roostr reads, which is opened only when it is a regular file.
pub(crate) fn read_regular_text(path: &Path) -> Result<String, Unread> {
    regular(path)?;
    fs::read_to_string(path).map_err(Unread::Failed)
}

#[derive(Debug)]
pub(crate) struct ProcessRecord {
    pub(crate) id: String,
    pub(crate) file: String,
    pub(crate) parsed: Parsed<Process>,
}

pub(crate) struct Repository {
    pub(crate) root: PathBuf,
    pub(crate) project: Parsed<Project>,
    pub(crate) processes: Vec<ProcessRecord>,
    pub(crate) inventory: Option<Inventory>,
    /// Files and folders that could not be read at all.
    pub(crate) problems: Vec<Finding>,
    /// What `validate` says of the names of files in `.roostr/processes/`, in path order: files
    /// that look like records and are not read, and records whose names the map cannot print.
    pub(crate) misplaced: Vec<Finding>,
}

fn finding(file: &str, place: Place, message: impl Into<String>) -> Finding {
    Finding {
        file: file.to_owned(),
        problem: Problem::new(place, message),
    }
}

impl Repository {
    pub(crate) fn discover(start: &Path) -> Option<PathBuf> {
        for directory in start.ancestors() {
            if directory.join(PROJECT_FILE).exists() {
                return Some(directory.to_owned());
            }
            // Do not accidentally open an enclosing project's metadata across a repository boundary.
            if directory.join(".git").exists() {
                break;
            }
        }
        None
    }

    pub(crate) fn load(root: PathBuf) -> Self {
        Self::load_with(root, true)
    }

    /// `project.yml` and the records, without `entrypoints.tsv`: what the project map is made
    /// of.
    pub(crate) fn load_records(root: PathBuf) -> Self {
        Self::load_with(root, false)
    }

    fn load_with(root: PathBuf, with_inventory: bool) -> Self {
        let project = read_yaml(&root.join(PROJECT_FILE), parse::project);
        let mut problems = Vec::new();
        let text = with_inventory.then(|| read_regular_text(&root.join(ENTRYPOINTS_FILE)));
        let inventory = match text {
            None => None,
            Some(Ok(text)) => Some(Inventory::parse(&text)),
            Some(Err(unread)) if unread.is_missing() => None,
            Some(Err(unread)) => {
                problems.push(finding(ENTRYPOINTS_FILE, Place::File, unread.to_string()));
                None
            }
        };
        let mut processes = Vec::new();
        let mut misplaced = Vec::new();
        read_processes(
            &root,
            PROCESSES_FOLDER,
            &mut processes,
            &mut misplaced,
            &mut problems,
        );
        processes.sort_by(|a, b| a.id.cmp(&b.id));
        Self {
            root,
            project,
            processes,
            inventory,
            problems,
            misplaced,
        }
    }

    pub(crate) fn readable_processes(&self) -> impl Iterator<Item = (&str, &Process)> {
        self.processes.iter().filter_map(|record| {
            record
                .parsed
                .readable()
                .map(|process| (record.id.as_str(), process))
        })
    }

    /// The process records that are not readable, and `entrypoints.tsv` when it has a
    /// problem, in path order: what the overview lists as needing checking.
    pub(crate) fn unreadable(&self) -> BTreeSet<&str> {
        self.processes
            .iter()
            .filter(|record| record.parsed.readable().is_none())
            .map(|record| record.file.as_str())
            .chain(
                self.inventory
                    .as_ref()
                    .filter(|inventory| !inventory.problems.is_empty())
                    .map(|_| ENTRYPOINTS_FILE),
            )
            .collect()
    }

    fn reading_problems(&self) -> Vec<Finding> {
        let mut findings = self.problems.clone();
        let mut add = |file: &str, problems: &[Problem]| {
            findings.extend(problems.iter().map(|problem| Finding {
                file: file.to_owned(),
                problem: problem.clone(),
            }));
        };
        add(PROJECT_FILE, &self.project.problems);
        for record in &self.processes {
            add(&record.file, &record.parsed.problems);
        }
        if let Some(inventory) = &self.inventory {
            add(ENTRYPOINTS_FILE, &inventory.problems);
        }
        findings
    }

    pub(crate) fn validate(&self) -> Vec<Finding> {
        let mut findings = self.reading_problems();
        findings.extend(self.misplaced.iter().cloned());
        let areas: BTreeSet<&str> = self
            .inventory
            .iter()
            .flat_map(|inventory| &inventory.entries)
            .filter_map(|entry| entry.area.as_deref())
            .collect();
        let mut needs_scan = false;
        for record in &self.processes {
            let Some(process) = &record.parsed.value else {
                continue;
            };
            if let Some(group) = &process.group {
                let near: Vec<_> = areas
                    .iter()
                    .filter(|area| loose(area) == loose(group))
                    .copied()
                    .collect();
                if !areas.contains(group.as_str()) && !near.is_empty() {
                    findings.push(finding(
                        &record.file,
                        Place::Key("group".into()),
                        format!(
                            "{group:?} matches no area; write {}, as the scan writes the area",
                            near.join(" or ")
                        ),
                    ));
                }
            }
            for item in &process.files {
                let file = &item.value;
                let path = Path::new(&file.path);
                let key = Place::Key(item.key.clone());
                if file.path.is_empty()
                    || path.components().any(|component| {
                        matches!(
                            component,
                            Component::ParentDir | Component::RootDir | Component::Prefix(_)
                        )
                    })
                {
                    findings.push(finding(
                        &record.file,
                        key,
                        format!(
                            "{:?} must be a path relative to the project root without '..'",
                            file.path
                        ),
                    ));
                } else if !self.root.join(path).is_file() {
                    findings.push(finding(
                        &record.file,
                        key,
                        format!("file {} does not exist or is not a regular file", file.path),
                    ));
                }
            }
            if let Some(inventory) = &self.inventory {
                for item in &process.entrypoints {
                    let reference = &item.value;
                    if inventory.find(reference).is_none() {
                        findings.push(finding(
                            &record.file,
                            Place::Key(item.key.clone()),
                            format!("entrypoint {reference} is not in entrypoints.tsv"),
                        ));
                    }
                }
            } else if !process.entrypoints.is_empty() {
                needs_scan = true;
            }
        }
        // When the file exists and could not be read, that is the finding about it.
        if needs_scan
            && !self
                .problems
                .iter()
                .any(|finding| finding.file == ENTRYPOINTS_FILE)
        {
            findings.push(finding(
                ENTRYPOINTS_FILE,
                Place::File,
                "does not exist; run roostr scan to check process entrypoints",
            ));
        }
        findings
    }
}

// The records in `folder`, a path from `root`, and the files below it that look like records
// and are not read. Hidden files and folders are skipped: editors keep files that start with
// a dot beside the one open.
fn read_processes(
    root: &Path,
    folder: &str,
    processes: &mut Vec<ProcessRecord>,
    misplaced: &mut Vec<Finding>,
    problems: &mut Vec<Finding>,
) {
    let entries = match fs::read_dir(root.join(folder)) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound && folder == PROCESSES_FOLDER => {
            return;
        }
        Err(error) => {
            problems.push(finding(
                folder,
                Place::File,
                format!("cannot read: {error}"),
            ));
            return;
        }
    };
    let mut names = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => names.push(entry),
            Err(error) => problems.push(finding(
                folder,
                Place::File,
                format!("cannot read directory entry: {error}"),
            )),
        }
    }
    names.sort_by_key(fs::DirEntry::file_name);
    for entry in names {
        let name = entry.file_name();
        if name.as_encoded_bytes().starts_with(b".") {
            continue;
        }
        let Some(name) = name.to_str() else {
            problems.push(finding(
                folder,
                Place::File,
                "process filename is not UTF-8",
            ));
            continue;
        };
        let path = format!("{folder}/{name}");
        let is_folder = entry.file_type().is_ok_and(|kind| kind.is_dir())
            || (entry.file_type().is_ok_and(|kind| kind.is_symlink()) && root.join(&path).is_dir());
        if is_folder {
            read_processes(root, &path, processes, misplaced, problems);
            continue;
        }
        let at_top = folder == PROCESSES_FOLDER;
        if let (true, Some(id)) = (at_top, name.strip_suffix(".yml")) {
            // The map would print another id, one `roostr show` does not know.
            let changed = characters::changed(id);
            if !changed.is_empty() {
                misplaced.push(finding(
                    &path,
                    Place::File,
                    characters::file_name_finding(&changed),
                ));
            }
            processes.push(ProcessRecord {
                id: id.to_owned(),
                file: path.clone(),
                parsed: read_yaml(&root.join(&path), parse::process),
            });
            continue;
        }
        let lower = name.to_ascii_lowercase();
        let Some(stem) = [".yml", ".yaml"]
            .iter()
            .find(|extension| lower.ends_with(*extension))
            .map(|extension| &name[..name.len() - extension.len()])
        else {
            continue;
        };
        let target = format!("{PROCESSES_FOLDER}/{stem}.yml");
        let mut message =
            format!("not read: a record is a file named <id>.yml directly in {PROCESSES_FOLDER}/");
        // By the listing, by exact name: on a file system that ignores case, `Refund.yml`
        // "exists" when only `Refund.YML` is there.
        let taken = fs::read_dir(root.join(PROCESSES_FOLDER)).is_ok_and(|entries| {
            entries
                .filter_map(Result::ok)
                .any(|entry| entry.file_name().to_str() == Some(&format!("{stem}.yml")))
        });
        if !taken {
            let verb = if at_top { "rename" } else { "move" };
            message.push_str(&format!("; {verb} it to {target}"));
        }
        misplaced.push(finding(&path, Place::File, message));
    }
}

fn read_yaml<T>(path: &Path, parse: fn(&str) -> Parsed<T>) -> Parsed<T> {
    match read_regular_text(path) {
        Ok(text) => parse(&text),
        Err(unread) => Parsed::failed(unread.to_string()),
    }
}

// A group and an area compared with case, spaces, `-` and `_` ignored.
fn loose(name: &str) -> String {
    name.chars()
        .filter(|ch| !matches!(ch, ' ' | '-' | '_'))
        .flat_map(char::to_lowercase)
        .collect()
}
