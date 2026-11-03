use std::collections::{BTreeMap, BTreeSet, HashSet};

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::entrypoints::Inventory;
use crate::git::Changes;
use crate::model::{Kind, Process, Project};
use crate::repository::Repository;

const WIDTH: usize = 80;

fn width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

fn pad(text: &str, columns: usize) -> String {
    format!("{text}{}", " ".repeat(columns.saturating_sub(width(text))))
}

fn truncate(text: &str, columns: usize) -> String {
    if width(text) <= columns {
        return text.to_owned();
    }
    if columns <= 3 {
        return ".".repeat(columns);
    }
    let mut result = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let size = ch.width().unwrap_or(0);
        if used + size > columns - 3 {
            break;
        }
        result.push(ch);
        used += size;
    }
    result.push_str("...");
    result
}

// Preserve explicit newlines, and split a long word when it cannot fit on an empty line.
fn wrap(text: &str, columns: usize) -> Vec<String> {
    let columns = columns.max(2);
    let mut result = Vec::new();
    for source in text.lines() {
        let mut line = String::new();
        for word in source.split_whitespace() {
            if !line.is_empty() && width(&line) + 1 + width(word) > columns {
                result.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            let mut used = width(&line);
            for ch in word.chars() {
                let size = ch.width().unwrap_or(0);
                if used + size > columns {
                    result.push(std::mem::take(&mut line));
                    used = 0;
                }
                line.push(ch);
                used += size;
            }
        }
        result.push(line);
    }
    result
}

fn paragraph(text: &str, indent: usize) -> String {
    wrap(text, WIDTH - indent)
        .into_iter()
        .map(|line| {
            if line.is_empty() {
                line
            } else {
                format!("{}{line}", " ".repeat(indent))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn heading(left: &str, right: &str) -> String {
    if right.is_empty() {
        return paragraph(left, 0);
    }
    if width(left) + 3 + width(right) <= WIDTH {
        return format!("{}{right}", pad(left, WIDTH - width(right)));
    }
    format!("{}\n{}", paragraph(left, 0), paragraph(right, 0))
}

fn labeled(label: &str, value: &str, column: usize) -> String {
    let prefix = pad(&format!("  {label}"), column);
    wrapped_with_prefix(&prefix, value)
}

fn wrapped_with_prefix(prefix: &str, value: &str) -> String {
    let column = width(prefix);
    let lines = wrap(value, WIDTH - column);
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                format!("{prefix}{line}").trim_end().to_owned()
            } else if line.is_empty() {
                String::new()
            } else {
                format!("{}{line}", " ".repeat(column))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// A manual snippet is literal text. Only the display's label and left margin are added.
fn literal_labeled(label: &str, value: &str, column: usize) -> String {
    let prefix = pad(&format!("  {label}"), column);
    let margin = " ".repeat(column);
    value
        .split_terminator('\n')
        .enumerate()
        .map(|(index, line)| {
            let prefix = if index == 0 { &prefix } else { &margin };
            format!("{prefix}{line}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn overview(
    project: &Project,
    repository: &Repository,
    changes: Option<&Changes>,
    changed_files: &[(String, String)],
) -> String {
    let mut sections = Vec::new();
    let mut title = heading(&project.name, project.stack.as_deref().unwrap_or_default());
    if let Some(description) = &project.description {
        if !description.is_empty() {
            title.push('\n');
            title.push_str(&paragraph(description, 0));
        }
    }
    sections.push(title);
    if !project.watch_out.is_empty() {
        let mut lines = vec!["WATCH OUT".to_owned()];
        lines.extend(project.watch_out.iter().map(|text| labeled("-", text, 4)));
        sections.push(lines.join("\n"));
    }
    sections.push(functionality(repository));
    if !project.operations.is_empty() {
        let label_width = project
            .operations
            .iter()
            .map(|(name, _)| width(name))
            .max()
            .unwrap_or(0)
            .min(32);
        let column = 2 + label_width + 4;
        let mut lines = vec!["WORK WITH THIS PROJECT".to_owned()];
        for (name, operation) in &project.operations {
            lines.push(table_line(name, &operation.command, label_width, 4));
            if let Some(note) = &operation.note {
                if !note.is_empty() {
                    lines.push(labeled("", note, column));
                }
            }
        }
        sections.push(lines.join("\n"));
    }
    if !project.environments.is_empty() {
        let label_width = project
            .environments
            .iter()
            .map(|(name, _)| width(name))
            .max()
            .unwrap_or(0)
            .min(32);
        let column = 2 + label_width + 4;
        let mut lines = vec!["ENVIRONMENTS".to_owned()];
        for (name, destination) in &project.environments {
            lines.push(labeled(&truncate(name, label_width), destination, column));
        }
        sections.push(lines.join("\n"));
    }
    if let Some(changes) = changes.filter(|changes| !changes.is_empty()) {
        sections.push(format!(
            "LAST 7 DAYS\n{}\n  run roostr changes to see them",
            table_line("", &changes.summary(), 0, 0)
        ));
    }
    if let Some(attention) = attention(repository, changed_files) {
        sections.push(attention);
    }
    format!("{}\n", sections.join("\n\n"))
}

fn attention(repository: &Repository, changed_files: &[(String, String)]) -> Option<String> {
    let attention_files: BTreeSet<_> = repository
        .processes
        .iter()
        .filter(|record| !record.parsed.problems.is_empty())
        .map(|record| record.file.as_str())
        .chain(
            repository
                .inventory
                .as_ref()
                .filter(|inventory| !inventory.problems.is_empty())
                .map(|_| crate::repository::ENTRYPOINTS_FILE),
        )
        .collect();
    if attention_files.is_empty() && repository.problems.is_empty() && changed_files.is_empty() {
        return None;
    }
    let mut lines = vec!["NEEDS ATTENTION".to_owned()];
    lines.extend(
        attention_files
            .iter()
            .map(|file| paragraph(&format!("{file}: needs checking; run roostr validate"), 2)),
    );
    lines.extend(
        repository
            .problems
            .iter()
            .map(|problem| paragraph(&format!("{problem}; run roostr validate"), 2)),
    );
    let label_width = changed_files
        .iter()
        .map(|(id, _)| width(id))
        .max()
        .unwrap_or(0)
        .min(36);
    for (id, file) in changed_files {
        lines.push(table_line(
            id,
            &format!("{file} changed since this record was updated"),
            label_width,
            3,
        ));
    }
    Some(lines.join("\n"))
}

pub(crate) fn changes(changes: &Changes) -> String {
    let mut sections = vec![format!("CHANGES SINCE {}", changes.first_day)];
    if let Some(entries) = &changes.entrypoints {
        if !entries.added.is_empty() || !entries.removed.is_empty() {
            let mut lines = vec!["ENTRYPOINTS".to_owned()];
            lines.extend(entries.added.iter().map(|entry| labeled("+", entry, 4)));
            lines.extend(entries.removed.iter().map(|entry| labeled("-", entry, 4)));
            sections.push(lines.join("\n"));
        }
    }
    if !changes.processes.is_empty() {
        let mut lines = vec!["PROCESSES".to_owned()];
        for (id, subjects) in &changes.processes {
            lines.push(paragraph(id, 2));
            lines.extend(subjects.iter().map(|subject| paragraph(subject, 4)));
        }
        sections.push(lines.join("\n"));
    }
    sections.push(changes.summary());
    format!("{}\n", sections.join("\n\n"))
}

fn functionality(repository: &Repository) -> String {
    let processes: Vec<_> = repository.readable_processes().collect();
    let summary = if let Some(inventory) = &repository.inventory {
        let references: HashSet<_> = processes
            .iter()
            .flat_map(|(_, process)| process.entrypoints.iter().map(|item| &item.value))
            .collect();
        let described = inventory
            .entries
            .iter()
            .filter(|entry| references.contains(&entry.reference))
            .count();
        format!(
            "{} entrypoints, {described} described",
            inventory.entries.len()
        )
    } else {
        "not scanned yet".to_owned()
    };
    let mut lines = vec![heading("FUNCTIONALITY", &summary)];
    let mut groups: BTreeMap<&str, Vec<(&str, &Process)>> = BTreeMap::new();
    for (id, process) in &processes {
        let group = process
            .group
            .as_deref()
            .filter(|group| !group.is_empty())
            .unwrap_or("other");
        groups.entry(group).or_default().push((id, process));
    }
    if let Some(inventory) = &repository.inventory {
        for entry in &inventory.entries {
            groups
                .entry(entry.area.as_deref().unwrap_or("other"))
                .or_default();
        }
    }
    let group_width = groups
        .keys()
        .map(|group| width(group))
        .max()
        .unwrap_or(0)
        .min(36);
    let id_width = processes
        .iter()
        .map(|(id, _)| width(id))
        .max()
        .unwrap_or(0)
        .min(36);
    let group_names: Vec<_> = groups
        .keys()
        .copied()
        .filter(|group| *group != "other")
        .chain(groups.contains_key("other").then_some("other"))
        .collect();
    for group in group_names {
        let mut counts = Vec::new();
        if let Some(inventory) = &repository.inventory {
            for kind in Kind::ALL {
                let count = inventory
                    .entries
                    .iter()
                    .filter(|entry| {
                        entry.reference.kind == kind
                            && entry.area.as_deref().unwrap_or("other") == group
                    })
                    .count();
                if count > 0 {
                    counts.push(format!("{} {count}", kind.as_str()));
                }
            }
        }
        if counts.is_empty() {
            lines.push(format!("  {}", truncate(group, WIDTH - 2)));
        } else {
            lines.push(table_line(group, &counts.join("   "), group_width, 3));
        }
        for (id, process) in &groups[group] {
            let prefix = format!("    {}   ", pad(&truncate(id, id_width), id_width));
            lines.push(wrapped_with_prefix(&prefix, &process.description));
        }
    }
    if let Some(inventory) = &repository.inventory {
        let none_found: Vec<_> = Kind::ALL
            .into_iter()
            .filter(|kind| {
                inventory.covers.contains(kind)
                    && !inventory
                        .entries
                        .iter()
                        .any(|entry| entry.reference.kind == *kind)
            })
            .map(Kind::as_str)
            .collect();
        let not_scanned: Vec<_> = Kind::ALL
            .into_iter()
            .filter(|kind| !inventory.covers.contains(kind))
            .map(Kind::as_str)
            .collect();
        if !none_found.is_empty() {
            lines.push(format!("  none found: {}", none_found.join(" ")));
        }
        if !not_scanned.is_empty() {
            lines.push(format!("  not scanned: {}", not_scanned.join(" ")));
        }
    }
    lines.join("\n")
}

// Retain intentional spacing in short commands and note columns.
fn table_line(label: &str, value: &str, label_width: usize, gap: usize) -> String {
    let label = truncate(label, label_width);
    let prefix = format!("  {}{}", pad(&label, label_width), " ".repeat(gap));
    if width(&prefix) + width(value) <= WIDTH {
        format!("{prefix}{value}")
    } else {
        labeled(&label, value, width(&prefix))
    }
}

pub(crate) fn show(id: &str, process: &Process, inventory: Option<&Inventory>) -> String {
    let mut sections = Vec::new();
    sections.push(format!(
        "{}\n{}",
        heading(
            &format!("{id}   {}", process.name),
            process.group.as_deref().unwrap_or_default()
        ),
        paragraph(&process.description, 0)
    ));
    let mut details = Vec::new();
    for item in &process.entrypoints {
        let reference = &item.value;
        details.push(labeled("entry", &reference.to_string(), 11));
        let file = inventory
            .and_then(|inventory| inventory.find(reference))
            .map(|entry| entry.file.as_str())
            .unwrap_or("not in entrypoints.tsv");
        details.push(labeled("", file, 11));
    }
    let path_width = process
        .files
        .iter()
        .map(|file| width(&file.value.path))
        .max()
        .unwrap_or(0)
        .min(40);
    for (index, item) in process.files.iter().enumerate() {
        let file = &item.value;
        let label = if index == 0 { "code" } else { "" };
        if let Some(note) = &file.note {
            let prefix = pad(&format!("  {label}"), 11);
            let path = pad(&file.path, path_width);
            let line = format!("{prefix}{path}   {note}");
            if width(&line) <= WIDTH {
                details.push(line);
            } else {
                details.push(labeled(label, &file.path, 11));
                details.push(labeled("", note, 11));
            }
        } else {
            details.push(labeled(label, &file.path, 11));
        }
    }
    if let Some(text) = &process.try_text {
        if !text.is_empty() {
            details.push(literal_labeled("try", text, 11));
        }
    }
    if let Some(checked) = &process.checked {
        details.push(labeled("checked", checked, 11));
    }
    if !details.is_empty() {
        sections.push(details.join("\n"));
    }
    if !process.outcomes.is_empty() {
        let mut lines = vec!["OUTCOMES".to_owned()];
        let label_width = process
            .outcomes
            .iter()
            .map(|(name, _)| width(name))
            .max()
            .unwrap_or(0)
            .min(32);
        for (name, outcome) in &process.outcomes {
            lines.push(labeled(
                &truncate(name, label_width),
                &format!("{}: {}", outcome.when, outcome.then),
                2 + label_width + 3,
            ));
        }
        sections.push(lines.join("\n"));
    }
    if !process.why.is_empty() {
        sections.push(format!(
            "WHY\n{}",
            process
                .why
                .iter()
                .map(|text| paragraph(text, 2))
                .collect::<Vec<_>>()
                .join("\n\n")
        ));
    }
    format!("{}\n", sections.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_characters_and_combining_accents_fit_the_columns() {
        let text = "庭園 café e\u{301} 新しい庭園";
        for columns in 3..20 {
            let lines = wrap(text, columns);
            assert!(lines.iter().all(|line| width(line) <= columns));
            let content: String = lines
                .concat()
                .chars()
                .filter(|ch| !ch.is_whitespace())
                .collect();
            let original: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
            assert_eq!(content, original);
            let shortened = truncate(text, columns);
            assert!(width(&shortened) <= columns);
            if width(text) > columns {
                assert!(shortened.ends_with("..."));
            }
        }
    }

    #[test]
    fn long_words_are_wrapped_without_losing_text() {
        let text = "a".repeat(241);
        let lines = wrap(&text, 80);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines.concat(), text);
        assert!(lines.iter().all(|line| width(line) <= 80));
    }

    #[test]
    fn detail_view_keeps_long_file_paths_with_notes() {
        let path = "src/very/long/nested/path/to/the/order/export/implementation.rs";
        let parsed = crate::parse::process(&format!(
            "name: Export\ndescription: Export orders.\nfiles:\n  - path: {path}\n    note: Converts orders into records.\n"
        ));
        let output = show("export", parsed.valid().unwrap(), None);
        assert!(output.contains(path));
        assert!(output.contains("Converts orders into records."));
        assert!(output.lines().all(|line| width(line) <= 80));
    }

    #[test]
    fn try_preserves_blank_lines_tabs_trailing_spaces_and_long_lines() {
        let mut process = crate::parse::process("name: Inspect\ndescription: Inspect an export.\n")
            .value
            .unwrap();
        let long_line = "x".repeat(100);
        process.try_text = Some(format!(
            "Start:\n  run  export\t# check  \n\n{long_line}  \n"
        ));
        let output = show("inspect", &process, None);
        assert!(
            output
                .contains("  try      Start:\n             run  export\t# check  \n           \n")
        );
        assert!(output.ends_with(&format!("           {long_line}  \n")));
        assert_eq!(output.lines().filter(|line| width(line) > WIDTH).count(), 1);
    }
}
