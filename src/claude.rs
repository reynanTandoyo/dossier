//! Transcripts of Claude Code. This module is the only place that knows the transcript format of
//! Claude Code, which is internal and may change with any release (D44). Reading is lenient: a
//! line that is not understood is skipped, and nothing here fails.

use serde_json::Value;

use crate::session::Turn;

/// The turns of a transcript of Claude Code: one per JSON line whose `type` is `user` or
/// `assistant`, with text in `message.content`. The text is the content when it is a string, or
/// the `text` values of its items of type `text` joined with a line break. A turn whose text is
/// empty is left out, and every other line is skipped.
pub fn turns(transcript: &str) -> Vec<Turn> {
    transcript.lines().filter_map(turn_of_line).collect()
}

/// The turn of one line of a transcript, or None when the line holds no turn.
fn turn_of_line(line: &str) -> Option<Turn> {
    let value: Value = serde_json::from_str(line).ok()?;
    let role = value.get("type")?.as_str()?;
    if !matches!(role, "user" | "assistant") {
        return None;
    }
    let content = value.get("message")?.get("content")?;
    let text = match content {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .filter(|item| item.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|item| item.get("text").and_then(Value::as_str))
            .collect::<Vec<&str>>()
            .join("\n"),
        _ => return None,
    };
    if text.trim().is_empty() {
        return None;
    }
    Some(Turn {
        role: role.to_string(),
        text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRANSCRIPT: &str = r#"{"type":"summary","summary":"Sizing the SST run"}
{"type":"user","message":{"role":"user","content":"We go with SST."}}
not json at all
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Recorded as DEC-012."},{"type":"tool_use","id":"toolu_1","name":"Bash","input":{"command":"dossier list"}}]}}
"#;

    #[test]
    fn a_string_and_an_array_give_two_turns() {
        let turns = turns(TRANSCRIPT);
        assert_eq!(
            turns,
            vec![
                Turn {
                    role: "user".to_string(),
                    text: "We go with SST.".to_string(),
                },
                Turn {
                    role: "assistant".to_string(),
                    text: "Recorded as DEC-012.".to_string(),
                },
            ]
        );
    }

    #[test]
    fn a_line_that_is_not_json_and_a_summary_line_are_skipped() {
        assert!(turns("not json\n{\"type\":\"summary\",\"summary\":\"x\"}\n\n42\n").is_empty());
    }

    #[test]
    fn text_items_are_joined_with_a_line_break_and_a_turn_without_text_is_left_out() {
        let transcript = concat!(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"a"},{"type":"thinking","thinking":"b"},{"type":"text","text":"c"}]}}"#,
            "\n",
            r#"{"type":"user","message":{"content":[{"type":"tool_result","content":"ok"}]}}"#,
            "\n",
            r#"{"type":"user","message":{"content":"   "}}"#,
        );
        let turns = turns(transcript);
        assert_eq!(
            turns,
            vec![Turn {
                role: "assistant".to_string(),
                text: "a\nc".to_string(),
            }]
        );
    }
}
