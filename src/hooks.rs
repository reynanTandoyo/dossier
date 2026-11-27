//! The Claude Code hooks that `dossier hooks install` keeps in `.claude/settings.json`.

use crate::error::{Code, Error, Result};
use serde_json::{Map, Value, json};

/// The command of the SessionStart hook: it prints the brief.
pub const BRIEF: &str = "dossier brief";

/// The command of the PreToolUse hook: it blocks an edit inside a dossier folder.
pub const GUARD: &str = "dossier guard";

/// The command of the PostToolUse hook: after a `todo` command it prints what the dossier has for
/// that moment (R-AGENT-5).
pub const AFTER: &str = "dossier after";

/// The start of the command of the SessionEnd hook: it runs the team's review command when a
/// session ends (R-DRAFT-9). The full command is this text, `--run` and the quoted review command.
pub const SESSION_END: &str = "dossier session-end";

/// The `shell` of the SessionEnd hook: its command is quoted for a POSIX shell. Claude Code runs a
/// hook on Windows in Git Bash, or in PowerShell when Git Bash is not installed, unless `shell`
/// says which (code.claude.com/docs/en/hooks).
const POSIX_SHELL: &str = "bash";

/// The matcher of the PreToolUse hook: the tools that write a file.
const EDIT_TOOLS: &str = "Edit|Write|MultiEdit";

/// The matcher of the PostToolUse hook: the tools that run a shell command.
const SHELL_TOOLS: &str = "Bash|PowerShell";

/// Seconds Claude Code waits for `dossier after` (a server that does not answer takes 10).
const AFTER_TIMEOUT: u64 = 30;

/// The variable of the local settings that names the writer of Claude's writes (D34).
const AGENT_VAR: &str = "DOSSIER_AGENT";

/// .claude/settings.json with a SessionStart hook running BRIEF, a PreToolUse hook (matcher
/// "Edit|Write|MultiEdit") running GUARD and a PostToolUse hook (matcher "Bash|PowerShell") running
/// AFTER with its timeout, each added once; other keys kept. Empty text is "{}".
/// Code::Parse when the text is not a JSON object.
pub fn installed(existing: &str) -> Result<String> {
    let mut root = settings(existing)?;
    let hooks = object_or_new(&mut root, "hooks")?;
    add_once(hooks, "SessionStart", None, BRIEF, None)?;
    add_once(hooks, "PreToolUse", Some(EDIT_TOOLS), GUARD, None)?;
    add_once(
        hooks,
        "PostToolUse",
        Some(SHELL_TOOLS),
        AFTER,
        Some(AFTER_TIMEOUT),
    )?;
    render(&root)
}

/// The settings text with exactly the entries of `installed` and of `review_installed` removed;
/// other keys kept. An array or object that the removal left empty goes too; one that was empty
/// before stays.
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

/// .claude/settings.json with a SessionEnd hook running `dossier session-end --run <command>`, the
/// command quoted for a POSIX shell (a single quote is written as '\''), and the hook says
/// `"shell": "bash"`. The hook is added once; a SessionEnd hook that runs SESSION_END takes the
/// new command in its place, and only one such hook is kept. Other keys and hooks are kept.
/// Code::BadValue for a command that is empty or only spaces; Code::Parse as in `installed`.
pub fn review_installed(existing: &str, command: &str) -> Result<String> {
    if command.trim().is_empty() {
        return Err(Error::new(
            Code::BadValue,
            "the review command must not be empty",
        ));
    }
    let mut root = settings(existing)?;
    let hooks = object_or_new(&mut root, "hooks")?;
    let run = format!("{SESSION_END} --run {}", shell_quoted(command));
    set_session_end(array_mut(hooks, "SessionEnd")?, &run);
    render(&root)
}

/// The message to block with when a Claude Code PreToolUse payload edits a file inside a folder
/// named DOSSIER; None to allow.
pub fn guard(payload: &str) -> Option<String> {
    let value: Value = serde_json::from_str(without_mark(payload)).ok()?;
    let path = input_path(&value)?;
    if !in_dossier(path) {
        return None;
    }
    Some(format!(
        "{path} is inside DOSSIER: change entries with the dossier commands, for example dossier set or dossier edit"
    ))
}

/// The file that an Edit, Write or MultiEdit payload is about; None for another tool, for a path
/// inside a folder named DOSSIER (guard blocks those) and for anything that is not such a payload.
pub fn edited_path(payload: &str) -> Option<String> {
    let value: Value = serde_json::from_str(without_mark(payload)).ok()?;
    let tool = value.get("tool_name")?.as_str()?;
    if !matches!(tool, "Edit" | "Write" | "MultiEdit") {
        return None;
    }
    let path = input_path(&value)?;
    if in_dossier(path) {
        return None;
    }
    Some(path.to_owned())
}

/// The text of the payload's `session_id`; None when the key is missing or not a text.
pub fn session_of(payload: &str) -> Option<String> {
    let value: Value = serde_json::from_str(without_mark(payload)).ok()?;
    value.get("session_id")?.as_str().map(str::to_owned)
}

/// The `file_path` text of the payload's tool input, or else its `path` text.
fn input_path(value: &Value) -> Option<&str> {
    let input = value.get("tool_input")?;
    input
        .get("file_path")
        .and_then(Value::as_str)
        .or_else(|| input.get("path").and_then(Value::as_str))
}

/// True when a part of the path is a folder named DOSSIER.
fn in_dossier(path: &str) -> bool {
    path.split(['/', '\\']).any(|part| part == "DOSSIER")
}

/// A `todo` command that the hook after shell commands reacts to, with its task ID in upper case.
#[derive(Debug, Clone, PartialEq)]
pub enum TodoEvent {
    Claimed(String),
    Done(String),
    Stuck(String),
}

/// The todo event of a Claude Code PostToolUse payload of a shell tool; None for anything else.
/// The first event of the command text counts; a simple command with `--dry-run`, `--help` or
/// `-h` is no event.
pub fn todo_event(payload: &str) -> Option<TodoEvent> {
    let value: Value = serde_json::from_str(without_mark(payload)).ok()?;
    let command = value.get("tool_input")?.get("command")?.as_str()?;
    command_parts(command).find_map(|part| {
        let words: Vec<&str> = part.split_whitespace().map(unquote).collect();
        if words.iter().any(|word| NO_EVENT_FLAGS.contains(word)) {
            return None;
        }
        part_event(&words, &value)
    })
}

/// {"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":<text>}} as one line.
pub fn context(text: &str) -> String {
    json!({"hookSpecificOutput": {"hookEventName": "PostToolUse", "additionalContext": text}})
        .to_string()
}

/// The words of a simple command that make it no event.
const NO_EVENT_FLAGS: [&str; 3] = ["--dry-run", "--help", "-h"];

/// The simple commands of a command text: the text cut at `&&`, `||`, `;`, `|` and line breaks.
fn command_parts(command: &str) -> impl Iterator<Item = &str> {
    command
        .split([';', '|', '\n', '\r'])
        .flat_map(|part| part.split("&&"))
}

/// The word without one pair of surrounding double or single quotes.
fn unquote(word: &str) -> &str {
    ['"', '\'']
        .into_iter()
        .find_map(|quote| word.strip_prefix(quote)?.strip_suffix(quote))
        .unwrap_or(word)
}

/// The event of one simple command, given as its words with the quotes removed; None when it runs
/// no todo command with an event. `payload` is the whole PostToolUse payload.
fn part_event(words: &[&str], payload: &Value) -> Option<TodoEvent> {
    let arguments = todo_arguments(words)?;
    let (verb, rest) = arguments.split_first()?;
    match *verb {
        "done" => task_id(rest.first()?).map(TodoEvent::Done),
        "stuck" => task_id(rest.first()?).map(TodoEvent::Stuck),
        "claim" => task_id(rest.first()?).map(TodoEvent::Claimed),
        "next" if rest.contains(&"--claim") => {
            let stdout = payload.get("tool_response")?.get("stdout")?.as_str()?;
            stdout
                .split_whitespace()
                .find_map(task_id)
                .map(TodoEvent::Claimed)
        }
        _ => None,
    }
}

/// The words after `todo`, or after `dossier todo`, in a simple command; None when it runs no todo.
/// The program is the first word after any leading environment assignments. Only a program `todo`,
/// or `dossier` followed by `todo`, counts; a `todo` or `dossier` later in the command is an argument.
fn todo_arguments<'a, 'b>(words: &'a [&'b str]) -> Option<&'a [&'b str]> {
    let index = words.iter().position(|word| !is_assignment(word))?;
    let (program, after) = words.get(index..)?.split_first()?;
    if is_program(program, "todo") {
        return Some(after);
    }
    let (next, arguments) = after.split_first()?;
    (is_program(program, "dossier") && is_program(next, "todo")).then_some(arguments)
}

/// Whether a word is an environment assignment, `NAME=value`, where NAME is not empty and holds
/// no `/` or `\`.
fn is_assignment(word: &str) -> bool {
    word.split_once('=')
        .is_some_and(|(name, _)| !name.is_empty() && !name.contains(['/', '\\']))
}

/// Whether a word names the program `name`: the text after the last `/` or `\`, without a trailing
/// `.exe`, equals `name` in any case.
fn is_program(word: &str, name: &str) -> bool {
    let file = word.rsplit(['/', '\\']).next().unwrap_or(word);
    without_exe(file).eq_ignore_ascii_case(name)
}

/// The text without a trailing `.exe` in any case.
fn without_exe(file: &str) -> &str {
    match file.len().checked_sub(4) {
        Some(at) if file.is_char_boundary(at) && file[at..].eq_ignore_ascii_case(".exe") => {
            &file[..at]
        }
        _ => file,
    }
}

/// The task ID that `word` is, in upper case: ASCII letters, `-` and ASCII digits, with at least
/// one letter or digit. None for any other word.
fn task_id(word: &str) -> Option<String> {
    let shaped = word.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    let has_text = word.chars().any(|c| c.is_ascii_alphanumeric());
    (shaped && has_text).then(|| word.to_ascii_uppercase())
}

/// .claude/settings.local.json with env.DOSSIER_AGENT set to `agent` (D34), other keys kept.
/// An older DOSSIER_AGENT is replaced. Blank text is "{}"; Code::Parse as in `installed`.
pub fn local_installed(existing: &str, agent: &str) -> Result<String> {
    let mut root = settings(existing)?;
    let env = object_or_new(&mut root, "env")?;
    env.insert(AGENT_VAR.to_string(), Value::String(agent.to_string()));
    render(&root)
}

/// The local settings text with that one key removed. An "env" object that the removal left
/// empty goes too; one that was empty before stays.
pub fn local_uninstalled(existing: &str) -> Result<String> {
    let mut root = settings(existing)?;
    let env_emptied = match root.get_mut("env") {
        None => false,
        Some(value) => {
            let env = object_mut(value, "env")?;
            let had_keys = !env.is_empty();
            env.remove(AGENT_VAR);
            had_keys && env.is_empty()
        }
    };
    if env_emptied {
        root.remove("env");
    }
    render(&root)
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

/// The object at `key` of the settings (such as "hooks" or "env"), added when it is missing.
fn object_or_new<'a>(
    root: &'a mut Map<String, Value>,
    key: &str,
) -> Result<&'a mut Map<String, Value>> {
    let value = root.entry(key).or_insert_with(|| json!({}));
    object_mut(value, key)
}

/// The array of entries of `event`, added when it is missing.
fn array_mut<'a>(hooks: &'a mut Map<String, Value>, event: &str) -> Result<&'a mut Vec<Value>> {
    let value = hooks.entry(event).or_insert_with(|| json!([]));
    value
        .as_array_mut()
        .ok_or_else(|| shape(&format!("hooks.{event}"), "an array"))
}

/// Adds an entry running `command` (with `timeout` seconds when there is one) to the entries of
/// `event`, unless one of them runs it already.
fn add_once(
    hooks: &mut Map<String, Value>,
    event: &str,
    matcher: Option<&str>,
    command: &str,
    timeout: Option<u64>,
) -> Result<()> {
    let entries = array_mut(hooks, event)?;
    if entries.iter().any(|entry| runs(entry, command)) {
        return Ok(());
    }
    let hook = match timeout {
        Some(seconds) => json!({"type": "command", "command": command, "timeout": seconds}),
        None => json!({"type": "command", "command": command}),
    };
    let entry = match matcher {
        Some(matcher) => json!({"matcher": matcher, "hooks": [hook]}),
        None => json!({"hooks": [hook]}),
    };
    entries.push(entry);
    Ok(())
}

/// Gives the SessionEnd entries the hook that runs `run`. The first hook that runs SESSION_END
/// takes `run` as its command and POSIX_SHELL as its shell, in its place; any other such hook is
/// removed, and an entry the removal leaves without hooks goes too. With no such hook, an entry
/// running `run` is added at the end with the same shell.
fn set_session_end(entries: &mut Vec<Value>, run: &str) {
    let mut found = false;
    entries.retain_mut(|entry| {
        let Some(hooks) = entry.get_mut("hooks").and_then(Value::as_array_mut) else {
            return true;
        };
        let before = hooks.len();
        hooks.retain_mut(|hook| {
            if !is_session_end(hook) {
                return true;
            }
            if found {
                return false;
            }
            found = true;
            if let Some(object) = hook.as_object_mut() {
                object.insert("command".to_string(), Value::String(run.to_string()));
                object.insert("shell".to_string(), Value::String(POSIX_SHELL.to_string()));
            }
            true
        });
        before == 0 || !hooks.is_empty()
    });
    if !found {
        entries.push(json!({"hooks": [
            {"type": "command", "command": run, "shell": POSIX_SHELL}
        ]}));
    }
}

/// `text` as one word for a POSIX shell: in single quotes, each single quote written as '\''.
fn shell_quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
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

/// Whether a hook object runs a command that starts with SESSION_END.
fn is_session_end(hook: &Value) -> bool {
    command_of(hook).is_some_and(|command| command.starts_with(SESSION_END))
}

/// Whether a hook object runs BRIEF, GUARD or AFTER, or a command that starts with SESSION_END.
fn is_dossier(hook: &Value) -> bool {
    matches!(command_of(hook), Some(BRIEF | GUARD | AFTER)) || is_session_end(hook)
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
    fn installs_three_hooks_into_an_empty_object() {
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
                    }],
                    "PostToolUse": [{
                        "matcher": "Bash|PowerShell",
                        "hooks": [{
                            "type": "command",
                            "command": "dossier after",
                            "timeout": 30
                        }]
                    }]
                }
            })
        );
    }

    #[test]
    fn the_after_hook_has_the_shell_matcher_and_a_timeout() {
        let value = parsed(&installed(OTHER).unwrap());
        let after = &value["hooks"]["PostToolUse"];
        assert_eq!(after.as_array().unwrap().len(), 1, "{value}");
        assert_eq!(after[0]["matcher"], "Bash|PowerShell");
        assert_eq!(after[0]["hooks"][0]["command"], AFTER);
        assert_eq!(after[0]["hooks"][0]["timeout"], AFTER_TIMEOUT);
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
    fn uninstalling_removes_the_after_hook_and_keeps_its_neighbours() {
        let text = r#"{"hooks": {
  "PostToolUse": [
    {"matcher": "Bash|PowerShell", "hooks": [
      {"type": "command", "command": "dossier after", "timeout": 30},
      {"type": "command", "command": "echo shell"}
    ]}
  ]
}}"#;
        assert_eq!(
            parsed(&uninstalled(text).unwrap()),
            json!({"hooks": {"PostToolUse": [{
                "matcher": "Bash|PowerShell",
                "hooks": [{"type": "command", "command": "echo shell"}]
            }]}})
        );
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

    #[test]
    fn edited_path_gives_the_file_of_an_edit() {
        assert_eq!(
            edited_path(&payload("src/main.rs")).as_deref(),
            Some("src/main.rs")
        );
    }

    #[test]
    fn edited_path_reads_the_path_field_of_a_write() {
        let text = json!({"tool_name": "Write", "tool_input": {"path": "src/lib.rs"}});
        assert_eq!(
            edited_path(&text.to_string()).as_deref(),
            Some("src/lib.rs")
        );
    }

    #[test]
    fn edited_path_keeps_windows_separators_and_non_ascii_names() {
        assert_eq!(
            edited_path(&payload(r"C:\Projekte\Ärger\My App\src\x.rs")).as_deref(),
            Some(r"C:\Projekte\Ärger\My App\src\x.rs")
        );
    }

    #[test]
    fn edited_path_is_none_for_a_shell_command() {
        let text = json!({"tool_name": "Bash", "tool_input": {"command": "cargo test"}});
        assert_eq!(edited_path(&text.to_string()), None);
    }

    #[test]
    fn edited_path_is_none_for_a_file_inside_dossier() {
        assert_eq!(
            edited_path(&payload("DOSSIER/decisions/DEC-001-x.md")),
            None
        );
        assert_eq!(edited_path(&payload(r"C:\My App\DOSSIER\x.md")), None);
    }

    #[test]
    fn edited_path_is_none_for_text_that_is_not_json() {
        assert_eq!(edited_path("not json"), None);
        assert_eq!(edited_path(""), None);
    }

    #[test]
    fn session_of_gives_the_session_id() {
        let text = json!({"session_id": "invented-session-1", "tool_name": "Edit"});
        assert_eq!(
            session_of(&text.to_string()).as_deref(),
            Some("invented-session-1")
        );
    }

    #[test]
    fn session_of_is_none_without_the_key_or_for_a_non_text() {
        assert_eq!(session_of(&payload("src/main.rs")), None);
        assert_eq!(session_of(r#"{"session_id": 3}"#), None);
        assert_eq!(session_of("not json"), None);
    }

    /// A PostToolUse payload of the Bash tool that ran `command` and printed `stdout`.
    fn shell(command: &str, stdout: &str) -> String {
        json!({
            "tool_name": "Bash",
            "tool_input": {"command": command},
            "tool_response": {"stdout": stdout}
        })
        .to_string()
    }

    /// The todo event of `command` when the shell printed nothing.
    fn event_of(command: &str) -> Option<TodoEvent> {
        todo_event(&shell(command, ""))
    }

    #[test]
    fn todo_done_gives_done_with_the_id_in_upper_case() {
        assert_eq!(
            event_of("todo done T-012 --commit HEAD --agent a"),
            Some(TodoEvent::Done("T-012".to_string()))
        );
        assert_eq!(
            event_of("todo done t-12"),
            Some(TodoEvent::Done("T-12".to_string()))
        );
    }

    #[test]
    fn a_chained_stuck_command_gives_stuck() {
        assert_eq!(
            event_of(r#"cd /x && todo stuck t-7 --note "no data""#),
            Some(TodoEvent::Stuck("T-7".to_string()))
        );
    }

    #[test]
    fn todo_claim_gives_claimed() {
        assert_eq!(
            event_of("todo claim T-3"),
            Some(TodoEvent::Claimed("T-3".to_string()))
        );
    }

    #[test]
    fn dossier_todo_gives_the_same_event() {
        assert_eq!(
            event_of("dossier todo done T-9"),
            Some(TodoEvent::Done("T-9".to_string()))
        );
    }

    #[test]
    fn a_todo_word_that_is_an_argument_gives_no_event() {
        assert_eq!(
            event_of(r#"git commit -m "T-1: todo done does work""#),
            None
        );
        assert_eq!(event_of("echo todo done T-4"), None);
    }

    #[test]
    fn leading_environment_assignments_are_skipped() {
        assert_eq!(
            event_of("TODO_AGENT=anna todo done T-5 --commit HEAD"),
            Some(TodoEvent::Done("T-5".to_string()))
        );
        assert_eq!(
            event_of("A=1 B=2 dossier todo claim T-6"),
            Some(TodoEvent::Claimed("T-6".to_string()))
        );
        assert_eq!(
            event_of("cd x && todo claim T-2"),
            Some(TodoEvent::Claimed("T-2".to_string()))
        );
        assert_eq!(
            event_of("dossier todo done T-3"),
            Some(TodoEvent::Done("T-3".to_string()))
        );
    }

    #[test]
    fn a_word_with_a_slash_before_the_equals_sign_is_the_program() {
        assert_eq!(event_of("tools/x=1 todo claim T-1"), None);
        assert_eq!(event_of(r"C:\bin\x=1 todo claim T-1"), None);
    }

    #[test]
    fn a_program_with_a_path_and_exe_counts_as_todo() {
        assert_eq!(
            event_of(r"C:\tools\todo.exe done T-1"),
            Some(TodoEvent::Done("T-1".to_string()))
        );
        assert_eq!(
            event_of("/usr/local/bin/Todo done T-1"),
            Some(TodoEvent::Done("T-1".to_string()))
        );
    }

    #[test]
    fn a_quoted_word_loses_one_pair_of_quotes() {
        assert_eq!(
            event_of(r#"todo done "T-5""#),
            Some(TodoEvent::Done("T-5".to_string()))
        );
        assert_eq!(
            event_of("'todo' claim 'T-6'"),
            Some(TodoEvent::Claimed("T-6".to_string()))
        );
    }

    #[test]
    fn next_with_claim_takes_the_first_task_id_of_the_output() {
        let payload = shell(
            "todo next --claim --brief --agent a",
            "T-053 · P2 · claimed · a · Title\n",
        );
        assert_eq!(
            todo_event(&payload),
            Some(TodoEvent::Claimed("T-053".to_string()))
        );
    }

    #[test]
    fn next_with_claim_and_no_task_id_in_the_output_gives_none() {
        assert_eq!(event_of("todo next --claim --brief --agent a"), None);
        let no_output = json!({"tool_input": {"command": "todo next --claim"}});
        assert_eq!(todo_event(&no_output.to_string()), None);
    }

    #[test]
    fn commands_that_are_no_event_give_none() {
        let commands = [
            "todo next --brief",
            "todo done T-1 --dry-run",
            "todo stuck T-1 --help",
            "todo claim T-1 -h",
            "todo list",
            "echo todo done",
            r#"git commit -m "todo done T-1""#,
            "dossier brief",
        ];
        for command in commands {
            assert_eq!(event_of(command), None, "{command}");
        }
    }

    #[test]
    fn payloads_that_are_no_shell_command_give_none() {
        let payloads = [
            "",
            "not json",
            "[]",
            "\"todo done T-1\"",
            "{}",
            r#"{"tool_input": {}}"#,
            r#"{"tool_input": {"command": 3}}"#,
        ];
        for payload in payloads {
            assert_eq!(todo_event(payload), None, "{payload}");
        }
    }

    #[test]
    fn the_first_event_of_the_command_text_counts() {
        assert_eq!(
            event_of("todo list && todo done T-2"),
            Some(TodoEvent::Done("T-2".to_string()))
        );
        assert_eq!(
            event_of("todo done T-1 --dry-run; todo claim T-2"),
            Some(TodoEvent::Claimed("T-2".to_string()))
        );
        assert_eq!(
            event_of("todo claim T-3 | tee log\ntodo done T-4"),
            Some(TodoEvent::Claimed("T-3".to_string()))
        );
    }

    #[test]
    fn a_byte_order_mark_is_skipped() {
        let payload = format!("\u{feff}{}", shell("todo claim T-3", ""));
        assert_eq!(
            todo_event(&payload),
            Some(TodoEvent::Claimed("T-3".to_string()))
        );
    }

    #[test]
    fn context_is_one_line_that_parses_back_to_the_text() {
        let text = "first line\nsecond \"line\" · ä";
        let line = context(text);
        assert!(!line.contains('\n'), "{line}");
        let value: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(value["hookSpecificOutput"]["hookEventName"], "PostToolUse");
        assert_eq!(value["hookSpecificOutput"]["additionalContext"], text);
        assert_eq!(value.as_object().unwrap().len(), 1, "{line}");
    }

    /// Local settings with other keys and other environment variables in them.
    const LOCAL_OTHER: &str = r#"{
  "model": "invented-model",
  "env": {
    "INVENTED_VARIABLE": "1"
  }
}
"#;

    #[test]
    fn local_install_into_an_empty_object_sets_the_agent() {
        let text = local_installed("{}", "tony - claude").unwrap();
        assert_eq!(
            parsed(&text),
            json!({"env": {"DOSSIER_AGENT": "tony - claude"}})
        );
        assert!(text.ends_with("}\n") && !text.ends_with("\n\n"), "{text}");
    }

    #[test]
    fn local_blank_text_counts_as_an_empty_object() {
        let empty = local_installed("{}", "tony - claude").unwrap();
        assert_eq!(local_installed("", "tony - claude").unwrap(), empty);
        assert_eq!(local_installed(" \n\t", "tony - claude").unwrap(), empty);
    }

    #[test]
    fn local_install_keeps_other_keys_and_environment_variables() {
        let value = parsed(&local_installed(LOCAL_OTHER, "tony - claude").unwrap());
        assert_eq!(
            value,
            json!({
                "model": "invented-model",
                "env": {"INVENTED_VARIABLE": "1", "DOSSIER_AGENT": "tony - claude"}
            })
        );
    }

    #[test]
    fn local_install_replaces_an_older_agent() {
        let text = r#"{"env": {"DOSSIER_AGENT": "old - claude", "INVENTED_VARIABLE": "1"}}"#;
        let value = parsed(&local_installed(text, "tony - claude").unwrap());
        assert_eq!(
            value,
            json!({"env": {"DOSSIER_AGENT": "tony - claude", "INVENTED_VARIABLE": "1"}})
        );
    }

    #[test]
    fn local_install_twice_gives_the_same_text() {
        let once = local_installed(LOCAL_OTHER, "tony - claude").unwrap();
        assert_eq!(local_installed(&once, "tony - claude").unwrap(), once);
        let empty_once = local_installed("{}", "tony - claude").unwrap();
        assert_eq!(
            local_installed(&empty_once, "tony - claude").unwrap(),
            empty_once
        );
    }

    #[test]
    fn local_uninstall_after_install_gives_back_the_other_keys() {
        let installed_text = local_installed(LOCAL_OTHER, "tony - claude").unwrap();
        let text = local_uninstalled(&installed_text).unwrap();
        assert_eq!(parsed(&text), parsed(LOCAL_OTHER));
    }

    #[test]
    fn local_uninstall_after_install_into_an_empty_object_gives_an_empty_object() {
        let installed_text = local_installed("{}", "tony - claude").unwrap();
        assert_eq!(local_uninstalled(&installed_text).unwrap(), "{}\n");
    }

    #[test]
    fn local_uninstall_removes_only_the_agent() {
        let text = r#"{"env": {"DOSSIER_AGENT": "tony - claude", "INVENTED_VARIABLE": "1"}, "model": "invented-model"}"#;
        assert_eq!(
            parsed(&local_uninstalled(text).unwrap()),
            json!({"env": {"INVENTED_VARIABLE": "1"}, "model": "invented-model"})
        );
    }

    #[test]
    fn local_uninstall_keeps_an_env_that_was_empty_before() {
        let text = r#"{"env": {}}"#;
        assert_eq!(parsed(&local_uninstalled(text).unwrap()), parsed(text));
        // Installing made the env non-empty, so uninstalling empties it again and it goes, as
        // `uninstalled` does with an empty "hooks" object.
        let round = local_uninstalled(&local_installed(text, "tony - claude").unwrap()).unwrap();
        assert_eq!(parsed(&round), json!({}));
    }

    #[test]
    fn local_refuses_text_that_is_not_a_settings_object() {
        let texts = [
            "[]",
            "42",
            "not json",
            r#"{"env": []}"#,
            r#"{"env": "text"}"#,
        ];
        for text in texts {
            let installed_error = local_installed(text, "tony - claude").unwrap_err();
            assert_eq!(installed_error.code, Code::Parse, "{text}");
            assert_eq!(
                local_uninstalled(text).unwrap_err().code,
                Code::Parse,
                "{text}"
            );
        }
    }

    #[test]
    fn local_byte_order_mark_is_skipped_and_not_written_back() {
        let mark = "\u{feff}";
        let plain = local_installed(LOCAL_OTHER, "tony - claude").unwrap();
        assert_eq!(
            local_installed(&format!("{mark}{LOCAL_OTHER}"), "tony - claude").unwrap(),
            plain
        );
        assert_eq!(
            local_uninstalled(&format!("{mark}{plain}")).unwrap(),
            local_uninstalled(&plain).unwrap()
        );
        assert_eq!(local_uninstalled(&format!("{mark}  \n")).unwrap(), "{}\n");
        assert!(!plain.starts_with(mark));
    }

    /// A settings text with other keys and a SessionEnd hook of the team's own in it.
    const SESSION_END_OTHER: &str = r#"{
  "model": "invented-model",
  "hooks": {
    "SessionEnd": [
      {"hooks": [{"type": "command", "command": "echo end"}]}
    ]
  }
}
"#;

    #[test]
    fn review_installs_one_session_end_hook_into_an_empty_object() {
        let text = review_installed("{}", "echo done").unwrap();
        assert_eq!(
            parsed(&text),
            json!({"hooks": {"SessionEnd": [
                {"hooks": [{
                    "type": "command",
                    "command": "dossier session-end --run 'echo done'",
                    "shell": "bash"
                }]}
            ]}})
        );
        assert!(text.ends_with("}\n") && !text.ends_with("\n\n"), "{text}");
    }

    #[test]
    fn review_treats_blank_text_as_an_empty_object() {
        let empty = review_installed("{}", "echo done").unwrap();
        assert_eq!(review_installed("", "echo done").unwrap(), empty);
        assert_eq!(review_installed(" \n\t", "echo done").unwrap(), empty);
        let marked = format!("\u{feff}{SESSION_END_OTHER}");
        assert_eq!(
            review_installed(&marked, "echo done").unwrap(),
            review_installed(SESSION_END_OTHER, "echo done").unwrap()
        );
    }

    #[test]
    fn review_quotes_a_single_quote_in_the_command() {
        let value = parsed(&review_installed("{}", "echo it's done").unwrap());
        assert_eq!(
            value["hooks"]["SessionEnd"][0]["hooks"][0]["command"],
            r"dossier session-end --run 'echo it'\''s done'"
        );
    }

    #[test]
    fn review_keeps_other_keys_and_hooks() {
        let original = parsed(SESSION_END_OTHER);
        let value = parsed(&review_installed(SESSION_END_OTHER, "echo done").unwrap());
        assert_eq!(value["model"], "invented-model");
        assert_eq!(
            value["hooks"]["SessionEnd"][0],
            original["hooks"]["SessionEnd"][0]
        );
        let text = review_installed(OTHER, "echo done").unwrap();
        assert_eq!(
            parsed(&text)["hooks"]["Stop"],
            parsed(OTHER)["hooks"]["Stop"]
        );
        assert_eq!(
            parsed(&text)["hooks"]["SessionStart"],
            parsed(OTHER)["hooks"]["SessionStart"]
        );
    }

    #[test]
    fn review_twice_with_the_same_command_gives_the_same_text_with_one_entry() {
        let empty_once = review_installed("{}", "echo done").unwrap();
        assert_eq!(
            review_installed(&empty_once, "echo done").unwrap(),
            empty_once
        );
        let once = review_installed(SESSION_END_OTHER, "echo done").unwrap();
        assert_eq!(review_installed(&once, "echo done").unwrap(), once);
        let value = parsed(&once);
        assert_eq!(
            value["hooks"]["SessionEnd"].as_array().unwrap().len(),
            2,
            "{value}"
        );
    }

    #[test]
    fn review_with_another_command_replaces_the_old_one_in_place() {
        let first = review_installed(SESSION_END_OTHER, "echo old").unwrap();
        let second = review_installed(&first, "echo new").unwrap();
        let value = parsed(&second);
        let session_end = value["hooks"]["SessionEnd"].as_array().unwrap();
        assert_eq!(session_end.len(), 2, "{value}");
        assert_eq!(
            session_end[0],
            json!({"hooks": [{"type": "command", "command": "echo end"}]})
        );
        let review = json!({"hooks": [{
            "type": "command",
            "command": "dossier session-end --run 'echo new'",
            "shell": "bash"
        }]});
        assert_eq!(session_end[1], review);
        assert_eq!(
            second,
            review_installed(SESSION_END_OTHER, "echo new").unwrap()
        );
    }

    #[test]
    fn review_replaces_the_hook_inside_its_entry_and_keeps_its_neighbours() {
        let text = r#"{"hooks": {"SessionEnd": [
  {"hooks": [
    {"type": "command", "command": "echo end"},
    {"type": "command", "command": "dossier session-end --run 'old'"}
  ]}
]}}"#;
        assert_eq!(
            parsed(&review_installed(text, "new").unwrap()),
            json!({"hooks": {"SessionEnd": [{"hooks": [
                {"type": "command", "command": "echo end"},
                {"type": "command", "command": "dossier session-end --run 'new'", "shell": "bash"}
            ]}]}})
        );
    }

    #[test]
    fn review_keeps_one_session_end_hook_when_there_are_several() {
        let text = r#"{"hooks": {"SessionEnd": [
  {"hooks": [{"type": "command", "command": "dossier session-end --run 'old'"}]},
  {"hooks": [{"type": "command", "command": "echo end"}]},
  {"hooks": [{"type": "command", "command": "dossier session-end --run 'older'"}]}
]}}"#;
        assert_eq!(
            parsed(&review_installed(text, "new").unwrap()),
            json!({"hooks": {"SessionEnd": [
                {"hooks": [{
                    "type": "command",
                    "command": "dossier session-end --run 'new'",
                    "shell": "bash"
                }]},
                {"hooks": [{"type": "command", "command": "echo end"}]}
            ]}})
        );
    }

    #[test]
    fn review_refuses_text_that_is_not_a_settings_object() {
        let texts = [
            "[]",
            "not json",
            r#"{"hooks": []}"#,
            r#"{"hooks": {"SessionEnd": {}}}"#,
            r#"{"hooks": {"SessionEnd": "text"}}"#,
        ];
        for text in texts {
            assert_eq!(
                review_installed(text, "echo done").unwrap_err().code,
                Code::Parse,
                "{text}"
            );
        }
    }

    #[test]
    fn uninstalling_removes_only_the_session_review_hook() {
        let text = r#"{"hooks": {"SessionEnd": [
  {"hooks": [{"type": "command", "command": "echo end"}]},
  {"hooks": [{"type": "command", "command": "dossier session-end --run 'echo done'"}]}
]}}"#;
        assert_eq!(
            parsed(&uninstalled(text).unwrap()),
            json!({"hooks": {"SessionEnd": [
                {"hooks": [{"type": "command", "command": "echo end"}]}
            ]}})
        );
    }

    #[test]
    fn uninstalling_after_the_review_gives_back_the_other_entries() {
        let both = review_installed(&installed(OTHER).unwrap(), "echo done").unwrap();
        assert_eq!(parsed(&uninstalled(&both).unwrap()), parsed(OTHER));
        let review_only = review_installed(SESSION_END_OTHER, "echo done").unwrap();
        assert_eq!(
            parsed(&uninstalled(&review_only).unwrap()),
            parsed(SESSION_END_OTHER)
        );
    }

    #[test]
    fn uninstalling_after_the_review_into_an_empty_object_gives_an_empty_object() {
        let text = review_installed("{}", "echo done").unwrap();
        assert_eq!(uninstalled(&text).unwrap(), "{}\n");
    }

    /// The SessionEnd hook objects of a settings text that have a `shell` field.
    fn hooks_with_shell(text: &str) -> Vec<Value> {
        let value = parsed(text);
        value["hooks"]["SessionEnd"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|entry| entry["hooks"].as_array().unwrap().clone())
            .filter(|hook| hook.get("shell").is_some())
            .collect()
    }

    #[test]
    fn the_installed_session_end_hook_says_bash() {
        let text = review_installed("{}", "echo done").unwrap();
        let hooks = hooks_with_shell(&text);
        assert_eq!(hooks.len(), 1, "{text}");
        assert_eq!(hooks[0]["type"], "command");
        assert_eq!(hooks[0]["shell"], "bash");
    }

    #[test]
    fn replacing_the_review_keeps_exactly_one_hook_with_the_shell_field() {
        // An earlier version wrote the review hook without the field; replacing it adds the field.
        let earlier = r#"{"hooks": {"SessionEnd": [
  {"hooks": [{"type": "command", "command": "echo end"}]},
  {"hooks": [{"type": "command", "command": "dossier session-end --run 'old'"}]}
]}}"#;
        let once = review_installed(earlier, "new").unwrap();
        let hooks = hooks_with_shell(&once);
        assert_eq!(hooks.len(), 1, "{once}");
        assert_eq!(hooks[0]["command"], "dossier session-end --run 'new'");
        let again = review_installed(&once, "newer").unwrap();
        let hooks = hooks_with_shell(&again);
        assert_eq!(hooks.len(), 1, "{again}");
        assert_eq!(hooks[0]["command"], "dossier session-end --run 'newer'");
        assert_eq!(
            parsed(&again)["hooks"]["SessionEnd"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn the_session_start_hook_has_no_shell_field() {
        let text = installed("{}").unwrap();
        let value = parsed(&text);
        let session_start = &value["hooks"]["SessionStart"][0]["hooks"][0];
        assert_eq!(session_start["command"], BRIEF);
        assert!(session_start.get("shell").is_none(), "{text}");
    }

    #[test]
    fn review_refuses_a_command_that_is_empty_or_only_spaces() {
        for command in ["", "   ", " \t "] {
            let error = review_installed(SESSION_END_OTHER, command).unwrap_err();
            assert_eq!(error.code, Code::BadValue, "{command:?}");
            assert_eq!(
                error.message, "the review command must not be empty",
                "{command:?}"
            );
        }
    }
}
