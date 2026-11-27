//! Which entries rule something out, and the line shown for each (R-RULED-2, D59). `decide` and
//! `search` show these lines through this one module, so that both show the same.

use crate::entry::{Entry, Kind, Status};
use crate::select::{self, Query};
use crate::store::Item;

/// The most ruled-out entries that decide shows.
pub const RULED_LIMIT: usize = 3;
/// The field that holds the condition (format.md §38).
pub const REVISIT_IF: &str = "revisit-if";

/// Whether the entry rules something out on its own: a decision with status declined, or an
/// approved finding whose outcome is `dead end`.
pub fn is_ruled_out(item: &Item) -> bool {
    let entry = &item.entry;
    match entry.kind() {
        Some(Kind::Decision) => entry.status() == Some(Status::Declined),
        Some(Kind::Finding) => {
            entry.status() == Some(Status::Approved)
                && entry
                    .field("outcome")
                    .is_some_and(|outcome| outcome.trim() == "dead end")
        }
        _ => false,
    }
}

/// The trimmed revisit-if field of the entry; None when it is missing or empty.
pub fn condition(entry: &Entry) -> Option<String> {
    entry
        .field(REVISIT_IF)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// The title, with ` — revisit if: <condition>` after it when the entry has a condition.
pub fn titled(entry: &Entry) -> String {
    let title = entry.title();
    let title = title.trim();
    match condition(entry) {
        Some(condition) => format!("{title} — revisit if: {condition}"),
        None => title.to_string(),
    }
}

/// The entries of `items` most related to the query, best first, at most `limit` (select::top),
/// each as `ruled out: <ID> <title> — revisit if: <condition>`, or with
/// ` — no condition recorded` in place of the condition. `items` holds only ruled-out entries.
pub fn lines(items: &[Item], query: &Query, limit: usize) -> Vec<String> {
    select::top(items, query, limit)
        .into_iter()
        .map(|item| {
            let entry = &item.entry;
            let shown = match condition(entry) {
                Some(_) => titled(entry),
                None => format!("{} — no condition recorded", entry.title().trim()),
            };
            format!("ruled out: {} {shown}", entry.id())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::new::{Draft, render};
    use std::path::PathBuf;

    /// An item as `store::all` gives it: the text `render` makes for these fields, read back.
    fn item(kind: Kind, id: &str, title: &str, fields: &[(&str, &str)]) -> Item {
        let draft = Draft {
            kind,
            id: id.to_string(),
            title: title.to_string(),
            fields: fields
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            sections: Vec::new(),
        };
        let text = render(&draft);
        Item {
            path: PathBuf::from(format!("{id}-invented.md")),
            raw: text.as_bytes().to_vec(),
            entry: Entry::parse(&text).expect("a rendered entry parses"),
        }
    }

    fn query(words: &[&str]) -> Query {
        Query {
            tags: Vec::new(),
            words: words.iter().map(|word| word.to_string()).collect(),
        }
    }

    #[test]
    fn a_declined_decision_and_an_approved_dead_end_finding_are_ruled_out() {
        let declined = item(
            Kind::Decision,
            "DEC-001",
            "Pump seal",
            &[("status", "declined")],
        );
        let dead_end = item(
            Kind::Finding,
            "FND-001",
            "Seal wear",
            &[("status", "approved"), ("outcome", "dead end")],
        );

        assert!(is_ruled_out(&declined));
        assert!(is_ruled_out(&dead_end));
    }

    #[test]
    fn other_decisions_and_findings_are_not_ruled_out() {
        let not_ruled = [
            item(
                Kind::Decision,
                "DEC-002",
                "Approved",
                &[("status", "approved")],
            ),
            item(
                Kind::Decision,
                "DEC-003",
                "Proposed",
                &[("status", "proposed")],
            ),
            item(
                Kind::Finding,
                "FND-002",
                "No outcome",
                &[("status", "approved")],
            ),
            item(
                Kind::Finding,
                "FND-003",
                "Draft dead end",
                &[("status", "draft"), ("outcome", "dead end")],
            ),
        ];

        for entry in &not_ruled {
            assert!(
                !is_ruled_out(entry),
                "{} is not ruled out",
                entry.entry.id()
            );
        }
    }

    #[test]
    fn condition_is_trimmed_and_none_when_empty_or_missing() {
        let padded = item(
            Kind::Decision,
            "DEC-001",
            "Pump seal",
            &[
                ("status", "declined"),
                ("revisit-if", "  The supplier ships a seal  "),
            ],
        );
        let blank = item(
            Kind::Decision,
            "DEC-002",
            "Pump housing",
            &[("status", "declined"), ("revisit-if", "   ")],
        );
        let missing = item(
            Kind::Decision,
            "DEC-003",
            "Pump housing",
            &[("status", "declined")],
        );

        assert_eq!(
            condition(&padded.entry).as_deref(),
            Some("The supplier ships a seal")
        );
        assert_eq!(condition(&blank.entry), None);
        assert_eq!(condition(&missing.entry), None);
    }

    #[test]
    fn titled_adds_the_condition_only_when_there_is_one() {
        let with = item(
            Kind::Decision,
            "DEC-001",
            "Pump seal",
            &[
                ("status", "declined"),
                ("revisit-if", "The supplier ships a seal"),
            ],
        );
        let without = item(
            Kind::Decision,
            "DEC-002",
            "Pump housing",
            &[("status", "declined")],
        );

        assert_eq!(
            titled(&with.entry),
            "Pump seal — revisit if: The supplier ships a seal"
        );
        assert_eq!(titled(&without.entry), "Pump housing");
    }

    #[test]
    fn lines_gives_the_best_match_first_at_most_the_limit_in_both_forms() {
        let items = vec![
            item(
                Kind::Decision,
                "DEC-003",
                "Pump housing",
                &[("status", "declined")],
            ),
            item(
                Kind::Decision,
                "DEC-001",
                "Pump seal",
                &[
                    ("status", "declined"),
                    ("revisit-if", "The supplier ships a seal for the new pump"),
                ],
            ),
            item(
                Kind::Decision,
                "DEC-004",
                "Gearbox oil",
                &[
                    ("status", "declined"),
                    ("revisit-if", "The oil is recalled"),
                ],
            ),
        ];
        let words = query(&["pump", "seal"]);

        assert_eq!(
            lines(&items, &words, RULED_LIMIT),
            vec![
                "ruled out: DEC-001 Pump seal — revisit if: The supplier ships a seal for the new pump",
                "ruled out: DEC-003 Pump housing — no condition recorded",
            ]
        );
        assert_eq!(
            lines(&items, &words, 1),
            vec![
                "ruled out: DEC-001 Pump seal — revisit if: The supplier ships a seal for the new pump",
            ]
        );
    }

    #[test]
    fn a_query_with_no_words_and_no_tags_gives_no_lines() {
        let items = vec![item(
            Kind::Decision,
            "DEC-001",
            "Pump seal",
            &[("status", "declined")],
        )];

        assert!(lines(&items, &query(&[]), RULED_LIMIT).is_empty());
    }
}
