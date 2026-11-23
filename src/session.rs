//! Session files (format.md §34, D44): what a person and an LLM said in one session, as plain
//! text, in dossier's own format. `review-session` reads them. Only `claude.rs` knows the
//! transcript format of Claude Code.

use crate::error::{Code, Error, Result};

/// The version this program writes and reads (format.md §34).
pub const VERSION: u32 = 1;

/// The characters kept of one turn by `relevant`; the rest is cut.
const TURN_CHARS: usize = 1500;

/// The words that mark a turn as a decision or a discussed result (R-DRAFT-7), in lower case.
pub const CUES: [&str; 16] = [
    "decide",
    "decided",
    "decision",
    "we will",
    "let's use",
    "instead of",
    "because",
    "assume",
    "result",
    "converge",
    "diverge",
    "failed",
    "works",
    "wrong",
    "fixed",
    "turned out",
];

/// One turn of a session. `role` is "user" or "assistant".
#[derive(Debug, Clone, PartialEq)]
pub struct Turn {
    pub role: String,
    pub text: String,
}

/// The turns of a session file (format.md §34). A version above VERSION is Code::TooNew. A
/// missing format line reads as version 1. Lenient: CRLF and a byte-order mark are accepted, a
/// `## ` heading with another name starts a block that is skipped, and a turn with empty text is
/// left out.
pub fn parse(text: &str) -> Result<Vec<Turn>> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let lines: Vec<&str> = text.lines().collect();
    // The format line stands before the first heading.
    let heading = lines
        .iter()
        .position(|line| line.starts_with('#'))
        .unwrap_or(lines.len());
    let found = lines[..heading].iter().find_map(|line| format_digits(line));
    if let Some(digits) = found {
        let number: u64 = digits.parse().unwrap_or(u64::MAX);
        if number > u64::from(VERSION) {
            return Err(Error::new(
                Code::TooNew,
                format!("this session file has format {digits}; update dossier"),
            ));
        }
    }
    let mut turns = Vec::new();
    let mut open: Option<(&'static str, Vec<&str>)> = None;
    for &line in &lines {
        if let Some(role) = turn_role(line) {
            close(open.take(), &mut turns);
            open = Some((role, Vec::new()));
        } else if line.starts_with("## ") {
            // Another heading: its block is skipped.
            close(open.take(), &mut turns);
        } else if let Some((_, body)) = open.as_mut() {
            body.push(line);
        }
    }
    close(open, &mut turns);
    Ok(turns)
}

/// A session file of the current version (format.md §34): the format line, `# Session`, a
/// `- tool:` line, then one `## <role>` block per turn. A text line that starts with `## ` gets
/// one space in front. parse(render(tool, turns)) gives the turns back.
pub fn render(tool: &str, turns: &[Turn]) -> String {
    let tool = tool.replace(['\r', '\n'], " ");
    let mut out = format!("<!-- dossier-session: {VERSION} -->\n# Session\n\n- tool: {tool}\n");
    for turn in turns {
        let role = &turn.role;
        out.push_str(&format!("\n## {role}\n\n"));
        for line in turn.text.lines() {
            if needs_space(line) {
                out.push(' ');
            }
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// The turns that state a decision or discuss a result (a word of CUES in their text, in lower
/// case), each with the turn before it, in order, as `<role>: <text>` with the text cut to 1500
/// characters. While the total length is above `limit` characters, the oldest is dropped.
pub fn relevant(turns: &[Turn], limit: usize) -> Vec<String> {
    let mut keep = vec![false; turns.len()];
    for (index, turn) in turns.iter().enumerate() {
        let lower = turn.text.to_lowercase();
        if CUES.iter().any(|cue| lower.contains(cue)) {
            keep[index] = true;
            if index > 0 {
                keep[index - 1] = true;
            }
        }
    }
    let mut lines: Vec<String> = turns
        .iter()
        .enumerate()
        .filter(|(index, _)| keep[*index])
        .map(|(_, turn)| {
            let text: String = turn.text.chars().take(TURN_CHARS).collect();
            format!("{}: {text}", turn.role)
        })
        .collect();
    let mut total: usize = lines.iter().map(|line| line.chars().count()).sum();
    let mut dropped = 0;
    while total > limit && dropped < lines.len() {
        total -= lines[dropped].chars().count();
        dropped += 1;
    }
    lines.split_off(dropped)
}

/// The digits N of a format line `<!-- dossier-session: N -->`, with spaces around them free.
fn format_digits(line: &str) -> Option<&str> {
    let inner = line
        .trim()
        .strip_prefix("<!-- dossier-session:")?
        .strip_suffix("-->")?
        .trim();
    let digits = !inner.is_empty() && inner.bytes().all(|byte| byte.is_ascii_digit());
    digits.then_some(inner)
}

/// The role of a turn heading `## user` or `## assistant`, with trailing spaces free.
fn turn_role(line: &str) -> Option<&'static str> {
    match line.trim_end() {
        "## user" => Some("user"),
        "## assistant" => Some("assistant"),
        _ => None,
    }
}

/// Whether a text line is written with one space more in front. After its spaces it starts with
/// `## `, so that it is never read as a heading. Only such lines are changed, and `unescape` takes
/// that one space away again, so every text reads back as it was written.
fn needs_space(line: &str) -> bool {
    line.trim_start_matches(' ').starts_with("## ")
}

/// A text line as it is read: the one space that `needs_space` added is taken away.
fn unescape(line: &str) -> &str {
    if line.starts_with(' ') && needs_space(line) {
        &line[1..]
    } else {
        line
    }
}

/// Adds the turn of an open block to `turns`. Its text is the lines after the blank line that
/// follows the heading, without blank lines at its end. A turn whose text is white space only is
/// left out.
fn close(open: Option<(&'static str, Vec<&str>)>, turns: &mut Vec<Turn>) {
    let Some((role, mut body)) = open else {
        return;
    };
    if body.first().is_some_and(|line| line.is_empty()) {
        body.remove(0);
    }
    while body.last().is_some_and(|line| line.is_empty()) {
        body.pop();
    }
    let text = body
        .iter()
        .map(|&line| unescape(line))
        .collect::<Vec<&str>>()
        .join("\n");
    if !text.trim().is_empty() {
        turns.push(Turn {
            role: role.to_string(),
            text,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// format.md §34, the example as it is written there.
    const EXAMPLE: &str = "<!-- dossier-session: 1 -->\n# Session\n\n- tool: claude code\n\n## user\n\nWe go with the SST model because the test data is for attached flow.\n\n## assistant\n\nRecorded as DEC-012. The first run diverges at iteration 40.\n";

    fn turn(role: &str, text: &str) -> Turn {
        Turn {
            role: role.to_string(),
            text: text.to_string(),
        }
    }

    #[test]
    fn render_then_parse_gives_the_turns_back() {
        let turns = vec![
            turn(
                "user",
                "We go with the SST model.\n\nIt is because of the test data.",
            ),
            turn(
                "assistant",
                "## user\nA line that starts with a heading\n  ## indented\n## assistant",
            ),
            turn("user", " ## one space\n## \n##no space\n\nlast line  "),
            turn("assistant", "\nA first line that is blank"),
        ];
        let text = render("claude code", &turns);
        assert_eq!(parse(&text).unwrap(), turns);
    }

    #[test]
    fn render_writes_a_text_line_with_a_heading_start_with_one_space_in_front() {
        let text = render("claude code", &[turn("user", "## user")]);
        assert_eq!(
            text,
            "<!-- dossier-session: 1 -->\n# Session\n\n- tool: claude code\n\n## user\n\n ## user\n"
        );
    }

    #[test]
    fn render_writes_the_format_line_the_heading_and_the_tool() {
        let text = render("claude code", &[turn("user", "hi")]);
        assert_eq!(
            text,
            "<!-- dossier-session: 1 -->\n# Session\n\n- tool: claude code\n\n## user\n\nhi\n"
        );
    }

    #[test]
    fn the_example_of_format_section_34_parses_to_two_turns() {
        let turns = parse(EXAMPLE).unwrap();
        assert_eq!(
            turns,
            vec![
                turn(
                    "user",
                    "We go with the SST model because the test data is for attached flow."
                ),
                turn(
                    "assistant",
                    "Recorded as DEC-012. The first run diverges at iteration 40."
                ),
            ]
        );
    }

    #[test]
    fn a_format_line_above_the_version_is_too_new() {
        let error = parse("<!-- dossier-session: 2 -->\n# Session\n\n## user\n\nhi\n").unwrap_err();
        assert_eq!(error.code, Code::TooNew);
        assert_eq!(
            error.message,
            "this session file has format 2; update dossier"
        );
    }

    #[test]
    fn a_format_number_too_large_for_u64_is_too_new_with_its_digits() {
        let error =
            parse("<!-- dossier-session: 99999999999999999999999 -->\n# Session\n").unwrap_err();
        assert_eq!(error.code, Code::TooNew);
        assert!(
            error.message.contains("99999999999999999999999"),
            "{}",
            error.message
        );
    }

    #[test]
    fn a_file_without_a_format_line_reads_as_version_1() {
        let turns = parse("# Session\n\n## user\n\nhi\n").unwrap();
        assert_eq!(turns, vec![turn("user", "hi")]);
    }

    #[test]
    fn a_format_line_inside_a_turn_is_text_not_a_version() {
        let turns = parse("# Session\n\n## user\n\n<!-- dossier-session: 2 -->\n").unwrap();
        assert_eq!(turns, vec![turn("user", "<!-- dossier-session: 2 -->")]);
    }

    #[test]
    fn crlf_lines_and_a_byte_order_mark_are_accepted() {
        let turns =
            parse("\u{feff}<!-- dossier-session: 1 -->\r\n# Session\r\n\r\n## user\r\n\r\nhi\r\n")
                .unwrap();
        assert_eq!(turns, vec![turn("user", "hi")]);
    }

    #[test]
    fn a_heading_with_another_name_starts_a_block_that_is_skipped() {
        let turns = parse("# Session\n\n## notes\n\nsecret\n\n## assistant\n\nhi\n").unwrap();
        assert_eq!(turns, vec![turn("assistant", "hi")]);
    }

    #[test]
    fn a_turn_with_empty_text_is_left_out() {
        let turns =
            parse("# Session\n\n## user\n\n\n## assistant\n\n   \n## user\n\nkept\n").unwrap();
        assert_eq!(turns, vec![turn("user", "kept")]);
    }

    #[test]
    fn an_empty_file_has_no_turns() {
        assert_eq!(parse("").unwrap(), Vec::new());
    }

    #[test]
    fn relevant_keeps_a_cue_turn_and_the_turn_before_it_only() {
        let turns = vec![
            turn("user", "Hello."),
            turn("assistant", "Hi, what do you need?"),
            turn("user", "We will use the SST model."),
            turn("assistant", "Good."),
            turn("user", "Thanks."),
        ];
        assert_eq!(
            relevant(&turns, 10_000),
            vec![
                "assistant: Hi, what do you need?".to_string(),
                "user: We will use the SST model.".to_string(),
            ]
        );
    }

    #[test]
    fn relevant_matches_cues_in_any_case() {
        let turns = vec![turn("assistant", "The first run DIVERGED at 40.")];
        assert_eq!(
            relevant(&turns, 1000),
            vec!["assistant: The first run DIVERGED at 40.".to_string()]
        );
    }

    #[test]
    fn relevant_drops_the_oldest_turns_above_the_limit() {
        let turns = vec![
            turn("user", "This decision is final."),
            turn("assistant", "It failed on run 3."),
        ];
        assert_eq!(relevant(&turns, 1000).len(), 2);
        let newest = "assistant: It failed on run 3.";
        assert_eq!(
            relevant(&turns, newest.chars().count()),
            vec![newest.to_string()]
        );
    }

    #[test]
    fn relevant_cuts_a_turn_to_1500_characters() {
        let long = format!("We will {}", "a".repeat(2000));
        let lines = relevant(&[turn("user", &long)], usize::MAX);
        assert_eq!(lines[0].chars().count(), "user: ".len() + 1500);
    }

    #[test]
    fn relevant_of_a_session_without_cues_is_empty() {
        let turns = vec![turn("user", "Hello."), turn("assistant", "Hi.")];
        assert!(relevant(&turns, 1000).is_empty());
    }
}
