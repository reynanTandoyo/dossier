//! Entries: the files of decisions, assumptions, findings, lessons, usage records, how-tos and
//! tools.

pub mod edit;
pub mod new;
pub mod rules;

use std::ops::Range;

use crate::error::{Code, Error, Result};

/// The byte-order mark that may stand at the start of a file (format.md §2).
const BYTE_ORDER_MARK: char = '\u{feff}';

/// The kind of an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Decision,
    Assumption,
    Finding,
    Lesson,
    Usage,
    Howto,
    Tool,
}

impl Kind {
    /// The seven kinds.
    pub fn all() -> [Kind; 7] {
        [
            Kind::Decision,
            Kind::Assumption,
            Kind::Finding,
            Kind::Lesson,
            Kind::Usage,
            Kind::Howto,
            Kind::Tool,
        ]
    }

    /// The name of the kind.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Decision => "decision",
            Kind::Assumption => "assumption",
            Kind::Finding => "finding",
            Kind::Lesson => "lesson",
            Kind::Usage => "usage",
            Kind::Howto => "howto",
            Kind::Tool => "tool",
        }
    }

    /// The prefix of IDs of this kind.
    pub fn prefix(self) -> &'static str {
        match self {
            Kind::Decision => "DEC",
            Kind::Assumption => "ASM",
            Kind::Finding => "FND",
            Kind::Lesson => "LES",
            Kind::Usage => "USE",
            Kind::Howto => "HOW",
            Kind::Tool => "TOOL",
        }
    }

    /// The folder of this kind inside DOSSIER.
    pub fn folder(self) -> &'static str {
        match self {
            Kind::Decision => "decisions",
            Kind::Assumption => "assumptions",
            Kind::Finding => "findings",
            Kind::Lesson => "lessons",
            Kind::Usage => "usage",
            Kind::Howto => "howtos",
            Kind::Tool => "tools",
        }
    }

    /// The kind by name or prefix, in any case.
    pub fn parse(text: &str) -> Option<Kind> {
        Kind::all().into_iter().find(|kind| {
            text.eq_ignore_ascii_case(kind.name()) || text.eq_ignore_ascii_case(kind.prefix())
        })
    }

    /// The kind of an ID, with or without a place prefix (`P-017:DEC-012`), in any case.
    /// The prefix is the part before the first `-` after the last `:`.
    pub fn of_id(id: &str) -> Option<Kind> {
        let without_place = id.rsplit_once(':').map_or(id, |(_, rest)| rest);
        let prefix = without_place
            .split_once('-')
            .map_or(without_place, |(head, _)| head);
        Kind::all()
            .into_iter()
            .find(|kind| prefix.eq_ignore_ascii_case(kind.prefix()))
    }
}

/// The status of an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Draft,
    Approved,
    Rejected,
    Superseded,
    Retired,
    Proposed,
    Declined,
}

impl Status {
    /// The name of the status, as the file writes it.
    pub fn name(self) -> &'static str {
        match self {
            Status::Draft => "draft",
            Status::Approved => "approved",
            Status::Rejected => "rejected",
            Status::Superseded => "superseded",
            Status::Retired => "retired",
            Status::Proposed => "proposed",
            Status::Declined => "declined",
        }
    }

    /// The status by name, in any case.
    pub fn parse(text: &str) -> Option<Status> {
        [
            Status::Draft,
            Status::Approved,
            Status::Rejected,
            Status::Superseded,
            Status::Retired,
            Status::Proposed,
            Status::Declined,
        ]
        .into_iter()
        .find(|status| text.eq_ignore_ascii_case(status.name()))
    }
}

/// One entry file, kept as its lines so that every byte that is not changed stays as it was.
#[derive(Debug, Clone)]
pub struct Entry {
    /// Every line, with its own line ending.
    pub lines: Vec<String>,
}

impl Entry {
    /// Parses an entry file. Code::Parse with the line number when it is malformed.
    ///
    /// A format line above 1 is Code::TooNew. A text without a heading line (`# `) is
    /// Code::Parse; its line number is the line where the heading should be.
    pub fn parse(text: &str) -> Result<Entry> {
        let lines: Vec<String> = text.split_inclusive('\n').map(str::to_string).collect();
        let heading = heading_index(&lines);
        let format = format_line(&lines, heading.unwrap_or(lines.len()));
        if format.is_some_and(|(_, number)| number > 1) {
            return Err(Error::new(
                Code::TooNew,
                "this entry needs a newer dossier program",
            ));
        }
        if heading.is_none() {
            let line = format.map_or(1, |(index, _)| index + 2);
            return Err(Error::new(Code::Parse, format!("line {line}: no heading")));
        }
        Ok(Entry { lines })
    }

    /// The lines joined; equals the text that was parsed.
    pub fn text(&self) -> String {
        self.lines.concat()
    }

    /// The ID of the entry: the first word of the heading, upper-cased.
    pub fn id(&self) -> String {
        self.heading().map_or_else(String::new, |heading| {
            let heading = heading.trim();
            let end = heading.find(ends_id).unwrap_or(heading.len());
            heading[..end].to_uppercase()
        })
    }

    /// The title of the entry: the heading after the ID and its separator.
    pub fn title(&self) -> String {
        self.heading().map_or_else(String::new, |heading| {
            let heading = heading.trim();
            let end = heading.find(ends_id).unwrap_or(heading.len());
            strip_separator(heading[end..].trim()).to_string()
        })
    }

    /// The value of the first line with that key.
    pub fn field(&self, key: &str) -> Option<String> {
        self.fields()
            .into_iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    /// Every field in file order, history included.
    pub fn fields(&self) -> Vec<(String, String)> {
        self.lines[field_range(&self.lines)]
            .iter()
            .filter_map(|line| parse_field(content(line)))
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    /// A comma-separated field, trimmed. Empty items are left out.
    pub fn list(&self, key: &str) -> Vec<String> {
        self.field(key).map_or_else(Vec::new, |value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect()
        })
    }

    /// The body of a section (format.md §3). The name is matched exactly.
    pub fn section(&self, name: &str) -> Option<String> {
        section_ranges(&self.lines)
            .into_iter()
            .find(|(section, _)| section == name)
            .map(|(_, range)| self.section_body(range))
    }

    /// Every section, in file order.
    pub fn sections(&self) -> Vec<(String, String)> {
        section_ranges(&self.lines)
            .into_iter()
            .map(|(name, range)| (name, self.section_body(range)))
            .collect()
    }

    /// The kind of the entry, from its `kind` field.
    pub fn kind(&self) -> Option<Kind> {
        self.field("kind").and_then(|value| Kind::parse(&value))
    }

    /// The status of the entry, from its `status` field.
    pub fn status(&self) -> Option<Status> {
        self.field("status").and_then(|value| Status::parse(&value))
    }

    /// The values of the history lines, in file order.
    pub fn history(&self) -> Vec<String> {
        self.fields()
            .into_iter()
            .filter(|(key, _)| key == "history")
            .map(|(_, value)| value)
            .collect()
    }

    /// The text after `# ` in the heading line, without its line ending.
    fn heading(&self) -> Option<&str> {
        let index = heading_index(&self.lines)?;
        plain(&self.lines, index).strip_prefix("# ")
    }

    /// The body of the section whose range (heading line included) is `range`.
    fn section_body(&self, range: Range<usize>) -> String {
        body_of(&self.lines[range.start + 1..range.end])
    }
}

/// Whether `c` ends the ID at the start of a heading: a space or a colon.
fn ends_id(c: char) -> bool {
    c.is_whitespace() || c == ':'
}

/// The title after a separator of format.md §3: ` · `, ` - ` or `: `. A separator with no
/// title after it gives an empty title. Anything else is the title as it is.
fn strip_separator(rest: &str) -> &str {
    for mark in ["·", "-", ":"] {
        if let Some(after) = rest
            .strip_prefix(mark)
            .filter(|after| after.is_empty() || after.starts_with(char::is_whitespace))
        {
            return after.trim();
        }
    }
    rest
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

/// The index of the first line that starts with `# ` (the heading, format.md §3).
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

/// A field line, `- key: value` (format.md §3, §5). The key is lower-case letters, digits and
/// `-`; the value is trimmed and may be empty.
fn parse_field(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("- ")?;
    let (key, value) = rest.split_once(':')?;
    let valid_key = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    valid_key.then_some((key, value.trim()))
}

/// The field block: the field lines after the heading, before the first section, from the first
/// field line up to the first line that is not one. Without a field line it is empty, at the
/// line after the heading. Without a heading it is empty at 0.
fn field_range(lines: &[String]) -> Range<usize> {
    let Some(heading) = heading_index(lines) else {
        return 0..0;
    };
    let after = heading + 1;
    let first_section = (after..lines.len())
        .find(|&index| content(&lines[index]).starts_with("## "))
        .unwrap_or(lines.len());
    match (after..first_section).find(|&index| parse_field(content(&lines[index])).is_some()) {
        Some(start) => {
            let end = (start..lines.len())
                .find(|&index| parse_field(content(&lines[index])).is_none())
                .unwrap_or(lines.len());
            start..end
        }
        None => after..after,
    }
}

/// Every section: its name and the range of its lines, from its `## ` line up to the next
/// `## ` line or the end of the file.
fn section_ranges(lines: &[String]) -> Vec<(String, Range<usize>)> {
    let starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| content(line).starts_with("## "))
        .map(|(index, _)| index)
        .collect();
    starts
        .iter()
        .enumerate()
        .map(|(number, &start)| {
            let end = starts.get(number + 1).copied().unwrap_or(lines.len());
            let line = content(&lines[start]);
            let name = line.strip_prefix("## ").unwrap_or(line).trim().to_string();
            (name, start..end)
        })
        .collect()
}

/// The body of a section: its lines without blank lines at the start and the end, each line's
/// ending removed, joined with LF.
fn body_of(lines: &[String]) -> String {
    let texts: Vec<&str> = lines.iter().map(|line| content(line)).collect();
    let first = texts.iter().position(|text| !text.trim().is_empty());
    let last = texts.iter().rposition(|text| !text.trim().is_empty());
    match (first, last) {
        (Some(first), Some(last)) => texts[first..=last].join("\n"),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// format.md section 1, the first example, copied exactly.
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

    /// format.md section 1, the second example (an assumption), copied exactly.
    const ASSUMPTION: &str = r#"<!-- dossier-format: 1 -->
# ASM-003 · Inlet profile is uniform

- kind: assumption
- status: approved
- created: 2026-11-20T09:12:05+01:00
- author: ben
- approved: 2026-11-20T09:12:05+01:00 by ben
- valid-from: 2026-11-20T09:12:05+01:00
- state: open
- task: T-040
- history: 2026-11-20T09:12:05+01:00 ben: created
- history: 2026-11-20T09:12:05+01:00 ben: approved

## Assumption

The total pressure and temperature at the inlet are uniform over the span.

## Reason

No measured profile was delivered.

## Affects

T-042, T-043
"#;

    /// format.md section 1, the third example (a finding that is a dead end), copied exactly.
    const FINDING: &str = r#"<!-- dossier-format: 1 -->
# FND-007 · Halving the time step did not remove the oscillation

- kind: finding
- status: approved
- created: 2026-11-21T16:40:11+01:00
- author: anna
- via: anna-claude
- approved: 2026-11-21T16:40:11+01:00 by anna
- valid-from: 2026-11-21T16:40:11+01:00
- outcome: dead end
- task: T-042
- history: 2026-11-21T16:40:11+01:00 anna via anna-claude: created
- history: 2026-11-21T16:40:11+01:00 anna via anna-claude: approved

## Observation

The mass flow still oscillated by 0.4 % with half the time step. The cause is not the time step.

## Evidence

Runs 12 and 13, monitor plots in results/monitors/.
"#;

    /// format.md section 1, the fourth example (a lesson drafted by an LLM), copied exactly.
    const LESSON: &str = r#"<!-- dossier-format: 1 -->
# LES-004 · Steady mixing-plane runs overpredict stall margin

- kind: lesson
- status: draft
- created: 2026-11-21T17:02:44+01:00
- author: anna
- via: anna-claude
- confidence: simulated
- task: T-042
- machine: axial compressor
- analysis: steady RANS, speedline
- history: 2026-11-21T17:02:44+01:00 anna via anna-claude: created

## Statement

Near the stability limit, steady runs with a mixing-plane interface converged at mass flows
where the transient run already showed rotating stall.

## Valid for

Transonic front stages, tip Mach number above 1.1. Not checked for subsonic rear stages.

## Evidence

Comparison of steady and transient speedlines, report section 4.2.
"#;

    #[test]
    fn the_four_examples_of_format_section_1_parse_and_keep_every_byte() {
        for example in [DECISION, ASSUMPTION, FINDING, LESSON] {
            let entry = Entry::parse(example).expect("a format.md example parses");
            assert_eq!(entry.text(), example);
        }
    }

    #[test]
    fn decision_example_gives_its_id_title_fields_history_and_sections() {
        let entry = Entry::parse(DECISION).expect("parses");
        assert_eq!(entry.id(), "DEC-012");
        assert_eq!(
            entry.title(),
            "Use transient runs for the last three speedline points"
        );
        assert_eq!(entry.kind(), Some(Kind::Decision));
        assert_eq!(entry.status(), Some(Status::Approved));
        assert_eq!(entry.field("task").as_deref(), Some("T-042"));
        assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
        assert_eq!(entry.field("tags"), None);
        assert_eq!(
            entry.field("approved").as_deref(),
            Some("2026-11-20T14:03:27+01:00 by anna")
        );
        assert_eq!(
            entry.list("analysis"),
            vec!["steady RANS".to_string(), "speedline".to_string()]
        );
        assert_eq!(
            entry.history(),
            vec![
                "2026-11-20T14:03:27+01:00 anna via anna-claude: created".to_string(),
                "2026-11-20T14:03:27+01:00 anna via anna-claude: approved".to_string(),
            ]
        );
        assert_eq!(
            entry.section("Decision").as_deref(),
            Some("The last three points of every speedline are run transient.")
        );
        assert_eq!(
            entry.section("Rejected").as_deref(),
            Some("Steady runs with a finer mesh: same result, three times the cost.")
        );
        let names: Vec<String> = entry.sections().into_iter().map(|(name, _)| name).collect();
        assert_eq!(names, ["Decision", "Reason", "Rejected"]);
    }

    #[test]
    fn assumption_example_reads_its_state_and_affects_section() {
        let entry = Entry::parse(ASSUMPTION).expect("parses");
        assert_eq!(entry.id(), "ASM-003");
        assert_eq!(entry.title(), "Inlet profile is uniform");
        assert_eq!(entry.kind(), Some(Kind::Assumption));
        assert_eq!(entry.field("state").as_deref(), Some("open"));
        assert_eq!(entry.section("Affects").as_deref(), Some("T-042, T-043"));
        assert_eq!(
            entry.section("Assumption").as_deref(),
            Some("The total pressure and temperature at the inlet are uniform over the span.")
        );
    }

    #[test]
    fn finding_example_reads_its_outcome() {
        let entry = Entry::parse(FINDING).expect("parses");
        assert_eq!(entry.id(), "FND-007");
        assert_eq!(entry.kind(), Some(Kind::Finding));
        assert_eq!(entry.status(), Some(Status::Approved));
        assert_eq!(entry.field("outcome").as_deref(), Some("dead end"));
        assert_eq!(
            entry.section("Evidence").as_deref(),
            Some("Runs 12 and 13, monitor plots in results/monitors/.")
        );
    }

    #[test]
    fn lesson_example_is_a_draft_and_its_multi_line_section_is_joined_with_lf() {
        let entry = Entry::parse(LESSON).expect("parses");
        assert_eq!(entry.id(), "LES-004");
        assert_eq!(entry.kind(), Some(Kind::Lesson));
        assert_eq!(entry.status(), Some(Status::Draft));
        assert_eq!(entry.field("confidence").as_deref(), Some("simulated"));
        assert_eq!(
            entry.section("Statement").as_deref(),
            Some(
                "Near the stability limit, steady runs with a mixing-plane interface converged at mass flows\nwhere the transient run already showed rotating stall."
            )
        );
        assert_eq!(
            entry.section("Valid for").as_deref(),
            Some(
                "Transonic front stages, tip Mach number above 1.1. Not checked for subsonic rear stages."
            )
        );
    }

    #[test]
    fn crlf_endings_read_the_same_values_and_keep_every_byte() {
        let crlf = DECISION.replace('\n', "\r\n");
        let entry = Entry::parse(&crlf).expect("parses");
        assert_eq!(entry.text(), crlf);
        let lf = Entry::parse(DECISION).expect("parses");
        assert_eq!(entry.id(), lf.id());
        assert_eq!(entry.title(), lf.title());
        assert_eq!(entry.fields(), lf.fields());
        assert_eq!(entry.sections(), lf.sections());
        assert_eq!(entry.field("status").as_deref(), Some("approved"));
        assert!(!entry.title().contains('\r'));
    }

    #[test]
    fn a_byte_order_mark_is_accepted_and_kept() {
        let with_mark = format!("\u{feff}{DECISION}");
        let entry = Entry::parse(&with_mark).expect("parses");
        assert_eq!(entry.text(), with_mark);
        assert_eq!(entry.id(), "DEC-012");
        assert_eq!(entry.kind(), Some(Kind::Decision));
        assert_eq!(entry.status(), Some(Status::Approved));
    }

    #[test]
    fn a_byte_order_mark_before_a_heading_without_format_line_is_skipped() {
        let entry = Entry::parse("\u{feff}# ASM-003 · Inlet profile is uniform\n").expect("parses");
        assert_eq!(entry.id(), "ASM-003");
        assert_eq!(entry.title(), "Inlet profile is uniform");
    }

    #[test]
    fn a_byte_order_mark_before_the_format_line_is_skipped_when_reading_the_version() {
        let error = Entry::parse("\u{feff}<!-- dossier-format: 2 -->\n# DEC-001 · x\n")
            .expect_err("a newer format is refused");
        assert_eq!(error.code, Code::TooNew);
    }

    #[test]
    fn a_file_without_final_line_ending_keeps_every_byte() {
        let text = DECISION
            .strip_suffix('\n')
            .expect("the example ends with LF");
        let entry = Entry::parse(text).expect("parses");
        assert_eq!(entry.text(), text);
        assert_eq!(entry.id(), "DEC-012");
        assert_eq!(
            entry.section("Rejected").as_deref(),
            Some("Steady runs with a finer mesh: same result, three times the cost.")
        );
    }

    #[test]
    fn unknown_key_and_unknown_section_are_both_returned_and_kept() {
        let text = DECISION
            .replace(
                "- task: T-042\n",
                "- task: T-042\n- custom-note: kept as written\n",
            )
            .replace(
                "## Rejected\n",
                "## Notes\n\nA note of the writer.\n\n## Rejected\n",
            );
        let entry = Entry::parse(&text).expect("parses");
        assert_eq!(entry.text(), text);
        assert_eq!(
            entry.field("custom-note").as_deref(),
            Some("kept as written")
        );
        assert!(
            entry
                .fields()
                .contains(&("custom-note".to_string(), "kept as written".to_string()))
        );
        assert_eq!(
            entry.section("Notes").as_deref(),
            Some("A note of the writer.")
        );
        let names: Vec<String> = entry.sections().into_iter().map(|(name, _)| name).collect();
        assert_eq!(names, ["Decision", "Reason", "Notes", "Rejected"]);
    }

    #[test]
    fn each_heading_separator_gives_the_same_id_and_title() {
        for heading in [
            "# DEC-012 · Use transient runs",
            "# DEC-012 - Use transient runs",
            "# DEC-012: Use transient runs",
        ] {
            let entry = Entry::parse(&format!("{heading}\n")).expect("parses");
            assert_eq!(entry.id(), "DEC-012", "{heading}");
            assert_eq!(entry.title(), "Use transient runs", "{heading}");
        }
    }

    #[test]
    fn the_id_is_upper_cased_and_the_title_keeps_its_own_separators() {
        let entry = Entry::parse("# dec-012 · Plan A - then B: no\n").expect("parses");
        assert_eq!(entry.id(), "DEC-012");
        assert_eq!(entry.title(), "Plan A - then B: no");
    }

    #[test]
    fn a_heading_with_no_title_has_an_empty_title() {
        let entry = Entry::parse("# DEC-012\n").expect("parses");
        assert_eq!(entry.id(), "DEC-012");
        assert_eq!(entry.title(), "");
        let with_separator = Entry::parse("# DEC-012 ·\n").expect("parses");
        assert_eq!(with_separator.title(), "");
    }

    #[test]
    fn a_heading_without_separator_takes_the_first_word_as_id() {
        let entry = Entry::parse("# LES-004 Steady runs overpredict\n").expect("parses");
        assert_eq!(entry.id(), "LES-004");
        assert_eq!(entry.title(), "Steady runs overpredict");
    }

    #[test]
    fn a_file_without_heading_fails_with_its_line_number() {
        let error =
            Entry::parse("<!-- dossier-format: 1 -->\nno heading here\n").expect_err("no heading");
        assert_eq!(error.code, Code::Parse);
        assert_eq!(error.message, "line 2: no heading");

        let error = Entry::parse("just some text\n").expect_err("no heading");
        assert_eq!(error.code, Code::Parse);
        assert_eq!(error.message, "line 1: no heading");

        let error = Entry::parse("").expect_err("empty file has no heading");
        assert_eq!(error.code, Code::Parse);
        assert_eq!(error.message, "line 1: no heading");
    }

    #[test]
    fn a_format_line_above_one_is_refused_as_too_new() {
        let error =
            Entry::parse("<!-- dossier-format: 2 -->\n# DEC-001 · x\n").expect_err("too new");
        assert_eq!(error.code, Code::TooNew);
        assert_eq!(error.message, "this entry needs a newer dossier program");
        assert_eq!(error.exit(), 4);
    }

    #[test]
    fn a_file_without_format_line_is_read_as_version_one() {
        let entry = Entry::parse("# DEC-001 · x\n\n- kind: decision\n").expect("parses");
        assert_eq!(entry.kind(), Some(Kind::Decision));
    }

    #[test]
    fn a_field_line_needs_a_valid_key_and_a_space_after_the_dash() {
        let text = "# DEC-001 · x\n- empty:\n- key-2: two\n- Key: upper case ends the block\n- later: never read\n";
        let entry = Entry::parse(text).expect("parses");
        assert_eq!(entry.field("empty").as_deref(), Some(""));
        assert_eq!(entry.field("key-2").as_deref(), Some("two"));
        assert_eq!(entry.field("later"), None);
        let keys: Vec<String> = entry.fields().into_iter().map(|(key, _)| key).collect();
        assert_eq!(keys, ["empty", "key-2"]);

        let bad = Entry::parse("# DEC-001 · x\n-nospace: a\n-  two: b\n").expect("parses");
        assert!(bad.fields().is_empty());
    }

    #[test]
    fn field_values_are_trimmed_and_never_hold_the_line_ending() {
        let entry = Entry::parse("# DEC-001 · x\r\n- task:   T-042   \r\n- status:draft\r\n")
            .expect("parses");
        assert_eq!(entry.field("task").as_deref(), Some("T-042"));
        assert_eq!(entry.field("status").as_deref(), Some("draft"));
        assert_eq!(entry.fields()[0].1, "T-042");
    }

    #[test]
    fn a_bullet_in_a_section_is_not_a_field() {
        let text = "# DEC-001 · x\n\n## Reason\n\n- status: rejected\n";
        let entry = Entry::parse(text).expect("parses");
        assert_eq!(entry.field("status"), None);
        assert_eq!(entry.status(), None);
        assert_eq!(
            entry.section("Reason").as_deref(),
            Some("- status: rejected")
        );
    }

    #[test]
    fn section_body_drops_blank_lines_at_both_ends_and_keeps_inner_ones() {
        let text =
            "# DEC-001 · x\r\n\r\n## Reason\r\n\r\nfirst\r\n\r\nsecond\r\n\r\n## Next\r\n\r\n";
        let entry = Entry::parse(text).expect("parses");
        assert_eq!(entry.section("Reason").as_deref(), Some("first\n\nsecond"));
        assert_eq!(entry.section("Next").as_deref(), Some(""));
    }

    #[test]
    fn section_names_are_matched_exactly() {
        let entry = Entry::parse(LESSON).expect("parses");
        assert!(entry.section("Valid for").is_some());
        assert_eq!(entry.section("valid for"), None);
        assert_eq!(entry.section("Valid"), None);
    }

    #[test]
    fn list_splits_on_commas_and_trims_each_item() {
        let entry = Entry::parse("# DEC-001 · x\n- analysis: a, ,  b ,c,\n").expect("parses");
        assert_eq!(entry.list("analysis"), ["a", "b", "c"]);
        assert!(entry.list("machine").is_empty());
    }

    #[test]
    fn kinds_have_the_names_prefixes_and_folders_of_format_section_4() {
        let expected = [
            (Kind::Decision, "decision", "DEC", "decisions"),
            (Kind::Assumption, "assumption", "ASM", "assumptions"),
            (Kind::Finding, "finding", "FND", "findings"),
            (Kind::Lesson, "lesson", "LES", "lessons"),
            (Kind::Usage, "usage", "USE", "usage"),
            (Kind::Howto, "howto", "HOW", "howtos"),
            (Kind::Tool, "tool", "TOOL", "tools"),
        ];
        assert_eq!(Kind::all().len(), expected.len());
        for (kind, name, prefix, folder) in expected {
            assert_eq!(kind.name(), name);
            assert_eq!(kind.prefix(), prefix);
            assert_eq!(kind.folder(), folder);
        }
    }

    #[test]
    fn kind_all_lists_the_seven_kinds_in_order() {
        assert_eq!(
            Kind::all(),
            [
                Kind::Decision,
                Kind::Assumption,
                Kind::Finding,
                Kind::Lesson,
                Kind::Usage,
                Kind::Howto,
                Kind::Tool,
            ]
        );
    }

    #[test]
    fn kind_parse_takes_a_name_or_a_prefix_in_any_case() {
        assert_eq!(Kind::parse("decision"), Some(Kind::Decision));
        assert_eq!(Kind::parse("DEC"), Some(Kind::Decision));
        assert_eq!(Kind::parse("Assumption"), Some(Kind::Assumption));
        assert_eq!(Kind::parse("asm"), Some(Kind::Assumption));
        assert_eq!(Kind::parse("FINDING"), Some(Kind::Finding));
        assert_eq!(Kind::parse("les"), Some(Kind::Lesson));
        assert_eq!(Kind::parse("usage"), Some(Kind::Usage));
        assert_eq!(Kind::parse("USE"), Some(Kind::Usage));
        assert_eq!(Kind::parse("howto"), Some(Kind::Howto));
        assert_eq!(Kind::parse("HOW"), Some(Kind::Howto));
        assert_eq!(Kind::parse("Tool"), Some(Kind::Tool));
        assert_eq!(Kind::parse("tool"), Some(Kind::Tool));
        assert_eq!(Kind::parse("note"), None);
        assert_eq!(Kind::parse(""), None);
    }

    #[test]
    fn kind_of_id_strips_the_place_prefix_and_reads_the_prefix_in_any_case() {
        assert_eq!(Kind::of_id("DEC-012"), Some(Kind::Decision));
        assert_eq!(Kind::of_id("dec-012"), Some(Kind::Decision));
        assert_eq!(Kind::of_id("P-017:DEC-012"), Some(Kind::Decision));
        assert_eq!(Kind::of_id("p-017:asm-003"), Some(Kind::Assumption));
        assert_eq!(Kind::of_id("FND-007"), Some(Kind::Finding));
        assert_eq!(Kind::of_id("LES-004"), Some(Kind::Lesson));
        assert_eq!(Kind::of_id("USE-003"), Some(Kind::Usage));
        assert_eq!(Kind::of_id("p-017:use-003"), Some(Kind::Usage));
        assert_eq!(Kind::of_id("HOW-002"), Some(Kind::Howto));
        assert_eq!(Kind::of_id("team:tool-010"), Some(Kind::Tool));
        assert_eq!(Kind::of_id("DEC"), Some(Kind::Decision));
        assert_eq!(Kind::of_id("XYZ-001"), None);
        assert_eq!(Kind::of_id("P-017:"), None);
        assert_eq!(Kind::of_id(""), None);
    }

    #[test]
    fn statuses_have_lower_case_names_and_parse_in_any_case() {
        let expected = [
            (Status::Draft, "draft"),
            (Status::Approved, "approved"),
            (Status::Rejected, "rejected"),
            (Status::Superseded, "superseded"),
            (Status::Retired, "retired"),
            (Status::Proposed, "proposed"),
            (Status::Declined, "declined"),
        ];
        for (status, name) in expected {
            assert_eq!(status.name(), name);
            assert_eq!(Status::parse(name), Some(status));
            assert_eq!(Status::parse(&name.to_uppercase()), Some(status));
        }
        assert_eq!(Status::parse("open"), None);
        assert_eq!(Status::parse(""), None);
    }
}
