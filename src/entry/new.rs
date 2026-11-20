//! The text of a new entry file.

use super::Kind;

/// What a new entry file holds before it is rendered.
pub struct Draft {
    pub kind: Kind,
    pub id: String,
    pub title: String,
    pub fields: Vec<(String, String)>,
    pub sections: Vec<(String, String)>,
}

/// The text of a new entry file exactly as format.md §1, fields in the order of §5 (unknown
/// keys after the known ones, in the order given), sections in the order of §8.
pub fn render(draft: &Draft) -> String {
    String::new()
}
