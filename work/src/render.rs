use std::collections::{BTreeMap, HashSet};

use unicode_width::UnicodeWidthStr;

use crate::characters::{written, written_file_name, written_name};
use crate::check::{Entrypoints, Report};
use crate::entrypoints::Inventory;
use crate::git::Changes;
use crate::model::{Kind, Process, Project};
use crate::repository::{ENTRYPOINTS_FILE, Repository};
use crate::scan::{Failure, SCAN_FILE};

const WIDTH: usize = 80;

fn width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

fn pad(text: &str, columns: usize) -> String {
    format!("{text}{}", " ".repeat(columns.saturating_sub(width(text))))
}

/// The widest a column of names gets; a longer name stands alone on its line.
const NAME_COLUMN: usize = 36;

fn name_width<S: AsRef<str>>(names: impl Iterator<Item = S>) -> usize {
    names
        .map(|name| width(name.as_ref()))
        .max()
        .unwrap_or(0)
        .min(NAME_COLUMN)
}

// Prose within `columns`, explicit newlines kept. Words are never broken: one longer than
// the room starts a line of its own and runs past it. A no-break space joins words.
fn wrap(text: &str, columns: usize) -> Vec<String> {
    text.lines()
        .flat_map(|source| {
            let words = source
                .split(|ch: char| ch.is_whitespace() && ch != NO_BREAK)
                .filter(|word| !word.is_empty());
            wrap_words(words, columns)
        })
        .collect()
}

const NO_BREAK: char = '\u{a0}';

// Words that must stay on one line, such as a command to copy: joined by no-break spaces,
// which `block` prints as spaces.
fn unbroken(text: &str) -> String {
    text.replace(' ', &NO_BREAK.to_string())
}

// Words laid out greedily within `columns`, each whole: a word may be a path with spaces.
fn wrap_words<'a>(words: impl IntoIterator<Item = &'a str>, columns: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in words {
        if !line.is_empty() && width(&line) + 1 + width(word) > columns {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    lines.push(line);
    lines
}

// Text kept as it is when it fits, such as counts spaced apart; wrapped when it does not.
fn fitted(text: &str, columns: usize) -> Vec<String> {
    if width(text) <= columns {
        vec![text.to_owned()]
    } else {
        wrap(text, columns)
    }
}

/// A row of a column of names: `name` at `indent`, padded to `name_width` and followed by
/// `gap` spaces, then `rest`, whose lines start in that column. A name wider than the column
/// stands alone on its line, whole, and `rest` starts on the next.
fn row(indent: usize, name: &str, name_width: usize, gap: usize, rest: &[String]) -> String {
    let margin = " ".repeat(indent + name_width + gap);
    let mut rest = rest.iter();
    let first = if width(name) <= name_width {
        let value = rest.next().map_or("", String::as_str);
        format!(
            "{}{}{value}",
            " ".repeat(indent),
            pad(name, name_width + gap)
        )
    } else {
        format!("{}{name}", " ".repeat(indent))
    };
    let mut lines = vec![first.trim_end().to_owned()];
    lines.extend(rest.map(|line| {
        if line.is_empty() {
            String::new()
        } else {
            format!("{margin}{line}")
        }
    }));
    lines.join("\n")
}

// A command is copied, not read: never wrapped. Of one of several lines, the first shows.
fn command_line(command: &str) -> String {
    match first_line(command) {
        (first, true) => format!("{first} ..."),
        (first, false) => first.to_owned(),
    }
}

/// The first line of a command, and whether more lines follow it. A command written as a
/// block ends with a newline, which is not one more line.
pub(crate) fn first_line(command: &str) -> (&str, bool) {
    let command = command.strip_suffix('\n').unwrap_or(command);
    match command.split_once(['\n', '\r']) {
        Some((first, _)) => (first, true),
        None => (command, false),
    }
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

// A heading that fits keeps its spacing, such as the three spaces between an id and a name.
// The name on the right, a group, is printed whole.
fn heading(left: &str, right: &str) -> String {
    let whole_left = if width(left) <= WIDTH {
        left.to_owned()
    } else {
        paragraph(left, 0)
    };
    if right.is_empty() {
        return whole_left;
    }
    if width(left) + 3 + width(right) <= WIDTH {
        return format!("{}{right}", pad(left, WIDTH - width(right)));
    }
    format!("{whole_left}\n{right}")
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
    git_unreadable: Option<&str>,
) -> String {
    let mut sections = Vec::new();
    let mut title = heading(
        &written(&project.name),
        &written(project.stack.as_deref().unwrap_or_default()),
    );
    if let Some(description) = &project.description {
        if !description.is_empty() {
            title.push('\n');
            title.push_str(&paragraph(&written(description), 0));
        }
    }
    sections.push(title);
    if !project.watch_out.is_empty() {
        let mut lines = vec!["WATCH OUT".to_owned()];
        lines.extend(
            project
                .watch_out
                .iter()
                .map(|text| labeled("-", &written(text), 4)),
        );
        sections.push(lines.join("\n"));
    }
    sections.push(functionality(repository));
    if !project.operations.is_empty() {
        let label_width = name_width(
            project
                .operations
                .iter()
                .map(|(name, _)| written_name(name)),
        );
        let column = 2 + label_width + 4;
        let mut lines = vec!["WORK WITH THIS PROJECT".to_owned()];
        for (name, operation) in &project.operations {
            lines.push(row(
                2,
                &written_name(name),
                label_width,
                4,
                &[written(&command_line(&operation.command)).into_owned()],
            ));
            if let Some(note) = &operation.note {
                if !note.is_empty() {
                    lines.push(labeled("", &written(note), column));
                }
            }
        }
        sections.push(lines.join("\n"));
    }
    if !project.environments.is_empty() {
        let label_width = name_width(
            project
                .environments
                .iter()
                .map(|(name, _)| written_name(name)),
        );
        let column = 2 + label_width + 4;
        let mut lines = vec!["ENVIRONMENTS".to_owned()];
        for (name, destination) in &project.environments {
            lines.push(row(
                2,
                &written_name(name),
                label_width,
                4,
                &wrap(&written(destination), WIDTH - column),
            ));
        }
        sections.push(lines.join("\n"));
    }
    if let Some(changes) = changes.filter(|changes| !changes.is_empty()) {
        sections.push(format!(
            "LAST 7 DAYS\n  {}\n  run roostr changes to see them",
            changes.summary()
        ));
    }
    if let Some(attention) = attention(repository, changed_files, git_unreadable) {
        sections.push(attention);
    }
    format!("{}\n", sections.join("\n\n"))
}

// A line that names a file and then says something about it: the path is kept whole.
fn about_file(file: &str, words: &str, indent: usize) -> String {
    wrap_words(
        std::iter::once(file).chain(words.split_whitespace()),
        WIDTH - indent,
    )
    .into_iter()
    .map(|line| format!("{}{line}", " ".repeat(indent)))
    .collect::<Vec<_>>()
    .join("\n")
}

// `git_unreadable` is why Git could not be read, when it could not.
fn attention(
    repository: &Repository,
    changed_files: &[(String, String)],
    git_unreadable: Option<&str>,
) -> Option<String> {
    let attention_files = repository.unreadable();
    if attention_files.is_empty()
        && repository.problems.is_empty()
        && changed_files.is_empty()
        && git_unreadable.is_none()
    {
        return None;
    }
    let mut lines = vec!["NEEDS ATTENTION".to_owned()];
    lines.extend(attention_files.iter().map(|file| {
        about_file(
            &format!("{}:", written_file_name(file)),
            "needs checking; run roostr validate",
            2,
        )
    }));
    lines.extend(repository.problems.iter().map(|finding| {
        about_file(
            &format!("{}:", written_file_name(&finding.file)),
            &format!("{}; run roostr validate", finding.problem),
            2,
        )
    }));
    if let Some(reason) = git_unreadable {
        // Only Git's first line, as it wrote it: a path in it must stay whole.
        lines.push("  Git could not be read, so changes to the code are not shown:".to_owned());
        lines.push(format!(
            "    {}",
            written(reason.lines().next().unwrap_or_default().trim_end())
        ));
    }
    let label_width = name_width(changed_files.iter().map(|(id, _)| written_file_name(id)));
    let column = 2 + label_width + 3;
    for (id, file) in changed_files {
        let file = written_name(file);
        let rest = wrap_words(
            std::iter::once(file.as_ref())
                .chain("changed since this record was updated".split_whitespace()),
            WIDTH - column,
        );
        lines.push(row(2, &written_file_name(id), label_width, 3, &rest));
    }
    Some(lines.join("\n"))
}

pub(crate) fn changes(changes: &Changes) -> String {
    let mut sections = vec![format!("CHANGES SINCE {}", changes.first_day)];
    if let Some(entries) = &changes.entrypoints {
        if !entries.added.is_empty() || !entries.removed.is_empty() {
            let mut lines = vec!["ENTRYPOINTS".to_owned()];
            lines.extend(
                entries
                    .added
                    .iter()
                    .map(|entry| format!("  + {}", written_name(&entry.to_string()))),
            );
            lines.extend(
                entries
                    .removed
                    .iter()
                    .map(|entry| format!("  - {}", written_name(&entry.to_string()))),
            );
            sections.push(lines.join("\n"));
        }
    }
    if !changes.processes.is_empty() {
        let mut lines = vec!["PROCESSES".to_owned()];
        for (id, subjects) in &changes.processes {
            lines.push(paragraph(&written_file_name(id), 2));
            lines.extend(
                subjects
                    .iter()
                    .map(|subject| paragraph(&written(subject), 4)),
            );
        }
        sections.push(lines.join("\n"));
    }
    sections.push(changes.summary());
    format!("{}\n", sections.join("\n\n"))
}

// Text shown as it was written, each line indented; never wrapped.
fn literal(text: &str, indent: usize) -> Vec<String> {
    text.lines()
        .map(|line| {
            let line = line.trim_end();
            if line.is_empty() {
                String::new()
            } else {
                format!("{}{line}", " ".repeat(indent))
            }
        })
        .collect()
}

fn block(heading: &str, advice: &str, items: Vec<String>) -> String {
    let mut lines = vec![
        heading.to_owned(),
        paragraph(advice, 2).replace(NO_BREAK, " "),
    ];
    lines.extend(items);
    lines.join("\n")
}

pub(crate) fn check(report: &Report) -> String {
    if report.is_empty() {
        return "No problems found\n".to_owned();
    }
    let mut blocks = Vec::new();
    if !report.not_staged.is_empty() {
        blocks.push(block(
            "NOT STAGED",
            &format!(
                "The commit would not hold these files as they are in the working tree, where \
                 check judged them. Stage each with {}, or undo the change.",
                unbroken("git add <path>")
            ),
            report
                .not_staged
                .iter()
                .map(|file| format!("    {}", written_name(file)))
                .collect(),
        ));
    }
    if !report.validate.is_empty() {
        blocks.push(block(
            "VALIDATE",
            "Correct each of these in the file it names. roostr validate lists the same \
             problems.",
            report
                .validate
                .iter()
                .flat_map(|finding| literal(&finding.to_string(), 4))
                .collect(),
        ));
    }
    if let Some(entrypoints) = &report.entrypoints {
        blocks.push(entrypoints_block(entrypoints));
    }
    if !report.processes.is_empty() {
        let mut items = Vec::new();
        for (id, files) in &report.processes {
            items.push(format!("    {}", written_file_name(id)));
            items.extend(
                files
                    .iter()
                    .map(|file| format!("      {}", written_name(file))),
            );
        }
        blocks.push(block(
            "PROCESSES",
            &format!(
                "Each file below changed after the record of its process was last updated. \
                 Update .roostr/processes/<id>.yml so that it describes the code as it is now. \
                 If the record still holds, write {} in it instead.",
                unbroken(&format!("checked: {}", report.checked))
            ),
            items,
        ));
    }
    if !report.not_described.is_empty() {
        blocks.push(block(
            "NOT DESCRIBED",
            &format!(
                "No process lists these entrypoints, though {} each was either new or listed by \
                 a process. Add each to the entrypoints of the process it belongs to in \
                 .roostr/processes/<id>.yml, or describe a new process for it. Where a process \
                 listed it before, restore that record or its entry instead.",
                if report.session.is_some() {
                    "when the session started".to_owned()
                } else {
                    format!("at {}", written(report.base.as_deref().unwrap_or_default()))
                }
            ),
            report
                .not_described
                .iter()
                .map(|reference| format!("    {}", written_name(reference)))
                .collect(),
        ));
    }
    format!("{}\n", blocks.join("\n\n"))
}

fn entrypoints_block(entrypoints: &Entrypoints) -> String {
    const UNKNOWN: &str = "check cannot tell whether .roostr/entrypoints.tsv is current";
    const CORRECT: &str = "Correct the script until roostr scan succeeds.";
    let (advice, mut items, stderr) = match entrypoints {
        Entrypoints::Behind { added, removed } => (
            format!(
                "{ENTRYPOINTS_FILE} is not what {SCAN_FILE} finds in the code now. Run roostr \
                 scan to update it."
            ),
            added
                .iter()
                .map(|reference| format!("    + {}", written_name(reference)))
                .chain(
                    removed
                        .iter()
                        .map(|reference| format!("    - {}", written_name(reference))),
                )
                .collect(),
            None,
        ),
        Entrypoints::NeverScanned => (
            format!(
                "{ENTRYPOINTS_FILE} does not exist yet. Run roostr scan to write it from what \
                 {SCAN_FILE} finds in the code."
            ),
            Vec::new(),
            None,
        ),
        Entrypoints::Failed(failure) => match failure {
            Failure::Missing(_) | Failure::NotExecutable | Failure::CannotStart { .. } => {
                let reason = written(&failure.message()).into_owned();
                let stop = if reason.ends_with('.') { "" } else { "." };
                (
                    format!("{reason}{stop} Until it runs, {UNKNOWN}."),
                    Vec::new(),
                    None,
                )
            }
            Failure::Exited { reason, stderr, .. } => (
                format!("{SCAN_FILE} {}, so {UNKNOWN}. {CORRECT}", written(reason)),
                Vec::new(),
                Some(stderr),
            ),
            Failure::Malformed { problems, stderr } => (
                format!(
                    "{SCAN_FILE} printed output that roostr scan refuses, so {UNKNOWN}. {CORRECT}"
                ),
                problems
                    .iter()
                    .flat_map(|problem| literal(&problem.to_string(), 4))
                    .collect(),
                Some(stderr),
            ),
        },
    };
    let stderr = stderr.map(|bytes| String::from_utf8_lossy(bytes).into_owned());
    if let Some(stderr) = stderr.filter(|text| !text.trim().is_empty()) {
        items.push(paragraph("It wrote this to stderr:", 2));
        let mut lines = literal(&written(&stderr), 4);
        while lines.last().is_some_and(String::is_empty) {
            lines.pop();
        }
        items.extend(lines);
    }
    block("ENTRYPOINTS", &advice, items)
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
    let group_width = name_width(groups.keys().map(|group| written_name(group)));
    let id_width = name_width(processes.iter().map(|(id, _)| written_file_name(id)));
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
        let shown_group = written_name(group);
        if counts.is_empty() {
            lines.push(format!("  {shown_group}"));
        } else {
            let counts = fitted(&counts.join("   "), WIDTH - (2 + group_width + 3));
            lines.push(row(2, &shown_group, group_width, 3, &counts));
        }
        for (id, process) in &groups[group] {
            let description = wrap(&written(&process.description), WIDTH - (4 + id_width + 3));
            lines.push(row(4, &written_file_name(id), id_width, 3, &description));
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

// A label in the details column and a value printed whole: an entrypoint or a path.
fn whole(label: &str, value: &str) -> String {
    format!("{}{value}", pad(&format!("  {label}"), 11))
}

pub(crate) fn show(id: &str, process: &Process, inventory: Option<&Inventory>) -> String {
    let mut sections = Vec::new();
    sections.push(format!(
        "{}\n{}",
        heading(
            &format!("{}   {}", written_file_name(id), written(&process.name)),
            &written_name(process.group.as_deref().unwrap_or_default())
        ),
        paragraph(&written(&process.description), 0)
    ));
    let mut details = Vec::new();
    for item in &process.entrypoints {
        let reference = &item.value;
        details.push(whole("entry", &written_name(&reference.to_string())));
        let file = inventory
            .and_then(|inventory| inventory.find(reference))
            .map_or("not in entrypoints.tsv".into(), |entry| {
                written_name(&entry.file)
            });
        details.push(whole("", &file));
    }
    let path_width = process
        .files
        .iter()
        .map(|file| width(&written_name(&file.value.path)))
        .max()
        .unwrap_or(0)
        .min(40);
    for (index, item) in process.files.iter().enumerate() {
        let file = &item.value;
        let label = if index == 0 { "code" } else { "" };
        let path = written_name(&file.path);
        if let Some(note) = &file.note {
            let note = written(note);
            let prefix = pad(&format!("  {label}"), 11);
            let line = format!("{prefix}{}   {note}", pad(&path, path_width));
            if width(&line) <= WIDTH {
                details.push(line);
            } else {
                details.push(whole(label, &path));
                details.push(labeled("", &note, 11));
            }
        } else {
            details.push(whole(label, &path));
        }
    }
    if let Some(text) = &process.try_text {
        if !text.is_empty() {
            details.push(literal_labeled("try", &written(text), 11));
        }
    }
    if let Some(checked) = &process.checked {
        details.push(labeled("checked", &written(checked), 11));
    }
    if !details.is_empty() {
        sections.push(details.join("\n"));
    }
    if !process.outcomes.is_empty() {
        let mut lines = vec!["OUTCOMES".to_owned()];
        let label_width = name_width(process.outcomes.iter().map(|(name, _)| written_name(name)));
        for (name, outcome) in &process.outcomes {
            let text = wrap(
                &format!("{}: {}", written(&outcome.when), written(&outcome.then)),
                WIDTH - (2 + label_width + 3),
            );
            lines.push(row(2, &written_name(name), label_width, 3, &text));
        }
        sections.push(lines.join("\n"));
    }
    if !process.why.is_empty() {
        sections.push(format!(
            "WHY\n{}",
            process
                .why
                .iter()
                .map(|text| paragraph(&written(text), 2))
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
    fn wide_characters_and_combining_accents_fit_the_columns_and_words_stay_whole() {
        let text = "庭園 café e\u{301} 新しい庭園";
        let words: Vec<_> = text.split_whitespace().collect();
        for columns in 3..20 {
            let lines = wrap(text, columns);
            // A line passes the edge only when it is one word that does not fit.
            assert!(
                lines
                    .iter()
                    .all(|line| width(line) <= columns || words.contains(&line.as_str()))
            );
            assert_eq!(lines.join(" "), words.join(" "));
        }
    }

    #[test]
    fn a_word_longer_than_the_line_stands_whole_on_a_line_of_its_own() {
        let path = format!("src/{}.rs", "a".repeat(90));
        let lines = wrap(&format!("see {path} for the rest"), 80);
        assert_eq!(lines, ["see", path.as_str(), "for the rest"]);
    }

    #[test]
    fn a_name_wider_than_its_column_stands_alone_and_the_rest_starts_in_the_column() {
        let long = "n".repeat(40);
        let rest = vec!["first".to_owned(), "second".to_owned()];
        assert_eq!(
            row(2, &long, 36, 3, &rest),
            format!("  {long}\n{0}first\n{0}second", " ".repeat(41))
        );
        assert_eq!(
            row(2, "short", 36, 3, &rest),
            format!("  short{}first\n{}second", " ".repeat(34), " ".repeat(41))
        );
        assert_eq!(name_width(["a", "bb"].into_iter()), 2);
        assert_eq!(name_width([long.as_str()].into_iter()), 36);
    }

    #[test]
    fn a_command_of_several_lines_shows_its_first_and_a_block_of_one_its_line() {
        assert_eq!(command_line("make setup\n"), "make setup");
        assert_eq!(
            command_line("bundle install\nbin/setup\n"),
            "bundle install ..."
        );
        assert_eq!(command_line("make test"), "make test");
    }

    #[test]
    fn detail_view_keeps_long_file_paths_with_notes() {
        let path = "src/very/long/nested/path/to/the/order/export/implementation.rs";
        let parsed = crate::parse::process(&format!(
            "name: Export\ndescription: Export orders.\nfiles:\n  - path: {path}\n    note: Converts orders into records.\n"
        ));
        let output = show("export", parsed.readable().unwrap(), None);
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
