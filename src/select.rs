//! Choosing the entries that are most related to a task or a question.

use crate::store::Item;

/// The lower-case words of `text`, split at non-alphanumerics; words under 3 characters are
/// dropped.
pub fn words(text: &str) -> Vec<String> {
    Vec::new()
}

/// What to look for: tags and words.
pub struct Query {
    pub tags: Vec<String>,
    pub words: Vec<String>,
}

/// 5 points per query word in the title, 3 per query tag among the entry's tag fields,
/// 1 per query word in the sections. 0 = no match.
pub fn score(item: &Item, query: &Query) -> u32 {
    0
}

/// Items with score > 0, highest first, then newest `created` first, then ID.
pub fn best<'a>(items: &'a [Item], query: &Query, limit: usize) -> Vec<&'a Item> {
    Vec::new()
}
