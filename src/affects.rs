//! The Affects section of an assumption: the items it names (format.md §14).

use crate::entry::Kind;

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
/// the attached stored paths. The text is split at commas and line breaks, each item is trimmed,
/// and empty items are dropped.
pub fn parse(text: &str, values: &[String], paths: &[String]) -> Vec<Item> {
    text.split([',', '\n', '\r'])
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| classify(item, values, paths))
        .collect()
}

/// The kind of one trimmed, non-empty item. The checks run in this order: task, entry, value,
/// path, text.
fn classify(item: &str, values: &[String], paths: &[String]) -> Item {
    if is_task(item) {
        return Item::Task(item.to_ascii_uppercase());
    }
    if let Some(id) = entry_id(item) {
        return Item::Entry(id);
    }
    if values.iter().any(|name| name.as_str() == item) {
        return Item::Value(item.to_string());
    }
    // A stored path has forward slashes (format.md §18); a typed one may have backslashes.
    let path = item.replace('\\', "/");
    if paths.contains(&path) {
        return Item::Path(path);
    }
    Item::Text(item.to_string())
}

/// The part before the `-` of `<head>-<digits>`, when the digits are not empty and all digits.
fn numbered_head(item: &str) -> Option<&str> {
    let (head, digits) = item.split_once('-')?;
    let digits_only = !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit());
    digits_only.then_some(head)
}

/// Whether `item` is a task ID: `T-<digits>`, in any case.
fn is_task(item: &str) -> bool {
    numbered_head(item).is_some_and(|head| head.eq_ignore_ascii_case("T"))
}

/// The entry ID of `item` (`<PREFIX>-<digits>` with the prefix of an entry kind, in any case),
/// upper-cased.
fn entry_id(item: &str) -> Option<String> {
    let head = numbered_head(item)?;
    let kind = Kind::of_id(item)?;
    head.eq_ignore_ascii_case(kind.prefix())
        .then(|| item.to_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn parse_gives_a_task_an_entry_a_value_a_path_and_text() {
        let items = parse(
            "T-042, FND-004\ndesign.massflow, sim/a.sim, the inlet",
            &names(&["design.massflow"]),
            &names(&["sim/a.sim"]),
        );
        assert_eq!(
            items,
            vec![
                Item::Task("T-042".to_string()),
                Item::Entry("FND-004".to_string()),
                Item::Value("design.massflow".to_string()),
                Item::Path("sim/a.sim".to_string()),
                Item::Text("the inlet".to_string()),
            ]
        );
    }

    #[test]
    fn parse_splits_at_commas_and_line_breaks_trims_and_drops_empty_items() {
        let items = parse(" a ,, b\r\n\n ,\r\nc ", &[], &[]);
        assert_eq!(
            items,
            vec![
                Item::Text("a".to_string()),
                Item::Text("b".to_string()),
                Item::Text("c".to_string()),
            ]
        );
        assert!(parse(" , \n\r\n", &[], &[]).is_empty());
        assert!(parse("", &[], &[]).is_empty());
    }

    #[test]
    fn task_and_entry_ids_are_upper_cased_and_need_digits() {
        let items = parse("t-7, fnd-004, les-12, use-003", &[], &[]);
        assert_eq!(
            items,
            vec![
                Item::Task("T-7".to_string()),
                Item::Entry("FND-004".to_string()),
                Item::Entry("LES-12".to_string()),
                Item::Entry("USE-003".to_string()),
            ]
        );
        for text in ["T-", "FND-", "FND", "T", "XYZ-004", "DEC-1x", "FND-004-1"] {
            assert_eq!(parse(text, &[], &[]), vec![Item::Text(text.to_string())]);
        }
    }

    #[test]
    fn a_place_prefixed_id_is_text_and_a_kind_name_is_not_an_entry() {
        let items = parse("P-017:DEC-012, decision-004", &[], &[]);
        assert_eq!(
            items,
            vec![
                Item::Text("P-017:DEC-012".to_string()),
                Item::Text("decision-004".to_string()),
            ]
        );
    }

    #[test]
    fn a_value_name_must_match_exactly() {
        let items = parse(
            "design.massflow, Design.massflow",
            &names(&["design.massflow"]),
            &[],
        );
        assert_eq!(
            items,
            vec![
                Item::Value("design.massflow".to_string()),
                Item::Text("Design.massflow".to_string()),
            ]
        );
    }

    #[test]
    fn a_typed_path_with_backslashes_matches_a_stored_path_with_slashes() {
        let items = parse("sim\\a.sim, sim\\b.sim", &[], &names(&["sim/a.sim"]));
        assert_eq!(
            items,
            vec![
                Item::Path("sim/a.sim".to_string()),
                Item::Text("sim\\b.sim".to_string()),
            ]
        );
    }
}
