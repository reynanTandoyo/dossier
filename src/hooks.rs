//! The Claude Code hooks that `dossier hooks install` keeps in `.claude/settings.json`.

use crate::error::{Code, Error, Result};
use serde_json::{Map, Value, json};

/// The command of the SessionStart hook: it prints the brief.
pub const BRIEF: &str = "dossier brief";

/// The command of the PreToolUse hook: it blocks an edit inside a dossier folder.
pub const GUARD: &str = "dossier guard";

/// The matcher of the PreToolUse hook: the tools that write a file.
const EDIT_TOOLS: &str = "Edit|Write|MultiEdit";

/// .claude/settings.json with a SessionStart hook running BRIEF and a PreToolUse hook (matcher
/// "Edit|Write|MultiEdit") running GUARD, each added once; other keys kept. Empty text is "{}".
/// Code::Parse when the text is not a JSON object.
pub fn installed(existing: &str) -> Result<String> {
    let mut root = settings(existing)?;
    let hooks = hooks_or_new(&mut root)?;
    add_once(hooks, "SessionStart", None, BRIEF)?;
    add_once(hooks, "PreToolUse", Some(EDIT_TOOLS), GUARD)?;
    render(&root)
}

/// The settings text with exactly the two entries of `installed` removed; other keys kept. An
/// array or object that the removal left empty goes too; one that was empty before stays.
pub fn uninstalled(existing: &str) -> Result<String> {
    let mut root = settings(existing)?;
    let hooks_emptied = match root.get_mut("hooks") {
        None => false,
        Some(value) => {
            let hooks = object_mut(value, "hooks")?;
            let had_events = !hooks.is_empty();
            remove_dossier_hooks(hooks)?;
            had_events && hooks.is_empty()
        }
    };
    if hooks_emptied {
        root.remove("hooks");
    }
    render(&root)
}

/// The message to block with when a Claude Code PreToolUse payload edits a file inside a folder
/// named DOSSIER; None to allow.
pub fn guard(payload: &str) -> Option<String> {
    let value: Value = serde_json::from_str(without_mark(payload)).ok()?;
    let input = value.get("tool_input")?;
    let path = input
        .get("file_path")
        .and_then(Value::as_str)
        .or_else(|| input.get("path").and_then(Value::as_str))?;
    if !path.split(['/', '\\']).any(|part| part == "DOSSIER") {
        return None;
    }
    Some(format!(
        "{path} is inside DOSSIER: change entries with the dossier commands, for example dossier set or dossier edit"
    ))
}

/// The text without a leading UTF-8 byte-order mark, which a Windows editor may have saved.
fn without_mark(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

/// The settings as a JSON object. Blank text is an empty object; any other text that is not a
/// JSON object is Code::Parse, so that nothing overwrites it.
fn settings(text: &str) -> Result<Map<String, Value>> {
    let text = without_mark(text);
    if text.trim().is_empty() {
        return Ok(Map::new());
    }
    match serde_json::from_str::<Value>(text) {
        Ok(Value::Object(object)) => Ok(object),
        Ok(_) => Err(Error::new(
            Code::Parse,
            "the settings are not a JSON object",
        )),
        Err(error) => Err(Error::new(
            Code::Parse,
            format!("the settings are not valid JSON: {error}"),
        )),
    }
}

/// The settings as pretty JSON with a final LF.
fn render(root: &Map<String, Value>) -> Result<String> {
    let mut text = serde_json::to_string_pretty(root)
        .map_err(|error| Error::new(Code::Io, error.to_string()))?;
    text.push('\n');
    Ok(text)
}

/// The refusal for a value of the settings that is not of the kind it must be.
fn shape(path: &str, kind: &str) -> Error {
    Error::new(
        Code::Parse,
        format!("{path} is not {kind} in the settings; fix it and try again"),
    )
}

/// The object at `value`, which is called `what` in a refusal.
fn object_mut<'a>(value: &'a mut Value, what: &str) -> Result<&'a mut Map<String, Value>> {
    value
        .as_object_mut()
        .ok_or_else(|| shape(what, "an object"))
}

/// The "hooks" object of the settings, added when it is missing.
fn hooks_or_new(root: &mut Map<String, Value>) -> Result<&mut Map<String, Value>> {
    let value = root.entry("hooks").or_insert_with(|| json!({}));
    object_mut(value, "hooks")
}

/// The array of entries of `event`, added when it is missing.
fn array_mut<'a>(hooks: &'a mut Map<String, Value>, event: &str) -> Result<&'a mut Vec<Value>> {
    let value = hooks.entry(event).or_insert_with(|| json!([]));
    value
        .as_array_mut()
        .ok_or_else(|| shape(&format!("hooks.{event}"), "an array"))
}

/// Adds an entry running `command` to the entries of `event`, unless one of them runs it already.
fn add_once(
    hooks: &mut Map<String, Value>,
    event: &str,
    matcher: Option<&str>,
    command: &str,
) -> Result<()> {
    let entries = array_mut(hooks, event)?;
    if entries.iter().any(|entry| runs(entry, command)) {
        return Ok(());
    }
    let hook = json!({"type": "command", "command": command});
    let entry = match matcher {
        Some(matcher) => json!({"matcher": matcher, "hooks": [hook]}),
        None => json!({"hooks": [hook]}),
    };
    entries.push(entry);
    Ok(())
}

/// Whether `entry` has a hook that runs `command`.
fn runs(entry: &Value, command: &str) -> bool {
    entry
        .get("hooks")
        .and_then(Value::as_array)
        .is_some_and(|hooks| hooks.iter().any(|hook| command_of(hook) == Some(command)))
}

/// The command of a hook object, when it has one.
fn command_of(hook: &Value) -> Option<&str> {
    hook.get("command").and_then(Value::as_str)
}

/// Whether a hook object runs BRIEF or GUARD.
fn is_dossier(hook: &Value) -> bool {
    matches!(command_of(hook), Some(BRIEF | GUARD))
}

/// Takes the dossier hooks out of `entry`. Returns false when that leaves the entry with no hooks
/// at all, which means it had only dossier hooks; the entry then goes too.
fn keep_entry(entry: &mut Value) -> bool {
    let Some(hooks) = entry.get_mut("hooks").and_then(Value::as_array_mut) else {
        return true;
    };
    let before = hooks.len();
    hooks.retain(|hook| !is_dossier(hook));
    before == 0 || !hooks.is_empty()
}

/// Removes the dossier hooks from every event of `hooks`. An event whose entries the removal
/// empties goes too.
fn remove_dossier_hooks(hooks: &mut Map<String, Value>) -> Result<()> {
    let mut emptied = Vec::new();
    for (event, value) in hooks.iter_mut() {
        let entries = value
            .as_array_mut()
            .ok_or_else(|| shape(&format!("hooks.{event}"), "an array"))?;
        let before = entries.len();
        entries.retain_mut(keep_entry);
        if before > 0 && entries.is_empty() {
            emptied.push(event.clone());
        }
    }
    for event in emptied {
        hooks.remove(&event);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A settings text with other keys and other hooks in it.
    const OTHER: &str = r#"{
  "model": "invented-model",
  "hooks": {
    "SessionStart": [
      {"hooks": [{"type": "command", "command": "echo start"}]}
    ],
    "Stop": [
      {"matcher": "", "hooks": [{"type": "command", "command": "echo stop"}]}
    ]
  }
}
"#;

    /// The settings text parsed, so that a test does not depend on spacing or key order.
    fn parsed(text: &str) -> Value {
        serde_json::from_str(text).unwrap()
    }

    /// A PreToolUse payload that edits `path`, as Claude Code sends it.
    fn payload(path: &str) -> String {
        json!({"tool_name": "Edit", "tool_input": {"file_path": path}}).to_string()
    }

    #[test]
    fn installs_both_hooks_into_an_empty_object() {
        let text = installed("{}").unwrap();
        assert_eq!(
            parsed(&text),
            json!({
                "hooks": {
                    "SessionStart": [
                        {"hooks": [{"type": "command", "command": "dossier brief"}]}
                    ],
                    "PreToolUse": [{
                        "matcher": "Edit|Write|MultiEdit",
                        "hooks": [{"type": "command", "command": "dossier guard"}]
                    }]
                }
            })
        );
    }

    #[test]
    fn blank_text_counts_as_an_empty_object() {
        assert_eq!(installed("").unwrap(), installed("{}").unwrap());
        assert_eq!(installed(" \n\t").unwrap(), installed("{}").unwrap());
        assert_eq!(uninstalled("  ").unwrap(), "{}\n");
    }

    #[test]
    fn text_ends_with_one_lf() {
        let text = installed("{}").unwrap();
        assert!(text.ends_with("}\n"), "{text}");
        assert!(!text.ends_with("\n\n"), "{text}");
    }

    #[test]
    fn installing_keeps_other_keys_and_hooks() {
        let original = parsed(OTHER);
        let value = parsed(&installed(OTHER).unwrap());
        assert_eq!(value["model"], "invented-model");
        assert_eq!(value["hooks"]["Stop"], original["hooks"]["Stop"]);
        let session = value["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(session.len(), 2, "{value}");
        assert_eq!(session[0], original["hooks"]["SessionStart"][0]);
        assert!(session.contains(&json!({"hooks": [{"type": "command", "command": BRIEF}]})));
        assert_eq!(value["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn installing_twice_gives_the_same_text_with_one_entry_each() {
        let once = installed(OTHER).unwrap();
        assert_eq!(installed(&once).unwrap(), once);
        let value = parsed(&installed(&once).unwrap());
        assert_eq!(value["hooks"]["SessionStart"].as_array().unwrap().len(), 2);
        assert_eq!(value["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
        let empty_once = installed("{}").unwrap();
        assert_eq!(installed(&empty_once).unwrap(), empty_once);
    }

    #[test]
    fn installing_adds_a_hook_only_when_its_command_is_missing() {
        let text = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "dossier guard"}]}]}}"#;
        let value = parsed(&installed(text).unwrap());
        assert_eq!(value["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
        assert_eq!(value["hooks"]["SessionStart"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn uninstalling_after_installing_gives_back_the_other_entries() {
        let text = uninstalled(&installed(OTHER).unwrap()).unwrap();
        assert_eq!(parsed(&text), parsed(OTHER));
    }

    #[test]
    fn uninstalling_after_installing_into_an_empty_object_gives_an_empty_object() {
        assert_eq!(uninstalled(&installed("{}").unwrap()).unwrap(), "{}\n");
    }

    #[test]
    fn uninstalling_removes_only_the_dossier_entries() {
        let text = r#"{"hooks": {
  "SessionStart": [
    {"hooks": [{"type": "command", "command": "dossier brief"}]},
    {"hooks": [{"type": "command", "command": "echo start"}]}
  ],
  "PreToolUse": [
    {"matcher": "Bash", "hooks": [
      {"type": "command", "command": "dossier guard"},
      {"type": "command", "command": "echo check"}
    ]}
  ]
}}"#;
        let value = parsed(&uninstalled(text).unwrap());
        assert_eq!(
            value,
            json!({"hooks": {
                "SessionStart": [{"hooks": [{"type": "command", "command": "echo start"}]}],
                "PreToolUse": [{
                    "matcher": "Bash",
                    "hooks": [{"type": "command", "command": "echo check"}]
                }]
            }})
        );
    }

    #[test]
    fn uninstalling_keeps_arrays_and_objects_that_were_empty_before() {
        let text = r#"{"hooks": {"Stop": []}}"#;
        assert_eq!(parsed(&uninstalled(text).unwrap()), parsed(text));
        let round = uninstalled(&installed(text).unwrap()).unwrap();
        assert_eq!(parsed(&round), parsed(text));
    }

    #[test]
    fn refuses_text_that_is_not_a_settings_object() {
        let texts = [
            "[]",
            "42",
            "not json",
            r#"{"hooks": []}"#,
            r#"{"hooks": {"SessionStart": {}}}"#,
            r#"{"hooks": {"PreToolUse": "text"}}"#,
        ];
        for text in texts {
            assert_eq!(installed(text).unwrap_err().code, Code::Parse, "{text}");
            assert_eq!(uninstalled(text).unwrap_err().code, Code::Parse, "{text}");
        }
    }

    #[test]
    fn a_byte_order_mark_is_skipped_and_not_written_back() {
        let mark = "\u{feff}";
        let installed_plain = installed(OTHER).unwrap();
        assert_eq!(
            installed(&format!("{mark}{OTHER}")).unwrap(),
            installed_plain
        );
        assert_eq!(
            uninstalled(&format!("{mark}{installed_plain}")).unwrap(),
            uninstalled(&installed_plain).unwrap()
        );
        assert_eq!(uninstalled(&format!("{mark}  \n")).unwrap(), "{}\n");
        assert!(
            !installed(&format!("{mark}{OTHER}"))
                .unwrap()
                .starts_with(mark)
        );
    }

    #[test]
    fn guard_skips_a_byte_order_mark_too() {
        assert!(guard(&format!("\u{feff}{}", payload("DOSSIER/x.md"))).is_some());
        assert_eq!(guard(&format!("\u{feff}{}", payload("src/main.rs"))), None);
    }

    #[test]
    fn guard_blocks_an_edit_of_a_decision_in_dossier() {
        assert_eq!(
            guard(&payload("DOSSIER/decisions/DEC-001-x.md")).as_deref(),
            Some(
                "DOSSIER/decisions/DEC-001-x.md is inside DOSSIER: change entries with the dossier commands, for example dossier set or dossier edit"
            )
        );
    }

    #[test]
    fn guard_splits_windows_and_non_ascii_paths_too() {
        assert!(
            guard(&payload(
                r"C:\Projects\My Dossier\DOSSIER\findings\FND-001-a.md"
            ))
            .is_some()
        );
        assert!(guard(&payload("C:/Projekte/DOSSIER/findings/FND-001-a.md")).is_some());
        assert!(guard(&payload(r"C:\Projekte\Ärger\DOSSIER\x.md")).is_some());
    }

    #[test]
    fn guard_allows_files_outside_a_dossier_folder() {
        assert_eq!(guard(&payload("src/main.rs")), None);
        assert_eq!(guard(&payload("DOSSIER-old/findings/x.md")), None);
        assert_eq!(guard(&payload("dossier/findings/x.md")), None);
        assert_eq!(guard(&payload("notes/DOSSIER.md")), None);
    }

    #[test]
    fn guard_reads_the_path_field_when_there_is_no_file_path() {
        let text = json!({"tool_name": "Write", "tool_input": {"path": "DOSSIER/x.md"}});
        assert!(guard(&text.to_string()).is_some());
    }

    #[test]
    fn guard_allows_what_it_cannot_read() {
        let texts = [
            "",
            "not json",
            "[]",
            "{}",
            r#"{"tool_input": {}}"#,
            r#"{"tool_input": {"file_path": 3}}"#,
        ];
        for text in texts {
            assert_eq!(guard(text), None, "{text}");
        }
    }
}
