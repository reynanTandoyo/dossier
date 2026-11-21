//! Block files (format.md §17): the format line, a heading, and blocks of `## <key>` with field
//! lines. Kept as lines, so that every byte not changed stays.
//!
//! A change touches only the lines it has to. Every other line keeps its bytes, including its own
//! line ending. A line the tool adds ends as format.md §2 says: with CRLF when the line before it
//! ends with CRLF, else LF.

use std::ops::Range;

use crate::error::{Code, Error, Result};
use crate::names::{self, Who};

/// The byte-order mark that may stand at the start of a file (format.md §2).
const BYTE_ORDER_MARK: char = '\u{feff}';

/// A block file (format.md §17), kept as its lines so that every byte not changed stays.
#[derive(Debug, Clone)]
pub struct BlockFile {
    /// Every line, with its own line ending.
    pub lines: Vec<String>,
}

impl BlockFile {
    /// Parses a block file. TooNew above format 1; Parse when no line starts with `# `, with the
    /// number of the line where the heading should be (as for entries).
    pub fn parse(text: &str) -> Result<BlockFile> {
        let lines: Vec<String> = text.split_inclusive('\n').map(str::to_string).collect();
        let heading = heading_index(&lines);
        let format = format_line(&lines, heading.unwrap_or(lines.len()));
        if format.is_some_and(|(_, number)| number > 1) {
            return Err(Error::new(
                Code::TooNew,
                "this file needs a newer dossier program",
            ));
        }
        if heading.is_none() {
            let line = format.map_or(1, |(index, _)| index + 2);
            return Err(Error::new(Code::Parse, format!("line {line}: no heading")));
        }
        Ok(BlockFile { lines })
    }

    /// Format line, `# <title>`, one blank line.
    pub fn empty(title: &str) -> BlockFile {
        BlockFile {
            lines: vec![
                "<!-- dossier-format: 1 -->\n".to_string(),
                format!("# {}\n", one_line(title)),
                "\n".to_string(),
            ],
        }
    }

    /// Equals the parsed text.
    pub fn text(&self) -> String {
        self.lines.concat()
    }

    /// `## ` keys in file order.
    pub fn keys(&self) -> Vec<String> {
        block_starts(&self.lines)
            .into_iter()
            .map(|(key, _)| key)
            .collect()
    }

    /// The value of the first field of a block with that name (history included).
    pub fn field(&self, key: &str, name: &str) -> Option<String> {
        let name = name.trim();
        self.fields(key)
            .into_iter()
            .find(|(field, _)| field == name)
            .map(|(_, value)| value)
    }

    /// The fields of a block in file order, history included. A block with no such key has none.
    pub fn fields(&self, key: &str) -> Vec<(String, String)> {
        block_range(&self.lines, key).map_or_else(Vec::new, |block| {
            self.lines[block]
                .iter()
                .filter_map(|line| parse_field(content(line)))
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect()
        })
    }

    /// The values of the history lines of a block, in file order.
    pub fn history(&self, key: &str) -> Vec<String> {
        self.fields(key)
            .into_iter()
            .filter(|(name, _)| name == "history")
            .map(|(_, value)| value)
            .collect()
    }

    /// Appends `## <key>`, a blank line, the fields (history pairs last), a blank line. A blank
    /// line comes first when the text does not end with one.
    ///
    /// Changes nothing when the key is empty, has a line break, or a block of that key exists
    /// already (a block is never written twice). A field with an empty value or a name that is
    /// not a valid key (format.md §3) is left out.
    pub fn add_block(&mut self, key: &str, fields: &[(String, String)]) {
        let key = key.trim();
        if key.is_empty() || key.contains(['\r', '\n']) || block_range(&self.lines, key).is_some() {
            return;
        }
        let mut texts = Vec::new();
        if self.lines.last().is_some_and(|line| !is_blank(line)) {
            texts.push(String::new());
        }
        texts.push(format!("## {key}"));
        texts.push(String::new());
        texts.extend(field_texts(fields));
        texts.push(String::new());
        let end = self.lines.len();
        let ending = ending_for(&self.lines, end);
        self.insert(end, &texts, ending);
    }

    /// Replaces the field or adds it before the block's first history line; an empty value
    /// removes it; `history` changes nothing; a missing block changes nothing.
    ///
    /// The name is trimmed and must be a valid key (format.md §3), else nothing changes. A new
    /// field with no field line in its block is placed right after the blank line under the
    /// block's heading (see `insert_field`).
    pub fn set_field(&mut self, key: &str, name: &str, value: &str) {
        let name = name.trim();
        if name == "history" || !valid_key(name) {
            return;
        }
        let Some(block) = block_range(&self.lines, key) else {
            return;
        };
        let value = one_line(value);
        let found = block
            .clone()
            .find(|&index| field_name(&self.lines[index]) == Some(name));
        if value.is_empty() {
            if let Some(index) = found {
                self.lines.remove(index);
            }
            return;
        }
        let line = format!("- {name}: {value}");
        if let Some(index) = found {
            let ending = own_ending(&self.lines[index]);
            self.lines[index] = format!("{line}{ending}");
            return;
        }
        let history = block
            .clone()
            .find(|&index| field_name(&self.lines[index]) == Some("history"));
        match history {
            Some(at) => {
                let ending = ending_for(&self.lines, at);
                self.insert(at, &[line], ending);
            }
            None => self.insert_field(block, line),
        }
    }

    /// Adds one history line to a block, after its last field line:
    /// `- history: <time> <person>[ via <llm>]: <event>[ — <detail>]` (format.md §6). The detail
    /// is left out when it is empty. A missing block changes nothing.
    pub fn add_history(&mut self, key: &str, time: &str, who: &Who, event: &str, detail: &str) {
        let Some(block) = block_range(&self.lines, key) else {
            return;
        };
        let mut line = format!(
            "- history: {} {}: {}",
            one_line(time),
            names::label(who),
            one_line(event)
        );
        let detail = one_line(detail);
        if !detail.is_empty() {
            line.push_str(" — ");
            line.push_str(&detail);
        }
        self.insert_field(block, line);
    }

    /// Adds a field line to a block: after its last field line (a history line counts), or, in a
    /// block with no field line, right after the blank line under its heading. Without that blank
    /// line the new line follows the heading, with a blank line before and after it, so that the
    /// block reads as format.md §17 shows it.
    fn insert_field(&mut self, block: Range<usize>, line: String) {
        let last = block
            .clone()
            .rfind(|&index| field_name(&self.lines[index]).is_some());
        if let Some(last) = last {
            let at = last + 1;
            let ending = ending_for(&self.lines, at);
            self.insert(at, &[line], ending);
            return;
        }
        let heading = block.start;
        let blank_under = heading + 1 < block.end && is_blank(&self.lines[heading + 1]);
        let at = if blank_under {
            heading + 2
        } else {
            heading + 1
        };
        let mut texts = Vec::new();
        if !blank_under {
            texts.push(String::new());
        }
        texts.push(line);
        if at < self.lines.len() && !is_blank(&self.lines[at]) {
            texts.push(String::new());
        }
        let ending = ending_for(&self.lines, at);
        self.insert(at, &texts, ending);
    }

    /// Inserts whole lines before the line at `at` (the end when `at` is the number of lines).
    /// Each new line ends with `ending`. The line before `at` gets `ending` first when it has
    /// none, which is the case only for the last line of a file.
    fn insert(&mut self, at: usize, texts: &[String], ending: &str) {
        let at = at.min(self.lines.len());
        if at > 0 && !self.lines[at - 1].ends_with('\n') {
            self.lines[at - 1].push_str(ending);
        }
        for (offset, text) in texts.iter().enumerate() {
            self.lines.insert(at + offset, format!("{text}{ending}"));
        }
    }
}

/// The key and the line index of every block (a `## ` line), in file order.
fn block_starts(lines: &[String]) -> Vec<(String, usize)> {
    lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            content(line)
                .strip_prefix("## ")
                .map(|key| (key.trim().to_string(), index))
        })
        .collect()
}

/// The lines of the block with `key` (trimmed), its `## ` line included: up to the next `## `
/// line or the end of the file. None when there is no such block; the first one counts.
fn block_range(lines: &[String], key: &str) -> Option<Range<usize>> {
    let key = key.trim();
    let starts = block_starts(lines);
    let position = starts.iter().position(|(name, _)| name == key)?;
    let start = starts[position].1;
    let end = starts
        .get(position + 1)
        .map_or(lines.len(), |(_, index)| *index);
    Some(start..end)
}

/// The field lines for `fields` as they are written: a name that is not a valid key and a value
/// that is empty are left out, and the `history` pairs come last.
fn field_texts(fields: &[(String, String)]) -> Vec<String> {
    let valid = fields.iter().filter_map(|(name, value)| {
        let name = name.trim();
        let value = one_line(value);
        (valid_key(name) && !value.is_empty()).then(|| (name.to_string(), value))
    });
    let (history, others): (Vec<_>, Vec<_>) = valid.partition(|(name, _)| name == "history");
    others
        .into_iter()
        .chain(history)
        .map(|(name, value)| format!("- {name}: {value}"))
        .collect()
}

/// The name of a field line (format.md §3), None for any other line.
fn field_name(line: &str) -> Option<&str> {
    parse_field(content(line)).map(|(name, _)| name)
}

/// The key of a field line and its trimmed value (format.md §3), None for any other line.
fn parse_field(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("- ")?;
    let (key, value) = rest.split_once(':')?;
    valid_key(key).then_some((key, value.trim()))
}

/// A key as format.md §3 allows it: lower-case letters, digits and `-`.
fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// A line without its line ending: LF or CRLF.
fn content(line: &str) -> &str {
    line.strip_suffix("\r\n")
        .or_else(|| line.strip_suffix('\n'))
        .unwrap_or(line)
}

/// A line without its line ending, and without a byte-order mark when it is the first line.
fn plain(lines: &[String], index: usize) -> &str {
    let line = content(&lines[index]);
    if index == 0 {
        line.strip_prefix(BYTE_ORDER_MARK).unwrap_or(line)
    } else {
        line
    }
}

/// The index of the first line that starts with `# ` (the heading, format.md §17).
fn heading_index(lines: &[String]) -> Option<usize> {
    (0..lines.len()).find(|&index| plain(lines, index).starts_with("# "))
}

/// The first format line before line `end`, with its number (format.md §3, §13).
fn format_line(lines: &[String], end: usize) -> Option<(usize, u64)> {
    (0..end).find_map(|index| format_number(plain(lines, index)).map(|number| (index, number)))
}

/// The number of a line `<!-- dossier-format: N -->`, after trimming. A number too large for
/// u64 counts as u64::MAX, so that it is refused as too new.
fn format_number(line: &str) -> Option<u64> {
    let inner = line
        .trim()
        .strip_prefix("<!-- dossier-format:")?
        .strip_suffix("-->")?
        .trim();
    if inner.is_empty() || !inner.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    Some(inner.parse().unwrap_or(u64::MAX))
}

/// Whether a line holds nothing but white space.
fn is_blank(line: &str) -> bool {
    content(line).trim().is_empty()
}

/// The line ending of one line: CRLF, LF, or nothing for the last line of a file without one.
fn own_ending(line: &str) -> &'static str {
    if line.ends_with("\r\n") {
        "\r\n"
    } else if line.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}

/// The ending that a new line takes at `index` (format.md §2): the ending of the line before it,
/// CRLF or LF. A line before with no ending (the last line of a file) gives the ending of the
/// line before that one. LF when there is no line before.
fn ending_for(lines: &[String], index: usize) -> &'static str {
    lines[..index.min(lines.len())]
        .iter()
        .rev()
        .map(|line| own_ending(line))
        .find(|ending| !ending.is_empty())
        .unwrap_or("\n")
}

/// A value on one line: line breaks become spaces, and the ends are trimmed.
fn one_line(text: &str) -> String {
    text.replace(['\r', '\n'], " ").trim().to_string()
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

    /// The values example with a second block after the first.
    fn two_blocks() -> String {
        format!("{VALUES}\n## design.speed\n\n- value: 3900\n- unit: rpm\n")
    }

    /// Anna's LLM, as `names::resolve` gives it.
    fn anna_claude() -> Who {
        Who {
            name: "anna-claude".to_string(),
            person: "anna".to_string(),
            is_llm: true,
        }
    }

    /// Anna herself, as `names::resolve` gives her.
    fn anna() -> Who {
        Who {
            name: "Anna".to_string(),
            person: "Anna".to_string(),
            is_llm: false,
        }
    }

    /// The text with every line break made CRLF.
    fn crlf(text: &str) -> String {
        text.replace('\n', "\r\n")
    }

    /// A field pair as `add_block` and `fields` take it.
    fn pair(name: &str, value: &str) -> (String, String) {
        (name.to_string(), value.to_string())
    }

    /// The pairs as borrowed text, for comparing with literals.
    fn pairs(list: &[(String, String)]) -> Vec<(&str, &str)> {
        list.iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect()
    }

    /// Applies `change` to `source` and checks the full text against `expected`. The same is
    /// done with the source and the expected text in CRLF, so that every line the tool adds must
    /// take the file's ending.
    fn check(source: &str, change: impl Fn(&mut BlockFile), expected: &str) {
        let mut file = BlockFile::parse(source).expect("the source parses");
        change(&mut file);
        assert_eq!(file.text(), expected);

        let mut crlf_file = BlockFile::parse(&crlf(source)).expect("the CRLF source parses");
        change(&mut crlf_file);
        assert_eq!(crlf_file.text(), crlf(expected));
    }

    #[test]
    fn the_values_example_of_format_section_17_parses_and_its_text_is_the_same() {
        let file = BlockFile::parse(VALUES).expect("the example parses");
        assert_eq!(file.text(), VALUES);
        assert_eq!(file.keys(), ["design.massflow"]);
    }

    #[test]
    fn empty_is_the_format_line_the_heading_and_one_blank_line() {
        let file = BlockFile::empty("Values of P-017");
        assert_eq!(
            file.text(),
            "<!-- dossier-format: 1 -->\n# Values of P-017\n\n"
        );
        assert!(file.keys().is_empty());
    }

    #[test]
    fn keys_are_the_block_keys_in_file_order_and_a_key_may_have_spaces() {
        let file = BlockFile::parse(&two_blocks()).expect("parses");
        assert_eq!(file.keys(), ["design.massflow", "design.speed"]);

        let spaced = BlockFile::parse("# T\n\n## my key  \n\n- a: 1\n").expect("parses");
        assert_eq!(spaced.keys(), ["my key"]);
        assert_eq!(spaced.field("my key", "a").as_deref(), Some("1"));
        assert_eq!(spaced.field(" my key ", "a").as_deref(), Some("1"));
    }

    #[test]
    fn field_reads_the_value_of_its_own_block_only() {
        let file = BlockFile::parse(&two_blocks()).expect("parses");
        assert_eq!(
            file.field("design.massflow", "unit").as_deref(),
            Some("kg/s")
        );
        assert_eq!(file.field("design.speed", "unit").as_deref(), Some("rpm"));
        assert_eq!(file.field("design.speed", "source"), None);
        assert_eq!(file.field("design.missing", "value"), None);
        assert_eq!(
            file.field("design.massflow", "history").as_deref(),
            Some("2026-11-20T10:00:00+01:00 anna: set — 12.1 kg/s")
        );
    }

    #[test]
    fn field_lines_are_found_anywhere_in_their_block_and_other_text_is_not_a_field() {
        let source = "# T\n\n## k\n\nSome text kept as written.\n- a:   spaced out   \r\n- b:\n\n- not a field\n- Upper: no\n";
        let file = BlockFile::parse(source).expect("parses");
        assert_eq!(
            file.fields("k"),
            vec![pair("a", "spaced out"), pair("b", "")]
        );
        assert_eq!(file.text(), source);
    }

    #[test]
    fn fields_lists_every_field_of_the_block_history_included_in_order() {
        let file = BlockFile::parse(VALUES).expect("parses");
        assert_eq!(
            pairs(&file.fields("design.massflow")),
            [
                ("value", "12.4"),
                ("unit", "kg/s"),
                ("source", "docs/spec-rev3.pdf"),
                ("key", "yes"),
                ("valid-from", "2026-12-01T09:00:00+01:00"),
                ("history", "2026-11-20T10:00:00+01:00 anna: set — 12.1 kg/s"),
                ("history", "2026-12-01T09:00:00+01:00 anna: set — 12.4 kg/s"),
            ]
        );
        assert!(file.fields("design.none").is_empty());
    }

    #[test]
    fn history_lists_the_history_values_in_order() {
        let file = BlockFile::parse(&two_blocks()).expect("parses");
        assert_eq!(
            file.history("design.massflow"),
            [
                "2026-11-20T10:00:00+01:00 anna: set — 12.1 kg/s",
                "2026-12-01T09:00:00+01:00 anna: set — 12.4 kg/s",
            ]
        );
        assert!(file.history("design.speed").is_empty());
    }

    #[test]
    fn set_field_replaces_the_value_of_its_line_and_keeps_every_other_line() {
        check(
            VALUES,
            |file| file.set_field("design.massflow", "value", "12.5"),
            &VALUES.replace("- value: 12.4\n", "- value: 12.5\n"),
        );
    }

    #[test]
    fn set_field_trims_the_value_and_keeps_it_on_one_line() {
        check(
            VALUES,
            |file| file.set_field("design.massflow", "source", "  docs/new\nspec.pdf "),
            &VALUES.replace(
                "- source: docs/spec-rev3.pdf\n",
                "- source: docs/new spec.pdf\n",
            ),
        );
    }

    #[test]
    fn set_field_adds_a_new_field_before_the_first_history_line() {
        check(
            VALUES,
            |file| file.set_field("design.massflow", "note", "checked by anna"),
            &VALUES.replace(
                "- history: 2026-11-20T10:00:00+01:00",
                "- note: checked by anna\n- history: 2026-11-20T10:00:00+01:00",
            ),
        );
    }

    #[test]
    fn set_field_adds_a_new_field_after_the_last_field_when_there_is_no_history() {
        check(
            "# T\n\n## k\n\n- a: 1\n- b: 2\n\n## other\n\n- a: 9\n",
            |file| file.set_field("k", "c", "3"),
            "# T\n\n## k\n\n- a: 1\n- b: 2\n- c: 3\n\n## other\n\n- a: 9\n",
        );
    }

    #[test]
    fn set_field_in_a_block_without_fields_goes_right_after_the_blank_line_under_the_heading() {
        check(
            "# T\n\n## k\n\nBody text.\n\n## other\n\n- a: 9\n",
            |file| file.set_field("k", "c", "3"),
            "# T\n\n## k\n\n- c: 3\n\nBody text.\n\n## other\n\n- a: 9\n",
        );
        check(
            "# T\n\n## k\nBody.\n",
            |file| file.set_field("k", "c", "3"),
            "# T\n\n## k\n\n- c: 3\n\nBody.\n",
        );
        check(
            "# T\n\n## k",
            |file| file.set_field("k", "c", "3"),
            "# T\n\n## k\n\n- c: 3\n",
        );
    }

    #[test]
    fn an_empty_value_removes_the_field_and_a_missing_field_changes_nothing() {
        check(
            VALUES,
            |file| file.set_field("design.massflow", "unit", ""),
            &VALUES.replace("- unit: kg/s\n", ""),
        );
        check(
            VALUES,
            |file| file.set_field("design.massflow", "unit", "  "),
            &VALUES.replace("- unit: kg/s\n", ""),
        );
        check(
            VALUES,
            |file| file.set_field("design.massflow", "note", ""),
            VALUES,
        );
    }

    #[test]
    fn set_field_for_history_a_missing_block_or_an_invalid_name_changes_nothing() {
        check(
            VALUES,
            |file| {
                file.set_field(
                    "design.massflow",
                    "history",
                    "2026-11-21T09:00:00+01:00 anna: changed",
                )
            },
            VALUES,
        );
        check(
            VALUES,
            |file| file.set_field("design.massflow", "history", ""),
            VALUES,
        );
        check(
            VALUES,
            |file| file.set_field("design.missing", "value", "1"),
            VALUES,
        );
        check(
            VALUES,
            |file| file.set_field("design.massflow", "Unit", "kg"),
            VALUES,
        );
        check(
            VALUES,
            |file| file.set_field("design.massflow", "a: b", "kg"),
            VALUES,
        );
        check(
            VALUES,
            |file| file.set_field("design.massflow", "", "kg"),
            VALUES,
        );
    }

    #[test]
    fn a_changed_line_keeps_its_own_line_ending() {
        let mut file = BlockFile::parse("# T\n\n## k\n\n- a: 1\r\n- b: 2\n").expect("parses");
        file.set_field("k", "a", "9");
        assert_eq!(file.text(), "# T\n\n## k\n\n- a: 9\r\n- b: 2\n");
    }

    #[test]
    fn a_file_without_last_line_ending_keeps_every_byte_and_gets_one_for_a_new_line() {
        check(
            "# T\n\n## k\n- a: 1",
            |file| file.set_field("k", "a", "2"),
            "# T\n\n## k\n- a: 2",
        );
        check(
            "# T\n\n## k\n- a: 1",
            |file| file.set_field("k", "b", "2"),
            "# T\n\n## k\n- a: 1\n- b: 2\n",
        );
    }

    #[test]
    fn add_history_appends_after_the_last_field_line_with_the_detail() {
        check(
            VALUES,
            |file| {
                file.add_history(
                    "design.massflow",
                    "2026-12-02T08:00:00+01:00",
                    &anna_claude(),
                    "changed",
                    "field value",
                )
            },
            &format!(
                "{VALUES}- history: 2026-12-02T08:00:00+01:00 anna via anna-claude: changed — field value\n"
            ),
        );
    }

    #[test]
    fn add_history_in_the_middle_of_a_file_stays_inside_its_block() {
        check(
            &two_blocks(),
            |file| {
                file.add_history(
                    "design.massflow",
                    "2026-12-02T08:00:00+01:00",
                    &anna_claude(),
                    "changed",
                    "x",
                )
            },
            &format!(
                "{VALUES}- history: 2026-12-02T08:00:00+01:00 anna via anna-claude: changed — x\n\n## design.speed\n\n- value: 3900\n- unit: rpm\n"
            ),
        );
    }

    #[test]
    fn add_history_without_a_detail_leaves_out_the_dash() {
        check(
            VALUES,
            |file| {
                file.add_history(
                    "design.massflow",
                    "2026-12-02T08:00:00+01:00",
                    &anna(),
                    "reopened",
                    "",
                )
            },
            &format!("{VALUES}- history: 2026-12-02T08:00:00+01:00 Anna: reopened\n"),
        );
    }

    #[test]
    fn add_history_on_a_block_without_fields_goes_under_its_heading() {
        check(
            "# T\n\n## k\n\nBody\n",
            |file| file.add_history("k", "2026-11-21T09:00:00+01:00", &anna(), "created", ""),
            "# T\n\n## k\n\n- history: 2026-11-21T09:00:00+01:00 Anna: created\n\nBody\n",
        );
    }

    #[test]
    fn add_history_for_a_missing_block_changes_nothing() {
        check(
            VALUES,
            |file| {
                file.add_history(
                    "design.missing",
                    "2026-12-02T08:00:00+01:00",
                    &anna(),
                    "created",
                    "",
                )
            },
            VALUES,
        );
    }

    #[test]
    fn add_block_on_an_empty_file_writes_the_block_with_its_history_last() {
        let mut file = BlockFile::empty("Values of P-017");
        file.add_block(
            "design.massflow",
            &[
                pair("history", "2026-11-20T10:00:00+01:00 anna: set — 12.1 kg/s"),
                pair("value", "12.1"),
                pair("unit", "kg/s"),
            ],
        );
        assert_eq!(
            file.text(),
            "<!-- dossier-format: 1 -->\n# Values of P-017\n\n## design.massflow\n\n- value: 12.1\n- unit: kg/s\n- history: 2026-11-20T10:00:00+01:00 anna: set — 12.1 kg/s\n\n"
        );
        assert_eq!(
            file.field("design.massflow", "value").as_deref(),
            Some("12.1")
        );
        assert_eq!(
            file.history("design.massflow"),
            ["2026-11-20T10:00:00+01:00 anna: set — 12.1 kg/s"]
        );
    }

    #[test]
    fn add_block_after_a_block_ends_with_a_blank_line_and_adds_one_only_when_needed() {
        check(
            VALUES,
            |file| {
                file.add_block(
                    "design.speed",
                    &[pair("value", "3900"), pair("unit", "rpm")],
                )
            },
            &format!("{VALUES}\n## design.speed\n\n- value: 3900\n- unit: rpm\n\n"),
        );
        let mut file = BlockFile::parse("# T\n\n## a\n\n- x: 1").expect("parses");
        file.add_block("b", &[pair("y", "2")]);
        assert_eq!(file.text(), "# T\n\n## a\n\n- x: 1\n\n## b\n\n- y: 2\n\n");
    }

    #[test]
    fn add_block_leaves_out_empty_and_invalid_fields_and_puts_history_last() {
        check(
            VALUES,
            |file| {
                file.add_block(
                    "design.speed",
                    &[
                        pair("note", ""),
                        pair("Unit", "rpm"),
                        pair("value", " 3900 "),
                    ],
                )
            },
            &format!("{VALUES}\n## design.speed\n\n- value: 3900\n\n"),
        );
    }

    #[test]
    fn add_block_changes_nothing_for_an_existing_key_an_empty_key_or_a_key_with_a_line_break() {
        check(
            VALUES,
            |file| file.add_block("design.massflow", &[pair("value", "1")]),
            VALUES,
        );
        check(
            VALUES,
            |file| file.add_block("  ", &[pair("value", "1")]),
            VALUES,
        );
        check(
            VALUES,
            |file| file.add_block("a\nb", &[pair("value", "1")]),
            VALUES,
        );
    }

    #[test]
    fn a_block_key_with_spaces_works_for_every_change() {
        let mut file = BlockFile::parse("# T\n\n## my key\n\n- value: 1\n").expect("parses");
        file.set_field("my key", "unit", "kg");
        file.add_history(
            "my key",
            "2026-11-21T09:00:00+01:00",
            &anna(),
            "changed",
            "",
        );
        assert_eq!(
            file.text(),
            "# T\n\n## my key\n\n- value: 1\n- unit: kg\n- history: 2026-11-21T09:00:00+01:00 Anna: changed\n"
        );
        file.add_block("other key", &[pair("value", "2")]);
        assert_eq!(file.keys(), ["my key", "other key"]);
        assert_eq!(file.field("other key", "value").as_deref(), Some("2"));
    }

    #[test]
    fn a_crlf_file_gets_crlf_lines_for_every_change() {
        let mut file = BlockFile::parse(&crlf(VALUES)).expect("parses");
        file.set_field("design.massflow", "note", "checked");
        file.add_history(
            "design.massflow",
            "2026-12-02T08:00:00+01:00",
            &anna(),
            "changed",
            "",
        );
        file.add_block("design.speed", &[pair("value", "3900")]);
        let expected = VALUES.replace(
            "- history: 2026-11-20T10:00:00+01:00",
            "- note: checked\n- history: 2026-11-20T10:00:00+01:00",
        ) + "- history: 2026-12-02T08:00:00+01:00 Anna: changed\n\n## design.speed\n\n- value: 3900\n\n";
        assert_eq!(file.text(), crlf(&expected));
        assert!(!file.text().replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn a_byte_order_mark_is_accepted_and_kept() {
        let source = "\u{feff}<!-- dossier-format: 1 -->\n# T\n\n## k\n\n- a: 1\n";
        check(
            source,
            |file| file.set_field("k", "a", "2"),
            &source.replace("- a: 1", "- a: 2"),
        );

        let heading_first = "\u{feff}# T\n## k\n- a: 1\n";
        let file = BlockFile::parse(heading_first).expect("parses");
        assert_eq!(file.text(), heading_first);
        assert_eq!(file.field("k", "a").as_deref(), Some("1"));
    }

    #[test]
    fn a_format_line_above_one_is_refused_as_too_new() {
        let error = BlockFile::parse("<!-- dossier-format: 2 -->\n# T\n").expect_err("too new");
        assert_eq!(error.code, Code::TooNew);
        assert_eq!(error.exit(), 4);
    }

    #[test]
    fn a_text_without_heading_fails_with_its_line_number() {
        let error =
            BlockFile::parse("<!-- dossier-format: 1 -->\nno heading\n").expect_err("no heading");
        assert_eq!(error.code, Code::Parse);
        assert_eq!(error.message, "line 2: no heading");

        let error = BlockFile::parse("## k\n- a: 1\n").expect_err("a block is not a heading");
        assert_eq!(error.code, Code::Parse);
        assert_eq!(error.message, "line 1: no heading");

        let error = BlockFile::parse("").expect_err("empty text has no heading");
        assert_eq!(error.message, "line 1: no heading");
    }

    #[test]
    fn a_file_without_format_line_is_read_as_version_one() {
        let file = BlockFile::parse("# T\n## k\n- a: 1\n").expect("parses");
        assert_eq!(file.field("k", "a").as_deref(), Some("1"));
    }

    #[test]
    fn ending_for_takes_the_line_before_and_falls_back_to_lf() {
        let lines: Vec<String> = vec!["# A\r\n".to_string(), "- b: c".to_string()];
        assert_eq!(ending_for(&lines, 0), "\n");
        assert_eq!(ending_for(&lines, 1), "\r\n");
        assert_eq!(ending_for(&lines, 2), "\r\n");
        let lf: Vec<String> = vec!["# A\n".to_string()];
        assert_eq!(ending_for(&lf, 1), "\n");
    }
}
