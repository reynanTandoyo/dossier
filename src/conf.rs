//! The confidential terms of a project and the check that keeps them out of the team dossier
//! (format.md §24).

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};
use std::path::Path;

/// The line a project states when it has no confidential terms.
pub const NONE: &str = "(no confidential terms)";

/// The file of a project dossier that holds the terms.
const FILE: &str = "confidential.md";

/// Lower case; runs of space, '-' and '_' become one space.
///
/// Any white space counts as a space, so that a tab or a no-break space in a text still matches.
pub fn normalize(text: &str) -> String {
    text.to_lowercase()
        .split(is_separator)
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Lower case with every space, '-' and '_' removed: the spelling of a text without separators.
fn compact(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|&c| !is_separator(c))
        .collect()
}

/// A white space, '-' or '_'.
fn is_separator(c: char) -> bool {
    c.is_whitespace() || c == '-' || c == '_'
}

/// The terms file of the dossier and its raw bytes, when it exists. A missing file is
/// BlockFile::empty("Confidential terms of <code>") and None.
pub fn load(dossier: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    let bytes = match std::fs::read(dossier.join(FILE)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let title = format!("Confidential terms of {code}");
            return Ok((BlockFile::empty(&title), None));
        }
        Err(error) => return Err(error.into()),
    };
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| Error::new(Code::Parse, format!("{FILE}: not UTF-8")))?;
    let file = BlockFile::parse(text)
        .map_err(|error| Error::new(error.code, format!("{FILE}: {}", error.message)))?;
    Ok((file, Some(bytes)))
}

/// The terms not removed, NONE left out, in file order. A term is removed when its block has a
/// `removed` field with a value. A key that normalizes to nothing is not a term and is left out.
pub fn terms(file: &BlockFile) -> Vec<String> {
    let mut terms: Vec<String> = Vec::new();
    for key in file.keys() {
        if key == NONE || normalize(&key).is_empty() || terms.contains(&key) {
            continue;
        }
        if !is_removed(file, &key) {
            terms.push(key);
        }
    }
    terms
}

/// Whether the file states that the project has no confidential terms: the NONE block has a
/// `stated` field with a value.
pub fn stated_none(file: &BlockFile) -> bool {
    file.field(NONE, "stated")
        .is_some_and(|when| !when.is_empty())
}

/// Each term found in `text` with the 1-based line it is on. A term is found on a line when the
/// normalized line contains the normalized term, or when the line without separators contains the
/// term without separators (format.md §24). Either comparison counts once per term and line. The
/// pairs are in line order, and within a line in the order of `terms`.
pub fn find(terms: &[String], text: &str) -> Vec<(String, usize)> {
    let wanted: Vec<(&String, String, String)> = terms
        .iter()
        .map(|term| (term, normalize(term), compact(term)))
        .filter(|(_, spaced, _)| !spaced.is_empty())
        .collect();
    let mut hits = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let spaced_line = normalize(line);
        let compact_line = compact(line);
        for (term, spaced, packed) in &wanted {
            let found =
                spaced_line.contains(spaced.as_str()) || compact_line.contains(packed.as_str());
            if found {
                hits.push((term.to_string(), index + 1));
            }
        }
    }
    hits
}

/// The terms of a project before writing to the team dossier: Code::Refused when there are none
/// and none was stated (R-CONF-5, the message says how to add them or state none).
pub fn gate(project: &Path, code: &str) -> Result<Vec<String>> {
    let (file, _) = load(project, code)?;
    let listed = terms(&file);
    if !listed.is_empty() {
        return Ok(listed);
    }
    if stated_none(&file) {
        return Ok(Vec::new());
    }
    Err(Error::new(
        Code::Refused,
        "this project has no confidential terms yet; add them with: dossier confidential add \"<term>\", or let a person state there are none: dossier confidential none --yes",
    ))
}

/// Code::Refused naming each term and line when `text` holds one (R-CONF-2).
pub fn check(terms: &[String], text: &str) -> Result<()> {
    let hits = find(terms, text);
    if hits.is_empty() {
        return Ok(());
    }
    let listed: Vec<String> = hits
        .iter()
        .map(|(term, line)| format!("{term} (line {line})"))
        .collect();
    Err(Error::new(
        Code::Refused,
        format!("confidential terms found: {}", listed.join("; ")),
    ))
}

/// Whether the first block with this key has a `removed` field with a value.
fn is_removed(file: &BlockFile, key: &str) -> bool {
    file.field(key, "removed")
        .is_some_and(|when| !when.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A terms file as `confidential add` and `remove` leave it: NT-450 active, Falcon removed,
    /// each with its history.
    const TERMS: &str = "<!-- dossier-format: 1 -->\n# Confidential terms of P-001\n\n## NT-450\n\n- history: 2026-11-02T09:00:00+01:00 anna: added\n\n## Falcon\n\n- removed: 2026-11-04T09:00:00+01:00 by anna\n- history: 2026-11-03T09:00:00+01:00 anna: added\n- history: 2026-11-04T09:00:00+01:00 anna: removed\n";

    /// A terms file in which a person stated that there are none.
    const NONE_STATED: &str = "<!-- dossier-format: 1 -->\n# Confidential terms of P-001\n\n## (no confidential terms)\n\n- stated: 2026-11-02T09:00:00+01:00 by anna\n";

    /// A project folder with the given terms file, or none at all when `text` is None.
    fn project_with(text: Option<&str>) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("a temporary folder");
        if let Some(text) = text {
            std::fs::write(dir.path().join(FILE), text).expect("writes the terms file");
        }
        dir
    }

    /// The terms as owned strings, for the functions that take a list.
    fn list(terms: &[&str]) -> Vec<String> {
        terms.iter().map(|term| term.to_string()).collect()
    }

    #[test]
    fn normalize_lowers_case_and_makes_spaces_hyphens_and_underscores_one_space() {
        assert_eq!(normalize("NT-450"), "nt 450");
        assert_eq!(normalize("NT_450"), "nt 450");
        assert_eq!(normalize("nt 450"), "nt 450");
        assert_eq!(normalize("  NT -- 450_ "), "nt 450");
        assert_eq!(normalize("a\tb\u{a0}c"), "a b c");
        assert_eq!(normalize(" - _ "), "");
    }

    #[test]
    fn a_term_matches_in_any_spelling_of_its_separators() {
        let terms = list(&["NT-450"]);
        assert_eq!(
            find(&terms, "the nt 450 stage"),
            [("NT-450".to_string(), 1)]
        );
        assert_eq!(find(&terms, "NT_450"), [("NT-450".to_string(), 1)]);
        assert_eq!(find(&terms, "see NT - 450"), [("NT-450".to_string(), 1)]);
    }

    #[test]
    fn a_term_matches_ignoring_case() {
        let terms = list(&["NT-450"]);
        assert_eq!(find(&terms, "nt-450"), [("NT-450".to_string(), 1)]);
        assert_eq!(find(&terms, "Nt 450"), [("NT-450".to_string(), 1)]);
        let terms = list(&["Falcon"]);
        assert_eq!(find(&terms, "FALCON stage"), [("Falcon".to_string(), 1)]);
    }

    #[test]
    fn a_term_matches_a_text_where_the_separators_are_left_out() {
        let terms = list(&["NT-450"]);
        assert_eq!(find(&terms, "the NT450 stage"), [("NT-450".to_string(), 1)]);
        assert_eq!(find(&terms, "nt450"), [("NT-450".to_string(), 1)]);
        let terms = list(&["Project Falcon"]);
        assert_eq!(
            find(&terms, "PROJECTFALCON"),
            [("Project Falcon".to_string(), 1)]
        );
    }

    #[test]
    fn a_term_written_without_separators_matches_a_text_with_them() {
        let terms = list(&["NT450"]);
        assert_eq!(find(&terms, "the NT-450 stage"), [("NT450".to_string(), 1)]);
        assert_eq!(find(&terms, "see nt 450"), [("NT450".to_string(), 1)]);
        assert_eq!(find(&terms, "NT_450"), [("NT450".to_string(), 1)]);
    }

    #[test]
    fn a_term_is_reported_once_per_line_when_both_comparisons_match() {
        let terms = list(&["NT-450"]);
        assert_eq!(
            find(&terms, "NT-450 and nt 450 and NT450"),
            [("NT-450".to_string(), 1)]
        );
    }

    #[test]
    fn a_term_does_not_match_a_different_number_without_separators() {
        assert!(find(&list(&["NT-450"]), "NT 45 and NT-405").is_empty());
    }

    #[test]
    fn find_gives_each_hit_with_its_line_in_line_order() {
        let terms = list(&["Beta", "Alpha"]);
        let text = "Alpha and Beta\nnothing here\nbeta again\nALPHA";
        assert_eq!(
            find(&terms, text),
            [
                ("Beta".to_string(), 1),
                ("Alpha".to_string(), 1),
                ("Beta".to_string(), 3),
                ("Alpha".to_string(), 4),
            ]
        );
    }

    #[test]
    fn find_is_empty_without_terms_or_without_a_match() {
        assert!(find(&[], "NT-450").is_empty());
        assert!(find(&list(&["NT-450"]), "nothing confidential here").is_empty());
        assert!(find(&list(&["NT-450"]), "").is_empty());
    }

    #[test]
    fn terms_leave_out_removed_terms_and_the_none_block() {
        let file = BlockFile::parse(TERMS).expect("the terms file parses");
        assert_eq!(terms(&file), ["NT-450"]);
        assert!(!stated_none(&file));

        let none = BlockFile::parse(NONE_STATED).expect("the none file parses");
        assert!(terms(&none).is_empty());
        assert!(stated_none(&none));
    }

    #[test]
    fn terms_are_read_from_the_file_of_the_project() {
        let dir = project_with(Some(TERMS));
        let (file, bytes) = load(dir.path(), "P-001").expect("loads");
        assert_eq!(terms(&file), ["NT-450"]);
        assert_eq!(bytes.as_deref(), Some(TERMS.as_bytes()));
    }

    #[test]
    fn load_of_a_missing_file_is_an_empty_file_and_no_bytes() {
        let dir = project_with(None);
        let (file, bytes) = load(dir.path(), "P-001").expect("loads");
        assert_eq!(
            file.text(),
            BlockFile::empty("Confidential terms of P-001").text()
        );
        assert!(bytes.is_none());
    }

    #[test]
    fn gate_refuses_a_project_with_no_file_and_says_how_to_add_or_state_none() {
        let dir = project_with(None);
        let error = gate(dir.path(), "P-001").expect_err("no terms and no statement");
        assert_eq!(error.code, Code::Refused);
        assert_eq!(
            error.message,
            "this project has no confidential terms yet; add them with: dossier confidential add \"<term>\", or let a person state there are none: dossier confidential none --yes"
        );
    }

    #[test]
    fn gate_refuses_an_empty_list_of_terms() {
        let only_heading = "<!-- dossier-format: 1 -->\n# Confidential terms of P-001\n\n";
        let dir = project_with(Some(only_heading));
        let error = gate(dir.path(), "P-001").expect_err("an empty list");
        assert_eq!(error.code, Code::Refused);

        let only_removed = "<!-- dossier-format: 1 -->\n# Confidential terms of P-001\n\n## NT-450\n\n- removed: 2026-11-04T09:00:00+01:00 by anna\n";
        let dir = project_with(Some(only_removed));
        assert_eq!(
            gate(dir.path(), "P-001").expect_err("all removed").code,
            Code::Refused
        );
    }

    #[test]
    fn gate_accepts_the_statement_that_there_are_none() {
        let dir = project_with(Some(NONE_STATED));
        assert_eq!(
            gate(dir.path(), "P-001").expect("accepted"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn gate_gives_the_terms_of_the_project() {
        let dir = project_with(Some(TERMS));
        assert_eq!(gate(dir.path(), "P-001").expect("accepted"), ["NT-450"]);
    }

    #[test]
    fn check_names_each_term_and_its_line() {
        let terms = list(&["NT-450", "Falcon"]);
        let text = "line one\nthe nt 450 stage\nfalcon here\nfine";
        let error = check(&terms, text).expect_err("two terms found");
        assert_eq!(error.code, Code::Refused);
        assert_eq!(
            error.message,
            "confidential terms found: NT-450 (line 2); Falcon (line 3)"
        );
    }

    #[test]
    fn check_passes_text_without_a_term_and_passes_with_no_terms() {
        assert!(check(&list(&["NT-450"]), "nothing confidential here").is_ok());
        assert!(check(&[], "NT-450 in the text").is_ok());
    }
}
