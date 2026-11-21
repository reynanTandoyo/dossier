//! The glossary of the team dossier (format.md §20).

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};
use std::path::Path;

/// The file of a dossier that holds the glossary, `glossary.md`.
const FILE: &str = "glossary.md";

/// One term of the glossary.
pub struct Term {
    pub term: String,
    pub meaning: String,
    pub symbol: Option<String>,
    pub unit: Option<String>,
    pub also: Vec<String>,
}

/// The glossary file of the dossier and its raw bytes, when it exists. A missing file is
/// BlockFile::empty("Glossary of <code>") and None.
pub fn load(dossier: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    let bytes = match std::fs::read(dossier.join(FILE)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((BlockFile::empty(&format!("Glossary of {code}")), None));
        }
        Err(error) => return Err(error.into()),
    };
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| Error::new(Code::Parse, format!("{FILE}: not UTF-8")))?;
    let file = BlockFile::parse(text)
        .map_err(|error| Error::new(error.code, format!("{FILE}: {}", error.message)))?;
    Ok((file, Some(bytes)))
}

/// The terms of the glossary file in file order. A term that occurs twice is read once, from its
/// first block, as `BlockFile::field` does. A block with no name is no term.
pub fn terms(file: &BlockFile) -> Vec<Term> {
    let mut names: Vec<String> = Vec::new();
    for key in file.keys() {
        if !key.is_empty() && !names.contains(&key) {
            names.push(key);
        }
    }
    names
        .into_iter()
        .map(|name| read_term(file, name))
        .collect()
}

/// The term of the block `name`, from its fields. A missing or empty `meaning` is an empty text,
/// a missing or empty `symbol` or `unit` is None, and `also` is split at its commas with each
/// spelling trimmed (empty spellings are left out).
fn read_term(file: &BlockFile, name: String) -> Term {
    let text = |field: &str| file.field(&name, field).filter(|text| !text.is_empty());
    let also: Vec<String> = text("also")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|spelling| !spelling.is_empty())
        .map(str::to_string)
        .collect();
    Term {
        meaning: text("meaning").unwrap_or_default(),
        symbol: text("symbol"),
        unit: text("unit"),
        also,
        term: name,
    }
}

/// Every spelling of the terms that `word` is a spelling of (term and `also`, any case). Each
/// spelling is listed once, in the order of the terms, and two spellings that differ only in case
/// count as one. A blank word finds nothing.
pub fn spellings(terms: &[Term], word: &str) -> Vec<String> {
    let word = fold(word);
    if word.is_empty() {
        return Vec::new();
    }
    let mut found: Vec<String> = Vec::new();
    for term in terms {
        let names: Vec<&str> = std::iter::once(term.term.as_str())
            .chain(term.also.iter().map(String::as_str))
            .collect();
        if names.iter().any(|name| fold(name) == word) {
            for name in names {
                let spelling = name.trim();
                if !spelling.is_empty() && !found.iter().any(|known| fold(known) == fold(spelling))
                {
                    found.push(spelling.to_string());
                }
            }
        }
    }
    found
}

/// A spelling as it is compared: trimmed and in lower case, so that case is ignored.
fn fold(text: &str) -> String {
    text.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A team glossary of invented terms, in the format of format.md section 20.
    const GLOSSARY: &str = r#"<!-- dossier-format: 1 -->
# Glossary of TEAM

## mass flow

- meaning: fluid through a section per second
- symbol: m-dot
- unit: kg/s
- also: Massenstrom, mdot
- history: 2026-11-20T10:00:00+01:00 lena: added — mass flow

## pressure ratio

- meaning: outlet pressure divided by inlet pressure
- unit:
- also:  ,Druckverhältnis,
- history: 2026-11-21T10:00:00+01:00 lena: added — pressure ratio
"#;

    /// A term with the given name and spellings, and nothing else.
    fn term(name: &str, also: &[&str]) -> Term {
        Term {
            term: name.to_string(),
            meaning: String::new(),
            symbol: None,
            unit: None,
            also: also.iter().map(|spelling| spelling.to_string()).collect(),
        }
    }

    #[test]
    fn terms_reads_each_block_with_its_fields_and_splits_the_also_list() {
        let file = BlockFile::parse(GLOSSARY).expect("the glossary parses");
        let read = terms(&file);
        assert_eq!(read.len(), 2);
        let mass = &read[0];
        assert_eq!(mass.term, "mass flow");
        assert_eq!(mass.meaning, "fluid through a section per second");
        assert_eq!(mass.symbol.as_deref(), Some("m-dot"));
        assert_eq!(mass.unit.as_deref(), Some("kg/s"));
        assert_eq!(mass.also, ["Massenstrom", "mdot"]);
        let ratio = &read[1];
        assert_eq!(ratio.term, "pressure ratio");
        assert_eq!(ratio.symbol, None);
        assert_eq!(ratio.unit, None);
        assert_eq!(ratio.also, ["Druckverhältnis"]);
    }

    #[test]
    fn a_term_that_occurs_twice_is_read_once_from_its_first_block() {
        let text = "<!-- dossier-format: 1 -->\n# Glossary of TEAM\n\n## mass flow\n\n- meaning: first\n\n## mass flow\n\n- meaning: second\n";
        let file = BlockFile::parse(text).expect("parses");
        let read = terms(&file);
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].meaning, "first");
    }

    #[test]
    fn a_german_word_finds_its_english_term_and_all_its_spellings() {
        let file = BlockFile::parse(GLOSSARY).expect("parses");
        let read = terms(&file);
        assert_eq!(
            spellings(&read, "Massenstrom"),
            ["mass flow", "Massenstrom", "mdot"]
        );
    }

    #[test]
    fn spellings_ignores_case_and_finds_the_term_from_any_of_its_spellings() {
        let file = BlockFile::parse(GLOSSARY).expect("parses");
        let read = terms(&file);
        let all = ["mass flow", "Massenstrom", "mdot"];
        for word in [
            "massenstrom",
            "MASSENSTROM",
            "mdot",
            "Mass Flow",
            " mass flow ",
        ] {
            assert_eq!(spellings(&read, word), all, "{word}");
        }
        for word in ["druckverhältnis", "DRUCKVERHÄLTNIS", "Pressure Ratio"] {
            assert!(!spellings(&read, word).is_empty(), "{word}");
        }
        assert_eq!(
            spellings(&read, "DRUCKVERHÄLTNIS"),
            ["pressure ratio", "Druckverhältnis"]
        );
    }

    #[test]
    fn an_unknown_word_a_blank_word_and_no_terms_find_nothing() {
        let file = BlockFile::parse(GLOSSARY).expect("parses");
        let read = terms(&file);
        assert!(spellings(&read, "enthalpy").is_empty());
        assert!(spellings(&read, "  ").is_empty());
        assert!(spellings(&read, "").is_empty());
        assert!(spellings(&[], "mdot").is_empty());
    }

    #[test]
    fn spellings_gathers_every_term_the_word_belongs_to_and_lists_each_spelling_once() {
        let terms = vec![
            term("mass flow", &["Massenstrom", "mass flow", "mdot"]),
            term("flow rate", &["massenstrom", "Durchsatz"]),
            term("pressure ratio", &["Druckverhältnis"]),
        ];
        assert_eq!(
            spellings(&terms, "Massenstrom"),
            ["mass flow", "Massenstrom", "mdot", "flow rate", "Durchsatz"]
        );
    }

    #[test]
    fn load_of_a_missing_file_is_an_empty_glossary_and_no_bytes() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let (file, bytes) = load(dir.path(), "TEAM").expect("loads");
        assert_eq!(file.text(), BlockFile::empty("Glossary of TEAM").text());
        assert!(bytes.is_none());
    }

    #[test]
    fn load_reads_the_file_and_keeps_its_bytes_in_a_folder_with_spaces_and_non_ascii_letters() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let team = dir.path().join("Team Dossier Ärger");
        std::fs::create_dir(&team).expect("makes the folder");
        std::fs::write(team.join(FILE), GLOSSARY).expect("writes the file");
        let (file, bytes) = load(&team, "TEAM").expect("loads");
        assert_eq!(file.text(), GLOSSARY);
        assert_eq!(bytes.as_deref(), Some(GLOSSARY.as_bytes()));
    }

    #[test]
    fn load_refuses_a_file_that_is_not_utf_8() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        std::fs::write(dir.path().join(FILE), [0xff, 0xfe, b'#']).expect("writes the file");
        let error = load(dir.path(), "TEAM").expect_err("not UTF-8");
        assert_eq!(error.code, Code::Parse);
    }
}
