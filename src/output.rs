//! Command results: the text or data a command returns, and how they are written out.

use crate::error::Result;
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
/// Skeleton version: it ignores `command` and `json`; the output task replaces it.
pub fn emit(command: &str, json: bool, result: &Result<Output>) -> u8 {
    match result {
        Ok(output) => {
            let mut text = output.text.clone();
            if !text.ends_with('\n') {
                text.push('\n');
            }
            let mut stdout = std::io::stdout();
            let _ = stdout.write_all(text.as_bytes());
            let _ = stdout.flush();
            let mut stderr = std::io::stderr();
            for warning in &output.warnings {
                let _ = writeln!(stderr, "warning: {warning}");
            }
            0
        }
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "error: {}", error.message);
            error.exit()
        }
    }
}
