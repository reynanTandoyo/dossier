//! The `schema` command (docs/design.md section 20, R-OUT-6): a machine-readable description of
//! the commands, the shape of the `--json` envelope, the exit codes and the error codes. It needs
//! no dossier.

use crate::cli::Global;
use crate::cmd::help::COMMANDS;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use serde_json::{Value, json};

/// The arguments of `schema`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Every variant of `Code`, in the order of error.rs.
const CODES: [Code; 21] = [
    Code::Usage,
    Code::NotImplemented,
    Code::Refused,
    Code::NotFound,
    Code::Exists,
    Code::NeedsPerson,
    Code::NeedsTerminal,
    Code::UnknownName,
    Code::MissingField,
    Code::BadValue,
    Code::Busy,
    Code::Changed,
    Code::NoDossier,
    Code::NetworkDrive,
    Code::Parse,
    Code::Config,
    Code::Io,
    Code::TooNew,
    Code::TodoMissing,
    Code::Ssh,
    Code::Blocked,
];

/// Runs `schema`. The text is the description pretty-printed as JSON; with `--json` the same
/// value is the `data` of the envelope.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    let value = description();
    let text = serde_json::to_string_pretty(&value).unwrap_or_default();
    Ok(Output::with(text, value))
}

/// The whole description: the commands, the envelope, the exit codes and the error codes.
fn description() -> Value {
    let commands: Vec<Value> = COMMANDS
        .iter()
        .map(|(name, description)| {
            json!({
                "name": name,
                "description": description,
                "schema": format!("dossier.{name}.v1"),
            })
        })
        .collect();
    let error_codes: Vec<Value> = CODES
        .iter()
        .map(|&code| {
            let error = Error::new(code, "");
            json!({"name": error.name(), "exit": error.exit()})
        })
        .collect();
    json!({
        "commands": commands,
        "envelope": {
            "ok": "bool",
            "data": "object",
            "warnings": "array of strings",
            "error": {"code": "string", "message": "string"},
        },
        "exit_codes": {
            "0": "done",
            "1": "refused by a rule, or problems found",
            "2": "wrong usage",
            "3": "busy, try again",
            "4": "file or settings problem",
            "5": "a needed program or connection failed",
        },
        "error_codes": error_codes,
    })
}

/// The place of `code` in `CODES`. The match has no catch-all arm, so a variant added to `Code`
/// does not compile the tests until it has a place here and in `CODES`.
#[cfg(test)]
fn place(code: Code) -> usize {
    match code {
        Code::Usage => 0,
        Code::NotImplemented => 1,
        Code::Refused => 2,
        Code::NotFound => 3,
        Code::Exists => 4,
        Code::NeedsPerson => 5,
        Code::NeedsTerminal => 6,
        Code::UnknownName => 7,
        Code::MissingField => 8,
        Code::BadValue => 9,
        Code::Busy => 10,
        Code::Changed => 11,
        Code::NoDossier => 12,
        Code::NetworkDrive => 13,
        Code::Parse => 14,
        Code::Config => 15,
        Code::Io => 16,
        Code::TooNew => 17,
        Code::TodoMissing => 18,
        Code::Ssh => 19,
        Code::Blocked => 20,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_code_is_listed_once_in_the_order_of_the_match() {
        for (index, code) in CODES.iter().enumerate() {
            assert_eq!(place(*code), index, "{code:?}");
        }
    }
}
