//! Comparing the versions of the PC program and the server program (R-XOS-9, format.md §31).
//!
//! The PC sets two variables for every command it runs on the server; the server program reads
//! them here before it runs the command.

use crate::error::{Code, Error, Result};

/// The format version this program reads and writes.
pub const FORMAT: u32 = 1;
/// The variable that carries the PC program's version, such as `0.5.0`. The PC sets it per command.
pub const CLIENT_VERSION: &str = "DOSSIER_CLIENT_VERSION";
/// The variable that carries the PC program's format version. The PC sets it per command.
pub const CLIENT_FORMAT: &str = "DOSSIER_CLIENT_FORMAT";

/// The program version of this build.
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// On the server, from the two variables: Err(Code::TooNew) naming the side to update when the
/// formats differ; Ok(Some(warning)) when only the program versions differ; Ok(None) when equal
/// or when the variables are not set.
pub fn check(client_version: Option<&str>, client_format: Option<&str>) -> Result<Option<String>> {
    let server_format = FORMAT.to_string();
    if let Some(client_format) = client_format {
        if client_format != server_format {
            return Err(Error::new(
                Code::TooNew,
                format_message(client_format, &server_format),
            ));
        }
        if let Some(client_version) = client_version
            && client_version != SERVER_VERSION
        {
            return Ok(Some(format!(
                "dossier on the PC is {client_version}, on the server {SERVER_VERSION}"
            )));
        }
    }
    Ok(None)
}

/// The refusal for two different formats. The server is to update when the PC's format is a
/// higher number; otherwise the PC is (a lower number, or a value that is no number).
fn format_message(client_format: &str, server_format: &str) -> String {
    let server_is_older = client_format
        .parse::<u32>()
        .is_ok_and(|number| number > FORMAT);
    let side = if server_is_older { "server" } else { "PC" };
    format!(
        "the PC uses dossier format {client_format}, the server format {server_format}: update dossier on the {side}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The error that `check` returns for these variables (the test fails when it returns Ok).
    fn refusal(client_version: Option<&str>, client_format: Option<&str>) -> Error {
        check(client_version, client_format).unwrap_err()
    }

    #[test]
    fn nothing_set_is_ok_and_silent() {
        assert_eq!(check(None, None).unwrap(), None);
    }

    #[test]
    fn a_version_without_a_format_is_not_compared() {
        assert_eq!(check(Some("0.0.0-other"), None).unwrap(), None);
    }

    #[test]
    fn a_lower_pc_format_is_refused_and_the_pc_is_to_update() {
        let error = refusal(Some("0.4.0"), Some("0"));
        assert_eq!(error.code, Code::TooNew);
        assert_eq!(
            error.message,
            "the PC uses dossier format 0, the server format 1: update dossier on the PC"
        );
    }

    #[test]
    fn a_higher_pc_format_is_refused_and_the_server_is_to_update() {
        let higher = (FORMAT + 1).to_string();
        let error = refusal(Some("0.6.0"), Some(&higher));
        assert_eq!(error.code, Code::TooNew);
        assert_eq!(
            error.message,
            "the PC uses dossier format 2, the server format 1: update dossier on the server"
        );
    }

    #[test]
    fn a_pc_format_that_is_no_number_is_refused_and_the_pc_is_to_update() {
        let error = refusal(None, Some("one"));
        assert_eq!(error.code, Code::TooNew);
        assert_eq!(
            error.message,
            "the PC uses dossier format one, the server format 1: update dossier on the PC"
        );
    }

    #[test]
    fn a_pc_format_with_the_same_number_but_other_text_is_refused_and_the_pc_is_to_update() {
        let error = refusal(None, Some("01"));
        assert_eq!(error.code, Code::TooNew);
        assert_eq!(
            error.message,
            "the PC uses dossier format 01, the server format 1: update dossier on the PC"
        );
    }

    #[test]
    fn equal_formats_with_other_program_versions_give_a_warning() {
        let warning = check(Some("0.0.0-other"), Some("1")).unwrap();
        assert_eq!(
            warning,
            Some(format!(
                "dossier on the PC is 0.0.0-other, on the server {}",
                env!("CARGO_PKG_VERSION")
            ))
        );
    }

    #[test]
    fn equal_formats_and_equal_program_versions_are_ok_and_silent() {
        assert_eq!(
            check(Some(env!("CARGO_PKG_VERSION")), Some("1")).unwrap(),
            None
        );
    }

    #[test]
    fn equal_formats_without_a_version_are_ok_and_silent() {
        assert_eq!(check(None, Some("1")).unwrap(), None);
    }
}
