//! The project map: what `roostr session start` gives an agent before its first step. It is
//! made of `.roostr/project.yml` and the readable records only, never Git, the scan, the
//! clock or the terminal, so that the same files give the same bytes.

use std::collections::BTreeMap;
use std::path::Path;

use crate::characters::{self, shown};
use crate::model::Project;
use crate::render;
use crate::repository::{PROJECT_FILE, Repository};

/// The most bytes a map has, as UTF-8, before JSON escapes it.
const LIMIT: usize = 8_000;

const ABOUT: &str = ", from .roostr/, kept by people and agents working here. Records can lag \
     the code; where they differ, the code is current. `roostr show` with a process id prints \
     a process in full: files, entrypoints, how to try it, outcomes. `roostr` prints the \
     overview and recent changes.";

/// The group of the processes without one, last in the map.
const OTHER: &str = "other";

const LEFT_OUT: &str = "Left out to keep this map within 8,000 bytes: each process's description";

/// The line `session start` prints for the project found from `cwd` up: the map in the
/// JSON a SessionStart hook gives the agent. Nothing where there is no project; an error,
/// one line, when its `project.yml` cannot be read.
pub(crate) fn hook_output(cwd: &Path) -> Result<Option<String>, String> {
    let Some(root) = Repository::discover(cwd) else {
        return Ok(None);
    };
    let repository = Repository::load_records(root);
    let Some(project) = repository.project.readable() else {
        let reasons = repository.project.reasons();
        return Err(format!(
            "roostr session start: no project map, as {PROJECT_FILE} cannot be read: {}. Run \
             roostr validate for details.",
            reasons.split_whitespace().collect::<Vec<_>>().join(" ")
        ));
    };
    let map = map(project, &processes(&repository));
    Ok(Some(format!(
        "{{\"hookSpecificOutput\":{{\"hookEventName\":\"SessionStart\",\"additionalContext\":\"{}\"}}}}\n",
        json_text(&map)
    )))
}

/// A process as the map shows it.
struct Entry {
    id: String,
    description: String,
    files: Vec<String>,
}

/// The readable records by group: the groups in byte order of their names as the map prints
/// them, each one's processes in byte order of their ids; last, whatever the names of the
/// others, one `other` with the processes without a group and those whose group is `other`.
fn processes(repository: &Repository) -> Vec<(String, Vec<Entry>)> {
    let mut groups: BTreeMap<String, Vec<Entry>> = BTreeMap::new();
    let mut other = Vec::new();
    for (id, process) in repository.readable_processes() {
        // The id the map would print is not the record's id: `validate` reports its file.
        if !characters::changed(id).is_empty() {
            continue;
        }
        let entry = Entry {
            id: id.to_owned(),
            description: shown(&process.description),
            files: process
                .files
                .iter()
                .map(|file| shown(&file.value.path))
                .filter(|path| !path.is_empty())
                .take(2)
                .collect(),
        };
        match process.group.as_deref().map(shown) {
            Some(group) if !group.is_empty() && group != OTHER => {
                groups.entry(group).or_default().push(entry);
            }
            _ => other.push(entry),
        }
    }
    let mut groups: Vec<_> = groups.into_iter().collect();
    if !other.is_empty() {
        groups.push((OTHER.into(), other));
    }
    for (_, entries) in &mut groups {
        entries.sort_by(|a, b| a.id.cmp(&b.id));
    }
    groups
}

/// The map, the first of its forms that is within the limit.
fn map(project: &Project, groups: &[(String, Vec<Entry>)]) -> String {
    let head = Head::of(project);
    let with = |processes: Vec<String>| {
        let mut parts = head.parts(&head.whole());
        if !groups.is_empty() {
            parts.push(processes);
        }
        lines(&parts)
    };
    let whole = with(by_group(
        "Processes by group, each as id: description [first files]:",
        groups,
        |entry| {
            let mut line = format!("- {}", entry.id);
            if !entry.description.is_empty() {
                line.push_str(&format!(": {}", entry.description));
            }
            if !entry.files.is_empty() {
                line.push_str(&format!(" [{}]", entry.files.join(", ")));
            }
            line
        },
    ));
    if size(&whole) <= LIMIT {
        return whole.join("\n");
    }
    let first_files = with(by_group(
        "Processes by group, each as id [first file]:",
        groups,
        |entry| match entry.files.first() {
            Some(file) => format!("- {} [{file}]", entry.id),
            None => format!("- {}", entry.id),
        },
    ));
    let first_files = closed(
        first_files,
        format!(
            "{LEFT_OUT} and all its files but the first. `roostr` prints the descriptions; \
             `roostr show` with a process id prints the files."
        ),
    );
    if size(&first_files) <= LIMIT {
        return first_files.join("\n");
    }
    let ids = closed(
        with(Ids::new(groups).lines()),
        format!(
            "{LEFT_OUT} and files. `roostr` prints the descriptions; `roostr show` with a \
             process id prints the files."
        ),
    );
    if size(&ids) <= LIMIT {
        return ids.join("\n");
    }
    cut(&head, groups).join("\n")
}

/// The lines, a blank line, and the closing line that says what was left out.
fn closed(mut lines: Vec<String>, closing: String) -> Vec<String> {
    lines.push(String::new());
    lines.push(closing);
    lines
}

/// Form (c): form (b) cut at its end, one item at a time, until it fits. The process ids go
/// first, from the last one back; then the operations, from the last; then the watch-out
/// items, from the last; then the Description line, then the Stack line. The header stays,
/// and with its name cut it fits with any closing line.
fn cut(head: &Head, groups: &[(String, Vec<Entry>)]) -> Vec<String> {
    let mut ids = Ids::new(groups);
    let whole = head.whole();
    let mut kept = whole;
    loop {
        if ids.left > 0 {
            ids.cut_one();
        } else if kept.operations > 0 {
            kept.operations -= 1;
        } else if kept.watch_out > 0 {
            kept.watch_out -= 1;
        } else if kept.description {
            kept.description = false;
        } else if kept.stack {
            kept.stack = false;
        }
        let closing = if ids.left > 0 {
            format!(
                "{LEFT_OUT} and files, and {} of the {} process ids. `roostr` prints the full \
                 overview.",
                ids.total - ids.left,
                ids.total
            )
        } else {
            format!(
                "Left out to keep this map within 8,000 bytes: {}. `roostr` prints the full \
                 overview.",
                head.left_out(&whole, &kept)
            )
        };
        // The parts kept, a blank line between each two; a blank line and the closing line.
        let mut parts = head.sizes(&kept);
        if ids.left > 0 {
            parts.push((ids.bytes, ids.lines));
        }
        let bytes: usize = parts.iter().map(|(bytes, _)| bytes).sum::<usize>() + closing.len();
        let count = parts.iter().map(|(_, count)| count).sum::<usize>() + parts.len() + 1;
        let everything_cut = ids.left == 0 && kept == Kept::default();
        if bytes + count - 1 <= LIMIT || everything_cut {
            let mut parts = head.parts(&kept);
            if ids.left > 0 {
                parts.push(ids.lines());
            }
            return closed(lines(&parts), closing);
        }
    }
}

/// How much of each part before the processes is kept.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Kept {
    stack: bool,
    description: bool,
    watch_out: usize,
    operations: usize,
}

/// The parts before the processes: the header; the stack and the description; what to watch
/// out for; the operations. Each item is a line.
struct Head {
    header: String,
    stack: Option<String>,
    description: Option<String>,
    watch_out: Vec<String>,
    operations: Vec<String>,
}

impl Head {
    fn of(project: &Project) -> Self {
        // A value that shows as nothing is not there.
        let present = |value: &str| Some(shown(value)).filter(|value| !value.is_empty());
        let operations = project
            .operations
            .iter()
            .filter_map(|(name, operation)| {
                let name = present(name)?;
                if ships(&name) {
                    return None;
                }
                let (first, more) = render::first_line(&operation.command);
                let mut line = format!("- {name}: `{}`", present(first)?);
                if more {
                    line.push_str(" ...");
                }
                if let Some(note) = operation.note.as_deref().and_then(present) {
                    line.push_str(&format!(" ({note})"));
                }
                Some(line)
            })
            .collect();
        Self {
            header: format!("roostr project map of {}{ABOUT}", name(&project.name)),
            stack: present(project.stack.as_deref().unwrap_or_default())
                .map(|stack| format!("Stack: {stack}")),
            description: present(project.description.as_deref().unwrap_or_default())
                .map(|description| format!("Description: {description}")),
            watch_out: project
                .watch_out
                .iter()
                .filter_map(|item| Some(format!("- {}", present(item)?)))
                .collect(),
            operations,
        }
    }

    fn whole(&self) -> Kept {
        Kept {
            stack: self.stack.is_some(),
            description: self.description.is_some(),
            watch_out: self.watch_out.len(),
            operations: self.operations.len(),
        }
    }

    /// The parts with what is `kept`, each as its lines; a part with nothing in it is left
    /// out with its heading.
    fn parts(&self, kept: &Kept) -> Vec<Vec<String>> {
        let mut parts = vec![vec![self.header.clone()]];
        let about: Vec<String> = [
            self.stack.as_ref().filter(|_| kept.stack),
            self.description.as_ref().filter(|_| kept.description),
        ]
        .into_iter()
        .flatten()
        .cloned()
        .collect();
        if !about.is_empty() {
            parts.push(about);
        }
        for (heading, items, count) in [
            (WATCH_OUT, &self.watch_out, kept.watch_out),
            (OPERATIONS, &self.operations, kept.operations),
        ] {
            if count > 0 {
                parts.push(
                    std::iter::once(heading.to_owned())
                        .chain(items[..count].iter().cloned())
                        .collect(),
                );
            }
        }
        parts
    }

    /// The bytes and the lines of each part `parts` gives.
    fn sizes(&self, kept: &Kept) -> Vec<(usize, usize)> {
        let mut sizes = vec![(self.header.len(), 1)];
        let about: Vec<usize> = [
            self.stack.as_ref().filter(|_| kept.stack),
            self.description.as_ref().filter(|_| kept.description),
        ]
        .into_iter()
        .flatten()
        .map(String::len)
        .collect();
        if !about.is_empty() {
            sizes.push((about.iter().sum(), about.len()));
        }
        for (heading, items, count) in [
            (WATCH_OUT, &self.watch_out, kept.watch_out),
            (OPERATIONS, &self.operations, kept.operations),
        ] {
            if count > 0 {
                let bytes: usize = items[..count].iter().map(String::len).sum();
                sizes.push((heading.len() + bytes, count + 1));
            }
        }
        sizes
    }

    /// What the closing line names once every process is cut: `every process, every
    /// operation, and 11 of the 80 watch-out items`.
    fn left_out(&self, whole: &Kept, kept: &Kept) -> String {
        let mut items = vec!["every process".to_owned()];
        for (cut, total, every, some) in [
            (
                whole.operations - kept.operations,
                whole.operations,
                "every operation",
                "operations",
            ),
            (
                whole.watch_out - kept.watch_out,
                whole.watch_out,
                "every watch-out item",
                "watch-out items",
            ),
        ] {
            if cut == total && cut > 0 {
                items.push(every.to_owned());
            } else if cut > 0 {
                items.push(format!("{cut} of the {total} {some}"));
            }
        }
        if whole.description && !kept.description {
            items.push("the Description line".to_owned());
        }
        if whole.stack && !kept.stack {
            items.push("the Stack line".to_owned());
        }
        match items.split_last() {
            Some((last, rest)) if !rest.is_empty() => format!("{}, and {last}", rest.join(", ")),
            _ => items.join(""),
        }
    }
}

const WATCH_OUT: &str = "Watch out:";
const OPERATIONS: &str = "Operations:";

/// The longest project name the header prints whole, in characters as the map prints it.
const NAME_KEPT: usize = 200;

/// The project's name as the header prints it: past 200 characters, cut, and `...` after.
fn name(name: &str) -> String {
    let name = shown(name);
    match name.char_indices().nth(NAME_KEPT) {
        Some((at, _)) => format!("{}...", &name[..at]),
        None => name,
    }
}

/// Whether an operation ships: one of the words of its name, its runs of letters, is
/// `deploy`, `release` or `publish`, in any letter case.
fn ships(name: &str) -> bool {
    name.split(|ch: char| !ch.is_alphabetic()).any(|word| {
        let word = word.to_lowercase();
        ["deploy", "release", "publish"].contains(&word.as_str())
    })
}

/// Form (b)'s lines of ids, `<group>: <id>, <id>` under their legend, as form (c) cuts them:
/// one id at a time from the end, a group line that has lost some ending in `, ...`, one
/// with none left gone, and the legend with the last.
struct Ids<'a> {
    groups: &'a [(String, Vec<Entry>)],
    total: usize,
    /// The ids left.
    left: usize,
    /// The groups with ids left, and how many the last of them has.
    groups_left: usize,
    last_left: usize,
    /// The last group line kept, as long as it is.
    last_length: usize,
    /// The bytes and the lines of the legend and the group lines left.
    bytes: usize,
    lines: usize,
}

const IDS_LEGEND: &str = "Process ids by group:";

impl<'a> Ids<'a> {
    fn new(groups: &'a [(String, Vec<Entry>)]) -> Self {
        let lengths: Vec<usize> = (0..groups.len())
            .map(|group| whole_line(&groups[group]))
            .collect();
        let total = groups.iter().map(|(_, entries)| entries.len()).sum();
        Self {
            groups,
            total,
            left: total,
            groups_left: groups.len(),
            last_left: groups.last().map_or(0, |(_, entries)| entries.len()),
            last_length: lengths.last().copied().unwrap_or(0),
            bytes: IDS_LEGEND.len() + lengths.iter().sum::<usize>(),
            lines: 1 + groups.len(),
        }
    }

    /// The last id left goes.
    fn cut_one(&mut self) {
        let (_, entries) = &self.groups[self.groups_left - 1];
        self.left -= 1;
        if self.left == 0 {
            self.groups_left = 0;
            self.bytes = 0;
            self.lines = 0;
            return;
        }
        if self.last_left == 1 {
            // The line goes; the group before it is whole.
            self.bytes -= self.last_length;
            self.lines -= 1;
            self.groups_left -= 1;
            let group = &self.groups[self.groups_left - 1];
            self.last_left = group.1.len();
            self.last_length = whole_line(group);
            return;
        }
        let whole = self.last_left == entries.len();
        let id = &entries[self.last_left - 1].id;
        let length = self.last_length - (2 + id.len()) + if whole { MORE.len() } else { 0 };
        self.bytes = self.bytes - self.last_length + length;
        self.last_length = length;
        self.last_left -= 1;
    }

    fn lines(&self) -> Vec<String> {
        if self.groups_left == 0 {
            return Vec::new();
        }
        let mut lines = vec![IDS_LEGEND.to_owned()];
        for (index, (group, entries)) in self.groups[..self.groups_left].iter().enumerate() {
            let last = index + 1 == self.groups_left;
            let count = if last { self.last_left } else { entries.len() };
            let ids: Vec<&str> = entries[..count]
                .iter()
                .map(|entry| entry.id.as_str())
                .collect();
            let more = if count < entries.len() { MORE } else { "" };
            lines.push(format!("{group}: {}{more}", ids.join(", ")));
        }
        lines
    }
}

/// What ends a group line that lost some of its ids.
const MORE: &str = ", ...";

/// The bytes of a group's whole line, `<group>: <id>, <id>`.
fn whole_line((group, entries): &(String, Vec<Entry>)) -> usize {
    group.len()
        + 2
        + entries.iter().map(|entry| entry.id.len()).sum::<usize>()
        + 2 * entries.len().saturating_sub(1)
}

/// The processes under `legend`: each group's name, then its processes, each made a line by
/// `line`.
fn by_group(
    legend: &str,
    groups: &[(String, Vec<Entry>)],
    line: impl Fn(&Entry) -> String,
) -> Vec<String> {
    let mut lines = vec![legend.to_owned()];
    for (group, entries) in groups {
        lines.push(format!("{group}:"));
        lines.extend(entries.iter().map(&line));
    }
    lines
}

/// The parts one after another, a blank line between each two.
fn lines(parts: &[Vec<String>]) -> Vec<String> {
    let mut lines = Vec::new();
    for part in parts {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.extend(part.iter().cloned());
    }
    lines
}

/// The bytes of the lines joined by newlines, as UTF-8.
fn size(lines: &[String]) -> usize {
    lines.iter().map(String::len).sum::<usize>() + lines.len().saturating_sub(1)
}

/// The map inside a JSON string: what does not show is out of it already, so only `"`, `\`
/// and the newline need escaping.
fn json_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len() + text.len() / 16);
    for ch in text.chars() {
        match ch {
            '"' | '\\' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            '\n' => escaped.push_str("\\n"),
            ch => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Operation;

    fn groups(spec: &[(&str, usize, usize)]) -> Vec<(String, Vec<Entry>)> {
        spec.iter()
            .map(|&(group, count, length)| {
                let entries = (0..count)
                    .map(|index| Entry {
                        id: format!("{group}-{index:0length$}").to_lowercase(),
                        description: String::new(),
                        files: Vec::new(),
                    })
                    .collect();
                (group.to_owned(), entries)
            })
            .collect()
    }

    fn project(watch_out: usize, operations: usize, description: usize) -> Project {
        Project {
            name: "Atelier".into(),
            stack: Some("Rails 8, PostgreSQL".into()),
            description: Some("d".repeat(description)),
            watch_out: (0..watch_out)
                .map(|index| format!("Shelf {index} holds glazes for the evening workshops."))
                .collect(),
            operations: (0..operations)
                .map(|index| {
                    let operation = Operation {
                        command: format!("bin/task-{index}"),
                        note: None,
                    };
                    (format!("task-{index}"), operation)
                })
                .collect(),
            environments: Vec::new(),
        }
    }

    // Form (c) as its rule says, measured whole at every step: what `cut` keeps by its
    // running total must be the same.
    fn cut_by_measuring(head: &Head, groups: &[(String, Vec<Entry>)]) -> Vec<String> {
        let whole = head.whole();
        let mut ids = Ids::new(groups);
        let mut kept = whole;
        let mut steps: Vec<(usize, Kept)> = Vec::new();
        while ids.left > 0 {
            ids.cut_one();
            steps.push((ids.total - ids.left, kept));
        }
        let total = ids.total;
        while kept != Kept::default() {
            if kept.operations > 0 {
                kept.operations -= 1;
            } else if kept.watch_out > 0 {
                kept.watch_out -= 1;
            } else if kept.description {
                kept.description = false;
            } else {
                kept.stack = false;
            }
            steps.push((total, kept));
        }
        if steps.is_empty() {
            steps.push((total, kept));
        }
        for (cut_ids, kept) in steps {
            let mut ids = Ids::new(groups);
            (0..cut_ids).for_each(|_| ids.cut_one());
            let mut parts = head.parts(&kept);
            if ids.left > 0 {
                parts.push(ids.lines());
            }
            let closing = if ids.left > 0 {
                format!(
                    "{LEFT_OUT} and files, and {cut_ids} of the {total} process ids. `roostr` \
                     prints the full overview."
                )
            } else {
                format!(
                    "Left out to keep this map within 8,000 bytes: {}. `roostr` prints the full \
                     overview.",
                    head.left_out(&whole, &kept)
                )
            };
            let map = closed(lines(&parts), closing);
            if size(&map) <= LIMIT {
                return map;
            }
        }
        unreachable!("the header and a closing line fit")
    }

    #[test]
    fn the_ids_left_are_counted_as_they_print() {
        let groups = groups(&[("Bookings", 3, 2), ("B", 1, 1), ("other", 4, 3)]);
        let mut ids = Ids::new(&groups);
        loop {
            let lines = ids.lines();
            let bytes: usize = lines.iter().map(String::len).sum();
            assert_eq!((bytes, lines.len()), (ids.bytes, ids.lines), "{lines:?}");
            if ids.left == 0 {
                break;
            }
            ids.cut_one();
        }
        let mut ids = Ids::new(&groups);
        (0..3).for_each(|_| ids.cut_one());
        assert_eq!(
            ids.lines(),
            [
                "Process ids by group:",
                "Bookings: bookings-00, bookings-01, bookings-02",
                "B: b-0",
                "other: other-000, ..."
            ]
        );
    }

    #[test]
    fn form_c_keeps_what_measuring_the_whole_map_at_each_step_keeps() {
        for (watch_out, operations, description, spec) in [
            (2, 2, 80, vec![("Group A", 120, 40), ("Group B", 100, 40)]),
            (2, 2, 80, vec![("other", 400, 30)]),
            (80, 3, 80, vec![("Bookings", 2, 5)]),
            (10, 400, 80, vec![("Bookings", 2, 5)]),
            (3, 3, 8_279, vec![("Bookings", 3, 5)]),
            (3, 3, 7_700, Vec::new()),
            (0, 0, 9_000, Vec::new()),
        ] {
            let project = project(watch_out, operations, description);
            let head = Head::of(&project);
            let groups = groups(&spec);
            let map = cut(&head, &groups);
            assert!(size(&map) <= LIMIT, "{}", size(&map));
            assert_eq!(map, cut_by_measuring(&head, &groups));
        }
    }

    #[test]
    fn the_closing_line_names_what_was_cut_in_order() {
        let head = Head::of(&project(80, 2, 10));
        let whole = head.whole();
        let kept = Kept {
            watch_out: 69,
            operations: 0,
            ..whole
        };
        assert_eq!(
            head.left_out(&whole, &kept),
            "every process, every operation, and 11 of the 80 watch-out items"
        );
        assert_eq!(
            head.left_out(&whole, &Kept::default()),
            "every process, every operation, every watch-out item, the Description line, and \
             the Stack line"
        );
        assert_eq!(head.left_out(&whole, &whole), "every process");
    }

    #[test]
    fn a_long_name_is_cut_and_shipping_operations_are_named_by_their_words() {
        let long = "é".repeat(250);
        assert_eq!(name(&long), format!("{}...", "é".repeat(200)));
        assert_eq!(name(&"a".repeat(200)), "a".repeat(200));
        for shipping in [
            "deploy",
            "Deploy",
            "deploy-production",
            "deploy:staging",
            "staging.deploy",
            "release",
            "publish_gem",
            "RELEASE 2",
            "x/publish",
        ] {
            assert!(ships(shipping), "{shipping}");
        }
        for other in ["deployment-status", "predeploy-check", "released", "start"] {
            assert!(!ships(other), "{other}");
        }
    }
}
