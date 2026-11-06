use crate::characters::written;
use crate::git::{Changes, Git, Unreadable};
use crate::repository::{PROJECT_FILE, Repository};
use crate::{check, dates::Period, init, json, map, render, scan, session};

pub(crate) struct Response {
    pub(crate) code: u8,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    /// What is lost when stdout cannot be written: the code stays. `session start` has done its
    /// work by then, and a hook's stdout that cannot be written loses the map and nothing else.
    pub(crate) stdout_optional: bool,
    /// Whether stdout is JSON whatever the command line: the map `session start` prints.
    json: bool,
}

impl Response {
    fn output(code: u8, stdout: String) -> Self {
        Self {
            code,
            stdout,
            stderr: String::new(),
            stdout_optional: false,
            json: false,
        }
    }

    fn error(code: u8, message: impl std::fmt::Display) -> Self {
        Self {
            code,
            stdout: String::new(),
            stderr: format!("{message}\n"),
            stdout_optional: false,
            json: false,
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
    Check(check::Scope<'a>),
    SessionStart,
}

const USAGE: &str = "Usage: roostr [show <id> | changes [--since <7d|YYYY-MM-DD>] | check [--base <commit> | --staged | --session] | validate | scan | init | session start]";

const CHANGES_USAGE: &str = "Usage: roostr changes [--since <7d|YYYY-MM-DD>]";

const CHECK_USAGE: &str = "Usage: roostr check [--base <commit> | --staged | --session]";

const SESSION_USAGE: &str = "Usage: roostr session start (reads the hook's JSON on stdin)";

const HELP: &str = "
  roostr             Show the project overview
  roostr show <id>   Show a process
  roostr changes     Show changes (defaults to --since 7d)
  roostr check       Report what in .roostr/ is behind the code. With
                     --base <commit>, only the work done after that commit;
                     with --staged, only the commit being made; with
                     --session, what an agent did in its session (a Claude
                     Code hook's JSON on stdin)
  roostr validate    Check project records and references
  roostr scan        Run the scan script and update entrypoints
  roostr init        Create project metadata and print setup instructions
  roostr session start
                     Note the commit an agent's session starts at, for
                     check --session (a Claude Code hook's JSON on stdin)
  --json             Print the result as JSON; scan, init and session start
                     do not take it
  --help, -h         Print this help
  roostr --version   Print the version
";

// `check [--base <commit> | --base=<commit> | --staged]`
fn check_scope<'a>(arguments: &[&'a str]) -> Result<check::Scope<'a>, String> {
    let mut base = None;
    let mut staged = false;
    let mut session = false;
    let mut arguments = arguments.iter();
    while let Some(&argument) = arguments.next() {
        let value = if argument == "--staged" {
            staged = true;
            continue;
        } else if argument == "--session" {
            session = true;
            continue;
        } else if argument == "--base" {
            arguments.next().copied()
        } else if let Some(value) = argument.strip_prefix("--base=") {
            Some(value)
        } else if argument.starts_with('-') {
            return Err(format!(
                "roostr check does not take the option {argument:?}. {CHECK_USAGE}"
            ));
        } else {
            return Err(format!(
                "roostr check takes no argument {argument:?}. {CHECK_USAGE}"
            ));
        };
        let Some(value) = value.filter(|value| !value.is_empty()) else {
            return Err(format!("--base needs a commit. {CHECK_USAGE}"));
        };
        if base.replace(value).is_some() {
            return Err(format!("--base can be given once. {CHECK_USAGE}"));
        }
    }
    let given: Vec<&str> = [
        base.map(|_| "--base"),
        staged.then_some("--staged"),
        session.then_some("--session"),
    ]
    .into_iter()
    .flatten()
    .collect();
    if let [first, second, ..] = given[..] {
        return Err(format!(
            "{first} and {second} cannot be used together: each says what the work is. \
             {CHECK_USAGE}"
        ));
    }
    Ok(match base {
        Some(base) => check::Scope::Base(base),
        None if staged => check::Scope::Staged,
        // What the hook passed comes from stdin, read once the command line is known to be
        // right.
        None if session => check::Scope::Session(None),
        None => check::Scope::Whole,
    })
}

/// What `check --session` prints when the agent is in plan mode.
const NOT_JUDGED: &str =
    "Not judged: the session is in plan mode, where the agent cannot change files.";

fn help() -> Response {
    Response::output(0, format!("{USAGE}\n{HELP}"))
}

fn no_json(name: &str, usage: &str) -> Response {
    Response::error(
        2,
        format!("roostr {name} does not take the option \"--json\". {usage}"),
    )
}

// The first argument that is an option the command does not know, up to `--`. An option in
// `takes_value` takes the argument after it, or its value after `=`.
fn unknown_option<'a>(arguments: &[&'a str], takes_value: &[&str]) -> Option<&'a str> {
    let mut arguments = arguments.iter();
    while let Some(&argument) = arguments.next() {
        if argument == "--" {
            return None;
        }
        if takes_value.contains(&argument) {
            arguments.next();
        } else if argument.len() > 1
            && argument.starts_with('-')
            && !takes_value.iter().any(|option| {
                argument
                    .strip_prefix(option)
                    .is_some_and(|rest| rest.starts_with('='))
            })
        {
            return Some(argument);
        }
    }
    None
}

fn parse<'a>(args: &[&'a str], json: bool) -> Result<Command<'a>, Response> {
    // `--help` among the arguments of a command, before `--`, is the one answer: never the
    // value of another option.
    if let [
        "show" | "validate" | "scan" | "init" | "changes" | "check" | "session",
        ref rest @ ..,
    ] = *args
    {
        if rest
            .iter()
            .take_while(|argument| **argument != "--")
            .any(|argument| matches!(*argument, "--help" | "-h"))
        {
            return Err(help());
        }
    }
    if let [
        name @ ("show" | "validate" | "scan" | "init" | "changes"),
        ref rest @ ..,
    ] = *args
    {
        let (takes_value, usage): (&[&str], String) = match name {
            "changes" => (&["--since"], CHANGES_USAGE.to_owned()),
            "show" => (&[], "Usage: roostr show <id>".to_owned()),
            _ => (&[], format!("Usage: roostr {name} (no arguments)")),
        };
        if let Some(option) = unknown_option(rest, takes_value) {
            return Err(Response::error(
                2,
                format!("roostr {name} does not take the option {option:?}. {usage}"),
            ));
        }
    }
    Ok(match *args {
        [] => Command::Overview,
        ["show", "--", id] => Command::Show(id),
        ["show", id] if id != "--" => Command::Show(id),
        ["validate"] => Command::Validate,
        [name @ ("scan" | "init")] if json => {
            return Err(no_json(
                name,
                &format!("Usage: roostr {name} (no arguments)"),
            ));
        }
        ["scan"] => Command::Scan,
        ["init"] => Command::Init,
        ["changes"] => Command::Changes("7d"),
        ["changes", "--since", when] => Command::Changes(when),
        ["changes", argument] => {
            let Some(when) = argument.strip_prefix("--since=") else {
                return Err(Response::error(2, CHANGES_USAGE));
            };
            Command::Changes(when)
        }
        ["check", ref arguments @ ..] => match check_scope(arguments) {
            Ok(scope) => Command::Check(scope),
            Err(message) => return Err(Response::error(2, message)),
        },
        ["--version" | "-V"] if json => return Err(no_json("--version", USAGE)),
        ["session", "start"] if json => return Err(no_json("session start", SESSION_USAGE)),
        ["session", "start"] => Command::SessionStart,
        ["session", ..] => {
            return Err(Response::error(
                2,
                format!("roostr session takes one word, start. {SESSION_USAGE}"),
            ));
        }
        ["--version" | "-V"] => {
            return Err(Response::output(
                0,
                format!("roostr {}\n", env!("CARGO_PKG_VERSION")),
            ));
        }
        ["--help" | "-h"] => return Err(help()),
        ["show", ..] => {
            return Err(Response::error(
                2,
                "Usage: roostr show <id> (exactly one process id is required)",
            ));
        }
        [command @ ("validate" | "scan" | "init"), ..] => {
            return Err(Response::error(
                2,
                format!("Usage: roostr {command} (no arguments)"),
            ));
        }
        ["changes", ..] => return Err(Response::error(2, CHANGES_USAGE)),
        [command, ..] => {
            return Err(Response::error(
                2,
                format!("Unknown command {command:?}. {USAGE}"),
            ));
        }
    })
}

/// What the command prints and its code. Text output, on stdout and stderr, writes each
/// character `validate` reports as `<U+XXXX>`, whatever printed it: a raw ESC never reaches the
/// terminal, and what does not show never reaches an agent unseen. JSON is left as it is.
pub(crate) fn run(args: &[String]) -> Response {
    let mut response = respond(args);
    let json = response.json || args.iter().any(|argument| argument == "--json");
    if !json {
        if let std::borrow::Cow::Owned(stdout) = written(&response.stdout) {
            response.stdout = stdout;
        }
    }
    if let std::borrow::Cow::Owned(stderr) = written(&response.stderr) {
        response.stderr = stderr;
    }
    response
}

fn respond(args: &[String]) -> Response {
    // `--json` may stand anywhere, and is never another option's value or an argument.
    let json = args.iter().any(|argument| argument == "--json");
    let args: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|argument| *argument != "--json")
        .collect();
    let command = match parse(&args, json) {
        Ok(command) => command,
        Err(response) => return response,
    };
    // Stdin is read only by `session start` and `check --session`, once the command line is
    // known to be right.
    let hook = match command {
        Command::SessionStart | Command::Check(check::Scope::Session(_)) => {
            Some(session::Hook::read())
        }
        _ => None,
    };
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        // The folder was deleted while the shell that started roostr was in it.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return match command {
                Command::SessionStart => Response::error(
                    0,
                    "roostr session start: the current folder no longer exists; nothing recorded",
                ),
                _ => Response::error(2, "Cannot run: the current folder no longer exists."),
            };
        }
        Err(error) => return Response::error(2, format!("Cannot read current directory: {error}")),
    };
    if let (Command::SessionStart, Some(hook)) = (&command, &hook) {
        // Two things, neither depending on the other: the session is recorded, and the agent
        // is given the project map. What a SessionStart hook prints on stdout goes into the
        // agent's context: the map's line of JSON, and nothing else. A stdout that cannot be
        // written loses the map and nothing else.
        let mut response = Response::output(0, String::new());
        response.stdout_optional = true;
        response.json = true;
        if let Err(reason) = session::start(&cwd, hook.session_id()) {
            response.stderr.push_str(&format!("{reason}\n"));
        }
        if hook.wants_map() {
            match map::hook_output(&cwd) {
                Ok(output) => response.stdout = output.unwrap_or_default(),
                Err(reason) => response.stderr.push_str(&format!("{reason}\n")),
            }
        }
        return response;
    }
    let command = match command {
        Command::Check(check::Scope::Session(_)) => {
            Command::Check(check::Scope::Session(hook.as_ref()))
        }
        command => command,
    };
    let root = Repository::discover(&cwd);
    if let Command::Init = command {
        let root = root.as_deref().unwrap_or(&cwd);
        return match init::create(root) {
            Ok(created) => Response::output(0, init::report(created, root)),
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
    if let Command::Check(scope) = command {
        if let check::Scope::Session(Some(hook)) = scope {
            if hook.in_plan_mode() {
                check::not_judged(&root, hook);
                return Response::output(
                    0,
                    if json {
                        json::not_judged(hook)
                    } else {
                        format!("{NOT_JUDGED}\n")
                    },
                );
            }
        }
        let report = match check::run(&root, scope) {
            Ok(report) => report,
            Err(error) => return Response::error(2, error),
        };
        let mut response = if json {
            Response::output(u8::from(!report.is_empty()), json::check(scope, &report))
        } else if check::let_through(&report, scope) {
            Response::output(0, format!("{}\n", check::NOT_HELD_AGAIN))
        } else {
            Response::output(u8::from(!report.is_empty()), render::check(&report))
        };
        if let Some(warning) = &report.warning {
            response.stderr = format!("{warning}\n");
        }
        return response;
    }
    let repository = Repository::load(root);
    if let Command::Validate = command {
        let findings = repository.validate();
        let code = u8::from(!findings.is_empty());
        return Response::output(
            code,
            if json {
                json::validate(&findings)
            } else if findings.is_empty() {
                "No problems found\n".into()
            } else {
                findings
                    .iter()
                    .map(|finding| format!("{finding}\n"))
                    .collect()
            },
        );
    }
    let Some(project) = repository.project.readable() else {
        return Response::error(
            2,
            format!(
                "Cannot read {PROJECT_FILE}: {}. Run roostr validate for details.",
                repository.project.reasons()
            ),
        );
    };
    match command {
        Command::Overview => {
            // Git that cannot be read leaves out what the overview reads from it, never the
            // rest. LAST 7 DAYS is not in the JSON, so then it is not read.
            let from_git = match Git::open(&repository.root) {
                Ok(None) => Ok(None),
                Ok(Some(git)) => {
                    let period = if json {
                        None
                    } else {
                        match Period::since("7d") {
                            Ok(period) => Some(period),
                            Err(error) => return Response::error(2, error),
                        }
                    };
                    overview_from_git(&git, &repository, period.as_ref()).map(Some)
                }
                Err(error) => Err(error),
            };
            let (changes, attention, unreadable) = match from_git {
                Ok(None) => (None, None, None),
                Ok(Some((changes, attention))) => (changes, Some(attention), None),
                Err(error) => (None, None, Some(error)),
            };
            if json {
                return Response::output(
                    0,
                    json::overview(project, &repository, attention.as_deref()),
                );
            }
            // The overview lists each process's files in path order.
            let attention: Vec<_> = attention
                .unwrap_or_default()
                .into_iter()
                .flat_map(|(id, files)| {
                    let files: std::collections::BTreeSet<_> = files.into_iter().collect();
                    files.into_iter().map(move |file| (id.clone(), file))
                })
                .collect();
            let reason = unreadable.as_ref().map(|error| error.0.as_str());
            Response::output(
                0,
                render::overview(project, &repository, changes.as_ref(), &attention, reason),
            )
        }
        Command::Changes(when) => {
            let git = match Git::open(&repository.root) {
                Ok(Some(git)) => git,
                Ok(None) => {
                    return Response::error(
                        2,
                        "This is not a Git repository; roostr changes needs Git history.",
                    );
                }
                Err(error) => return Response::error(2, error),
            };
            if let Err(error) = git.whole_history() {
                return Response::error(2, error);
            }
            let period = match Period::since(when) {
                Ok(period) => period,
                Err(error) => return Response::error(2, error),
            };
            let changes = match git
                .head_history()
                .and_then(|history| git.changes(&history, &repository, &period))
            {
                Ok(changes) => changes,
                Err(error) => return Response::error(2, error),
            };
            Response::output(
                0,
                if json {
                    json::changes(&changes)
                } else {
                    render::changes(&changes)
                },
            )
        }
        Command::Show(id) => {
            let Some(record) = repository.processes.iter().find(|record| record.id == id) else {
                let mut message = format!("No process with id {id:?} exists.");
                let close = close_ids(
                    id,
                    repository.processes.iter().map(|record| record.id.as_str()),
                );
                if !close.is_empty() {
                    message.push_str(&format!("\nDid you mean: {}?", close.join(", ")));
                }
                return Response::error(1, message);
            };
            let Some(process) = record.parsed.readable() else {
                return Response::error(
                    2,
                    format!(
                        "Cannot read {}: {}. Run roostr validate for details.",
                        record.file,
                        record.parsed.reasons()
                    ),
                );
            };
            let inventory = repository.inventory.as_ref();
            Response::output(
                0,
                if json {
                    json::show(id, process, inventory)
                } else {
                    render::show(id, process, inventory)
                },
            )
        }
        Command::Validate
        | Command::Scan
        | Command::Init
        | Command::Check(_)
        | Command::SessionStart => unreachable!(),
    }
}

type Attention = Vec<(String, Vec<String>)>;

/// What the overview reads from Git: with a period, what changed in it, and the processes
/// that need attention.
fn overview_from_git(
    git: &Git,
    repository: &Repository,
    period: Option<&Period>,
) -> Result<(Option<Changes>, Attention), Unreadable> {
    git.whole_history()?;
    let mut history = None;
    let changes = match period {
        Some(period) => {
            // LAST 7 DAYS is what HEAD reaches; during a merge, what needs attention also
            // counts the commits being merged, in a walk of its own.
            let walked = git.head_history()?;
            let changes = git.changes(&walked, repository, period)?;
            if !git.is_merging() {
                history = Some(walked);
            }
            Some(changes)
        }
        None => None,
    };
    let uncommitted = git.uncommitted()?;
    let attention = git.attention(repository, &uncommitted, None, &mut history)?;
    Ok((changes, attention))
}

// Ids compared with case, `-` and `_` ignored.
fn loose(text: &str) -> String {
    text.chars()
        .filter(|ch| !matches!(ch, '-' | '_'))
        .flat_map(char::to_lowercase)
        .collect()
}

/// The ids that contain `typed`, compared loosely: at most five, in the order given. What was
/// typed is first taken as a path, so the file name of a record names its id.
fn close_ids<'a>(typed: &str, ids: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let name = typed.rsplit('/').next().unwrap_or(typed);
    let typed = loose(name.strip_suffix(".yml").unwrap_or(name));
    if typed.is_empty() {
        return Vec::new();
    }
    ids.filter(|id| loose(id).contains(&typed))
        .take(5)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_ids_ignore_case_dashes_and_underscores_and_stop_at_five() {
        let ids = [
            "book-workshop",
            "cancel-booking",
            "export-bookings",
            "fix-double-booking",
            "rebook-workshop",
            "z-booking",
            "send-reminder",
        ];
        assert_eq!(
            close_ids("Book_Workshop", ids.into_iter()),
            ["book-workshop", "rebook-workshop"]
        );
        assert_eq!(close_ids("BOOKING", ids.into_iter()).len(), 4);
        assert_eq!(close_ids("book", ids.into_iter()).len(), 5);
        assert!(close_ids("invoice", ids.into_iter()).is_empty());
        assert_eq!(
            close_ids(".roostr/processes/rebook-workshop.yml", ids.into_iter()),
            ["rebook-workshop"]
        );
        assert_eq!(
            unknown_option(&["--since", "-3d", "--since=x", "--verbose"], &["--since"]),
            Some("--verbose")
        );
        assert_eq!(unknown_option(&["--", "--verbose"], &[]), None);
    }
}
