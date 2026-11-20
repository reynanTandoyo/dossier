//! Changes to an entry: its fields, title, sections and history.
//!
//! A change touches only the lines it has to. Every other line keeps its bytes, including its
//! own line ending. A line the tool adds ends as format.md §2 says: with CRLF when the line
//! before it ends with CRLF, else LF.

use std::ops::Range;

use super::{
    BYTE_ORDER_MARK, Entry, content, field_range, heading_index, parse_field, section_ranges,
};
use crate::names::{self, Who};

impl Entry {
    /// Replaces the field's line, or adds it before the first history line (else at the end of
    /// the block). New lines take their line ending as format.md §2 says.
    ///
    /// The key is trimmed and must be a valid key (format.md §3), else nothing changes. An empty
    /// value removes the field. `history` is never changed, so `history` changes nothing.
    pub fn set_field(&mut self, key: &str, value: &str) {
        let key = key.trim();
        if key == "history" || !valid_key(key) {
            return;
        }
        let value = one_line(value);
        if value.is_empty() {
            self.remove_field(key);
            return;
        }
        let line = format!("- {key}: {value}");
        let block = field_range(&self.lines);
        if block.is_empty() {
            self.insert_field_line(line);
            return;
        }
        if let Some(index) = block
            .clone()
            .find(|&index| field_key(&self.lines[index]) == Some(key))
        {
            let ending = own_ending(&self.lines[index]);
            self.lines[index] = format!("{line}{ending}");
            return;
        }
        let at = block
            .clone()
            .find(|&index| field_key(&self.lines[index]) == Some("history"))
            .unwrap_or(block.end);
        let ending = ending_for(&self.lines, at);
        self.insert(at, &[line], ending);
    }

    /// Removes the first line with that key from the field block. A history line is never
    /// removed, so `history` changes nothing.
    pub fn remove_field(&mut self, key: &str) {
        let key = key.trim();
        if key == "history" {
            return;
        }
        let block = field_range(&self.lines);
        if let Some(index) = block
            .into_iter()
            .find(|&index| field_key(&self.lines[index]) == Some(key))
        {
            self.lines.remove(index);
        }
    }

    /// Rewrites the heading as `# <ID> · <title>`, keeping its line ending and a byte-order mark
    /// at the start. Line breaks in the title become spaces; an empty title leaves out the
    /// separator.
    pub fn set_title(&mut self, title: &str) {
        let Some(index) = heading_index(&self.lines) else {
            return;
        };
        let id = self.id();
        let title = one_line(title);
        let text = match (id.is_empty(), title.is_empty()) {
            (false, false) => format!("# {id} · {title}"),
            (false, true) => format!("# {id}"),
            (true, false) => format!("# {title}"),
            (true, true) => return,
        };
        let mark = if index == 0 && self.lines[0].starts_with(BYTE_ORDER_MARK) {
            BYTE_ORDER_MARK.to_string()
        } else {
            String::new()
        };
        let ending = own_ending(&self.lines[index]);
        self.lines[index] = format!("{mark}{text}{ending}");
    }

    /// Replaces the body of the section `name` (matched exactly), or adds the section at the end
    /// of the file.
    ///
    /// `body` has LF (or CRLF) between its lines; blank lines at its start and end are dropped.
    /// The body is set off by one blank line before it. A section followed by another gets one
    /// blank line after it too; the last section ends with its body, as format.md §1 shows.
    /// An empty body removes the section and its heading. A body line that starts with `## ` would
    /// start a section of its own: callers refuse such text.
    pub fn set_section(&mut self, name: &str, body: &str) {
        let name = name.trim();
        if name.is_empty() || name.contains(['\r', '\n']) {
            return;
        }
        let Some(heading) = heading_index(&self.lines) else {
            return;
        };
        let body = body_lines(body);
        let found = section_ranges(&self.lines)
            .into_iter()
            .find(|(section, _)| section == name)
            .map(|(_, range)| range);
        let ending = ending_for(&self.lines, heading + 1);
        match found {
            Some(range) if body.is_empty() => self.remove_section(range),
            Some(range) => {
                let mut texts = vec![String::new()];
                texts.extend(body);
                if range.end < self.lines.len() {
                    texts.push(String::new());
                }
                self.lines.drain(range.start + 1..range.end);
                self.insert(range.start + 1, &texts, ending);
            }
            None if body.is_empty() => {}
            None => {
                let mut texts = Vec::new();
                if self.lines.last().is_some_and(|line| !is_blank(line)) {
                    texts.push(String::new());
                }
                texts.push(format!("## {name}"));
                texts.push(String::new());
                texts.extend(body);
                let end = self.lines.len();
                self.insert(end, &texts, ending);
            }
        }
    }

    /// Adds a history line after the last line of the field block:
    /// `- history: <time> <person>[ via <llm>]: <event>[ — <detail>]` (format.md §6). The detail
    /// is left out when it is empty.
    pub fn add_history(&mut self, time: &str, who: &Who, event: &str, detail: &str) {
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
        let block = field_range(&self.lines);
        if block.is_empty() {
            self.insert_field_line(line);
        } else {
            let ending = ending_for(&self.lines, block.end);
            self.insert(block.end, &[line], ending);
        }
    }

    /// Adds the first field line of an entry that has none: after the heading and one blank line,
    /// with one blank line after the block, as format.md §1 shows. Does nothing without a heading.
    fn insert_field_line(&mut self, line: String) {
        let Some(heading) = heading_index(&self.lines) else {
            return;
        };
        let after = heading + 1;
        let len = self.lines.len();
        if after < len && is_blank(&self.lines[after]) {
            let mut texts = vec![line];
            if after + 1 < len && !is_blank(&self.lines[after + 1]) {
                texts.push(String::new());
            }
            let ending = ending_for(&self.lines, after + 1);
            self.insert(after + 1, &texts, ending);
        } else {
            let mut texts = vec![String::new(), line];
            if after < len {
                texts.push(String::new());
            }
            let ending = ending_for(&self.lines, after);
            self.insert(after, &texts, ending);
        }
    }

    /// Removes a section's lines. The last section also takes the blank lines before its
    /// heading, so that the file ends as it would without it.
    fn remove_section(&mut self, range: Range<usize>) {
        let mut start = range.start;
        if range.end == self.lines.len() {
            while start > 0 && is_blank(&self.lines[start - 1]) {
                start -= 1;
            }
        }
        self.lines.drain(start..range.end);
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

/// The key of a field line (format.md §3), None for any other line.
fn field_key(line: &str) -> Option<&str> {
    parse_field(content(line)).map(|(key, _)| key)
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

/// A key as format.md §3 allows it: lower-case letters, digits and `-`.
fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// A value on one line: line breaks become spaces, and the ends are trimmed.
fn one_line(text: &str) -> String {
    text.replace(['\r', '\n'], " ").trim().to_string()
}

/// The lines of a section body, without the line endings and without blank lines at either end.
fn body_lines(body: &str) -> Vec<String> {
    let lines: Vec<&str> = body
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();
    let first = lines.iter().position(|line| !line.trim().is_empty());
    let last = lines.iter().rposition(|line| !line.trim().is_empty());
    match (first, last) {
        (Some(first), Some(last)) => lines[first..=last]
            .iter()
            .map(|line| line.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// format.md section 1, the first example, copied exactly, with its last line break.
    const DECISION: &str = r#"<!-- dossier-format: 1 -->
# DEC-012 · Use transient runs for the last three speedline points

- kind: decision
- status: approved
- created: 2026-11-20T14:03:27+01:00
- author: anna
- via: anna-claude
- approved: 2026-11-20T14:03:27+01:00 by anna
- valid-from: 2026-11-20T14:03:27+01:00
- task: T-042
- machine: axial compressor
- analysis: steady RANS, speedline
- history: 2026-11-20T14:03:27+01:00 anna via anna-claude: created
- history: 2026-11-20T14:03:27+01:00 anna via anna-claude: approved

## Decision

The last three points of every speedline are run transient.

## Reason

Steady runs converged at mass flows where the transient run already showed rotating stall.

## Rejected

Steady runs with a finer mesh: same result, three times the cost.
"#;

    const HISTORY_CREATED: &str =
        "- history: 2026-11-20T14:03:27+01:00 anna via anna-claude: created\n";
    const HISTORY_APPROVED: &str =
        "- history: 2026-11-20T14:03:27+01:00 anna via anna-claude: approved\n";

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

    /// Applies `change` to `source` and checks the full text against `expected`. The same is
    /// done with the source and the expected text in CRLF, so that every line the tool adds must
    /// take the file's ending.
    fn check(source: &str, change: impl Fn(&mut Entry), expected: &str) {
        let mut entry = Entry::parse(source).expect("the source parses");
        change(&mut entry);
        assert_eq!(entry.text(), expected);

        let mut crlf_entry = Entry::parse(&crlf(source)).expect("the CRLF source parses");
        change(&mut crlf_entry);
        assert_eq!(crlf_entry.text(), crlf(expected));
    }

    /// The example without its history lines.
    fn without_history() -> String {
        DECISION.replace(&format!("{HISTORY_CREATED}{HISTORY_APPROVED}"), "")
    }

    #[test]
    fn set_field_replaces_the_value_of_its_line_and_keeps_every_other_line() {
        check(
            DECISION,
            |entry| entry.set_field("status", "superseded"),
            &DECISION.replace("- status: approved\n", "- status: superseded\n"),
        );
    }

    #[test]
    fn set_field_trims_the_value_and_keeps_it_on_one_line() {
        check(
            DECISION,
            |entry| entry.set_field("analysis", "  steady\nRANS  "),
            &DECISION.replace(
                "- analysis: steady RANS, speedline\n",
                "- analysis: steady RANS\n",
            ),
        );
    }

    #[test]
    fn set_field_adds_a_new_field_before_the_first_history_line() {
        check(
            DECISION,
            |entry| entry.set_field("tags", "rotor, stall"),
            &DECISION.replace(
                HISTORY_CREATED,
                &format!("- tags: rotor, stall\n{HISTORY_CREATED}"),
            ),
        );
    }

    #[test]
    fn set_field_adds_a_new_field_at_the_end_of_the_block_when_there_is_no_history() {
        check(
            &without_history(),
            |entry| entry.set_field("tags", "rotor"),
            &without_history().replace(
                "- analysis: steady RANS, speedline\n",
                "- analysis: steady RANS, speedline\n- tags: rotor\n",
            ),
        );
    }

    #[test]
    fn an_empty_value_removes_the_field() {
        check(
            DECISION,
            |entry| entry.set_field("machine", "  "),
            &DECISION.replace("- machine: axial compressor\n", ""),
        );
    }

    #[test]
    fn set_field_for_history_changes_nothing_whatever_the_value() {
        check(
            DECISION,
            |entry| entry.set_field("history", "2026-11-21T09:00:00+01:00 anna: changed"),
            DECISION,
        );
        check(DECISION, |entry| entry.set_field("history", ""), DECISION);
    }

    #[test]
    fn set_field_with_an_invalid_key_changes_nothing() {
        check(
            DECISION,
            |entry| entry.set_field("Machine", "rotor"),
            DECISION,
        );
        check(DECISION, |entry| entry.set_field("a: b", "rotor"), DECISION);
        check(DECISION, |entry| entry.set_field("", "rotor"), DECISION);
    }

    #[test]
    fn remove_field_deletes_the_first_line_with_that_key_and_only_that_line() {
        check(
            DECISION,
            |entry| entry.remove_field("analysis"),
            &DECISION.replace("- analysis: steady RANS, speedline\n", ""),
        );
        check(
            DECISION,
            |entry| entry.remove_field("status"),
            &DECISION.replace("- status: approved\n", ""),
        );
    }

    #[test]
    fn remove_field_never_removes_a_history_line() {
        check(DECISION, |entry| entry.remove_field("history"), DECISION);
    }

    #[test]
    fn remove_field_of_a_missing_key_changes_nothing() {
        check(DECISION, |entry| entry.remove_field("tags"), DECISION);
    }

    #[test]
    fn set_title_rewrites_the_heading_and_keeps_its_line_ending() {
        check(
            DECISION,
            |entry| entry.set_title("Run the last points transient"),
            &DECISION.replace(
                "# DEC-012 · Use transient runs for the last three speedline points\n",
                "# DEC-012 · Run the last points transient\n",
            ),
        );
    }

    #[test]
    fn set_title_keeps_a_byte_order_mark_at_the_start_of_the_file() {
        let source = "\u{feff}# DEC-012 · Old title\n\n## Decision\n\nThe decision.\n";
        check(
            source,
            |entry| entry.set_title("New title"),
            "\u{feff}# DEC-012 · New title\n\n## Decision\n\nThe decision.\n",
        );
    }

    #[test]
    fn set_title_with_an_empty_title_leaves_out_the_separator() {
        check(
            DECISION,
            |entry| entry.set_title(" "),
            &DECISION.replace(
                "# DEC-012 · Use transient runs for the last three speedline points\n",
                "# DEC-012\n",
            ),
        );
    }

    #[test]
    fn set_section_replaces_the_body_and_keeps_one_blank_line_around_it() {
        check(
            DECISION,
            |entry| entry.set_section("Reason", "New reason.\n\nSecond paragraph."),
            &DECISION.replace(
                "Steady runs converged at mass flows where the transient run already showed rotating stall.\n",
                "New reason.\n\nSecond paragraph.\n",
            ),
        );
    }

    #[test]
    fn set_section_with_the_same_body_changes_nothing() {
        check(
            DECISION,
            |entry| {
                entry.set_section(
                    "Decision",
                    "\nThe last three points of every speedline are run transient.\n",
                )
            },
            DECISION,
        );
    }

    #[test]
    fn set_section_on_the_last_section_ends_the_file_after_its_body() {
        check(
            DECISION,
            |entry| entry.set_section("Rejected", "Not run: too costly."),
            &DECISION.replace(
                "Steady runs with a finer mesh: same result, three times the cost.\n",
                "Not run: too costly.\n",
            ),
        );
    }

    #[test]
    fn set_section_appends_a_missing_section_at_the_end_after_a_blank_line() {
        check(
            DECISION,
            |entry| entry.set_section("Confirmation", "Checked against run 14."),
            &format!("{DECISION}\n## Confirmation\n\nChecked against run 14.\n"),
        );
    }

    #[test]
    fn set_section_does_not_add_a_second_blank_line_when_the_file_already_ends_with_one() {
        let source = format!("{DECISION}\n");
        check(
            &source,
            |entry| entry.set_section("Confirmation", "Checked against run 14."),
            &format!("{DECISION}\n## Confirmation\n\nChecked against run 14.\n"),
        );
    }

    #[test]
    fn an_empty_body_removes_the_section_and_its_heading() {
        check(
            DECISION,
            |entry| entry.set_section("Reason", ""),
            &DECISION.replace(
                "## Reason\n\nSteady runs converged at mass flows where the transient run already showed rotating stall.\n\n",
                "",
            ),
        );
    }

    #[test]
    fn an_empty_body_on_the_last_section_also_takes_the_blank_line_before_it() {
        check(
            DECISION,
            |entry| entry.set_section("Rejected", "  \n"),
            &DECISION.replace(
                "\n## Rejected\n\nSteady runs with a finer mesh: same result, three times the cost.\n",
                "",
            ),
        );
    }

    #[test]
    fn an_empty_body_for_a_missing_section_changes_nothing() {
        check(
            DECISION,
            |entry| entry.set_section("Confirmation", ""),
            DECISION,
        );
    }

    #[test]
    fn a_section_name_with_a_line_break_changes_nothing() {
        check(
            DECISION,
            |entry| entry.set_section("Reason\nRejected", "text"),
            DECISION,
        );
    }

    #[test]
    fn a_file_without_a_last_line_ending_gets_one_when_a_line_is_added_after_it() {
        let source = DECISION
            .strip_suffix('\n')
            .expect("the example ends with a line break");
        check(
            source,
            |entry| entry.set_section("Confirmation", "Checked."),
            &format!("{DECISION}\n## Confirmation\n\nChecked.\n"),
        );
        check(
            "# DEC-012 · Title\n- status: draft",
            |entry| entry.set_field("tags", "rotor"),
            "# DEC-012 · Title\n- status: draft\n- tags: rotor\n",
        );
    }

    #[test]
    fn add_history_appends_after_the_last_history_line_with_the_detail() {
        let line =
            "- history: 2026-11-21T09:00:00+01:00 anna via anna-claude: changed — field machine\n";
        check(
            DECISION,
            |entry| {
                entry.add_history(
                    "2026-11-21T09:00:00+01:00",
                    &anna_claude(),
                    "changed",
                    "field machine",
                )
            },
            &DECISION.replace(HISTORY_APPROVED, &format!("{HISTORY_APPROVED}{line}")),
        );
    }

    #[test]
    fn add_history_without_a_detail_leaves_out_the_dash() {
        check(
            DECISION,
            |entry| entry.add_history("2026-11-21T09:00:00+01:00", &anna(), "reopened", ""),
            &DECISION.replace(
                HISTORY_APPROVED,
                &format!("{HISTORY_APPROVED}- history: 2026-11-21T09:00:00+01:00 Anna: reopened\n"),
            ),
        );
    }

    #[test]
    fn a_history_line_added_by_the_tool_reads_back_with_its_value() {
        let mut entry = Entry::parse(DECISION).expect("the example parses");
        entry.add_history(
            "2026-11-21T09:00:00+01:00",
            &anna_claude(),
            "changed",
            "field machine",
        );
        let history = entry.history();
        assert_eq!(history.len(), 3);
        assert_eq!(
            history[2],
            "2026-11-21T09:00:00+01:00 anna via anna-claude: changed — field machine"
        );
    }

    #[test]
    fn add_history_on_an_entry_without_field_lines_adds_the_block_it_needs() {
        let source = "# DEC-012 · Title\n\n## Decision\n\nBody\n";
        check(
            source,
            |entry| entry.add_history("2026-11-21T09:00:00+01:00", &anna(), "created", ""),
            "# DEC-012 · Title\n\n- history: 2026-11-21T09:00:00+01:00 Anna: created\n\n## Decision\n\nBody\n",
        );
    }

    #[test]
    fn set_field_on_an_entry_without_field_lines_adds_the_block_after_the_heading() {
        check(
            "# DEC-012 · Title\n## Decision\n\nBody\n",
            |entry| entry.set_field("status", "draft"),
            "# DEC-012 · Title\n\n- status: draft\n\n## Decision\n\nBody\n",
        );
        check(
            "# DEC-012 · Title\n\n## Decision\n\nBody\n",
            |entry| entry.set_field("status", "draft"),
            "# DEC-012 · Title\n\n- status: draft\n\n## Decision\n\nBody\n",
        );
    }

    #[test]
    fn a_heading_alone_gets_the_block_after_it_with_lf_when_no_line_says_otherwise() {
        // No line of this file has a CRLF ending, so there is nothing to follow but LF.
        let mut entry = Entry::parse("# DEC-012 · Title").expect("the heading parses");
        entry.set_field("status", "draft");
        assert_eq!(entry.text(), "# DEC-012 · Title\n\n- status: draft\n");
    }

    #[test]
    fn a_changed_last_line_keeps_having_no_line_ending() {
        // The line's own ending is kept, and this line had none.
        check(
            "# DEC-012 · Title\n- status: draft",
            |entry| entry.set_field("status", "approved"),
            "# DEC-012 · Title\n- status: approved",
        );
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
