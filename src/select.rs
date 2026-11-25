//! Choosing the entries most related to a task or a question, by word stems and rare words.

use crate::glossary::Term;
use crate::store::Item;
use std::collections::{HashMap, HashSet};

/// The entry fields whose comma-separated values are tags.
const TAG_FIELDS: [&str; 5] = ["machine", "analysis", "software", "tags", "topic"];

/// The words of the COMMON_WORDS lists of decide.rs and learn.rs, each once, in alphabetical order.
pub const COMMON_WORDS: &[&str] = &[
    "about", "after", "all", "also", "although", "and", "any", "are", "been", "before", "being",
    "between", "both", "but", "can", "could", "did", "does", "during", "each", "for", "from",
    "had", "has", "have", "here", "into", "its", "less", "may", "more", "most", "must", "new",
    "not", "off", "one", "only", "our", "out", "over", "per", "should", "some", "such", "than",
    "that", "the", "their", "them", "then", "there", "these", "they", "this", "those", "through",
    "two", "under", "use", "used", "uses", "using", "very", "via", "was", "were", "what", "when",
    "where", "which", "while", "who", "will", "with", "without", "would", "you", "your",
];

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

/// The lower-case word stemmed with the Snowball English stemmer; a word with a non-ASCII
/// character is only lower-cased.
pub fn stem(word: &str) -> String {
    let lower = word.to_lowercase();
    if !lower.is_ascii() {
        return lower;
    }
    rust_stemmers::Stemmer::create(rust_stemmers::Algorithm::English)
        .stem(&lower)
        .into_owned()
}

/// True when the lower-case word is one of COMMON_WORDS.
pub fn is_common(word: &str) -> bool {
    COMMON_WORDS.contains(&word.to_lowercase().as_str())
}

/// The stems of the words of `text` that are not common, each once, at its first place.
pub fn stems(text: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    words(text)
        .into_iter()
        .filter(|word| !is_common(word))
        .map(|word| stem(&word))
        .filter(|stem| seen.insert(stem.clone()))
        .collect()
}

/// What to look for: tags and words.
pub struct Query {
    pub tags: Vec<String>,
    pub words: Vec<String>,
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

/// How many entries hold each stem of a title or section body, and each tag value, counted once
/// per entry. `entries` is the number of entries the index was made from.
pub struct Index {
    entries: usize,
    stems: HashMap<String, usize>,
    tags: HashMap<String, usize>,
}

impl Index {
    /// Counts the stems of each title and section body, and the TAG_FIELDS values, once per entry.
    pub fn of(items: &[Item]) -> Index {
        let mut stem_counts = HashMap::new();
        let mut tag_counts = HashMap::new();
        for item in items {
            let mut held = title_stems(item);
            held.extend(section_stems(item));
            for stem in held {
                *stem_counts.entry(stem).or_insert(0) += 1;
            }
            for tag in item_tags(item) {
                *tag_counts.entry(tag).or_insert(0) += 1;
            }
        }
        Index {
            entries: items.len(),
            stems: stem_counts,
            tags: tag_counts,
        }
    }

    /// 0 when no entry holds the word; else 1 + log2(entries / holders), so a rare word weighs more.
    pub fn weight(&self, holders: usize) -> u32 {
        let holders = holders.min(self.entries);
        if holders == 0 {
            return 0;
        }
        1 + (self.entries / holders).ilog2()
    }

    /// The weight of a stem; 0 when no entry holds it.
    fn stem_weight(&self, stem: &str) -> u32 {
        self.weight(self.stems.get(stem).copied().unwrap_or(0))
    }

    /// The weight of a tag value; 0 when no entry carries it.
    fn tag_weight(&self, tag: &str) -> u32 {
        self.weight(self.tags.get(tag).copied().unwrap_or(0))
    }
}

/// The stems of the title of `item`, each once.
fn title_stems(item: &Item) -> HashSet<String> {
    stems(&item.entry.title()).into_iter().collect()
}

/// The stems of all section bodies of `item`, each once.
fn section_stems(item: &Item) -> HashSet<String> {
    let bodies: Vec<String> = item
        .entry
        .sections()
        .into_iter()
        .map(|(_, body)| body)
        .collect();
    stems(&bodies.join(" ")).into_iter().collect()
}

/// The lower-case values of the TAG_FIELDS fields of `item`.
fn item_tags(item: &Item) -> HashSet<String> {
    TAG_FIELDS
        .iter()
        .flat_map(|key| item.entry.list(key))
        .map(|tag| tag.to_lowercase())
        .collect()
}

/// The distinct stems of the query words.
fn query_stems(query: &Query) -> Vec<String> {
    stems(&query.words.join(" "))
}

/// True when `item` carries one of the folded query tags.
fn carries_tag(item: &Item, query_tags: &HashSet<String>) -> bool {
    item_tags(item).iter().any(|tag| query_tags.contains(tag))
}

/// Points for `item` against `query`: 5 times the weight of each query stem in the title, 1 time
/// in the sections, and 3 times the weight of each query tag the item carries.
pub fn weighted(item: &Item, query: &Query, index: &Index) -> u32 {
    let title = title_stems(item);
    let sections = section_stems(item);
    let mut points = 0;
    for stem in query_stems(query) {
        let weight = index.stem_weight(&stem);
        if title.contains(&stem) {
            points += 5 * weight;
        }
        if sections.contains(&stem) {
            points += weight;
        }
    }
    let tags = item_tags(item);
    for tag in folded(&query.tags) {
        if tags.contains(&tag) {
            points += 3 * index.tag_weight(&tag);
        }
    }
    points
}

/// The points an entry must reach: a third of the sum of the weights of the query stems, rounded up.
pub fn needed(query: &Query, index: &Index) -> u32 {
    let sum: u32 = query_stems(query)
        .iter()
        .map(|stem| index.stem_weight(stem))
        .sum();
    sum.div_ceil(3)
}

/// The items with a weighted score above 0 that carry a query tag or reach `needed`, with their
/// score, highest first, then newest `created` first, then ID, then path.
pub fn ranked<'a>(items: &'a [Item], query: &Query) -> Vec<(u32, &'a Item)> {
    ranked_in(items, query, &Index::of(items))
}

/// As ranked, but every weight comes from index (an Index of the whole dossier), not from items.
pub fn ranked_in<'a>(items: &'a [Item], query: &Query, index: &Index) -> Vec<(u32, &'a Item)> {
    let threshold = needed(query, index);
    let query_tags = folded(&query.tags);
    let mut scored: Vec<(u32, &'a Item)> = items
        .iter()
        .filter_map(|item| {
            let points = weighted(item, query, index);
            let listed = carries_tag(item, &query_tags) || points >= threshold;
            (points > 0 && listed).then_some((points, item))
        })
        .collect();
    scored.sort_by(|(left_points, left), (right_points, right)| {
        right_points
            .cmp(left_points)
            .then_with(|| created(right).cmp(&created(left)))
            .then_with(|| left.entry.id().cmp(&right.entry.id()))
            .then_with(|| left.path.cmp(&right.path))
    });
    scored
}

/// The items of the first `limit` rows of `ranked`.
pub fn top<'a>(items: &'a [Item], query: &Query, limit: usize) -> Vec<&'a Item> {
    top_in(items, query, &Index::of(items), limit)
}

/// The items of the first `limit` rows of `ranked_in`.
pub fn top_in<'a>(items: &'a [Item], query: &Query, index: &Index, limit: usize) -> Vec<&'a Item> {
    ranked_in(items, query, index)
        .into_iter()
        .take(limit)
        .map(|(_, item)| item)
        .collect()
}

/// Some("tag match") when `item` carries a query tag, else Some("word match") when a query stem
/// is among the stems of its title or sections, else None.
pub fn why(item: &Item, query: &Query) -> Option<&'static str> {
    if carries_tag(item, &folded(&query.tags)) {
        return Some("tag match");
    }
    let title = title_stems(item);
    let sections = section_stems(item);
    let shares_word = query_stems(query)
        .iter()
        .any(|stem| title.contains(stem) || sections.contains(stem));
    shares_word.then_some("word match")
}

/// The query with the other spellings of every glossary term it names added to its words. A term
/// is named when one of its spellings has stems that are all among the stems of the query words,
/// or when a query tag equals one of its spellings, ignoring case. The tags and words keep their
/// order; a word already among the words is not added again, ignoring case.
pub fn widened(query: &Query, terms: &[Term]) -> Query {
    let known = query_stems(query);
    let tags = folded(&query.tags);
    let mut widened_words = query.words.clone();
    for term in terms {
        let spellings: Vec<&str> = std::iter::once(term.term.as_str())
            .chain(term.also.iter().map(String::as_str))
            .collect();
        let named = spellings.iter().any(|spelling| {
            let spelling_stems = stems(spelling);
            let by_words = !spelling_stems.is_empty()
                && spelling_stems.iter().all(|stem| known.contains(stem));
            by_words || tags.contains(&spelling.trim().to_lowercase())
        });
        if named {
            for spelling in &spellings {
                for word in words(spelling) {
                    if !widened_words
                        .iter()
                        .any(|present| present.to_lowercase() == word)
                    {
                        widened_words.push(word);
                    }
                }
            }
        }
    }
    Query {
        tags: query.tags.clone(),
        words: widened_words,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::new::{Draft, render};
    use crate::entry::{Entry, Kind};
    use std::path::PathBuf;

    const EARLY: &str = "2026-11-20T10:00:00+01:00";
    const LATE: &str = "2026-11-21T10:00:00+01:00";

    /// A glossary term with the given name and spellings, and nothing else.
    fn glossary_term(name: &str, also: &[&str]) -> Term {
        Term {
            term: name.to_string(),
            meaning: String::new(),
            symbol: None,
            unit: None,
            also: also.iter().map(|spelling| spelling.to_string()).collect(),
        }
    }

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
    fn word_forms_share_a_stem() {
        assert_eq!(stem("reuse"), stem("reusing"));
        assert_eq!(stem("reuse"), stem("reused"));
        assert_eq!(stem("connections"), stem("connection"));
        assert_eq!(stem("ÉTÉ"), "été");
    }

    #[test]
    fn common_words_are_known_in_any_case() {
        assert!(is_common("And"));
        assert!(!is_common("pump"));
    }

    #[test]
    fn stems_keep_the_first_stem_of_each_meaningful_word() {
        assert_eq!(stems("The pump and the pumps"), vec![stem("pump")]);
        assert!(stems("").is_empty());
    }

    #[test]
    fn a_rare_word_weighs_more_than_a_common_one() {
        let mut items = vec![item("DEC-001", EARLY, "Pump seal", &[], &[])];
        for n in 2..=8 {
            items.push(item(&format!("DEC-00{n}"), EARLY, "Pump unit", &[], &[]));
        }
        let index = Index::of(&items);
        let holders = |word: &str| index.stems.get(&stem(word)).copied().unwrap_or(0);

        assert_eq!(index.weight(holders("pump")), 1);
        assert_eq!(index.weight(holders("seal")), 4);
        assert_eq!(index.weight(0), 0);
        assert_eq!(index.weight(3), 2);

        let q = query(&[], &["pump", "seal"]);
        assert_eq!(ids(&top(&items, &q, 8))[0], "DEC-001");
    }

    #[test]
    fn ranked_in_weighs_words_by_the_whole_dossier_not_by_the_pool() {
        let valve_check = || item("DEC-001", EARLY, "Valve check", &[], &[]);
        let gasket_check = || item("DEC-008", EARLY, "Gasket check", &[], &[]);
        let mut all = vec![valve_check()];
        for n in 2..=7 {
            all.push(item(&format!("DEC-00{n}"), EARLY, "Valve test", &[], &[]));
        }
        all.push(gasket_check());
        let pool = vec![valve_check(), gasket_check()];
        let index = Index::of(&all);
        let q = query(&[], &["valve", "gasket"]);

        let found: Vec<(u32, String)> = ranked_in(&pool, &q, &index)
            .into_iter()
            .map(|(points, item)| (points, item.entry.id()))
            .collect();
        assert_eq!(
            found,
            vec![(20, "DEC-008".to_string()), (5, "DEC-001".to_string())]
        );
        assert_eq!(ids(&top_in(&pool, &q, &index, 1)), vec!["DEC-008"]);

        let alone: Vec<u32> = ranked(&pool, &q)
            .iter()
            .map(|(points, _)| *points)
            .collect();
        assert_eq!(alone, vec![10, 10]);
    }

    #[test]
    fn common_words_alone_rank_nothing() {
        let items = sample();

        assert!(ranked(&items, &query(&[], &["and", "the"])).is_empty());
        assert!(top(&items, &query(&[], &["and", "the"]), 10).is_empty());
    }

    #[test]
    fn an_entry_sharing_only_a_weak_word_is_left_out() {
        let items = vec![
            item("DEC-001", EARLY, "Valve test", &[], &[]),
            item(
                "DEC-002",
                EARLY,
                "Seal check",
                &[],
                &[("Reason", "The valve was checked.")],
            ),
            item("DEC-003", EARLY, "Rotor run", &[], &[]),
            item("DEC-004", EARLY, "Casing repair", &[], &[]),
            item("DEC-005", EARLY, "Nozzle layout", &[], &[]),
            item("DEC-006", EARLY, "Cooling plan", &[], &[]),
            item("DEC-007", EARLY, "Pump choice", &[], &[]),
            item("DEC-008", EARLY, "Gearbox notes", &[], &[]),
        ];
        let q = query(&[], &["valve", "rotor", "casing", "nozzle"]);

        assert_eq!(
            ids(&top(&items, &q, 10)),
            vec!["DEC-003", "DEC-004", "DEC-005", "DEC-001"]
        );
    }

    #[test]
    fn a_word_finds_another_form_of_it_in_a_title() {
        let items = vec![
            item("DEC-001", EARLY, "Reusing the seal", &[], &[]),
            item("DEC-002", EARLY, "Pump curves", &[], &[]),
        ];
        let q = query(&[], &["reuse"]);

        assert_eq!(ids(&top(&items, &q, 10)), vec!["DEC-001"]);
    }

    #[test]
    fn an_entry_sharing_only_a_tag_is_kept() {
        let mut items = sample();
        items.push(item(
            "DEC-010",
            LATE,
            "Compressor run",
            &[("tags", "Inbox")],
            &[],
        ));
        let q = query(&["inbox"], &["pump"]);

        assert_eq!(
            ids(&top(&items, &q, 10)),
            vec!["DEC-010", "DEC-001", "DEC-002", "DEC-003", "DEC-009"]
        );
    }

    #[test]
    fn top_gives_the_same_order_when_the_input_is_reversed() {
        let expected = vec!["DEC-001", "DEC-002", "DEC-003", "DEC-009"];
        let q = query(&[], &["pump"]);

        let items = sample();
        assert_eq!(ids(&top(&items, &q, 10)), expected);

        let reversed: Vec<Item> = sample().into_iter().rev().collect();
        assert_eq!(ids(&top(&reversed, &q, 10)), expected);
    }

    #[test]
    fn an_empty_query_ranks_nothing() {
        let items = sample();

        assert!(ranked(&items, &query(&[], &[])).is_empty());
        assert!(top(&items, &query(&[" "], &[""]), 10).is_empty());
    }

    #[test]
    fn why_names_the_rule_that_matched() {
        let tagged = item(
            "DEC-001",
            EARLY,
            "Compressor run",
            &[("tags", "inbox")],
            &[],
        );
        let titled = item("DEC-002", EARLY, "Pump seal choice", &[], &[]);
        let sectioned = item(
            "DEC-003",
            EARLY,
            "Other",
            &[],
            &[("Reason", "The pump was checked.")],
        );
        let unrelated = item(
            "DEC-004",
            EARLY,
            "Compressor run",
            &[],
            &[("Reason", "Fits the rig.")],
        );
        let q = query(&["inbox"], &["pump"]);

        assert_eq!(why(&tagged, &q), Some("tag match"));
        assert_eq!(why(&titled, &q), Some("word match"));
        assert_eq!(why(&sectioned, &q), Some("word match"));
        assert_eq!(why(&unrelated, &q), None);
    }

    #[test]
    fn a_term_named_by_its_other_spellings_adds_their_words_to_the_query() {
        let terms = [glossary_term("ControlLine", &["valve connection"])];
        let q = query(&["inbox"], &["valve", "connection", "check"]);

        let wide = widened(&q, &terms);

        assert_eq!(wide.tags, vec!["inbox".to_string()]);
        assert_eq!(
            wide.words,
            vec!["valve", "connection", "check", "controlline"]
        );
    }

    #[test]
    fn one_word_of_a_spelling_names_nothing() {
        let terms = [glossary_term("ControlLine", &["valve connection"])];
        let q = query(&[], &["valve"]);

        assert_eq!(widened(&q, &terms).words, vec!["valve"]);
    }

    #[test]
    fn a_query_that_names_no_term_comes_back_unchanged() {
        let terms = [glossary_term("ControlLine", &["valve connection"])];
        let q = query(&["inbox"], &["pump", "seal"]);

        let wide = widened(&q, &terms);

        assert_eq!(wide.tags, q.tags);
        assert_eq!(wide.words, q.words);
        assert_eq!(widened(&q, &[]).words, q.words);
    }

    #[test]
    fn widening_twice_adds_nothing_new() {
        let terms = [glossary_term("ControlLine", &["valve connection"])];
        let once = widened(&query(&[], &["valve", "connection"]), &terms);

        let twice = widened(&once, &terms);

        assert_eq!(twice.words, once.words);
        assert_eq!(twice.words, vec!["valve", "connection", "controlline"]);
    }

    #[test]
    fn a_query_tag_equal_to_a_spelling_names_the_term_ignoring_case() {
        let terms = [glossary_term("ControlLine", &["valve connection"])];
        let q = query(&["CONTROLLINE"], &["check", "Valve"]);

        let wide = widened(&q, &terms);

        assert_eq!(wide.tags, vec!["CONTROLLINE".to_string()]);
        assert_eq!(
            wide.words,
            vec!["check", "Valve", "controlline", "connection"]
        );
    }
}
