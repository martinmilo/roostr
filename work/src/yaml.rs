//! A YAML document as roostr reads it: every scalar keeps the text it was written as, so a
//! name like `200` stays `"200"`; each key knows its line; and a key written twice in one
//! mapping is found, with the value written last the one read.

use std::collections::HashMap;

use saphyr::{Scalar, ScalarStyle};
use saphyr_parser::{Event, Parser};

/// What YAML's core schema makes of a scalar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Kind {
    String,
    Null,
    /// A number, a boolean, or a value with a tag roostr does not know.
    Other,
}

#[derive(Clone, Debug)]
pub(crate) enum Node {
    Scalar {
        /// As written, after YAML's quoting and folding.
        text: String,
        kind: Kind,
        line: usize,
    },
    Sequence(Vec<Node>),
    Mapping(Vec<(Node, Node)>),
}

/// A key written more than once in one mapping: its dotted path and the line of each.
#[derive(Debug)]
pub(crate) struct Repeated {
    pub(crate) path: String,
    pub(crate) lines: Vec<usize>,
}

#[derive(Debug)]
pub(crate) struct Document {
    pub(crate) root: Node,
    pub(crate) repeated: Vec<Repeated>,
}

impl Node {
    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Self::Scalar {
                text,
                kind: Kind::String,
                ..
            } => Some(text),
            _ => None,
        }
    }

    /// The scalar's text whatever YAML would make of it: a name, such as the key `200:`.
    pub(crate) fn name(&self) -> Option<&str> {
        match self {
            Self::Scalar { text, .. } => Some(text),
            _ => None,
        }
    }

    pub(crate) fn as_sequence(&self) -> Option<&[Self]> {
        match self {
            Self::Sequence(items) => Some(items),
            _ => None,
        }
    }

    pub(crate) fn is_mapping(&self) -> bool {
        matches!(self, Self::Mapping(_))
    }

    pub(crate) fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    /// Nothing written: a key with nothing after it, or the empty string.
    pub(crate) fn is_empty_value(&self) -> bool {
        match self {
            Self::Scalar {
                kind: Kind::Null, ..
            } => true,
            Self::Scalar {
                text,
                kind: Kind::String,
                ..
            } => text.is_empty(),
            _ => false,
        }
    }

    /// The entries of a mapping, each key once: where it was first written, with the value
    /// written last.
    pub(crate) fn entries(&self) -> Option<Vec<(&Self, &Self)>> {
        let Self::Mapping(entries) = self else {
            return None;
        };
        let mut result: Vec<(&Self, &Self)> = Vec::new();
        let mut seen: HashMap<&str, usize> = HashMap::new();
        for (key, value) in entries {
            match key.name().and_then(|name| seen.get(name)) {
                Some(&index) => result[index].1 = value,
                None => {
                    if let Some(name) = key.name() {
                        seen.insert(name, result.len());
                    }
                    result.push((key, value));
                }
            }
        }
        Some(result)
    }

    /// The value of `key` in a mapping, the one written last.
    pub(crate) fn get(&self, key: &str) -> Option<&Self> {
        let Self::Mapping(entries) = self else {
            return None;
        };
        entries
            .iter()
            .rev()
            .find(|(name, _)| name.name() == Some(key))
            .map(|(_, value)| value)
    }
}

enum Frame {
    Sequence(Vec<Node>, usize),
    Mapping {
        entries: Vec<(Node, Node)>,
        key: Option<Node>,
        anchor: usize,
    },
}

/// Reads `text` as one YAML document. An error says why it cannot be read.
pub(crate) fn load(text: &str) -> Result<Document, String> {
    let mut parser = Parser::new_from_str(text);
    let mut stack: Vec<Frame> = Vec::new();
    let mut anchors: HashMap<usize, Node> = HashMap::new();
    let mut documents = Vec::new();
    while let Some(event) = parser.next_event() {
        let (event, span) = event.map_err(|error| format!("cannot parse YAML: {error}"))?;
        let node = match event {
            Event::SequenceStart(anchor, _) => {
                stack.push(Frame::Sequence(Vec::new(), anchor));
                continue;
            }
            Event::MappingStart(anchor, _) => {
                stack.push(Frame::Mapping {
                    entries: Vec::new(),
                    key: None,
                    anchor,
                });
                continue;
            }
            Event::SequenceEnd | Event::MappingEnd => {
                let (node, anchor) = match stack.pop() {
                    Some(Frame::Sequence(items, anchor)) => (Node::Sequence(items), anchor),
                    Some(Frame::Mapping {
                        entries, anchor, ..
                    }) => (Node::Mapping(entries), anchor),
                    None => return Err("cannot parse YAML: unbalanced collection".into()),
                };
                if anchor > 0 {
                    anchors.insert(anchor, node.clone());
                }
                node
            }
            Event::Scalar(value, style, anchor, tag) => {
                let kind =
                    match Scalar::parse_from_cow_and_metadata(value.clone(), style, tag.as_ref()) {
                        // Nothing written is null in the core schema.
                        Some(Scalar::String(text))
                            if text.is_empty() && style == ScalarStyle::Plain =>
                        {
                            Kind::Null
                        }
                        Some(Scalar::String(_)) => Kind::String,
                        Some(Scalar::Null) => Kind::Null,
                        _ => Kind::Other,
                    };
                let node = Node::Scalar {
                    text: value.into_owned(),
                    kind,
                    line: span.start.line(),
                };
                if anchor > 0 {
                    anchors.insert(anchor, node.clone());
                }
                node
            }
            Event::Alias(anchor) => anchors
                .get(&anchor)
                .cloned()
                .ok_or("cannot parse YAML: an alias names no anchor")?,
            Event::StreamStart
            | Event::StreamEnd
            | Event::DocumentStart(_)
            | Event::DocumentEnd
            | Event::Nothing => continue,
        };
        match stack.last_mut() {
            None => documents.push(node),
            Some(Frame::Sequence(items, _)) => items.push(node),
            Some(Frame::Mapping { entries, key, .. }) => match key.take() {
                None => *key = Some(node),
                Some(name) => entries.push((name, node)),
            },
        }
    }
    let mut documents = documents.into_iter();
    let (Some(root), None) = (documents.next(), documents.next()) else {
        return Err("expected one YAML mapping document".into());
    };
    if !root.is_mapping() {
        return Err("expected a YAML mapping".into());
    }
    let mut repeated = Vec::new();
    find_repeated(&root, "", &mut repeated);
    Ok(Document { root, repeated })
}

fn path(parent: &str, step: &str) -> String {
    if parent.is_empty() {
        step.to_owned()
    } else {
        format!("{parent}.{step}")
    }
}

fn find_repeated(node: &Node, at: &str, repeated: &mut Vec<Repeated>) {
    match node {
        Node::Scalar { .. } => {}
        Node::Sequence(items) => {
            for (index, item) in items.iter().enumerate() {
                find_repeated(item, &path(at, &(index + 1).to_string()), repeated);
            }
        }
        Node::Mapping(entries) => {
            let mut lines: Vec<(&str, Vec<usize>)> = Vec::new();
            for (key, _) in entries {
                if let Node::Scalar { text, line, .. } = key {
                    match lines.iter_mut().find(|(name, _)| name == text) {
                        Some((_, found)) => found.push(*line),
                        None => lines.push((text, vec![*line])),
                    }
                }
            }
            for (name, lines) in lines {
                if lines.len() > 1 {
                    repeated.push(Repeated {
                        path: path(at, name),
                        lines,
                    });
                }
            }
            // A repeated key's earlier values are not read; only the one that counts is
            // looked into.
            if let Some(entries) = node.entries() {
                for (key, value) in entries {
                    if let Some(name) = key.name() {
                        find_repeated(value, &path(at, name), repeated);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalars_keep_their_text_and_kind() {
        let document =
            load("a: 200\nb: '200'\nc:\nd: \"\"\ne: true\nf: 1.0\ng: on\nh: |\n  x\n").unwrap();
        let kind = |key: &str| match document.root.get(key).unwrap() {
            Node::Scalar { kind, .. } => *kind,
            _ => unreachable!(),
        };
        assert_eq!(kind("a"), Kind::Other);
        assert_eq!(document.root.get("a").unwrap().name(), Some("200"));
        assert_eq!(kind("b"), Kind::String);
        assert_eq!(kind("c"), Kind::Null);
        assert!(document.root.get("c").unwrap().is_empty_value());
        assert!(document.root.get("d").unwrap().is_empty_value());
        assert_eq!(kind("e"), Kind::Other);
        assert_eq!(document.root.get("f").unwrap().name(), Some("1.0"));
        assert_eq!(document.root.get("g").unwrap().as_str(), Some("on"));
        assert_eq!(document.root.get("h").unwrap().as_str(), Some("x\n"));
    }

    #[test]
    fn a_key_written_twice_names_both_lines_and_the_last_value_counts() {
        let document = load(
            "name: One\nfiles:\n  - a\nstack: x\nouter:\n  when: a\n  when: b\nfiles:\n  - b\n",
        )
        .unwrap();
        let found: Vec<_> = document
            .repeated
            .iter()
            .map(|repeated| (repeated.path.as_str(), repeated.lines.clone()))
            .collect();
        assert_eq!(found, [("files", vec![2, 8]), ("outer.when", vec![6, 7])]);
        let files = document.root.get("files").unwrap().as_sequence().unwrap();
        assert_eq!(files[0].as_str(), Some("b"));
        let entries = document.root.entries().unwrap();
        let names: Vec<_> = entries.iter().map(|(key, _)| key.name().unwrap()).collect();
        assert_eq!(names, ["name", "files", "stack", "outer"]);
        assert_eq!(entries[1].1.as_sequence().unwrap()[0].as_str(), Some("b"));
    }

    #[test]
    fn anchors_documents_and_errors() {
        let document = load("a: &x hello\nb: *x\n").unwrap();
        assert_eq!(document.root.get("b").unwrap().as_str(), Some("hello"));
        assert!(load("").is_err());
        assert!(load("a: 1\n---\nb: 2\n").is_err());
        assert!(load("[1, 2]").is_err());
        assert!(
            load("a: [unfinished")
                .unwrap_err()
                .starts_with("cannot parse YAML: ")
        );
    }
}
