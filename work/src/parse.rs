use std::collections::HashSet;

use crate::characters;
use crate::model::{
    CodeFile, EntrypointRef, Located, Operation, Outcome, Parsed, Place, Problem, Process, Project,
};
use crate::yaml::{self, Node};

// The YAML document in `text`, a byte order mark ignored. A key written twice is a
// problem the file is still read with.
fn document(text: &str, check: &mut Check) -> Result<Node, String> {
    let document = yaml::load(text.strip_prefix('\u{feff}').unwrap_or(text))?;
    for repeated in &document.repeated {
        check.readable(&repeated.path, written_more_than_once(&repeated.lines));
    }
    hidden_characters(&document.root, "", check);
    Ok(document.root)
}

// Every string the document holds, keys and values, checked for characters that are invisible
// or change how text shows: one finding per string, on the path of the key whose value or
// name it is. Of a key written twice, only the value read counts.
fn hidden_characters(node: &Node, at: &str, check: &mut Check) {
    match node {
        Node::Scalar { text, .. } => check.hidden(at, text),
        Node::Sequence(items) => {
            for (index, item) in items.iter().enumerate() {
                hidden_characters(item, &key_path(at, &(index + 1).to_string()), check);
            }
        }
        Node::Mapping(_) => {
            for (key, value) in node.entries().unwrap_or_default() {
                let Some(name) = key.name() else {
                    continue;
                };
                let path = key_path(at, name);
                check.hidden(&path, name);
                hidden_characters(value, &path, check);
            }
        }
    }
}

fn written_more_than_once(lines: &[usize]) -> String {
    let Some((last, earlier)) = lines.split_last() else {
        return "written more than once".into();
    };
    let times = if lines.len() == 2 {
        "twice".to_owned()
    } else {
        format!("{} times", lines.len())
    };
    let earlier: Vec<_> = earlier.iter().map(ToString::to_string).collect();
    format!(
        "written {times}, on lines {} and {last}; the value on line {last} is the one read",
        earlier.join(", ")
    )
}

/// The value written for `key`, unless nothing is: a key with nothing after it, or with the
/// empty string, is not there.
fn present<'a>(node: &'a Node, key: &str) -> Option<&'a Node> {
    node.get(key).filter(|value| !value.is_empty_value())
}

#[derive(Default)]
struct Check {
    problems: Vec<Problem>,
}

fn key_path(parent: &str, key: &str) -> String {
    if parent.is_empty() {
        key.to_owned()
    } else {
        format!("{parent}.{key}")
    }
}

impl Check {
    fn problem(&mut self, key: &str, message: impl std::fmt::Display) {
        let place = if key.is_empty() {
            Place::File
        } else {
            Place::Key(key.to_owned())
        };
        self.problems.push(Problem::new(place, message.to_string()));
    }

    // A problem the file is still read with: a limit on length or count, a key written
    // twice, an operation with no command.
    fn readable(&mut self, key: &str, message: impl Into<String>) {
        self.problems.push(Problem {
            place: Place::Key(key.to_owned()),
            message: message.into(),
            readable: true,
        });
    }

    // A string holding characters that are invisible or change how text shows: the file is
    // still read, and the overview prints the string as written.
    fn hidden(&mut self, key: &str, text: &str) {
        let found = characters::reported(text);
        if !found.is_empty() {
            self.problems.push(Problem {
                place: if key.is_empty() {
                    Place::File
                } else {
                    Place::Key(key.to_owned())
                },
                message: characters::finding(&found),
                readable: true,
            });
        }
    }

    fn max_length(&mut self, text: &str, key: &str, limit: usize) {
        if text.chars().count() > limit {
            self.readable(key, format!("must have at most {limit} characters"));
        }
    }

    fn string(&mut self, value: &Node, key: &str, multiline: bool) -> Option<String> {
        match value.as_str() {
            Some(text) if multiline || !text.contains(['\n', '\r']) => Some(text.to_owned()),
            _ => {
                self.problem(
                    key,
                    if multiline {
                        "must be a string"
                    } else {
                        "must be a one-line string"
                    },
                );
                None
            }
        }
    }

    fn field(
        &mut self,
        node: &Node,
        key: &str,
        parent: &str,
        required: bool,
        multiline: bool,
    ) -> Option<String> {
        let path = key_path(parent, key);
        let Some(value) = present(node, key) else {
            if required {
                self.problem(&path, "is required");
            }
            return None;
        };
        let value = self.string(value, &path, multiline)?;
        if required && value.trim().is_empty() {
            self.problem(&path, "must not be empty");
            return None;
        }
        Some(value)
    }

    // The entries of a map, each named as written: `200:` is the name "200".
    fn mapping<'a>(&mut self, value: &'a Node, key: &str) -> Vec<(String, &'a Node)> {
        let Some(entries) = value.entries() else {
            self.problem(key, "must be a map");
            return Vec::new();
        };
        entries
            .into_iter()
            .filter_map(|(name, value)| match name.name() {
                // A key holding a line break is a name all the same: text output writes the
                // break out, so that its findings stay on their lines.
                Some(name) if !name.is_empty() => Some((name.to_owned(), value)),
                _ => {
                    // A key that is not a name has no dotted path to name it by.
                    let location = if key.is_empty() {
                        "key".to_owned()
                    } else {
                        format!("{key} key")
                    };
                    self.problems.push(Problem::new(
                        Place::File,
                        format!("{location}: must be a one-line string"),
                    ));
                    None
                }
            })
            .collect()
    }

    fn record(&mut self, node: &Node, parent: &str, allowed_keys: &[&str]) {
        for (key, value) in self.mapping(node, parent) {
            // A key with nothing after it is as if it were not written.
            if !allowed_keys.contains(&key.as_str()) && !value.is_empty_value() {
                self.problem(&key_path(parent, &key), "unknown key");
            }
        }
    }

    fn map_field<'a>(&mut self, node: &'a Node, key: &str) -> Vec<(String, &'a Node)> {
        present(node, key)
            .map(|value| self.mapping(value, key))
            .unwrap_or_default()
    }

    fn sequence<'a>(&mut self, node: &'a Node, key: &str) -> &'a [Node] {
        let Some(value) = present(node, key) else {
            return &[];
        };
        if let Some(sequence) = value.as_sequence() {
            sequence
        } else {
            self.problem(key, "must be a list");
            &[]
        }
    }

    fn strings(&mut self, node: &Node, key: &str, multiline: bool) -> Vec<String> {
        self.sequence(node, key)
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                self.string(value, &format!("{key}.{}", index + 1), multiline)
            })
            .collect()
    }

    // An operation's command: any string, one line asked for but several read.
    fn command(&mut self, value: &Node, key: &str) -> Option<String> {
        let Some(command) = value.as_str() else {
            self.problem(key, "must be a one-line string");
            return None;
        };
        if command.trim().is_empty() {
            return None;
        }
        if command
            .strip_suffix('\n')
            .unwrap_or(command)
            .contains(['\n', '\r'])
        {
            self.readable(
                key,
                "must be one line; put the steps in a script and name the script here",
            );
        }
        Some(command.to_owned())
    }
}

pub(crate) fn project(text: &str) -> Parsed<Project> {
    let mut check = Check::default();
    let node = match document(text, &mut check) {
        Ok(node) => node,
        Err(error) => return Parsed::failed(error),
    };
    check.record(
        &node,
        "",
        &[
            "name",
            "stack",
            "description",
            "watch_out",
            "operations",
            "environments",
        ],
    );
    let name = check
        .field(&node, "name", "", true, false)
        .unwrap_or_default();
    let stack = check.field(&node, "stack", "", false, false);
    let description = check.field(&node, "description", "", false, false);
    let watch_out = check.strings(&node, "watch_out", false);
    if let Some(items) = present(&node, "watch_out").and_then(Node::as_sequence) {
        if items.len() > 7 {
            check.readable("watch_out", "must have at most 7 items");
        }
        for (index, item) in items.iter().enumerate() {
            if let Some(text) = item.as_str() {
                check.max_length(text, &format!("watch_out.{}", index + 1), 160);
            }
        }
    }
    let mut operations = Vec::new();
    for (name, value) in check.map_field(&node, "operations") {
        let key = key_path("operations", &name);
        let (command, note) = if value.is_empty_value() {
            (None, None)
        } else if value.is_mapping() {
            check.record(value, &key, &["command", "note"]);
            (
                present(value, "command")
                    .and_then(|command| check.command(command, &key_path(&key, "command"))),
                check.field(value, "note", &key, false, false),
            )
        } else if value.is_string() {
            (check.command(value, &key), None)
        } else {
            check.problem(&key, "must be a command string or a map");
            continue;
        };
        if let Some(note) = &note {
            check.max_length(note, &key_path(&key, "note"), 160);
        }
        match command {
            Some(command) => operations.push((name, Operation { command, note })),
            None => check.readable(&key, "has no command; write the command that runs it"),
        }
    }
    let mut environments = Vec::new();
    for (name, value) in check.map_field(&node, "environments") {
        let key = key_path("environments", &name);
        if value.is_empty_value() {
            check.readable(&key, "has no value; write where the environment runs");
            continue;
        }
        if let Some(value) = check.string(value, &key, false) {
            check.max_length(&value, &key, 160);
            environments.push((name, value));
        }
    }
    Parsed {
        value: Some(Project {
            name,
            stack,
            description,
            watch_out,
            operations,
            environments,
        }),
        problems: check.problems,
    }
}

pub(crate) fn process(text: &str) -> Parsed<Process> {
    let mut check = Check::default();
    let node = match document(text, &mut check) {
        Ok(node) => node,
        Err(error) => return Parsed::failed(error),
    };
    check.record(
        &node,
        "",
        &[
            "name",
            "description",
            "group",
            "entrypoints",
            "files",
            "try",
            "outcomes",
            "why",
            "checked",
        ],
    );
    let name = check
        .field(&node, "name", "", true, false)
        .unwrap_or_default();
    let description = check
        .field(&node, "description", "", true, false)
        .unwrap_or_default();
    let group = check.field(&node, "group", "", false, false);
    let mut entrypoints = Vec::new();
    let mut seen = HashSet::new();
    for (index, value) in check.sequence(&node, "entrypoints").iter().enumerate() {
        let key = format!("entrypoints.{}", index + 1);
        let Some(text) = check.string(value, &key, false) else {
            continue;
        };
        match EntrypointRef::parse(&text) {
            Ok(reference) => {
                if !seen.insert(reference.clone()) {
                    check.problem(&key, format!("duplicate entrypoint {reference}"));
                }
                entrypoints.push(Located {
                    key,
                    value: reference,
                });
            }
            Err(error) => check.problem(&key, error),
        }
    }
    let mut files = Vec::new();
    for (index, value) in check.sequence(&node, "files").iter().enumerate() {
        let key = format!("files.{}", index + 1);
        let (path, note) = if value.is_string() {
            (check.string(value, &key, false), None)
        } else if value.is_mapping() {
            check.record(value, &key, &["path", "note"]);
            (
                check.field(value, "path", &key, true, false),
                check.field(value, "note", &key, false, false),
            )
        } else {
            check.problem(&key, "must be a path string or a map");
            (None, None)
        };
        if let Some(path) = path {
            files.push(Located {
                key: if value.is_mapping() {
                    key_path(&key, "path")
                } else {
                    key
                },
                value: CodeFile { path, note },
            });
        }
    }
    let try_text = check.field(&node, "try", "", false, true);
    let mut outcomes = Vec::new();
    for (name, value) in check.map_field(&node, "outcomes") {
        let key = key_path("outcomes", &name);
        if value.is_empty_value() {
            continue;
        }
        if !value.is_mapping() {
            check.problem(&key, "must be a map");
            continue;
        }
        check.record(value, &key, &["when", "then"]);
        let when = check.field(value, "when", &key, true, false);
        let then = check.field(value, "then", &key, true, false);
        if let (Some(when), Some(then)) = (when, then) {
            outcomes.push((name, Outcome { when, then }));
        }
    }
    let why = check.strings(&node, "why", true);
    let checked = present(&node, "checked").and_then(|value| {
        if let Some(text) = value
            .as_str()
            .filter(|text| crate::dates::parse_with_time(text).is_some())
        {
            Some(text.to_owned())
        } else {
            check.problem(
                "checked",
                "must be an existing date written YYYY-MM-DD, or a date and time written \
                 YYYY-MM-DD HH:MM",
            );
            None
        }
    });
    Parsed {
        value: Some(Process {
            name,
            description,
            group,
            entrypoints,
            files,
            try_text,
            outcomes,
            why,
            checked,
        }),
        problems: check.problems,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines<T>(parsed: &Parsed<T>) -> Vec<String> {
        parsed.problems.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn empty_values_are_absent_and_the_file_stays_readable() {
        let parsed = process(
            "\u{feff}name: Refund\ndescription: Refunds an order.\ngroup:\ntry: \"\"\nwhy:\nchecked:\ncolour:\noutcomes:\n  refunded:\n  200:\n    when: paid\n    then: refunded\nfiles:\n  - path: src/refund.rs\n    note:\n",
        );
        assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);
        let process = parsed.value.unwrap();
        assert_eq!(process.group, None);
        assert_eq!(process.try_text, None);
        assert!(process.why.is_empty());
        let names: Vec<_> = process
            .outcomes
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(names, ["200"]);
        assert_eq!(process.files[0].value.note, None);
        let parsed = super::process("name: \"\"\ndescription: Refunds an order.\n");
        assert_eq!(lines(&parsed), ["name: is required"]);
    }

    #[test]
    fn commands_environments_and_repeated_keys_are_findings_the_file_is_read_with() {
        let parsed = project(
            "name: Ledger\noperations:\n  setup: |\n    make setup\n  deploy:\n  release:\n    command:\n  ci: |\n    bundle install\n    bin/test\n  1.0: make one\nenvironments:\n  8080: localhost:8080\n  staging:\nname: Ledgerline\n",
        );
        assert_eq!(
            lines(&parsed),
            [
                "name: written twice, on lines 1 and 15; the value on line 15 is the one read",
                "operations.deploy: has no command; write the command that runs it",
                "operations.release: has no command; write the command that runs it",
                "operations.ci: must be one line; put the steps in a script and name the script here",
                "environments.staging: has no value; write where the environment runs",
            ]
        );
        let project = parsed.readable().unwrap();
        assert_eq!(project.name, "Ledgerline");
        let operations: Vec<_> = project
            .operations
            .iter()
            .map(|(name, operation)| (name.as_str(), operation.command.as_str()))
            .collect();
        assert_eq!(
            operations,
            [
                ("setup", "make setup\n"),
                ("ci", "bundle install\nbin/test\n"),
                ("1.0", "make one")
            ]
        );
        assert_eq!(
            project.environments,
            [("8080".to_owned(), "localhost:8080".to_owned())]
        );
    }

    #[test]
    fn yaml_12_strings_and_mapping_order_are_preserved() {
        let parsed = project(
            "name: on\noperations:\n  z-last: run-last\n  a-first:\n    command: run-first\n    note: 'yes'\nenvironments:\n  z: one\n  a: two\n",
        );
        assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);
        let project = parsed.value.unwrap();
        assert_eq!(project.name, "on");
        assert_eq!(project.operations[0].0, "z-last");
        assert_eq!(project.operations[1].1.note.as_deref(), Some("yes"));
        assert_eq!(project.environments[0].0, "z");
    }

    #[test]
    fn invalid_fields_do_not_discard_valid_references() {
        let parsed = process(
            "name: Export\ndescription: Exports orders.\nwhy: not-a-list\nentrypoints:\n  - job ExportOrders\nfiles:\n  - src/export.rs\n  - path: src/order.rs\n    note: []\noutcomes:\n  exported:\n    when: true\n    then: done\n",
        );
        for key in ["why", "files.2.note", "outcomes.exported.when"] {
            assert!(
                parsed
                    .problems
                    .iter()
                    .any(|problem| problem.place == Place::Key(key.into())),
                "{key}: {:?}",
                parsed.problems
            );
        }
        assert!(parsed.readable().is_none());
        let process = parsed.value.unwrap();
        assert_eq!(process.entrypoints[0].value.to_string(), "job ExportOrders");
        assert_eq!(process.files[0].value.path, "src/export.rs");
    }

    #[test]
    fn invalid_list_items_do_not_shift_later_diagnostic_positions() {
        let parsed = process(
            "name: Prune\ndescription: Remove old plants.\nentrypoints:\n  - false\n  - job Prune\n  - unknown Prune\n  - job Prune\nfiles:\n  - 17\n  - src/prune.rs\n  - {}\n  - path: src/garden.rs\n",
        );
        assert!(
            lines(&parsed)
                .iter()
                .any(|problem| { problem == "entrypoints.4: duplicate entrypoint job Prune" })
        );
        let process = parsed.value.unwrap();
        assert_eq!(process.entrypoints[0].key, "entrypoints.2");
        assert_eq!(process.entrypoints[1].key, "entrypoints.4");
        assert_eq!(process.files[0].key, "files.2");
        assert_eq!(process.files[1].key, "files.4.path");
    }

    #[test]
    fn literal_and_folded_text_have_yaml_semantics() {
        let parsed = process(
            "name: Check stock\ndescription: Check available stock.\ntry: |\n  First line.\n  Second line.\nwhy:\n  - >-\n    Keep inventory\n    accurate.\n",
        );
        let process = parsed.readable().unwrap();
        assert_eq!(
            process.try_text.as_deref(),
            Some("First line.\nSecond line.\n")
        );
        assert_eq!(process.why, ["Keep inventory accurate."]);
    }

    #[test]
    fn malformed_and_multiple_documents_are_rejected() {
        let unfinished = process("name: [unfinished");
        assert!(unfinished.value.is_none());
        assert_eq!(unfinished.problems[0].place, Place::File);
        assert!(project("name: One\n---\nname: Two\n").value.is_none());
        assert!(project("[one, two]").value.is_none());
    }

    #[test]
    fn unknown_keys_are_reported_in_every_record_shape() {
        let project = project(
            "name: Inventory\nowner: someone\noperations:\n  export:\n    command: bin/export\n    environment: local\n",
        );
        assert_eq!(
            lines(&project),
            [
                "owner: unknown key",
                "operations.export.environment: unknown key"
            ]
        );

        let process = process(
            "name: Export\ndescription: Export orders.\nretries: 3\nfiles:\n  - path: src/export.rs\n    purpose: Converts orders.\noutcomes:\n  ready:\n    when: processing finishes\n    then: download becomes available\n    because: a file was saved\n",
        );
        assert_eq!(
            lines(&process),
            [
                "retries: unknown key",
                "files.1.purpose: unknown key",
                "outcomes.ready.because: unknown key"
            ]
        );
    }

    #[test]
    fn a_key_that_is_not_a_string_is_named_by_where_it_is_without_a_key_path() {
        let parsed = project("name: Inventory\noperations:\n  [1, 2]: make\n");
        assert_eq!(
            lines(&parsed),
            ["operations key: must be a one-line string"]
        );
        assert_eq!(parsed.problems[0].place, Place::File);
    }

    #[test]
    fn wrong_nested_shapes_name_their_paths_without_trailing_dots() {
        let project = project("name: Inventory\noperations:\n  export: []\n");
        assert_eq!(
            lines(&project),
            ["operations.export: must be a command string or a map"]
        );
        let process = process(
            "name: Export\ndescription: Export orders.\nfiles: [17]\noutcomes:\n  ready: done\n",
        );
        assert_eq!(
            lines(&process),
            [
                "files.1: must be a path string or a map",
                "outcomes.ready: must be a map"
            ]
        );
    }

    #[test]
    fn watch_out_limits_count_unicode_characters_and_all_list_items() {
        let at_limit = format!(
            "name: Inventory\nwatch_out:\n{}",
            format!("  - {}\n", "庭".repeat(160)).repeat(7)
        );
        let parsed = project(&at_limit);
        assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);

        let too_long = project(&format!(
            "name: Inventory\nwatch_out:\n  - {}\n",
            "庭".repeat(161)
        ));
        assert_eq!(
            lines(&too_long),
            ["watch_out.1: must have at most 160 characters"]
        );

        let too_many = project(&format!("{at_limit}  - 17\n"));
        assert!(
            lines(&too_many)
                .iter()
                .any(|problem| problem == "watch_out: must have at most 7 items")
        );
        assert!(
            lines(&too_many)
                .iter()
                .any(|problem| problem.starts_with("watch_out.8:"))
        );
    }

    #[test]
    fn values_over_a_limit_are_read_whole_and_a_wrong_shape_is_not() {
        let long = "x".repeat(161);
        let parsed = project(&format!(
            "name: Inventory\nwatch_out:\n{}operations:\n  test:\n    command: make test\n    note: {long}\nenvironments:\n  staging: {long}\n",
            "  - Careful.\n".repeat(8)
        ));
        assert_eq!(parsed.problems.len(), 3);
        assert_eq!(parsed.reasons(), "");
        let project_value = parsed.readable().unwrap();
        assert_eq!(project_value.watch_out.len(), 8);
        assert_eq!(
            project_value.operations[0].1.note.as_deref(),
            Some(long.as_str())
        );
        assert_eq!(project_value.environments[0].1, long);

        let wrong_shape = project(&format!(
            "name: Inventory\nwatch_out: Careful.\nenvironments:\n  staging: {long}\n"
        ));
        assert!(wrong_shape.readable().is_none());
        assert_eq!(wrong_shape.reasons(), "watch_out: must be a list");
    }
}
