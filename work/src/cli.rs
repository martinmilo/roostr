use crate::repository::{PROJECT_FILE, Repository};
use crate::{dates::Period, git::Git, init, render, scan};

pub(crate) struct Response {
    pub(crate) code: u8,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

impl Response {
    fn output(code: u8, stdout: String) -> Self {
        Self {
            code,
            stdout,
            stderr: String::new(),
        }
    }

    fn error(code: u8, message: impl std::fmt::Display) -> Self {
        Self {
            code,
            stdout: String::new(),
            stderr: format!("{message}\n"),
        }
    }
}

enum Command<'a> {
    Overview,
    Show(&'a str),
    Validate,
    Scan,
    Init,
    Changes(&'a str),
}

const USAGE: &str =
    "Usage: roostr [show <id> | changes [--since <7d|YYYY-MM-DD>] | validate | scan | init]";

const CHANGES_USAGE: &str = "Usage: roostr changes [--since <7d|YYYY-MM-DD>]";

pub(crate) fn run(args: &[String]) -> Response {
    let command = match args {
        [] => Command::Overview,
        [command, id] if command == "show" => Command::Show(id),
        [command] if command == "validate" => Command::Validate,
        [command] if command == "scan" => Command::Scan,
        [command] if command == "init" => Command::Init,
        [command] if command == "changes" => Command::Changes("7d"),
        [command, flag, when] if command == "changes" && flag == "--since" => {
            Command::Changes(when)
        }
        [command, argument] if command == "changes" => {
            let Some(when) = argument.strip_prefix("--since=") else {
                return Response::error(2, CHANGES_USAGE);
            };
            Command::Changes(when)
        }
        [command] if command == "--help" || command == "-h" => {
            return Response::output(
                0,
                format!(
                    "{USAGE}\n\n  roostr            Show the project overview\n  roostr show <id>   Show a process\n  roostr changes     Show changes (defaults to --since 7d)\n  roostr validate    Check project records and references\n  roostr scan        Run the scan script and update entrypoints\n  roostr init        Create project metadata and print setup instructions\n"
                ),
            );
        }
        [command, ..] if command == "show" => {
            return Response::error(
                2,
                "Usage: roostr show <id> (exactly one process id is required)",
            );
        }
        [command, ..] if matches!(command.as_str(), "validate" | "scan" | "init") => {
            return Response::error(2, format!("Usage: roostr {command} (no arguments)"));
        }
        [command, ..] if command == "changes" => {
            return Response::error(2, CHANGES_USAGE);
        }
        [command, ..] => {
            return Response::error(2, format!("Unknown command {command:?}. {USAGE}"));
        }
    };
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => return Response::error(2, format!("Cannot read current directory: {error}")),
    };
    let root = Repository::discover(&cwd);
    if let Command::Init = command {
        return match init::create(root.as_deref().unwrap_or(&cwd)) {
            Ok(created) => Response::output(0, init::report(created)),
            Err(error) => Response::error(2, format!("Cannot initialise {PROJECT_FILE}: {error}")),
        };
    }
    let Some(root) = root else {
        return Response::error(
            2,
            format!("No {PROJECT_FILE} found; run roostr init to initialise this project."),
        );
    };
    if let Command::Scan = command {
        return match scan::run(&root) {
            Ok(report) => Response::output(0, report),
            Err(error) => Response::error(error.code, error.message),
        };
    }
    let repository = Repository::load(root);
    if let Command::Validate = command {
        let problems = repository.validate();
        return if problems.is_empty() {
            Response::output(0, "No problems found\n".into())
        } else {
            Response::output(1, format!("{}\n", problems.join("\n")))
        };
    }
    let Some(project) = repository.project.valid() else {
        return Response::error(
            2,
            format!(
                "Cannot read {PROJECT_FILE}: {}. Run roostr validate for details.",
                repository.project.problems.join("; ")
            ),
        );
    };
    match command {
        Command::Overview | Command::Changes(_) => {
            let git = match Git::read(&repository.root) {
                Ok(git) => git,
                Err(error) => return Response::error(2, error),
            };
            let Some(git) = git else {
                return match command {
                    Command::Changes(_) => Response::error(
                        2,
                        "This is not a Git repository; roostr changes needs Git history.",
                    ),
                    _ => Response::output(0, render::overview(project, &repository, None, &[])),
                };
            };
            let when = if let Command::Changes(when) = command {
                when
            } else {
                "7d"
            };
            let period = match Period::since(when) {
                Ok(period) => period,
                Err(error) => return Response::error(2, error),
            };
            let changes = match git.changes(&repository, &period) {
                Ok(changes) => changes,
                Err(error) => return Response::error(2, error),
            };
            match command {
                Command::Changes(_) => Response::output(0, render::changes(&changes)),
                _ => {
                    let attention = match git.needs_attention(&repository) {
                        Ok(attention) => attention,
                        Err(error) => return Response::error(2, error),
                    };
                    Response::output(
                        0,
                        render::overview(project, &repository, Some(&changes), &attention),
                    )
                }
            }
        }
        Command::Show(id) => {
            let Some(record) = repository.processes.iter().find(|record| record.id == id) else {
                return Response::error(1, format!("No process with id {id:?} exists."));
            };
            let Some(process) = record.parsed.valid() else {
                return Response::error(
                    2,
                    format!(
                        "Cannot read {}: {}. Run roostr validate for details.",
                        record.file,
                        record.parsed.problems.join("; ")
                    ),
                );
            };
            Response::output(0, render::show(id, process, repository.inventory.as_ref()))
        }
        Command::Validate | Command::Scan | Command::Init => unreachable!(),
    }
}
