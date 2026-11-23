//! Choosing the entries that are most related to a task or a question.

use crate::store::Item;
use std::collections::HashSet;

/// The entry fields whose comma-separated values are tags.
const TAG_FIELDS: [&str; 4] = ["machine", "analysis", "software", "tags"];

/// The lower-case words of `text`, split at non-alphanumerics; words under 3 characters are
/// dropped. Each word is listed once, at its first place in the text.
pub fn words(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for word in lower.split(|c: char| !c.is_alphanumeric()) {
        if word.chars().count() >= 3 && seen.insert(word) {
            found.push(word.to_string());
        }
    }
    found
}

/// What to look for: tags and words.
pub struct Query {
    pub tags: Vec<String>,
    pub words: Vec<String>,
}

/// What an entry offers to a query: its title words, its section words and its tags, all in
/// lower case. `score` and `reason` both compare the query with these sets.
struct Matchable {
    title: HashSet<String>,
    sections: HashSet<String>,
    tags: HashSet<String>,
}

/// The title words, section words and tags of `item`.
fn matchable(item: &Item) -> Matchable {
    let title: HashSet<String> = words(&item.entry.title()).into_iter().collect();
    let bodies: Vec<String> = item
        .entry
        .sections()
        .into_iter()
        .map(|(_, body)| body)
        .collect();
    let sections: HashSet<String> = words(&bodies.join(" ")).into_iter().collect();
    let tags: HashSet<String> = TAG_FIELDS
        .iter()
        .flat_map(|key| item.entry.list(key))
        .map(|tag| tag.to_lowercase())
        .collect();
    Matchable {
        title,
        sections,
        tags,
    }
}

/// 5 points per query word in the title, 3 per query tag among the entry's tag fields,
/// 1 per query word in the sections. 0 = no match.
///
/// Query words and tags are compared ignoring case, and each distinct one counts at most once.
/// The title words and the section words are `words` of the title and of all section bodies
/// joined with a space, so a query word in both gets 6 points.
pub fn score(item: &Item, query: &Query) -> u32 {
    let found = matchable(item);
    let mut points = 0;
    for word in folded(&query.words) {
        if found.title.contains(&word) {
            points += 5;
        }
        if found.sections.contains(&word) {
            points += 1;
        }
    }
    for tag in folded(&query.tags) {
        if found.tags.contains(&tag) {
            points += 3;
        }
    }
    points
}

/// Which rule gave the points: Some("tag match") when a query tag matched, else
/// Some("word match") when a word matched, else None.
pub fn reason(item: &Item, query: &Query) -> Option<&'static str> {
    let found = matchable(item);
    if folded(&query.tags)
        .iter()
        .any(|tag| found.tags.contains(tag))
    {
        Some("tag match")
    } else if folded(&query.words)
        .iter()
        .any(|word| found.title.contains(word) || found.sections.contains(word))
    {
        Some("word match")
    } else {
        None
    }
}

/// Items with score > 0, highest first, then newest `created` first, then ID. The ID is
/// compared as text; the file path settles any tie that is left, so the order does not depend
/// on the order of `items`.
pub fn best<'a>(items: &'a [Item], query: &Query, limit: usize) -> Vec<&'a Item> {
    let mut scored: Vec<(u32, &'a Item)> = items
        .iter()
        .map(|item| (score(item, query), item))
        .filter(|(points, _)| *points > 0)
        .collect();
    scored.sort_by(|(left_points, left), (right_points, right)| {
        right_points
            .cmp(left_points)
            .then_with(|| created(right).cmp(&created(left)))
            .then_with(|| left.entry.id().cmp(&right.entry.id()))
            .then_with(|| left.path.cmp(&right.path))
    });
    scored
        .into_iter()
        .take(limit)
        .map(|(_, item)| item)
        .collect()
}

/// The `created` field of an entry as text; empty when it has none.
fn created(item: &Item) -> String {
    item.entry.field("created").unwrap_or_default()
}

/// The distinct values of `values`, trimmed and in lower case, without the empty ones.
fn folded(values: &[String]) -> HashSet<String> {
    values
        .iter()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::new::{Draft, render};
    use crate::entry::{Entry, Kind};
    use std::path::PathBuf;

    const EARLY: &str = "2026-11-20T10:00:00+01:00";
    const LATE: &str = "2026-11-21T10:00:00+01:00";

    /// An entry written by `render` and read back with `Entry::parse`, as the store reads it.
    /// `fields` are added after `created`; `sections` are the bodies, by name.
    fn item(
        id: &str,
        created: &str,
        title: &str,
        fields: &[(&str, &str)],
        sections: &[(&str, &str)],
    ) -> Item {
        let mut all_fields = vec![("created".to_string(), created.to_string())];
        all_fields.extend(
            fields
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string())),
        );
        let draft = Draft {
            kind: Kind::Decision,
            id: id.to_string(),
            title: title.to_string(),
            fields: all_fields,
            sections: sections
                .iter()
                .map(|(name, body)| (name.to_string(), body.to_string()))
                .collect(),
        };
        let text = render(&draft);
        let entry = Entry::parse(&text).expect("a rendered entry parses");
        Item {
            path: PathBuf::from(format!("{id}.md")),
            raw: text.into_bytes(),
            entry,
        }
    }

    fn query(tags: &[&str], words: &[&str]) -> Query {
        Query {
            tags: tags.iter().map(|tag| tag.to_string()).collect(),
            words: words.iter().map(|word| word.to_string()).collect(),
        }
    }

    fn ids(found: &[&Item]) -> Vec<String> {
        found.iter().map(|item| item.entry.id()).collect()
    }

    /// Four entries that all match the query word `pump` in some way, in no particular order:
    /// DEC-003 and DEC-001 match in the title and were created on different days, DEC-002 matches
    /// in the title on the same day as DEC-001, and DEC-009 matches only in a section.
    fn sample() -> Vec<Item> {
        vec![
            item("DEC-003", EARLY, "Pump A", &[], &[]),
            item("DEC-001", LATE, "Pump B", &[], &[]),
            item(
                "DEC-009",
                LATE,
                "Other",
                &[],
                &[("Reason", "The pump was checked.")],
            ),
            item("DEC-002", LATE, "Pump C", &[], &[]),
        ]
    }

    #[test]
    fn words_are_lower_case_split_and_listed_once() {
        assert_eq!(
            words("The Pump-pump, pump! AB 3d ÉTÉ"),
            vec!["the", "pump", "été"]
        );
        assert!(words("").is_empty());
        assert!(words("a b cd -- !!").is_empty());
    }

    #[test]
    fn a_title_match_outranks_a_text_match() {
        let text_only = item(
            "DEC-002",
            LATE,
            "Speedline run mode",
            &[],
            &[("Reason", "Pump curves were checked.")],
        );
        let title_only = item(
            "DEC-001",
            EARLY,
            "Pump seal choice",
            &[],
            &[("Reason", "Steady runs converged.")],
        );
        let both = item(
            "DEC-003",
            EARLY,
            "Pump seal choice",
            &[],
            &[("Reason", "The pump was run transient.")],
        );
        let q = query(&[], &["pump"]);

        assert_eq!(score(&text_only, &q), 1);
        assert_eq!(score(&title_only, &q), 5);
        assert_eq!(score(&both, &q), 6);

        let items = vec![text_only, title_only];
        assert_eq!(ids(&best(&items, &q, 10)), vec!["DEC-001", "DEC-002"]);
    }

    #[test]
    fn a_tag_match_scores_three_ignoring_case() {
        let entry = item(
            "DEC-004",
            EARLY,
            "Compressor run",
            &[
                ("machine", "axial compressor, fan"),
                ("analysis", "steady RANS"),
                ("tags", "inbox"),
            ],
            &[],
        );

        assert_eq!(score(&entry, &query(&["Axial Compressor"], &[])), 3);
        assert_eq!(score(&entry, &query(&["steady rans"], &[])), 3);
        assert_eq!(score(&entry, &query(&["fan", "INBOX"], &[])), 6);
        assert_eq!(score(&entry, &query(&["fan", "fan"], &[])), 3);
        assert_eq!(score(&entry, &query(&["axial"], &[])), 0);
    }

    #[test]
    fn a_query_word_counts_once_and_section_names_do_not_count() {
        let entry = item(
            "DEC-005",
            EARLY,
            "Pump seal choice",
            &[],
            &[("Reason", "Fits the rig.")],
        );

        assert_eq!(score(&entry, &query(&[], &["pump", "PUMP"])), 5);
        assert_eq!(score(&entry, &query(&[], &["reason"])), 0);
    }

    #[test]
    fn a_tag_match_is_the_reason_even_when_a_word_matches_too() {
        let entry = item(
            "DEC-006",
            EARLY,
            "Pump seal choice",
            &[("tags", "Inbox")],
            &[("Reason", "Fits the rig.")],
        );
        let q = query(&["inbox"], &["pump"]);

        assert_eq!(reason(&entry, &q), Some("tag match"));
        assert_eq!(score(&entry, &q), 8);
    }

    #[test]
    fn a_title_or_section_word_is_a_word_match() {
        let title_only = item("DEC-007", EARLY, "Pump seal choice", &[], &[]);
        let section_only = item(
            "DEC-008",
            EARLY,
            "Other",
            &[],
            &[("Reason", "The pump was checked.")],
        );
        let q = query(&[], &["pump"]);

        assert_eq!(reason(&title_only, &q), Some("word match"));
        assert_eq!(reason(&section_only, &q), Some("word match"));
    }

    #[test]
    fn an_entry_sharing_nothing_has_no_reason_and_no_score() {
        let entry = item(
            "DEC-010",
            EARLY,
            "Compressor run",
            &[("tags", "inbox")],
            &[("Reason", "Fits the rig.")],
        );
        let q = query(&["axial"], &["pump"]);

        assert_eq!(reason(&entry, &q), None);
        assert_eq!(score(&entry, &q), 0);
    }

    #[test]
    fn the_order_is_the_same_when_the_input_order_is_reversed() {
        let expected = vec!["DEC-001", "DEC-002", "DEC-003", "DEC-009"];
        let q = query(&[], &["pump"]);

        let items = sample();
        assert_eq!(ids(&best(&items, &q, 10)), expected);

        let reversed: Vec<Item> = sample().into_iter().rev().collect();
        assert_eq!(ids(&best(&reversed, &q, 10)), expected);
    }

    #[test]
    fn the_limit_cuts_the_list() {
        let items = sample();
        let q = query(&[], &["pump"]);

        assert_eq!(ids(&best(&items, &q, 2)), vec!["DEC-001", "DEC-002"]);
        assert_eq!(best(&items, &q, 100).len(), 4);
        assert!(best(&items, &q, 0).is_empty());
    }

    #[test]
    fn an_empty_query_gives_nothing() {
        let items = sample();
        let empty = query(&[], &[]);
        let blank = query(&[" "], &[""]);

        assert!(items.iter().all(|item| score(item, &empty) == 0));
        assert!(best(&items, &empty, 10).is_empty());
        assert!(best(&items, &blank, 10).is_empty());
    }
}
