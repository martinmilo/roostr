use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::entrypoints::Inventory;
use crate::model::{Parsed, Process, Project};
use crate::parse;

pub(crate) const PROJECT_FILE: &str = ".roostr/project.yml";
pub(crate) const ENTRYPOINTS_FILE: &str = ".roostr/entrypoints.tsv";

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
    pub(crate) problems: Vec<String>,
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
        let project = read_yaml(&root.join(PROJECT_FILE), parse::project);
        let mut problems = Vec::new();
        let inventory = match fs::read_to_string(root.join(ENTRYPOINTS_FILE)) {
            Ok(text) => Some(Inventory::parse(&text)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => {
                problems.push(format!("{ENTRYPOINTS_FILE}: cannot read: {error}"));
                None
            }
        };
        let mut processes = Vec::new();
        match fs::read_dir(root.join(".roostr/processes")) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            if path.extension().and_then(|extension| extension.to_str())
                                != Some("yml")
                            {
                                continue;
                            }
                            let Some(id) = path.file_stem().and_then(|id| id.to_str()) else {
                                problems.push(
                                    ".roostr/processes: process filename is not UTF-8".into(),
                                );
                                continue;
                            };
                            processes.push(ProcessRecord {
                                id: id.to_owned(),
                                file: format!(".roostr/processes/{id}.yml"),
                                parsed: read_yaml(&path, parse::process),
                            });
                        }
                        Err(error) => problems.push(format!(
                            ".roostr/processes: cannot read directory entry: {error}"
                        )),
                    }
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => problems.push(format!(".roostr/processes: cannot read: {error}")),
        }
        processes.sort_by(|a, b| a.id.cmp(&b.id));
        Self {
            root,
            project,
            processes,
            inventory,
            problems,
        }
    }

    pub(crate) fn readable_processes(&self) -> impl Iterator<Item = (&str, &Process)> {
        self.processes.iter().filter_map(|record| {
            record
                .parsed
                .valid()
                .map(|process| (record.id.as_str(), process))
        })
    }

    pub(crate) fn reading_problems(&self) -> Vec<String> {
        let mut problems = self.problems.clone();
        problems.extend(
            self.project
                .problems
                .iter()
                .map(|problem| format!("{PROJECT_FILE}: {problem}")),
        );
        for record in &self.processes {
            problems.extend(
                record
                    .parsed
                    .problems
                    .iter()
                    .map(|problem| format!("{}: {problem}", record.file)),
            );
        }
        if let Some(inventory) = &self.inventory {
            problems.extend(
                inventory
                    .problems
                    .iter()
                    .map(|problem| format!("{ENTRYPOINTS_FILE}: {problem}")),
            );
        }
        problems
    }

    pub(crate) fn validate(&self) -> Vec<String> {
        let mut problems = self.reading_problems();
        let mut needs_scan = false;
        for record in &self.processes {
            let Some(process) = &record.parsed.value else {
                continue;
            };
            for item in &process.files {
                let file = &item.value;
                let path = Path::new(&file.path);
                if file.path.is_empty()
                    || path.components().any(|component| {
                        matches!(
                            component,
                            Component::ParentDir | Component::RootDir | Component::Prefix(_)
                        )
                    })
                {
                    problems.push(format!(
                        "{}: {}: {:?} must be a path relative to the project root without '..'",
                        record.file, item.key, file.path
                    ));
                } else if !self.root.join(path).is_file() {
                    problems.push(format!(
                        "{}: {}: file {} does not exist or is not a regular file",
                        record.file, item.key, file.path
                    ));
                }
            }
            if let Some(inventory) = &self.inventory {
                for item in &process.entrypoints {
                    let reference = &item.value;
                    if inventory.find(reference).is_none() {
                        problems.push(format!(
                            "{}: {}: entrypoint {} is not in entrypoints.tsv",
                            record.file, item.key, reference
                        ));
                    }
                }
            } else if !process.entrypoints.is_empty() {
                needs_scan = true;
            }
        }
        if needs_scan
            && !self
                .problems
                .iter()
                .any(|problem| problem.starts_with(ENTRYPOINTS_FILE))
        {
            problems.push(format!(
                "{ENTRYPOINTS_FILE} is missing; run roostr scan to check process entrypoints"
            ));
        }
        problems
    }
}

fn read_yaml<T>(path: &Path, parse: fn(&str) -> Parsed<T>) -> Parsed<T> {
    match fs::read_to_string(path) {
        Ok(text) => parse(&text),
        Err(error) => Parsed::failed(format!("cannot read: {error}")),
    }
}
