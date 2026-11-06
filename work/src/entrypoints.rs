use std::collections::{BTreeSet, HashSet};

use crate::model::{EntrypointRef, Kind, Place, Problem};

#[derive(Debug)]
pub(crate) struct Entrypoint {
    pub(crate) reference: EntrypointRef,
    pub(crate) file: String,
    pub(crate) area: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct Inventory {
    pub(crate) covers: BTreeSet<Kind>,
    pub(crate) entries: Vec<Entrypoint>,
    pub(crate) problems: Vec<Problem>,
}

impl Inventory {
    /// `entrypoints.tsv`, a byte order mark ignored.
    pub(crate) fn parse(text: &str) -> Self {
        Self::parse_with_format(text.strip_prefix('\u{feff}').unwrap_or(text), true)
    }

    pub(crate) fn parse_scan(text: &str) -> Self {
        Self::parse_with_format(text, false)
    }

    fn parse_with_format(text: &str, canonical: bool) -> Self {
        let mut inventory = Self::default();
        let mut lines = text.lines();
        let header = lines.next().unwrap_or_default();
        if let Some(kinds) = header
            .strip_prefix("# covers:")
            .filter(|kinds| kinds.is_empty() || kinds.starts_with(' '))
        {
            for name in kinds.split_whitespace() {
                match Kind::parse(name) {
                    Some(kind) => {
                        if !inventory.covers.insert(kind) && canonical {
                            inventory.problem(1, format!("covers: repeated kind {name}"));
                        }
                    }
                    None => inventory.problem(1, format!("covers: unknown kind {name:?}")),
                }
            }
        } else {
            inventory.problem(1, "expected '# covers: ' followed by scanned kinds");
        }
        if canonical && !text.ends_with('\n') {
            inventory
                .problems
                .push(Problem::new(Place::File, "must end with a newline"));
        }
        let mut previous = None;
        let mut reported_order = false;
        let mut seen = HashSet::new();
        for (index, line) in lines.enumerate() {
            let line_number = index + 2;
            if canonical
                && previous.is_some_and(|previous: &str| previous > line)
                && !reported_order
            {
                inventory.problem(line_number, "rows must be sorted in byte order");
                reported_order = true;
            }
            previous = Some(line);
            let columns: Vec<&str> = line.split('\t').collect();
            let valid_column_count = (3..=4).contains(&columns.len());
            if !valid_column_count {
                inventory.problem(
                    line_number,
                    format!(
                        "expected three or four tab-separated columns, got {}",
                        columns.len()
                    ),
                );
            }
            for (column, name) in columns.iter().zip(["kind", "name", "file"]) {
                if column.is_empty() {
                    inventory.problem(line_number, format!("{name} column must not be empty"));
                }
            }
            if canonical && columns.get(3) == Some(&"") {
                inventory.problem(line_number, "omit the empty area column and trailing tab");
            }
            if columns[0].is_empty() {
                continue;
            }
            let Some(kind) = Kind::parse(columns[0]) else {
                inventory.problem(
                    line_number,
                    format!("unknown entrypoint kind {:?}", columns[0]),
                );
                continue;
            };
            if !inventory.covers.contains(&kind) {
                inventory.problem(
                    line_number,
                    format!("kind {} is not listed in covers", kind.as_str()),
                );
            }
            let Some(name) = columns.get(1).filter(|name| !name.is_empty()) else {
                continue;
            };
            let reference = EntrypointRef {
                kind,
                name: (*name).into(),
            };
            if !seen.insert(reference.clone()) {
                inventory.problem(line_number, format!("duplicate entrypoint {reference}"));
            }
            if !valid_column_count || columns[2].is_empty() {
                continue;
            }
            inventory.entries.push(Entrypoint {
                reference,
                file: columns[2].into(),
                area: columns
                    .get(3)
                    .filter(|area| !area.is_empty())
                    .map(|area| (*area).into()),
            });
        }
        inventory
    }

    fn problem(&mut self, line: usize, message: impl Into<String>) {
        self.problems.push(Problem::new(Place::Line(line), message));
    }

    pub(crate) fn to_tsv(&self) -> String {
        let covers: Vec<_> = Kind::ALL
            .into_iter()
            .filter(|kind| self.covers.contains(kind))
            .map(Kind::as_str)
            .collect();
        let mut text = format!("# covers: {}\n", covers.join(" "));
        for (row, _) in self.sorted() {
            text.push_str(&row);
            text.push('\n');
        }
        text
    }

    // The entries in the order entrypoints.tsv keeps them: by their full row, byte order.
    pub(crate) fn sorted(&self) -> Vec<(String, &Entrypoint)> {
        let mut rows: Vec<_> = self
            .entries
            .iter()
            .map(|entry| {
                let mut row = format!(
                    "{}\t{}\t{}",
                    entry.reference.kind.as_str(),
                    entry.reference.name,
                    entry.file
                );
                if let Some(area) = &entry.area {
                    row.push('\t');
                    row.push_str(area);
                }
                (row, entry)
            })
            .collect();
        rows.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        rows
    }

    /// The entrypoints `self` has and `other` lacks, in the file's order.
    pub(crate) fn missing_from(&self, other: &Self) -> Vec<&EntrypointRef> {
        let theirs: HashSet<_> = other.entries.iter().map(|entry| &entry.reference).collect();
        let mut seen = HashSet::new();
        self.sorted()
            .into_iter()
            .map(|(_, entry)| &entry.reference)
            .filter(|reference| !theirs.contains(reference) && seen.insert(*reference))
            .collect()
    }

    pub(crate) fn references(&self) -> BTreeSet<String> {
        self.entries
            .iter()
            .map(|entry| entry.reference.to_string())
            .collect()
    }

    pub(crate) fn find(&self, reference: &EntrypointRef) -> Option<&Entrypoint> {
        self.entries
            .iter()
            .find(|entry| &entry.reference == reference)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_rows_keep_opaque_names_and_optional_areas() {
        let inventory = Inventory::parse(
            "# covers: http mail schedule\nhttp\tGET /orders/:id\tweb/orders.rs\tOrders\nmail\tOrderMailer#receipt\tmail/receipt.rs\n",
        );
        assert!(inventory.problems.is_empty(), "{:?}", inventory.problems);
        assert!(inventory.covers.contains(&Kind::Schedule));
        let reference = EntrypointRef::parse("http GET /orders/:id").unwrap();
        assert_eq!(
            inventory.find(&reference).unwrap().area.as_deref(),
            Some("Orders")
        );
        assert_eq!(inventory.entries[1].area, None);
    }

    #[test]
    fn malformed_rows_are_reported_without_losing_valid_rows() {
        let inventory = Inventory::parse(
            "# covers: job\njob\tExport\texport.rs\njob\tIncomplete\nunknown\tMystery\tmystery.rs\n",
        );
        assert_eq!(inventory.entries.len(), 1);
        assert_eq!(inventory.problems.len(), 2);
        assert_eq!(inventory.problems[0].place, Place::Line(3));
        assert_eq!(inventory.problems[1].place, Place::Line(4));
        assert!(inventory.problems[1].message.contains("unknown"));
    }

    #[test]
    fn scan_canonicalization_is_idempotent_across_row_orderings() {
        let rows = [
            "http\tGET /gardens/:id\tweb/gardens.rs\tGardens",
            "job\tPrune  Garden\tjobs/prune.rs\t",
            "mail\tGardenMail#notice\tmail/notice.rs",
            "schedule\tprune_old_plants\tjobs/prune.rs\tGardens",
            "event\t庭園.created\tevents/garden.rs",
            "cli\tgarden:inspect\tcommands/inspect.rs\tTools",
        ];
        let mut canonical = None;
        for reverse in [false, true] {
            for shift in 0..rows.len() {
                let mut rows = rows;
                rows.rotate_left(shift);
                if reverse {
                    rows.reverse();
                }
                // Repeated covers, CRLF, and an unterminated last row normalize on write.
                let input = format!(
                    "# covers: cli event schedule mail job http mail\r\n{}",
                    rows.join("\r\n")
                );
                let scanned = Inventory::parse_scan(&input);
                assert!(scanned.problems.is_empty(), "{:?}", scanned.problems);
                let output = scanned.to_tsv();
                assert!(output.starts_with("# covers: http job mail schedule event cli\n"));
                assert!(output.lines().all(|line| !line.ends_with('\t')));
                let saved = Inventory::parse(&output);
                assert!(saved.problems.is_empty(), "{:?}", saved.problems);
                assert_eq!(saved.references(), scanned.references());
                assert_eq!(saved.to_tsv(), output);
                assert_eq!(canonical.get_or_insert_with(|| output.clone()), &output);
            }
        }
    }

    #[test]
    fn scan_reports_each_bad_column_even_on_rows_with_other_errors() {
        let scanned = Inventory::parse_scan(
            "# covers: job\n\t\t\nunknown\tX\t\tarea\textra\njob\tPrune\njob\tPrune\tjobs/prune.rs\n",
        );
        let problems: Vec<String> = scanned.problems.iter().map(ToString::to_string).collect();
        for column in ["kind", "name", "file"] {
            assert!(
                problems
                    .iter()
                    .any(|problem| { problem.starts_with("line 2:") && problem.contains(column) })
            );
        }
        for detail in ["columns", "unknown", "file"] {
            assert!(
                problems
                    .iter()
                    .any(|problem| { problem.starts_with("line 3:") && problem.contains(detail) })
            );
        }
        assert!(problems.iter().any(|problem| {
            problem.starts_with("line 5:") && problem.contains("duplicate entrypoint job Prune")
        }));
    }
}
