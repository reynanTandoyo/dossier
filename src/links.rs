//! The linked entries of an entry (R-FIND-5, D53): the approved entries that an entry names in
//! its link fields (D61). `search`, `brief --task` and `which --binding` show these under an entry
//! through this one rule, so that they all show the same entries, each with its typed label.

use crate::entry::{Entry, Status};
use crate::store::Item;

/// The most linked entries shown under one entry.
pub const LINKED_LIMIT: usize = 3;

/// The field that lists the related entries (format.md §5).
const RELATED: &str = "related";

/// The link fields in the order their entries are shown, each with the word before the colon.
pub const LINK_FIELDS: [(&str, &str); 4] = [
    ("contradicts", "contradicts"),
    ("refines", "refines"),
    ("refined-by", "refined by"),
    ("related", "linked"),
];

/// The approved entries that `entry` names in its link fields, walking the fields of LINK_FIELDS
/// in their order and the IDs of each field in its order: at most `limit`, each once. IDs that
/// name no entry, the entry itself and entries that are not approved are left out.
pub fn linked<'a>(entry: &Entry, items: &'a [Item], limit: usize) -> Vec<&'a Item> {
    let own = entry.id();
    let mut found: Vec<&'a Item> = Vec::new();
    for (field, _) in LINK_FIELDS {
        for wanted in entry.list(field) {
            if found.len() >= limit {
                return found;
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
    }
    found
}

/// Whether the field `field` of `entry` names the ID of `other`, ignoring case.
fn names(entry: &Entry, field: &str, other: &Item) -> bool {
    let id = other.entry.id();
    entry
        .list(field)
        .iter()
        .any(|wanted| wanted.eq_ignore_ascii_case(&id))
}

/// The first field of LINK_FIELDS in which `entry` names `other`; `related` when none does.
pub fn field_of(entry: &Entry, other: &Item) -> &'static str {
    LINK_FIELDS
        .iter()
        .map(|&(field, _)| field)
        .find(|&field| names(entry, field, other))
        .unwrap_or(RELATED)
}

/// The word of LINK_FIELDS that belongs to `field_of(entry, other)`.
pub fn label(entry: &Entry, other: &Item) -> &'static str {
    let field = field_of(entry, other);
    // `field_of` only returns a name of LINK_FIELDS, so the fallback is never used.
    LINK_FIELDS
        .iter()
        .find(|&&(name, _)| name == field)
        .map_or("linked", |&(_, word)| word)
}

/// The line of a linked entry: two spaces, `linked: `, the ID, ` · ` and the title.
pub fn line(item: &Item) -> String {
    format!(
        "  linked: {} · {}",
        item.entry.id(),
        item.entry.title().trim()
    )
}

/// The line of a linked entry with its typed label: two spaces, the label of `field_of`, `: `,
/// the ID, ` · ` and the title.
pub fn typed_line(entry: &Entry, item: &Item) -> String {
    format!(
        "  {}: {} · {}",
        label(entry, item),
        item.entry.id(),
        item.entry.title().trim()
    )
}

/// The pairs of approved entries of which one names the other in its contradicts field, each
/// pair once, the item that comes first in `items` first, in the order of `items`.
pub fn contradictions<'a>(items: &[&'a Item]) -> Vec<(&'a Item, &'a Item)> {
    let mut pairs: Vec<(&'a Item, &'a Item)> = Vec::new();
    for (index, &first) in items.iter().enumerate() {
        for &second in &items[index + 1..] {
            let approved = first.entry.status() == Some(Status::Approved)
                && second.entry.status() == Some(Status::Approved);
            if approved
                && (names(&first.entry, "contradicts", second)
                    || names(&second.entry, "contradicts", first))
            {
                pairs.push((first, second));
            }
        }
    }
    pairs
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

    #[test]
    fn typed_links_come_in_the_order_of_the_link_fields() {
        let items = vec![
            item(
                Kind::Decision,
                "DEC-010",
                "Invented decision",
                &[
                    ("status", "approved"),
                    ("related", "FND-004"),
                    ("refined-by", "FND-003"),
                    ("refines", "FND-002"),
                    ("contradicts", "FND-001"),
                ],
            ),
            finding("FND-001", "One"),
            finding("FND-002", "Two"),
            finding("FND-003", "Three"),
            finding("FND-004", "Four"),
        ];
        let found = linked(&items[0].entry, &items, LINKED_LIMIT);
        assert_eq!(ids(&found), vec!["FND-001", "FND-002", "FND-003"]);
        let found = linked(&items[0].entry, &items, 4);
        assert_eq!(
            ids(&found),
            vec!["FND-001", "FND-002", "FND-003", "FND-004"]
        );
    }

    #[test]
    fn an_entry_named_in_two_fields_is_shown_once() {
        let items = vec![
            item(
                Kind::Decision,
                "DEC-010",
                "Invented decision",
                &[
                    ("status", "approved"),
                    ("related", "FND-002"),
                    ("refines", "FND-002, FND-001"),
                ],
            ),
            finding("FND-001", "One"),
            finding("FND-002", "Two"),
        ];
        let found = linked(&items[0].entry, &items, LINKED_LIMIT);
        assert_eq!(ids(&found), vec!["FND-002", "FND-001"]);
    }

    #[test]
    fn field_of_label_and_typed_line_for_each_field_and_an_entry_not_named() {
        let items = [
            item(
                Kind::Decision,
                "DEC-010",
                "Invented decision",
                &[
                    ("status", "approved"),
                    ("contradicts", "FND-001"),
                    ("refines", "FND-002"),
                    ("refined-by", "FND-003"),
                    ("related", "FND-004"),
                ],
            ),
            finding("FND-001", "One"),
            finding("FND-002", "Two"),
            finding("FND-003", "Three"),
            finding("FND-004", "Four"),
            finding("FND-005", "Not named"),
        ];
        let entry = &items[0].entry;
        let cases = [
            (
                1,
                "contradicts",
                "contradicts",
                "  contradicts: FND-001 · One",
            ),
            (2, "refines", "refines", "  refines: FND-002 · Two"),
            (
                3,
                "refined-by",
                "refined by",
                "  refined by: FND-003 · Three",
            ),
            (4, "related", "linked", "  linked: FND-004 · Four"),
            (5, "related", "linked", "  linked: FND-005 · Not named"),
        ];
        for (index, field, word, typed) in cases {
            let other = &items[index];
            assert_eq!(field_of(entry, other), field);
            assert_eq!(label(entry, other), word);
            assert_eq!(typed_line(entry, other), typed);
        }
    }

    #[test]
    fn a_link_field_is_matched_ignoring_case() {
        let items = [
            item(
                Kind::Decision,
                "DEC-010",
                "Invented decision",
                &[("status", "approved"), ("refines", "fnd-002")],
            ),
            finding("FND-002", "Two"),
        ];
        assert_eq!(field_of(&items[0].entry, &items[1]), "refines");
        assert_eq!(
            typed_line(&items[0].entry, &items[1]),
            "  refines: FND-002 · Two"
        );
    }

    #[test]
    fn one_approved_entry_naming_the_other_gives_one_pair() {
        let items = [
            item(
                Kind::Finding,
                "FND-001",
                "One",
                &[("status", "approved"), ("contradicts", "FND-002")],
            ),
            finding("FND-002", "Two"),
        ];
        let refs: Vec<&Item> = items.iter().collect();
        let pairs = contradictions(&refs);
        let pairs: Vec<(String, String)> = pairs
            .into_iter()
            .map(|(first, second)| (first.entry.id(), second.entry.id()))
            .collect();
        assert_eq!(pairs, vec![("FND-001".to_string(), "FND-002".to_string())]);
    }

    #[test]
    fn two_entries_naming_each_other_give_one_pair_in_the_order_of_the_list() {
        let items = [
            item(
                Kind::Finding,
                "FND-002",
                "Two",
                &[("status", "approved"), ("contradicts", "FND-001")],
            ),
            item(
                Kind::Finding,
                "FND-001",
                "One",
                &[("status", "approved"), ("contradicts", "fnd-002")],
            ),
        ];
        let refs: Vec<&Item> = items.iter().collect();
        let pairs = contradictions(&refs);
        let pairs: Vec<(String, String)> = pairs
            .into_iter()
            .map(|(first, second)| (first.entry.id(), second.entry.id()))
            .collect();
        assert_eq!(pairs, vec![("FND-002".to_string(), "FND-001".to_string())]);
    }

    #[test]
    fn no_pair_when_one_of_the_two_is_not_approved() {
        let items = [
            item(
                Kind::Finding,
                "FND-001",
                "One",
                &[("status", "approved"), ("contradicts", "FND-002")],
            ),
            item(
                Kind::Finding,
                "FND-002",
                "Draft two",
                &[("status", "draft"), ("contradicts", "FND-001")],
            ),
        ];
        let refs: Vec<&Item> = items.iter().collect();
        assert!(contradictions(&refs).is_empty());
    }
}
