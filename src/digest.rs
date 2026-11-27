//! Milestone digests (R-CTX-14, R-CTX-15): which approved digest sums up which entry.
//!
//! A digest lists the tasks of a closed milestone. `brief` and `status` show the digests in force
//! in place of the entries they cover; this module decides which those are.

use crate::entry::{Entry, Kind, Status};
use crate::filter::{in_force_on, newest_first};
use crate::store::Item;

/// The rules for the summary, printed by `digest`.
pub const RULES: [&str; 5] = [
    "At most 15 lines.",
    "What was decided and still binds.",
    "What was found.",
    "What was assumed and is still open.",
    "What did not work.",
];

/// The approved digests in force (on `day`, when given), newest first.
pub fn in_force<'a>(items: &'a [Item], day: Option<&str>) -> Vec<&'a Item> {
    let mut digests: Vec<&Item> = items
        .iter()
        .filter(|item| item.entry.kind() == Some(Kind::Digest))
        .filter(|item| item.entry.status() == Some(Status::Approved))
        .filter(|item| day.is_none_or(|day| in_force_on(&item.entry, day.trim())))
        .collect();
    digests.sort_by(|a, b| newest_first(&a.entry, &b.entry));
    digests
}

/// The digest that covers this entry: the entry's `task` is in the digest's `tasks` list, ignoring
/// case. Not covered: an entry with no task, a decision with `standing: yes`, and an assumption or
/// investigation that is still open. `digests` is in the order of `in_force`; the first that lists
/// the task is the one returned.
pub fn covering<'a>(digests: &[&'a Item], entry: &Entry) -> Option<&'a Item> {
    if !coverable(entry) {
        return None;
    }
    let task = entry.field("task").filter(|task| !task.is_empty())?;
    digests.iter().copied().find(|digest| {
        digest
            .entry
            .list("tasks")
            .iter()
            .any(|listed| listed.eq_ignore_ascii_case(&task))
    })
}

/// Whether a digest may cover the entry at all: a decision that stands and an open assumption or
/// investigation are kept as they are.
fn coverable(entry: &Entry) -> bool {
    match entry.kind() {
        Some(Kind::Decision) => !entry
            .field("standing")
            .is_some_and(|standing| standing.eq_ignore_ascii_case("yes")),
        Some(Kind::Assumption | Kind::Investigation) => !entry
            .field("state")
            .is_some_and(|state| state.eq_ignore_ascii_case("open")),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::new::{Draft, render};
    use std::path::PathBuf;

    /// An item as `store::all` gives it: the text `render` makes for these fields, read back.
    fn item(kind: Kind, id: &str, fields: &[(&str, &str)]) -> Item {
        let draft = Draft {
            kind,
            id: id.to_string(),
            title: format!("Invented title of {id}"),
            fields: fields
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            sections: Vec::new(),
        };
        let text = render(&draft);
        Item {
            path: PathBuf::from(format!("{id}-invented-title.md")),
            raw: text.as_bytes().to_vec(),
            entry: Entry::parse(&text).expect("a rendered entry parses"),
        }
    }

    /// The IDs of the digests, in order.
    fn ids(digests: &[&Item]) -> Vec<String> {
        digests.iter().map(|digest| digest.entry.id()).collect()
    }

    /// An approved digest of the invented milestone T-900 and its tasks T-901 and T-902.
    fn approved_digest(id: &str, created: &str) -> Item {
        item(
            Kind::Digest,
            id,
            &[
                ("status", "approved"),
                ("created", created),
                ("valid-from", created),
                ("milestone", "T-900"),
                ("tasks", "T-900, T-901, T-902"),
            ],
        )
    }

    #[test]
    fn a_decision_of_a_task_in_the_list_is_covered() {
        let items = vec![
            approved_digest("DIG-001", "2026-11-20T10:00:00+01:00"),
            item(
                Kind::Decision,
                "DEC-001",
                &[("status", "approved"), ("task", "T-902")],
            ),
            item(
                Kind::Decision,
                "DEC-002",
                &[("status", "approved"), ("task", "T-777")],
            ),
        ];
        let digests = in_force(&items, None);
        let covered = covering(&digests, &items[1].entry).expect("T-902 is in the list");
        assert_eq!(covered.entry.id(), "DIG-001");
        assert!(covering(&digests, &items[2].entry).is_none());
    }

    #[test]
    fn a_decision_with_standing_yes_is_not_covered() {
        let items = vec![
            approved_digest("DIG-001", "2026-11-20T10:00:00+01:00"),
            item(
                Kind::Decision,
                "DEC-001",
                &[
                    ("status", "approved"),
                    ("task", "T-901"),
                    ("standing", "yes"),
                ],
            ),
        ];
        let digests = in_force(&items, None);
        assert!(covering(&digests, &items[1].entry).is_none());
    }

    #[test]
    fn an_open_assumption_is_not_covered_and_a_confirmed_one_is() {
        let items = vec![
            approved_digest("DIG-001", "2026-11-20T10:00:00+01:00"),
            item(
                Kind::Assumption,
                "ASM-001",
                &[("status", "approved"), ("task", "T-901"), ("state", "open")],
            ),
            item(
                Kind::Assumption,
                "ASM-002",
                &[
                    ("status", "approved"),
                    ("task", "T-901"),
                    ("state", "confirmed"),
                ],
            ),
        ];
        let digests = in_force(&items, None);
        assert!(covering(&digests, &items[1].entry).is_none());
        let covered = covering(&digests, &items[2].entry).expect("a confirmed one is covered");
        assert_eq!(covered.entry.id(), "DIG-001");
    }

    #[test]
    fn an_open_investigation_is_not_covered_and_a_solved_one_is() {
        let items = vec![
            approved_digest("DIG-001", "2026-11-20T10:00:00+01:00"),
            item(
                Kind::Investigation,
                "INV-001",
                &[("status", "approved"), ("task", "T-902"), ("state", "open")],
            ),
            item(
                Kind::Investigation,
                "INV-002",
                &[
                    ("status", "approved"),
                    ("task", "T-902"),
                    ("state", "solved"),
                ],
            ),
        ];
        let digests = in_force(&items, None);
        assert!(covering(&digests, &items[1].entry).is_none());
        assert!(covering(&digests, &items[2].entry).is_some());
    }

    #[test]
    fn a_draft_digest_covers_nothing() {
        let items = vec![
            item(
                Kind::Digest,
                "DIG-001",
                &[
                    ("status", "draft"),
                    ("created", "2026-11-20T10:00:00+01:00"),
                    ("milestone", "T-900"),
                    ("tasks", "T-900, T-901"),
                ],
            ),
            item(
                Kind::Decision,
                "DEC-001",
                &[("status", "approved"), ("task", "T-901")],
            ),
        ];
        let digests = in_force(&items, None);
        assert!(digests.is_empty());
        assert!(covering(&digests, &items[1].entry).is_none());
    }

    #[test]
    fn a_task_is_matched_ignoring_case_and_an_entry_without_a_task_is_not_covered() {
        let items = vec![
            approved_digest("DIG-001", "2026-11-20T10:00:00+01:00"),
            item(
                Kind::Finding,
                "FND-001",
                &[("status", "approved"), ("task", "t-902")],
            ),
            item(Kind::Finding, "FND-002", &[("status", "approved")]),
        ];
        let digests = in_force(&items, None);
        assert!(covering(&digests, &items[1].entry).is_some());
        assert!(covering(&digests, &items[2].entry).is_none());
    }

    #[test]
    fn the_newest_digest_that_lists_the_task_covers_it() {
        let items = vec![
            approved_digest("DIG-001", "2026-11-10T10:00:00+01:00"),
            approved_digest("DIG-002", "2026-11-20T10:00:00+01:00"),
            item(
                Kind::Decision,
                "DEC-001",
                &[("status", "approved"), ("task", "T-901")],
            ),
        ];
        let digests = in_force(&items, None);
        assert_eq!(ids(&digests), ["DIG-002", "DIG-001"]);
        let covered = covering(&digests, &items[2].entry).expect("covered");
        assert_eq!(covered.entry.id(), "DIG-002");
    }

    #[test]
    fn in_force_gives_the_approved_digests_newest_first_and_on_the_day_only_those_in_force() {
        let items = vec![
            approved_digest("DIG-001", "2026-11-10T10:00:00+01:00"),
            item(
                Kind::Digest,
                "DIG-002",
                &[
                    ("status", "approved"),
                    ("created", "2026-11-20T10:00:00+01:00"),
                    ("valid-from", "2026-11-20T10:00:00+01:00"),
                    ("valid-until", "2026-11-25T10:00:00+01:00"),
                    ("milestone", "T-910"),
                    ("tasks", "T-910, T-911"),
                ],
            ),
            item(
                Kind::Digest,
                "DIG-003",
                &[
                    ("status", "approved"),
                    ("created", "2026-11-21T10:00:00+01:00"),
                    ("milestone", "T-920"),
                    ("tasks", "T-920"),
                ],
            ),
        ];
        assert_eq!(
            ids(&in_force(&items, None)),
            ["DIG-003", "DIG-002", "DIG-001"]
        );
        // DIG-002 had not begun, and DIG-003 has no valid-from, so it was never in force on a day.
        assert_eq!(ids(&in_force(&items, Some("2026-11-15"))), ["DIG-001"]);
        // DIG-002 began on the 20th; DIG-001 has no valid-until and stays in force.
        assert_eq!(
            ids(&in_force(&items, Some("2026-11-20"))),
            ["DIG-002", "DIG-001"]
        );
        // DIG-002 ended on the 25th, so the day it ends is not in force.
        assert_eq!(ids(&in_force(&items, Some("2026-11-25"))), ["DIG-001"]);
    }
}
