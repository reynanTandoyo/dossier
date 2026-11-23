//! The attempts of an investigation (format.md §28): one line each in its Attempts section, in
//! the order they were recorded, with a verdict, what was done, the result, a usage record, a
//! task, and who and when.

use crate::entry::{Entry, Kind};
use crate::time;

/// The section that holds the attempts of an investigation (format.md §28).
pub const SECTION: &str = "Attempts";

/// The verdicts an attempt can have (format.md §28).
pub const VERDICTS: [&str; 3] = ["failed", "worked", "unclear"];

/// The separators of the optional parts, in the order a line writes them (format.md §28): did,
/// result, run and task.
const SEPARATORS: [&str; 4] = [" — did: ", " — result: ", " — run: ", " — task: "];

/// One attempt of an investigation (format.md §28).
#[derive(Debug, Clone, PartialEq)]
pub struct Attempt {
    pub n: u32,
    pub verdict: String,
    pub text: String,
    pub did: Option<String>,
    pub result: Option<String>,
    pub run: Option<String>,
    pub task: Option<String>,
    pub time: String,
    pub who: String,
}

/// The attempts of an Attempts section body, in file order; other lines are skipped.
pub fn parse(body: &str) -> Vec<Attempt> {
    body.lines().filter_map(from_line).collect()
}

/// One attempt as its line (format.md §28). parse(line(a)) gives a back.
pub fn line(attempt: &Attempt) -> String {
    let mut text = format!("{}. [{}] {}", attempt.n, attempt.verdict, attempt.text);
    let parts = [&attempt.did, &attempt.result, &attempt.run, &attempt.task];
    for (separator, part) in SEPARATORS.iter().zip(parts) {
        if let Some(value) = part {
            text.push_str(separator);
            text.push_str(value);
        }
    }
    text.push_str(" (");
    text.push_str(&attempt.time);
    text.push(' ');
    text.push_str(&attempt.who);
    text.push(')');
    text
}

/// 1 + the highest number in the body; 1 for an empty body.
pub fn next_number(body: &str) -> u32 {
    parse(body)
        .iter()
        .map(|attempt| attempt.n)
        .max()
        .map_or(1, |highest| highest.saturating_add(1))
}

/// "<n>. <verdict>: <text>[ — <result>]", for brief, status and history.
pub fn summary(attempt: &Attempt) -> String {
    let mut text = format!("{}. {}: {}", attempt.n, attempt.verdict, attempt.text);
    if let Some(result) = &attempt.result {
        text.push_str(" — ");
        text.push_str(result);
    }
    text
}

/// The attempts of an entry (its Attempts section); empty for another kind.
pub fn of(entry: &Entry) -> Vec<Attempt> {
    if entry.kind() != Some(Kind::Investigation) {
        return Vec::new();
    }
    entry
        .section(SECTION)
        .map_or_else(Vec::new, |body| parse(&body))
}

/// A line as an attempt; None when it does not have the form of format.md §28.
fn from_line(line: &str) -> Option<Attempt> {
    let (number, rest) = line.split_once(". ")?;
    let n = number_of(number)?;
    let (verdict, rest) = rest.strip_prefix('[')?.split_once("] ")?;
    if !VERDICTS.contains(&verdict) {
        return None;
    }
    let inside = rest.strip_suffix(')')?;
    let start = inside.rfind(" (")?;
    let (middle, stamp) = (&inside[..start], &inside[start + 2..]);
    let (time, who) = stamp.split_once(' ')?;
    if !time::valid(time) || who.is_empty() {
        return None;
    }
    let (text, parts) = cut(middle);
    if text.is_empty() {
        return None;
    }
    let [did, result, run, task] = parts.map(|part| part.map(str::to_string));
    Some(Attempt {
        n,
        verdict: verdict.to_string(),
        text: text.to_string(),
        did,
        result,
        run,
        task,
        time: time.to_string(),
        who: who.to_string(),
    })
}

/// Splits the middle of a line into its text and its optional parts. Each separator is looked for
/// after the one before it, so the parts come in the order of SEPARATORS; a part that is not
/// there, or is empty, is None.
fn cut(middle: &str) -> (&str, [Option<&str>; 4]) {
    // (slot, start of the separator, start of its value) of each separator that is there.
    let mut found = Vec::new();
    let mut from = 0;
    for (slot, separator) in SEPARATORS.iter().enumerate() {
        if let Some(at) = middle[from..].find(separator) {
            let start = from + at;
            let value = start + separator.len();
            found.push((slot, start, value));
            from = value;
        }
    }
    let mut parts: [Option<&str>; 4] = [None; 4];
    for (index, &(slot, _, value)) in found.iter().enumerate() {
        let end = found.get(index + 1).map_or(middle.len(), |next| next.1);
        parts[slot] = Some(&middle[value..end]).filter(|part| !part.is_empty());
    }
    let text_end = found.first().map_or(middle.len(), |first| first.1);
    (&middle[..text_end], parts)
}

/// The number at the start of an attempt line: digits only.
fn number_of(text: &str) -> Option<u32> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST: &str = "1. [failed] The CFL number is too high — did: CFL 5 to 1 — result: diverges at iteration 180 (2026-12-03T09:30:00+01:00 anna via anna - claude)";
    const SECOND: &str = "2. [failed] The mesh is too coarse at the leading edge — did: one refinement level more — result: same divergence (2026-12-03T10:15:00+01:00 anna via anna - claude)";
    const THIRD: &str = "3. [worked] The flux at the outlet has the wrong sign — did: corrected the sign in the outlet flux — result: converges in 900 iterations (2026-12-03T11:35:00+01:00 anna via anna - claude)";

    fn full() -> Attempt {
        Attempt {
            n: 4,
            verdict: "unclear".to_string(),
            text: "The mesh is too coarse".to_string(),
            did: Some("one refinement level more".to_string()),
            result: Some("same divergence".to_string()),
            run: Some("USE-007".to_string()),
            task: Some("T-013".to_string()),
            time: "2026-12-03T10:15:00+01:00".to_string(),
            who: "anna via anna - claude".to_string(),
        }
    }

    fn investigation(kind: &str) -> Entry {
        let text = format!(
            "<!-- dossier-format: 1 -->\n# INV-003 · Residual blows up after 40 iterations\n\n- kind: {kind}\n- status: approved\n- state: solved\n\n## Problem\n\nThe density residual grows from iteration 40 on.\n\n## Attempts\n\n{FIRST}\n{SECOND}\n{THIRD}\n\n## Solution\n\nThe outlet flux had the wrong sign.\n"
        );
        Entry::parse(&text).unwrap()
    }

    #[test]
    fn a_line_with_every_part_reads_back_as_the_same_attempt() {
        let attempt = full();
        let text = line(&attempt);
        assert_eq!(
            text,
            "4. [unclear] The mesh is too coarse — did: one refinement level more — result: same divergence — run: USE-007 — task: T-013 (2026-12-03T10:15:00+01:00 anna via anna - claude)"
        );
        assert_eq!(parse(&text), vec![attempt]);
    }

    #[test]
    fn a_line_with_only_text_and_verdict_reads_back_as_the_same_attempt() {
        let attempt = Attempt {
            n: 1,
            verdict: "worked".to_string(),
            text: "Restarted from the converged solution".to_string(),
            did: None,
            result: None,
            run: None,
            task: None,
            time: "2026-12-03T09:30:00+01:00".to_string(),
            who: "ben".to_string(),
        };
        let text = line(&attempt);
        assert_eq!(
            text,
            "1. [worked] Restarted from the converged solution (2026-12-03T09:30:00+01:00 ben)"
        );
        assert_eq!(parse(&text), vec![attempt]);
    }

    #[test]
    fn the_example_lines_read_as_attempts_one_two_and_three() {
        let body = [FIRST, SECOND, THIRD].join("\n");
        let attempts = parse(&body);
        let numbers: Vec<u32> = attempts.iter().map(|attempt| attempt.n).collect();
        assert_eq!(numbers, [1, 2, 3]);
        let verdicts: Vec<&str> = attempts
            .iter()
            .map(|attempt| attempt.verdict.as_str())
            .collect();
        assert_eq!(verdicts, ["failed", "failed", "worked"]);
        assert_eq!(attempts[0].text, "The CFL number is too high");
        assert_eq!(attempts[0].did.as_deref(), Some("CFL 5 to 1"));
        assert_eq!(
            attempts[2].result.as_deref(),
            Some("converges in 900 iterations")
        );
        assert_eq!(attempts[1].time, "2026-12-03T10:15:00+01:00");
        assert_eq!(attempts[1].who, "anna via anna - claude");
        assert_eq!(attempts[0].run, None);
        assert_eq!(attempts[0].task, None);
    }

    #[test]
    fn a_free_text_line_is_skipped_and_the_next_number_is_four() {
        let body =
            format!("{FIRST}\nWe also looked at the inlet; nothing found.\n{SECOND}\n{THIRD}\n");
        assert_eq!(parse(&body).len(), 3);
        assert_eq!(next_number(&body), 4);
    }

    #[test]
    fn lines_that_do_not_have_the_form_are_skipped() {
        let body = [
            "1. [failed] A first attempt (2026-12-03T09:30:00+01:00 anna)",
            "2. [maybe] A verdict that does not exist (2026-12-03T09:30:00+01:00 anna)",
            "3. [worked] No time and no name",
            "4. [worked] A time that is not a time (2026-12-03 09:30 anna)",
            "5. [failed] Nobody wrote it (2026-12-03T09:30:00+01:00)",
            "6. [failed]  (2026-12-03T09:30:00+01:00 anna)",
            "[failed] Not numbered (2026-12-03T09:30:00+01:00 anna)",
        ]
        .join("\n");
        let attempts = parse(&body);
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].n, 1);
        assert_eq!(next_number(&body), 2);
    }

    #[test]
    fn the_last_bracket_starts_the_time_so_parentheses_in_the_text_stay() {
        let attempts = parse(
            "1. [failed] The CFL number (roughly) is too high (2026-12-03T09:30:00+01:00 anna)",
        );
        assert_eq!(attempts[0].text, "The CFL number (roughly) is too high");
        assert_eq!(attempts[0].who, "anna");
    }

    #[test]
    fn who_with_spaces_survives_the_round_trip() {
        let attempts = parse(FIRST);
        assert_eq!(attempts[0].who, "anna via anna - claude");
        assert_eq!(line(&attempts[0]), FIRST);
    }

    #[test]
    fn a_body_with_crlf_line_endings_reads_the_same_attempts() {
        let with_crlf = format!("{FIRST}\r\n{SECOND}\r\n");
        assert_eq!(parse(&with_crlf), parse(&format!("{FIRST}\n{SECOND}\n")));
    }

    #[test]
    fn the_summary_is_number_verdict_text_and_result() {
        let attempts = parse(FIRST);
        assert_eq!(
            summary(&attempts[0]),
            "1. failed: The CFL number is too high — diverges at iteration 180"
        );
        let without_result =
            parse("2. [unclear] Looked at the logs (2026-12-03T09:30:00+01:00 ben)");
        assert_eq!(
            summary(&without_result[0]),
            "2. unclear: Looked at the logs"
        );
    }

    #[test]
    fn next_number_is_one_for_a_body_without_attempts() {
        assert_eq!(next_number(""), 1);
        assert_eq!(next_number("Nothing recorded yet.\n"), 1);
    }

    #[test]
    fn of_reads_the_attempts_of_an_investigation_and_nothing_of_another_kind() {
        let attempts = of(&investigation("investigation"));
        assert_eq!(attempts.len(), 3);
        assert_eq!(attempts[2].verdict, "worked");
        assert!(of(&investigation("decision")).is_empty());
    }
}
