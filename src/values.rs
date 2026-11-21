//! Key values of a project (format.md §17, values.md): one block per value, with its history.

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};
use crate::time;
use std::path::Path;

/// The file of a dossier that holds the values, `values.md`.
pub const FILE: &str = "values.md";

/// The item name of the values file in snapshots and the journal.
pub const ITEM: &str = "VALUES";

/// The dash between the event of a history line and its detail (format.md §17).
const DASH: &str = " — ";

/// One value of the project, read from its block of values.md.
pub struct Value {
    pub name: String,
    pub value: String,
    pub unit: Option<String>,
    pub source: Option<String>,
    pub note: Option<String>,
    pub key: bool,
    pub valid_from: Option<String>,
    pub proposed: Option<String>,
}

/// A valid name of a value: lower-case letters, digits, `.`, `_` and `-`, starting with a letter
/// (format.md §17).
pub fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'))
}

/// "12.4 kg/s" -> ("12.4", Some("kg/s")): the unit is the last word. With `dimensionless` the
/// whole text is the value. One word without `dimensionless`: Code::BadValue.
pub fn split(text: &str, dimensionless: bool) -> Result<(String, Option<String>)> {
    let text = text.trim();
    if text.is_empty() {
        return Err(Error::new(Code::BadValue, "the value is empty"));
    }
    if dimensionless {
        return Ok((text.to_string(), None));
    }
    match text.rfind(char::is_whitespace) {
        Some(at) => Ok((
            text[..at].trim_end().to_string(),
            Some(text[at..].trim_start().to_string()),
        )),
        None => Err(Error::new(
            Code::BadValue,
            format!(
                "\"{text}\" has no unit: give a unit after the number, or --dimensionless for a value without one"
            ),
        )),
    }
}

/// The file and its bytes as read; a missing file is BlockFile::empty("Values of <code>"), None.
pub fn load(dossier: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    let bytes = match std::fs::read(dossier.join(FILE)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((BlockFile::empty(&format!("Values of {code}")), None));
        }
        Err(error) => return Err(error.into()),
    };
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| Error::new(Code::Parse, format!("{FILE}: not UTF-8")))?;
    let file = BlockFile::parse(text)
        .map_err(|error| Error::new(error.code, format!("{FILE}: {}", error.message)))?;
    Ok((file, Some(bytes)))
}

/// Every value of the file, in file order. A name that occurs twice is read once, from its first
/// block, as `BlockFile::field` does.
pub fn all(file: &BlockFile) -> Vec<Value> {
    let mut names: Vec<String> = Vec::new();
    for key in file.keys() {
        if !names.contains(&key) {
            names.push(key);
        }
    }
    names.iter().map(|name| read_value(file, name)).collect()
}

/// The value of the file with this name.
pub fn get(file: &BlockFile, name: &str) -> Option<Value> {
    file.keys()
        .iter()
        .any(|key| key == name)
        .then(|| read_value(file, name))
}

/// "<value>[ <unit>]" in force at the end of `date` (YYYY-MM-DD), from the `set` history lines.
/// The history lines are only ever added (format.md §17), so the last `set` line on or before
/// `date` is the newest one. Proposals and rejections do not count.
pub fn on(file: &BlockFile, name: &str, date: &str) -> Option<String> {
    let date = date.trim();
    file.history(name)
        .iter()
        .filter_map(|line| set_line(line))
        .rfind(|(stamp, _)| time::date(stamp) <= date)
        .map(|(_, detail)| detail.to_string())
}

/// The value of the block `name`, from its fields. A field that is missing or empty is None; a
/// missing `value` reads as an empty text.
fn read_value(file: &BlockFile, name: &str) -> Value {
    let text = |field: &str| file.field(name, field).filter(|text| !text.is_empty());
    Value {
        name: name.to_string(),
        value: file.field(name, "value").unwrap_or_default(),
        unit: text("unit"),
        source: text("source"),
        note: text("note"),
        key: file.field(name, "key").is_some_and(|text| text == "yes"),
        valid_from: text("valid-from"),
        proposed: text("proposed"),
    }
}

/// The time and the detail of a `set` history value, `<time> <who>: set — <detail>`
/// (format.md §17). None for any other event, or a line without the parts.
fn set_line(line: &str) -> Option<(&str, &str)> {
    let (stamp, rest) = line.split_once(' ')?;
    let (_who, rest) = rest.split_once(": ")?;
    let (event, detail) = rest.split_once(DASH)?;
    (event == "set").then_some((stamp, detail))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// format.md section 17, the values example, copied exactly, with its last line break.
    const VALUES: &str = r#"<!-- dossier-format: 1 -->
# Values of P-017

## design.massflow

- value: 12.4
- unit: kg/s
- source: docs/spec-rev3.pdf
- key: yes
- valid-from: 2026-12-01T09:00:00+01:00
- history: 2026-11-20T10:00:00+01:00 anna: set — 12.1 kg/s
- history: 2026-12-01T09:00:00+01:00 anna: set — 12.4 kg/s
"#;

    /// One value with a proposal between its two set lines and a rejection after them, and a
    /// set line written by an LLM for a person.
    const WITH_PROPOSALS: &str = "<!-- dossier-format: 1 -->\n# Values of P-017\n\n## design.massflow\n\n- value: 12.4\n- unit: kg/s\n- history: 2026-11-20T10:00:00+01:00 anna: set — 12.1 kg/s\n- history: 2026-11-25T10:00:00+01:00 ben via ben-codex: proposed — 12.9 kg/s\n- history: 2026-12-01T09:00:00+01:00 anna via anna-claude: set — 12.4 kg/s\n- history: 2026-12-03T09:00:00+01:00 anna: rejected — 13.0 kg/s\n";

    /// The values example followed by a dimensionless value and a value with no `value` line.
    fn two_values() -> String {
        format!(
            "{VALUES}\n## design.gas\n\n- value: air, ideal gas\n- dimensionless: yes\n- history: 2026-11-21T08:00:00+01:00 anna: set — air, ideal gas\n\n## design.speed\n\n- proposed: 3900 rpm by ben at 2026-12-02T08:00:00+01:00\n"
        )
    }

    #[test]
    fn valid_name_is_a_lower_case_letter_then_letters_digits_dots_underscores_and_dashes() {
        for name in ["design.massflow", "a", "gas-2.inlet_p", "x9"] {
            assert!(valid_name(name), "{name} is a valid name");
        }
        for name in [
            "",
            "1st",
            ".x",
            "_x",
            "-x",
            "Design",
            "design massflow",
            "design/x",
            "design:x",
            "é",
        ] {
            assert!(!valid_name(name), "{name} is not a valid name");
        }
    }

    #[test]
    fn split_takes_the_last_word_as_the_unit() {
        let (value, unit) = split("12.4 kg/s", false).expect("a value with a unit");
        assert_eq!(value, "12.4");
        assert_eq!(unit.as_deref(), Some("kg/s"));
        let (value, unit) = split("  12.4   kg/s  ", false).expect("spaces are trimmed");
        assert_eq!(value, "12.4");
        assert_eq!(unit.as_deref(), Some("kg/s"));
    }

    #[test]
    fn split_with_dimensionless_keeps_the_whole_text_as_the_value() {
        let (value, unit) = split("air, ideal gas", true).expect("dimensionless");
        assert_eq!(value, "air, ideal gas");
        assert_eq!(unit, None);
        let (value, unit) = split("0.92 … 0.95", true).expect("a range, dimensionless");
        assert_eq!(value, "0.92 … 0.95");
        assert_eq!(unit, None);
    }

    #[test]
    fn split_refuses_one_word_without_dimensionless_and_an_empty_value() {
        let error = split("12.4", false).expect_err("a value needs a unit");
        assert_eq!(error.code, Code::BadValue);
        assert_eq!(split("", true).expect_err("empty").code, Code::BadValue);
        assert_eq!(split("  ", false).expect_err("empty").code, Code::BadValue);
    }

    #[test]
    fn the_values_example_of_format_section_17_parses_and_is_read() {
        let file = BlockFile::parse(VALUES).expect("the example parses");
        assert_eq!(file.text(), VALUES);
        let values = all(&file);
        assert_eq!(values.len(), 1);
        let value = &values[0];
        assert_eq!(value.name, "design.massflow");
        assert_eq!(value.value, "12.4");
        assert_eq!(value.unit.as_deref(), Some("kg/s"));
        assert_eq!(value.source.as_deref(), Some("docs/spec-rev3.pdf"));
        assert_eq!(value.note, None);
        assert!(value.key);
        assert_eq!(
            value.valid_from.as_deref(),
            Some("2026-12-01T09:00:00+01:00")
        );
        assert_eq!(value.proposed, None);
    }

    #[test]
    fn all_lists_every_block_in_file_order_and_a_dimensionless_value_has_no_unit() {
        let file = BlockFile::parse(&two_values()).expect("parses");
        let names: Vec<String> = all(&file).into_iter().map(|value| value.name).collect();
        assert_eq!(names, ["design.massflow", "design.gas", "design.speed"]);
        let gas = get(&file, "design.gas").expect("the block exists");
        assert_eq!(gas.value, "air, ideal gas");
        assert_eq!(gas.unit, None);
    }

    #[test]
    fn a_block_without_a_value_line_reads_as_empty_and_its_proposal_is_kept() {
        let file = BlockFile::parse(&two_values()).expect("parses");
        let speed = get(&file, "design.speed").expect("the block exists");
        assert_eq!(speed.value, "");
        assert_eq!(speed.unit, None);
        assert_eq!(
            speed.proposed.as_deref(),
            Some("3900 rpm by ben at 2026-12-02T08:00:00+01:00")
        );
        assert!(!speed.key);
    }

    #[test]
    fn get_finds_a_value_by_its_name_and_nothing_else() {
        let file = BlockFile::parse(&two_values()).expect("parses");
        let massflow = get(&file, "design.massflow").expect("the block exists");
        assert_eq!(massflow.value, "12.4");
        assert!(get(&file, "design").is_none());
        assert!(get(&file, "Design.Massflow").is_none());
        assert!(get(&file, "design.pressure").is_none());
    }

    #[test]
    fn on_gives_the_value_in_force_before_on_and_after_each_set_line() {
        let file = BlockFile::parse(VALUES).expect("parses");
        let in_force = |date: &str| on(&file, "design.massflow", date);
        assert_eq!(in_force("2026-11-19"), None, "before the first set line");
        assert_eq!(in_force("2026-11-20").as_deref(), Some("12.1 kg/s"));
        assert_eq!(
            in_force("2026-11-30").as_deref(),
            Some("12.1 kg/s"),
            "between"
        );
        assert_eq!(in_force("2026-12-01").as_deref(), Some("12.4 kg/s"));
        assert_eq!(
            in_force("2026-12-05").as_deref(),
            Some("12.4 kg/s"),
            "after"
        );
        assert_eq!(on(&file, "design.pressure", "2026-12-05"), None);
    }

    #[test]
    fn on_skips_proposals_and_rejections_and_counts_a_set_line_of_an_llm() {
        let file = BlockFile::parse(WITH_PROPOSALS).expect("parses");
        let in_force = |date: &str| on(&file, "design.massflow", date);
        assert_eq!(in_force("2026-11-19"), None);
        assert_eq!(in_force("2026-11-25").as_deref(), Some("12.1 kg/s"));
        assert_eq!(in_force("2026-11-30").as_deref(), Some("12.1 kg/s"));
        assert_eq!(in_force("2026-12-01").as_deref(), Some("12.4 kg/s"));
        assert_eq!(in_force("2026-12-03").as_deref(), Some("12.4 kg/s"));
    }

    #[test]
    fn load_of_a_missing_file_is_an_empty_file_and_no_bytes() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let (file, bytes) = load(dir.path(), "P-017").expect("loads");
        assert_eq!(file.text(), BlockFile::empty("Values of P-017").text());
        assert!(bytes.is_none());
    }

    #[test]
    fn load_reads_the_file_and_keeps_its_bytes() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        std::fs::write(dir.path().join(FILE), VALUES).expect("writes the file");
        let (file, bytes) = load(dir.path(), "P-017").expect("loads");
        assert_eq!(file.text(), VALUES);
        assert_eq!(bytes.as_deref(), Some(VALUES.as_bytes()));
    }

    #[test]
    fn load_refuses_a_file_that_is_not_utf_8() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        std::fs::write(dir.path().join(FILE), [0xff, 0xfe, b'#']).expect("writes the file");
        let error = load(dir.path(), "P-017").expect_err("not UTF-8");
        assert_eq!(error.code, Code::Parse);
    }
}
