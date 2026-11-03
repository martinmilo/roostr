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

pub(crate) fn report(created: bool) -> String {
    let action = if created { "create" } else { "exists" };
    format!(
        "{action}  {PROJECT_FILE}\n\n{}",
        include_str!("templates/setup.txt")
    )
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
            assert_eq!(parsed.valid().unwrap().name, name);
        }
    }

    #[test]
    fn control_characters_in_directory_names_cannot_inject_yaml() {
        let parsed = crate::parse::project(&project_template("garden\nstack: false\u{2028}end"));
        assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);
        let project = parsed.valid().unwrap();
        assert_eq!(project.name, "garden stack: false end");
        assert_eq!(project.stack, None);
    }
}
