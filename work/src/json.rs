//! `--json`: what a command found, as one JSON object. The keys and values are those of
//! "JSON output" in FORMATS.md.

use std::collections::HashMap;

use crate::check::{Entrypoints, Report, Scope};
use crate::entrypoints::{Entrypoint, Inventory};
use crate::git::Changes;
use crate::model::{EntrypointRef, Finding, Kind, Place, Process, Project};
use crate::repository::Repository;
use crate::scan::Failure;
use crate::session::Hook;

/// The version of the format; it changes only when a change could break a consumer.
const FORMAT: usize = 1;

pub(crate) enum Json {
    Null,
    Bool(bool),
    /// The decimal digits of an integer.
    Number(String),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(&'static str, Json)>),
}

impl From<&str> for Json {
    fn from(text: &str) -> Self {
        Self::String(text.to_owned())
    }
}

impl From<String> for Json {
    fn from(text: String) -> Self {
        Self::String(text)
    }
}

impl From<bool> for Json {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<usize> for Json {
    fn from(number: usize) -> Self {
        Self::Number(number.to_string())
    }
}

impl From<i32> for Json {
    fn from(number: i32) -> Self {
        Self::Number(number.to_string())
    }
}

impl<T: Into<Self>> From<Option<T>> for Json {
    fn from(value: Option<T>) -> Self {
        value.map_or(Self::Null, Into::into)
    }
}

fn list<T: Into<Json>>(items: impl IntoIterator<Item = T>) -> Json {
    Json::Array(items.into_iter().map(Into::into).collect())
}

impl Json {
    /// The value as JSON text, indented two spaces a level, with a final newline.
    pub(crate) fn text(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, 0);
        out.push('\n');
        out
    }

    fn write(&self, out: &mut String, depth: usize) {
        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
            Self::Number(digits) => out.push_str(digits),
            Self::String(text) => string(out, text),
            Self::Array(items) if items.is_empty() => out.push_str("[]"),
            Self::Object(fields) if fields.is_empty() => out.push_str("{}"),
            Self::Array(items) => {
                out.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    newline(out, depth + 1);
                    item.write(out, depth + 1);
                }
                newline(out, depth);
                out.push(']');
            }
            Self::Object(fields) => {
                out.push('{');
                for (index, (key, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    newline(out, depth + 1);
                    string(out, key);
                    out.push_str(": ");
                    value.write(out, depth + 1);
                }
                newline(out, depth);
                out.push('}');
            }
        }
    }
}

fn newline(out: &mut String, depth: usize) {
    out.push('\n');
    for _ in 0..depth {
        out.push_str("  ");
    }
}

// Every control character is escaped; anything else, non-ASCII included, is written as is.
fn string(out: &mut String, text: &str) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\0'..='\u{1f}' => out.push_str(&format!("\\u{:04x}", u32::from(ch))),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

fn object(fields: Vec<(&'static str, Json)>) -> Json {
    Json::Object(fields)
}

fn process(id: &str, process: &Process) -> Json {
    object(vec![
        ("id", id.into()),
        ("name", process.name.as_str().into()),
        ("description", process.description.as_str().into()),
        ("group", process.group.as_deref().into()),
        (
            "entrypoints",
            list(
                process
                    .entrypoints
                    .iter()
                    .map(|item| item.value.to_string()),
            ),
        ),
        (
            "files",
            list(process.files.iter().map(|item| {
                object(vec![
                    ("path", item.value.path.as_str().into()),
                    ("note", item.value.note.as_deref().into()),
                ])
            })),
        ),
        ("try", process.try_text.as_deref().into()),
        (
            "outcomes",
            list(process.outcomes.iter().map(|(name, outcome)| {
                object(vec![
                    ("name", name.as_str().into()),
                    ("when", outcome.when.as_str().into()),
                    ("then", outcome.then.as_str().into()),
                ])
            })),
        ),
        ("why", list(process.why.iter().map(String::as_str))),
        ("checked", process.checked.as_deref().into()),
    ])
}

fn project(project: &Project) -> Json {
    object(vec![
        ("name", project.name.as_str().into()),
        ("stack", project.stack.as_deref().into()),
        ("description", project.description.as_deref().into()),
        (
            "watch_out",
            list(project.watch_out.iter().map(String::as_str)),
        ),
        (
            "operations",
            list(project.operations.iter().map(|(name, operation)| {
                object(vec![
                    ("name", name.as_str().into()),
                    ("command", operation.command.as_str().into()),
                    ("note", operation.note.as_deref().into()),
                ])
            })),
        ),
        (
            "environments",
            list(project.environments.iter().map(|(name, place)| {
                object(vec![
                    ("name", name.as_str().into()),
                    ("where", place.as_str().into()),
                ])
            })),
        ),
    ])
}

fn row(entry: &Entrypoint, listed_by: &HashMap<&EntrypointRef, Vec<&str>>) -> Json {
    let reference = &entry.reference;
    object(vec![
        ("entrypoint", reference.to_string().into()),
        ("kind", reference.kind.as_str().into()),
        ("name", reference.name.as_str().into()),
        ("file", entry.file.as_str().into()),
        ("area", entry.area.as_deref().into()),
        (
            "processes",
            list(listed_by.get(reference).into_iter().flatten().copied()),
        ),
    ])
}

fn attention(id: &str, files: &[String]) -> Vec<(&'static str, Json)> {
    vec![
        ("process", id.into()),
        ("files", list(files.iter().map(String::as_str))),
    ]
}

/// The overview. `attention` is `None` when Git could not tell which processes need it.
pub(crate) fn overview(
    project_value: &Project,
    repository: &Repository,
    attention_value: Option<&[(String, Vec<String>)]>,
) -> String {
    let processes: Vec<_> = repository.readable_processes().collect();
    // Ids are in id order, as the records are.
    let mut listed_by: HashMap<&EntrypointRef, Vec<&str>> = HashMap::new();
    for (id, process) in &processes {
        for item in &process.entrypoints {
            listed_by.entry(&item.value).or_default().push(id);
        }
    }
    let entrypoints = repository.inventory.as_ref().map(|inventory| {
        object(vec![
            (
                "covers",
                list(
                    Kind::ALL
                        .into_iter()
                        .filter(|kind| inventory.covers.contains(kind))
                        .map(Kind::as_str),
                ),
            ),
            (
                "rows",
                list(inventory.entries.iter().map(|entry| row(entry, &listed_by))),
            ),
        ])
    });
    let mut needs_checking = repository.unreadable();
    needs_checking.extend(
        repository
            .problems
            .iter()
            .map(|finding| finding.file.as_str()),
    );
    object(vec![
        ("format", FORMAT.into()),
        ("project", project(project_value)),
        (
            "processes",
            list(processes.iter().map(|(id, value)| process(id, value))),
        ),
        ("entrypoints", entrypoints.into()),
        (
            "needs_attention",
            attention_value
                .map(|items| list(items.iter().map(|(id, files)| object(attention(id, files)))))
                .into(),
        ),
        (
            "needs_checking",
            list(
                needs_checking
                    .into_iter()
                    .map(|file| object(vec![("file", file.into())])),
            ),
        ),
    ])
    .text()
}

pub(crate) fn show(id: &str, value: &Process, inventory: Option<&Inventory>) -> String {
    object(vec![
        ("format", FORMAT.into()),
        ("process", process(id, value)),
        (
            "entrypoints",
            list(value.entrypoints.iter().map(|item| {
                let file = inventory
                    .and_then(|inventory| inventory.find(&item.value))
                    .map(|entry| entry.file.as_str());
                object(vec![
                    ("entrypoint", item.value.to_string().into()),
                    ("file", file.into()),
                ])
            })),
        ),
    ])
    .text()
}

pub(crate) fn changes(changes: &Changes) -> String {
    object(vec![
        ("format", FORMAT.into()),
        ("since", changes.first_day.as_str().into()),
        (
            "entrypoints",
            changes
                .entrypoints
                .as_ref()
                .map(|entrypoints| {
                    object(vec![
                        ("added", list(entrypoints.added.iter().map(String::as_str))),
                        (
                            "removed",
                            list(entrypoints.removed.iter().map(String::as_str)),
                        ),
                    ])
                })
                .into(),
        ),
        (
            "processes",
            list(changes.processes.iter().map(|(id, subjects)| {
                object(vec![
                    ("id", id.as_str().into()),
                    ("subjects", list(subjects.iter().map(String::as_str))),
                ])
            })),
        ),
    ])
    .text()
}

fn invalid(finding: &Finding) -> Json {
    let (key, line) = match &finding.problem.place {
        Place::File => (None, None),
        Place::Key(key) => (Some(key.as_str()), None),
        Place::Line(line) => (None, Some(*line)),
    };
    object(vec![
        ("kind", "invalid".into()),
        ("file", finding.file.as_str().into()),
        ("key", key.into()),
        ("line", line.into()),
        ("message", finding.problem.message.as_str().into()),
    ])
}

pub(crate) fn validate(findings: &[Finding]) -> String {
    object(vec![
        ("format", FORMAT.into()),
        ("problems", list(findings.iter().map(invalid))),
    ])
    .text()
}

fn entrypoints_problem(entrypoints: &Entrypoints) -> Json {
    let names = |references: &[String]| list(references.iter().map(String::as_str));
    match entrypoints {
        Entrypoints::Behind { added, removed } => object(vec![
            ("kind", "entrypoints_behind".into()),
            ("added", names(added)),
            ("removed", names(removed)),
        ]),
        Entrypoints::NeverScanned => object(vec![("kind", "entrypoints_never_written".into())]),
        Entrypoints::Failed(Failure::Missing(_) | Failure::NotExecutable) => {
            object(vec![("kind", "scan_not_executable".into())])
        }
        Entrypoints::Failed(Failure::CannotStart { interpreter, .. }) => object(vec![
            ("kind", "scan_cannot_start".into()),
            ("interpreter", interpreter.as_deref().into()),
        ]),
        Entrypoints::Failed(Failure::Exited { code, stderr, .. }) => object(vec![
            ("kind", "scan_failed".into()),
            ("exit_code", (*code).into()),
            (
                "stderr",
                String::from_utf8_lossy(stderr).into_owned().into(),
            ),
        ]),
        Entrypoints::Failed(Failure::Malformed { problems, .. }) => object(vec![
            ("kind", "scan_malformed".into()),
            (
                "reasons",
                list(problems.iter().map(|problem| {
                    let line = match problem.place {
                        Place::Line(line) => Some(line),
                        Place::File | Place::Key(_) => None,
                    };
                    object(vec![
                        ("line", line.into()),
                        ("message", problem.message.as_str().into()),
                    ])
                })),
            ),
        ]),
    }
}

/// What check found, in the order of the text's blocks. `base` is as the user wrote it.
pub(crate) fn check(scope: Scope<'_>, report: &Report) -> String {
    let mut problems: Vec<Json> = report
        .not_staged
        .iter()
        .map(|file| {
            object(vec![
                ("kind", "not_staged".into()),
                ("file", file.as_str().into()),
            ])
        })
        .collect();
    problems.extend(report.validate.iter().map(invalid));
    problems.extend(report.entrypoints.as_ref().map(entrypoints_problem));
    problems.extend(report.processes.iter().map(|(id, files)| {
        let mut fields = vec![("kind", "needs_attention".into())];
        fields.extend(attention(id, files));
        object(fields)
    }));
    problems.extend(report.not_described.iter().map(|reference| {
        object(vec![
            ("kind", "not_described".into()),
            ("entrypoint", reference.as_str().into()),
        ])
    }));
    object(vec![
        ("format", FORMAT.into()),
        ("base", report.base.as_deref().into()),
        ("staged", matches!(scope, Scope::Staged).into()),
        (
            "session",
            match scope {
                Scope::Session(hook) => hook.and_then(Hook::session_id).into(),
                _ => Json::Null,
            },
        ),
        ("problems", Json::Array(problems)),
    ])
    .text()
}

/// `check --session --json` when it does not judge, the session being in plan mode.
pub(crate) fn not_judged(hook: &Hook) -> String {
    object(vec![
        ("format", FORMAT.into()),
        ("base", Json::Null),
        ("staged", false.into()),
        ("session", hook.session_id().into()),
        ("problems", Json::Null),
    ])
    .text()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded(text: &str) -> String {
        let mut out = String::new();
        string(&mut out, text);
        out
    }

    #[test]
    fn every_control_character_is_escaped_and_nothing_else_is() {
        assert_eq!(encoded("a\"b\\c"), r#""a\"b\\c""#);
        assert_eq!(encoded("\n\r\t\u{8}\u{c}"), r#""\n\r\t\b\f""#);
        assert_eq!(
            encoded("\0\u{1}\u{1b}\u{1f}"),
            r#""\u0000\u0001\u001b\u001f""#
        );
        for code in 0..0x20_u32 {
            let ch = char::from_u32(code).unwrap();
            let text = encoded(&ch.to_string());
            assert!(text.chars().all(|ch| !ch.is_control()), "{code}: {text:?}");
        }
        assert_eq!(encoded(" /~\u{7f}"), "\" /~\u{7f}\"");
        assert_eq!(encoded("Café 庭園 🌱"), "\"Café 庭園 🌱\"");
    }

    #[test]
    fn values_nest_with_empty_lists_and_objects_on_one_line() {
        let value = object(vec![
            ("format", FORMAT.into()),
            ("none", Option::<&str>::None.into()),
            ("empty", list(Vec::<&str>::new())),
            ("nothing", object(Vec::new())),
            ("exit_code", (-1_i32).into()),
            ("items", list(["one", "two"])),
            ("inner", object(vec![("line", 9_usize.into())])),
        ]);
        assert_eq!(
            value.text(),
            "{\n  \"format\": 1,\n  \"none\": null,\n  \"empty\": [],\n  \"nothing\": {},\n  \
             \"exit_code\": -1,\n  \"items\": [\n    \"one\",\n    \"two\"\n  ],\n  \
             \"inner\": {\n    \"line\": 9\n  }\n}\n"
        );
    }
}
