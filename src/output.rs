//! Command results: the text or data a command returns, and how they are written out.

use crate::error::{Code, Result};
use serde_json::json;
use std::io::Write;

/// What a successful command returns: text for people, data for `--json`, and warnings.
pub struct Output {
    pub text: String,
    pub data: serde_json::Value,
    pub warnings: Vec<String>,
}

impl Output {
    /// Text only; the data is `null`.
    pub fn text(text: impl Into<String>) -> Output {
        Output {
            text: text.into(),
            data: serde_json::Value::Null,
            warnings: Vec::new(),
        }
    }

    /// Text and data.
    pub fn with(text: impl Into<String>, data: serde_json::Value) -> Output {
        Output {
            text: text.into(),
            data,
            warnings: Vec::new(),
        }
    }

    /// Adds a warning.
    pub fn warn(mut self, warning: impl Into<String>) -> Output {
        self.warnings.push(warning.into());
        self
    }
}

/// Writes the result and returns the exit code. Never panics on a closed pipe.
///
/// Without `json`: the text to standard output, with a final line break unless the text is
/// empty; each warning as `warning: <text>` and an error as `error: <message>` to standard
/// error, except a `Code::Blocked` error, whose message is printed alone. With `json`: one line
/// on standard output, and nothing on standard error.
pub fn emit(command: &str, json: bool, result: &Result<Output>) -> u8 {
    let (stdout, stderr, code) = render(command, json, result);
    write_part(&mut std::io::stdout().lock(), &stdout);
    write_part(&mut std::io::stderr().lock(), &stderr);
    code
}

/// The text for standard output, the text for standard error and the exit code of a result.
fn render(command: &str, json: bool, result: &Result<Output>) -> (String, String, u8) {
    let schema = format!("dossier.{command}.v1");
    match result {
        Ok(output) if json => (
            json_line(&json!({
                "schema": schema,
                "ok": true,
                "data": output.data,
                "warnings": output.warnings,
            })),
            String::new(),
            0,
        ),
        Ok(output) => (
            with_line_break(&output.text),
            output
                .warnings
                .iter()
                .map(|warning| format!("warning: {warning}\n"))
                .collect(),
            0,
        ),
        Err(error) if json => (
            json_line(&json!({
                "schema": schema,
                "ok": false,
                "error": {"code": error.name(), "message": error.message},
                "warnings": [],
            })),
            String::new(),
            error.exit(),
        ),
        Err(error) if error.code == Code::Blocked => {
            (String::new(), with_line_break(&error.message), error.exit())
        }
        Err(error) => (
            String::new(),
            format!("error: {}\n", error.message),
            error.exit(),
        ),
    }
}

/// The text with a line break at its end, unless it is empty or already ends with one.
fn with_line_break(text: &str) -> String {
    if text.is_empty() || text.ends_with('\n') {
        text.to_string()
    } else {
        format!("{text}\n")
    }
}

/// The value as one line of JSON. A `Value` always serializes; the empty fallback only keeps
/// this free of `unwrap`.
fn json_line(value: &serde_json::Value) -> String {
    format!("{}\n", serde_json::to_string(value).unwrap_or_default())
}

/// Writes `text` and flushes it. A failed write, such as a closed pipe, is ignored: the
/// program ends quietly and the exit code still says what happened.
fn write_part(stream: &mut impl Write, text: &str) {
    if text.is_empty() {
        return;
    }
    let _ = stream.write_all(text.as_bytes());
    let _ = stream.flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{Code, Error};

    /// A stream whose every write fails, as a closed pipe does.
    struct ClosedPipe;

    impl Write for ClosedPipe {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
        }
    }

    /// The one JSON line of standard output, parsed.
    fn parse_line(stdout: &str) -> serde_json::Value {
        assert!(stdout.ends_with('\n'), "no line break: {stdout:?}");
        assert_eq!(stdout.matches('\n').count(), 1, "not one line: {stdout:?}");
        serde_json::from_str(stdout).expect("a JSON object")
    }

    /// The keys of a JSON object, sorted.
    fn keys(value: &serde_json::Value) -> Vec<&str> {
        let mut keys: Vec<&str> = value
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        keys
    }

    #[test]
    fn success_text_gets_a_final_line_break() {
        let (out, err, code) = render("where", false, &Ok(Output::text("/home/anna/project")));
        assert_eq!(out, "/home/anna/project\n");
        assert_eq!(err, "");
        assert_eq!(code, 0);
    }

    #[test]
    fn text_that_already_ends_in_a_line_break_gets_no_second_one() {
        let (out, _, _) = render("where", false, &Ok(Output::text("one\ntwo\n")));
        assert_eq!(out, "one\ntwo\n");
    }

    #[test]
    fn empty_text_gives_no_output_at_all() {
        let (out, err, code) = render("review-task", false, &Ok(Output::text("")));
        assert_eq!(out, "");
        assert_eq!(err, "");
        assert_eq!(code, 0);
    }

    #[test]
    fn warnings_go_to_standard_error_as_warning_lines() {
        let output = Output::text("done").warn("first").warn("second");
        let (out, err, code) = render("set", false, &Ok(output));
        assert_eq!(out, "done\n");
        assert_eq!(err, "warning: first\nwarning: second\n");
        assert_eq!(code, 0);
    }

    #[test]
    fn error_text_goes_to_standard_error_with_its_exit_code() {
        let error = Error::new(Code::NotFound, "no entry DEC-012");
        let (out, err, code) = render("show", false, &Err(error));
        assert_eq!(out, "");
        assert_eq!(err, "error: no entry DEC-012\n");
        assert_eq!(code, 1);
    }

    #[test]
    fn blocked_error_is_a_note_without_the_error_prefix() {
        let error = Error::new(Code::Blocked, "note text");
        let (out, err, code) = render("guard", false, &Err(error));
        assert_eq!(out, "");
        assert_eq!(err, "note text\n");
        assert_eq!(code, 2);
    }

    #[test]
    fn usage_error_keeps_the_error_prefix() {
        let error = Error::new(Code::Usage, "unknown option");
        let (out, err, code) = render("list", false, &Err(error));
        assert_eq!(out, "");
        assert_eq!(err, "error: unknown option\n");
        assert_eq!(code, 2);
    }

    #[test]
    fn json_blocked_error_has_the_code_blocked() {
        let error = Error::new(Code::Blocked, "note text");
        let (out, err, code) = render("guard", true, &Err(error));
        let value = parse_line(&out);
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "BLOCKED");
        assert_eq!(value["error"]["message"], "note text");
        assert_eq!(err, "");
        assert_eq!(code, 2);
    }

    #[test]
    fn json_success_has_exactly_the_keys_of_the_design() {
        let output = Output::with("2 entries", serde_json::json!({"count": 2})).warn("skipped one");
        let (out, err, code) = render("list", true, &Ok(output));
        let value = parse_line(&out);
        assert_eq!(keys(&value), ["data", "ok", "schema", "warnings"]);
        assert_eq!(value["schema"], "dossier.list.v1");
        assert_eq!(value["ok"], true);
        assert_eq!(value["data"], serde_json::json!({"count": 2}));
        assert_eq!(value["warnings"], serde_json::json!(["skipped one"]));
        assert!(
            !out.contains("2 entries"),
            "the text is not printed with --json"
        );
        assert_eq!(err, "", "warnings go only into the JSON");
        assert_eq!(code, 0);
    }

    #[test]
    fn json_success_without_data_has_null_data() {
        let (out, _, _) = render("where", true, &Ok(Output::text("x")));
        let value = parse_line(&out);
        assert!(value["data"].is_null());
    }

    #[test]
    fn json_failure_has_exactly_the_keys_of_the_design_on_one_line() {
        let error = Error::new(Code::NoDossier, "no dossier here\nrun dossier init");
        let (out, err, code) = render("show", true, &Err(error));
        let value = parse_line(&out);
        assert_eq!(keys(&value), ["error", "ok", "schema", "warnings"]);
        assert_eq!(keys(&value["error"]), ["code", "message"]);
        assert_eq!(value["schema"], "dossier.show.v1");
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "NO_DOSSIER");
        assert_eq!(
            value["error"]["message"],
            "no dossier here\nrun dossier init"
        );
        assert_eq!(value["warnings"], serde_json::json!([]));
        assert_eq!(err, "");
        assert_eq!(code, 4);
    }

    #[test]
    fn text_is_written_whole_and_empty_text_writes_nothing() {
        let mut bytes: Vec<u8> = Vec::new();
        write_part(&mut bytes, "line\n");
        write_part(&mut bytes, "");
        assert_eq!(bytes, b"line\n");
    }

    #[test]
    fn a_closed_pipe_does_not_panic() {
        write_part(&mut ClosedPipe, "error: lost\n");
    }
}
