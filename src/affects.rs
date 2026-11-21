//! The Affects section of an assumption: the items it names (format.md §14).

/// One item of an Affects section: a task, an entry, a value, an attached path, or plain text.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Task(String),
    Entry(String),
    Value(String),
    Path(String),
    Text(String),
}

/// The items of an Affects section (format.md §14). `values` are the known value names, `paths`
/// the attached stored paths.
pub fn parse(_text: &str, _values: &[String], _paths: &[String]) -> Vec<Item> {
    Vec::new()
}
