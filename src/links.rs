//! The linked entries of an entry (R-FIND-5, D53): the approved entries that an entry names in
//! its related field. `search`, `brief --task` and `which --binding` show these under an entry
//! through this one rule, so that they all show the same entries.

use crate::entry::{Entry, Status};
use crate::store::Item;

/// The most linked entries shown under one entry.
pub const LINKED_LIMIT: usize = 3;

/// The field that lists the related entries (format.md §5).
const RELATED: &str = "related";

/// The approved entries that `entry` names in its related field, in that order, at most `limit`,
/// each once. IDs that name no entry, the entry itself and entries that are not approved are left
/// out.
pub fn linked<'a>(entry: &Entry, items: &'a [Item], limit: usize) -> Vec<&'a Item> {
    let own = entry.id();
    let mut found: Vec<&'a Item> = Vec::new();
    for wanted in entry.list(RELATED) {
        if found.len() >= limit {
            break;
        }
        if wanted.eq_ignore_ascii_case(&own)
            || found
                .iter()
                .any(|item| item.entry.id().eq_ignore_ascii_case(&wanted))
        {
            continue;
        }
        let Some(item) = items
            .iter()
            .find(|item| item.entry.id().eq_ignore_ascii_case(&wanted))
        else {
            continue;
        };
        if item.entry.status() == Some(Status::Approved) {
            found.push(item);
        }
    }
    found
}

/// The line of a linked entry: two spaces, `linked: `, the ID, ` · ` and the title.
pub fn line(item: &Item) -> String {
    format!(
        "  linked: {} · {}",
        item.entry.id(),
        item.entry.title().trim()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::Kind;
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

    /// An approved finding with an invented title.
    fn finding(id: &str, title: &str) -> Item {
        item(Kind::Finding, id, title, &[("status", "approved")])
    }

    /// An approved decision that names `related` in its related field.
    fn decision(id: &str, related: &str) -> Item {
        item(
            Kind::Decision,
            id,
            "Invented decision",
            &[("status", "approved"), ("related", related)],
        )
    }

    /// The IDs of the linked entries, in order.
    fn ids(linked: &[&Item]) -> Vec<String> {
        linked.iter().map(|item| item.entry.id()).collect()
    }

    #[test]
    fn only_approved_entries_are_linked() {
        let items = vec![
            decision("DEC-010", "FND-001, FND-002, FND-003"),
            finding("FND-001", "Approved one"),
            item(
                Kind::Finding,
                "FND-002",
                "Superseded one",
                &[("status", "superseded")],
            ),
            item(
                Kind::Finding,
                "FND-003",
                "Draft one",
                &[("status", "draft")],
            ),
        ];
        let found = linked(&items[0].entry, &items, LINKED_LIMIT);
        assert_eq!(ids(&found), vec!["FND-001"]);
    }

    #[test]
    fn the_order_of_the_related_field_is_kept() {
        let items = vec![
            decision("DEC-010", "FND-003, DEC-004, FND-001"),
            finding("FND-001", "Listed first in the folder"),
            finding("FND-003", "Listed last in the folder"),
            decision("DEC-004", ""),
        ];
        let found = linked(&items[0].entry, &items, LINKED_LIMIT);
        assert_eq!(ids(&found), vec!["FND-003", "DEC-004", "FND-001"]);
    }

    #[test]
    fn a_fourth_approved_link_is_left_out_with_the_limit_of_three() {
        let items = vec![
            decision("DEC-010", "FND-001, FND-002, FND-003, FND-004"),
            finding("FND-001", "One"),
            finding("FND-002", "Two"),
            finding("FND-003", "Three"),
            finding("FND-004", "Four"),
        ];
        assert_eq!(LINKED_LIMIT, 3);
        let found = linked(&items[0].entry, &items, LINKED_LIMIT);
        assert_eq!(ids(&found), vec!["FND-001", "FND-002", "FND-003"]);
        let found = linked(&items[0].entry, &items, 2);
        assert_eq!(ids(&found), vec!["FND-001", "FND-002"]);
    }

    #[test]
    fn unknown_ids_and_the_entry_itself_are_skipped() {
        let items = vec![
            decision("DEC-010", "FND-777, dec-010, FND-002"),
            finding("FND-002", "Two"),
        ];
        let found = linked(&items[0].entry, &items, LINKED_LIMIT);
        assert_eq!(ids(&found), vec!["FND-002"]);
    }

    #[test]
    fn a_link_written_twice_gives_one_line() {
        let items = vec![
            decision("DEC-010", "FND-002, FND-002, fnd-002"),
            finding("FND-002", "Seal wear grows with the gap"),
        ];
        let found = linked(&items[0].entry, &items, LINKED_LIMIT);
        let lines: Vec<String> = found.into_iter().map(line).collect();
        assert_eq!(
            lines,
            vec!["  linked: FND-002 · Seal wear grows with the gap"]
        );
    }

    #[test]
    fn the_line_of_an_approved_finding_has_the_id_and_the_title() {
        let item = finding("FND-002", "Seal wear grows with the gap");
        assert_eq!(
            line(&item),
            "  linked: FND-002 · Seal wear grows with the gap"
        );
    }
}
