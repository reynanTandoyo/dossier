//! The bridge to the `todo` program: runs it and reads its JSON output.
//!
//! The program is `todo`, or the value of `DOSSIER_TODO` (tests point it at a stand-in).

use crate::error::{Code, Error, Result};
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// What the person sees when todo cannot be started.
const MISSING: &str = "todo is not installed or not on PATH; dossier needs todo 0.8 or newer";

/// How many characters of todo's output an error message quotes.
const EXCERPT_CHARS: usize = 200;

/// The program to run: the value of `DOSSIER_TODO` when it is set and not empty, else `todo`.
fn program() -> String {
    match std::env::var_os("DOSSIER_TODO") {
        Some(value) if !value.is_empty() => value.to_string_lossy().into_owned(),
        _ => "todo".to_string(),
    }
}

/// Whether the program runs: `todo --version` starts and exits with 0.
pub fn available() -> bool {
    Command::new(program())
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Runs `todo <args> --json` in `root` and returns the `data` of the envelope.
/// Program missing: Code::TodoMissing. `ok:false`: Code::Refused with todo's message.
pub fn json(root: &Path, args: &[&str]) -> Result<serde_json::Value> {
    let output = Command::new(program())
        .args(args)
        .arg("--json")
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .map_err(|_| Error::new(Code::TodoMissing, MISSING))?;
    envelope(&output)
}

/// The `data` of todo's envelope on standard output, or the error it reports. Output that is
/// not an envelope is Code::TodoMissing: the program is not the todo that dossier needs.
fn envelope(output: &Output) -> Result<serde_json::Value> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = match serde_json::from_str(stdout.trim()) {
        Ok(value) => value,
        Err(_) => return Err(not_json(output)),
    };
    match value.get("ok").and_then(serde_json::Value::as_bool) {
        Some(true) => Ok(value
            .get("data")
            .cloned()
            .unwrap_or(serde_json::Value::Null)),
        Some(false) => {
            let message = value
                .pointer("/error/message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("todo refused the command without a message");
            Err(Error::new(Code::Refused, message))
        }
        None => Err(not_json(output)),
    }
}

/// Code::TodoMissing for output that is not todo's envelope. The message quotes the start of
/// standard output, or of standard error when standard output is empty.
fn not_json(output: &Output) -> Error {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let text = if stdout.trim().is_empty() {
        String::from_utf8_lossy(&output.stderr).into_owned()
    } else {
        stdout.into_owned()
    };
    let mut shown = excerpt(&text);
    if shown.is_empty() {
        shown = "(no output)".to_string();
    }
    Error::new(
        Code::TodoMissing,
        format!("todo did not print a JSON envelope: {shown}"),
    )
}

/// The first 200 characters of `text` (not bytes), without surrounding white space.
fn excerpt(text: &str) -> String {
    text.trim().chars().take(EXCERPT_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const OK_LINE: &str =
        r#"{"schema":"todo.show.v1","ok":true,"data":{"id":"T-001"},"warnings":[]}"#;
    const REFUSED_LINE: &str = r#"{"schema":"todo.show.v1","ok":false,"error":{"code":"NOT_FOUND","message":"no task T-999 in TODO.md"},"warnings":[]}"#;

    /// Writes a stand-in for todo in `folder` that prints `line` and exits with `code`.
    #[cfg(windows)]
    fn stand_in(folder: &Path, name: &str, line: &str, code: u8) -> PathBuf {
        let path = folder.join(format!("{name}.cmd"));
        let text = format!("@echo off\r\necho {line}\r\nexit /b {code}\r\n");
        std::fs::write(&path, text).unwrap();
        path
    }

    /// Writes a stand-in for todo in `folder` that prints `line` and exits with `code`.
    #[cfg(not(windows))]
    fn stand_in(folder: &Path, name: &str, line: &str, code: u8) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = folder.join(name);
        let text = format!("#!/bin/sh\nprintf '%s\\n' '{line}'\nexit {code}\n");
        std::fs::write(&path, text).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[test]
    fn excerpt_takes_200_characters_not_bytes() {
        let text = format!("  {}  ", "é".repeat(300));
        assert_eq!(excerpt(&text).chars().count(), 200);
        assert_eq!(excerpt("  short \n"), "short");
        assert_eq!(excerpt(" \n"), "");
    }

    #[test]
    fn json_returns_the_envelope_data_and_reports_refusal_and_a_missing_todo() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // A folder name with a space and a non-ASCII letter, as project folders may have.
        let dir = tempfile::Builder::new()
            .prefix("todo bridge é ")
            .tempdir()
            .unwrap();
        let root = dir.path();
        let ok = stand_in(root, "todo-ok", OK_LINE, 0);
        let refused = stand_in(root, "todo-refused", REFUSED_LINE, 1);
        let text = stand_in(
            root,
            "todo-text",
            &format!("not json {}", "a".repeat(250)),
            0,
        );
        let missing = root.join("no-such-todo");

        // Runs the bridge with DOSSIER_TODO pointing at `program`.
        let run = |program: &Path| {
            // SAFETY: ENV_LOCK is held, so no other test in this process reads or changes the
            // environment while this one does.
            unsafe { std::env::set_var("DOSSIER_TODO", program) };
            (available(), json(root, &["show", "T-001"]))
        };
        let (ok_available, ok_json) = run(&ok);
        let (refused_available, refused_json) = run(&refused);
        let (_, text_json) = run(&text);
        let (missing_available, missing_json) = run(&missing);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::set_var("DOSSIER_TODO", "") };
        let empty_value = program();
        unsafe { std::env::remove_var("DOSSIER_TODO") };
        let unset_value = program();

        assert!(ok_available, "a stand-in that exits with 0 is available");
        assert_eq!(ok_json.unwrap(), serde_json::json!({"id": "T-001"}));

        assert!(
            !refused_available,
            "a program that exits with 1 is not available"
        );
        let error = refused_json.unwrap_err();
        assert_eq!(error.code, Code::Refused);
        assert_eq!(error.message, "no task T-999 in TODO.md");

        let error = text_json.unwrap_err();
        assert_eq!(error.code, Code::TodoMissing);
        assert_eq!(
            error.message,
            format!(
                "todo did not print a JSON envelope: not json {}",
                "a".repeat(191)
            )
        );

        assert!(!missing_available, "a missing program is not available");
        let error = missing_json.unwrap_err();
        assert_eq!(error.code, Code::TodoMissing);
        assert_eq!(error.message, MISSING);

        assert_eq!(empty_value, "todo");
        assert_eq!(unset_value, "todo");
    }
}
