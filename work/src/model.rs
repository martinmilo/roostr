use crate::characters::{written, written_file_name, written_name};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Kind {
    Http,
    Job,
    Mail,
    Schedule,
    Event,
    Cli,
}

impl Kind {
    pub(crate) const ALL: [Self; 6] = [
        Self::Http,
        Self::Job,
        Self::Mail,
        Self::Schedule,
        Self::Event,
        Self::Cli,
    ];

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Job => "job",
            Self::Mail => "mail",
            Self::Schedule => "schedule",
            Self::Event => "event",
            Self::Cli => "cli",
        }
    }

    pub(crate) fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == text)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct EntrypointRef {
    pub(crate) kind: Kind,
    pub(crate) name: String,
}

impl EntrypointRef {
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        let (kind, name) = text
            .split_once(' ')
            .ok_or_else(|| format!("expected '<kind> <name>', got {text:?}"))?;
        let kind = Kind::parse(kind).ok_or_else(|| format!("unknown entrypoint kind {kind:?}"))?;
        if name.trim().is_empty() {
            return Err("entrypoint name must not be empty".into());
        }
        Ok(Self {
            kind,
            name: name.to_owned(),
        })
    }
}

impl std::fmt::Display for EntrypointRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.kind.as_str(), self.name)
    }
}

#[derive(Debug)]
pub(crate) struct Project {
    pub(crate) name: String,
    pub(crate) stack: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) watch_out: Vec<String>,
    pub(crate) operations: Vec<(String, Operation)>,
    pub(crate) environments: Vec<(String, String)>,
}

#[derive(Debug)]
pub(crate) struct Operation {
    pub(crate) command: String,
    pub(crate) note: Option<String>,
}

#[derive(Debug)]
pub(crate) struct Process {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) group: Option<String>,
    pub(crate) entrypoints: Vec<Located<EntrypointRef>>,
    pub(crate) files: Vec<Located<CodeFile>>,
    pub(crate) try_text: Option<String>,
    pub(crate) outcomes: Vec<(String, Outcome)>,
    pub(crate) why: Vec<String>,
    pub(crate) checked: Option<String>,
}

#[derive(Debug)]
pub(crate) struct Located<T> {
    // Retain the original YAML key when other invalid list items are discarded.
    pub(crate) key: String,
    pub(crate) value: T,
}

#[derive(Debug)]
pub(crate) struct CodeFile {
    pub(crate) path: String,
    pub(crate) note: Option<String>,
}

#[derive(Debug)]
pub(crate) struct Outcome {
    pub(crate) when: String,
    pub(crate) then: String,
}

/// Where in a file a finding is: the whole file, a key as a dotted path, or a line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Place {
    File,
    Key(String),
    Line(usize),
}

/// A problem with one file's content, without the file.
#[derive(Clone, Debug)]
pub(crate) struct Problem {
    pub(crate) place: Place,
    pub(crate) message: String,
    /// Reported, but the file can still be read: a limit on length or count, a key written
    /// twice, an operation with no command or one of several lines, an environment with no
    /// value.
    pub(crate) readable: bool,
}

impl Problem {
    pub(crate) fn new(place: Place, message: impl Into<String>) -> Self {
        Self {
            place,
            message: message.into(),
            readable: false,
        }
    }
}

/// A problem as text output writes it: a key with what does not show written out, a tab and
/// a line break among them (see `characters::written`).
impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = written(&self.message);
        match &self.place {
            Place::File => write!(f, "{message}"),
            Place::Key(key) => write!(f, "{}: {message}", written_name(key)),
            Place::Line(line) => write!(f, "line {line}: {message}"),
        }
    }
}

/// A finding of `validate`: a problem and the file it is in.
#[derive(Clone, Debug)]
pub(crate) struct Finding {
    pub(crate) file: String,
    pub(crate) problem: Problem,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", written_file_name(&self.file), self.problem)
    }
}

#[derive(Debug)]
pub(crate) struct Parsed<T> {
    // Keep valid fields available to validation, even when another field is invalid.
    pub(crate) value: Option<T>,
    pub(crate) problems: Vec<Problem>,
}

impl<T> Parsed<T> {
    pub(crate) fn failed(message: String) -> Self {
        Self {
            value: None,
            problems: vec![Problem::new(Place::File, message)],
        }
    }

    /// The value, when no problem but a limit stands in the way of reading it.
    pub(crate) fn readable(&self) -> Option<&T> {
        if self.problems.iter().all(|problem| problem.readable) {
            self.value.as_ref()
        } else {
            None
        }
    }

    /// The problems that make the file unreadable, in their order.
    pub(crate) fn reasons(&self) -> String {
        self.problems
            .iter()
            .filter(|problem| !problem.readable)
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ")
    }
}
