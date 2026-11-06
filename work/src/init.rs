use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use saphyr::{Scalar, Yaml, YamlEmitter};

use crate::repository::PROJECT_FILE;

pub(crate) fn create(root: &Path) -> io::Result<bool> {
    fs::create_dir_all(root.join(".roostr"))?;
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(PROJECT_FILE))
    {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => return Ok(false),
        Err(error) => return Err(error),
    };
    let name = root
        .file_name()
        .unwrap_or(root.as_os_str())
        .to_string_lossy();
    file.write_all(project_template(&name).as_bytes())?;
    Ok(true)
}

fn project_template(name: &str) -> String {
    // Keep the project name on one line even for unusual directory names.
    let name: String = name
        .chars()
        .map(|ch| {
            if ch.is_control() || matches!(ch, '\u{2028}' | '\u{2029}') {
                ' '
            } else {
                ch
            }
        })
        .collect();
    let name = if name.trim().is_empty() {
        "unknown".to_owned()
    } else {
        name
    };
    let mut yaml = String::new();
    YamlEmitter::new(&mut yaml)
        .dump(&Yaml::Value(Scalar::String(name.into())))
        .expect("writing YAML to a String cannot fail");
    let scalar = yaml
        .strip_prefix("---\n")
        .expect("the YAML emitter starts a document with ---");
    include_str!("templates/project.yml").replacen("{{name}}", scalar, 1)
}

/// What `init` prints. Git runs hooks, and CI starts, at the top of the work tree: below it, the
/// commands that run there first change to the project folder `root`.
pub(crate) fn report(created: bool, root: &Path) -> String {
    let action = if created { "create" } else { "exists" };
    let folder = from_top(root);
    let cd = folder
        .as_deref()
        .map(|folder| format!("cd {} && ", quoted(folder)))
        .unwrap_or_default();
    format!(
        "{action}  {PROJECT_FILE}\n\n{}",
        include_str!("templates/setup.txt")
            .replace("{{cd}}", &cd)
            .replace("{{claude}}", &claude_hooks(folder.as_deref()))
    )
}

const SESSION_START: &str = "roostr session start";
const STOP: &str = "roostr check --session >&2 || { [ $? -eq 1 ] && exit 2; }";

// The line for `.claude/settings.json`. Claude Code runs a hook in its own current folder,
// any folder of the repository: below the top, the commands first change to the project
// folder, and the Stop command goes in braces, so that a failing `cd` exits 1 and holds
// nothing.
fn claude_hooks(folder: Option<&str>) -> String {
    let (start, stop) = match folder {
        None => (SESSION_START.to_owned(), STOP.to_owned()),
        Some(folder) => {
            let cd = format!(
                "cd \"$(git rev-parse --show-toplevel)/{}\" && ",
                in_double_quotes(folder)
            );
            (format!("{cd}{SESSION_START}"), format!("{cd}{{ {STOP}; }}"))
        }
    };
    format!(
        r#"{{"hooks":{{"SessionStart":[{{"hooks":[{{"type":"command","command":"{}","timeout":10}}]}}],"Stop":[{{"hooks":[{{"type":"command","command":"{}"}}]}}]}}}}"#,
        json_string(&start),
        json_string(&stop)
    )
}

// Text for inside the double quotes of `sh`: `"`, `$`, a backquote and `\` escaped.
fn in_double_quotes(text: &str) -> String {
    let mut escaped = String::new();
    for ch in text.chars() {
        if matches!(ch, '"' | '$' | '`' | '\\') {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

// Text for inside a JSON string. A folder name could hold a control character too.
fn json_string(text: &str) -> String {
    let mut escaped = String::new();
    for ch in text.chars() {
        match ch {
            '"' | '\\' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            ch if u32::from(ch) < 0x20 => escaped.push_str(&format!("\\u{:04x}", u32::from(ch))),
            ch => escaped.push(ch),
        }
    }
    escaped
}

// The project folder's path from the top of the Git work tree it is in, the nearest folder
// that holds `.git` (a folder, or a file in a linked work tree); none at the top, or outside
// a repository. Found without running Git, which `init` never reads.
fn from_top(root: &Path) -> Option<String> {
    let top = root
        .ancestors()
        .find(|folder| folder.join(".git").exists())?;
    let folder = root.strip_prefix(top).ok()?;
    if folder.as_os_str().is_empty() {
        return None;
    }
    Some(folder.to_string_lossy().into_owned())
}

// A word for `sh`: as it is when every character is safe there, otherwise in single quotes.
fn quoted(word: &str) -> String {
    let safe = |ch: char| ch.is_ascii_alphanumeric() || "/._-+,:@%=".contains(ch);
    if !word.is_empty() && word.chars().all(safe) {
        word.to_owned()
    } else {
        format!("'{}'", word.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_names_round_trip_as_yaml_strings() {
        for name in [
            "workshop",
            "true",
            "null",
            "123",
            "1e3",
            "~",
            "[a, b]",
            "a: b",
            "a # comment",
            " leading and trailing ",
            "quotes'and\"slashes\\",
            "庭園",
            "{{name}}",
        ] {
            let parsed = crate::parse::project(&project_template(name));
            assert!(
                parsed.problems.is_empty(),
                "{name:?}: {:?}",
                parsed.problems
            );
            assert_eq!(parsed.readable().unwrap().name, name);
        }
    }

    #[test]
    fn control_characters_in_directory_names_cannot_inject_yaml() {
        let parsed = crate::parse::project(&project_template("garden\nstack: false\u{2028}end"));
        assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);
        let project = parsed.readable().unwrap();
        assert_eq!(project.name, "garden stack: false end");
        assert_eq!(project.stack, None);
    }
}
