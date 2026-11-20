//! Running the program on a server through ssh.
//!
//! The ssh program is `ssh`, or the value of `DOSSIER_SSH` (tests point it at a stand-in).

use crate::error::{Code, Error, Result};
use std::ffi::OsString;

/// One shell word for a POSIX shell: wrapped in single quotes, each ' as '\''.
pub fn quote(arg: &str) -> String {
    format!("'{}'", arg.replace('\'', "'\\''"))
}

/// The argument list for the ssh program: -o BatchMode=yes -o ConnectTimeout=<n>,
/// -t when `terminal`, the server, then one string:
/// `cd <quoted folder> && DOSSIER_AGENT=<quoted> <quoted program> <quoted args…>`.
pub fn argv(
    server: &str,
    folder: &str,
    agent: Option<&str>,
    program: &str,
    args: &[String],
    terminal: bool,
    timeout: u64,
) -> Vec<String> {
    let mut list = vec![
        "-o".to_string(),
        "BatchMode=yes".to_string(),
        "-o".to_string(),
        format!("ConnectTimeout={timeout}"),
    ];
    if terminal {
        list.push("-t".to_string());
    }
    list.push(server.to_string());

    let mut command = format!("cd {} && ", quote(folder));
    if let Some(agent) = agent {
        command.push_str(&format!("DOSSIER_AGENT={} ", quote(agent)));
    }
    command.push_str(&quote(program));
    for argument in args {
        command.push(' ');
        command.push_str(&quote(argument));
    }
    list.push(command);
    list
}

/// Runs the ssh program with inherited input and output and returns the remote exit code.
/// ssh itself failing (exit 255, or the program missing): Code::Ssh.
pub fn run(argv: &[String]) -> Result<u8> {
    let program = std::env::var_os("DOSSIER_SSH")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| OsString::from("ssh"));
    let status = std::process::Command::new(program)
        .args(argv)
        .status()
        .map_err(|_| Error::new(Code::Ssh, "ssh is not installed or not on PATH"))?;
    outcome(argv, status.code())
}

/// What the exit code of the ssh program means: 0 to 254 is the remote exit code, 255 is ssh
/// failing to reach or sign in to the server, no code is a program ended by a signal, and any
/// other code cannot be passed on.
fn outcome(argv: &[String], code: Option<i32>) -> Result<u8> {
    match code {
        Some(255) => Err(Error::new(Code::Ssh, unreachable_message(argv))),
        Some(code) => u8::try_from(code)
            .map_err(|_| Error::new(Code::Ssh, format!("ssh ended with exit code {code}"))),
        None => Err(Error::new(Code::Ssh, "ssh ended without an exit code")),
    }
}

/// The message for exit code 255, naming the server from the arguments.
fn unreachable_message(argv: &[String]) -> String {
    match server_of(argv) {
        Some(server) => {
            format!("could not reach the server; sign in once by hand with: ssh {server}")
        }
        None => "could not reach the server".to_string(),
    }
}

/// The server in an argument list for ssh: the first argument that does not start with '-' and
/// is not the value of a -o option.
fn server_of(argv: &[String]) -> Option<&str> {
    let mut args = argv.iter();
    while let Some(arg) = args.next() {
        if arg == "-o" {
            args.next();
        } else if !arg.starts_with('-') {
            return Some(arg.as_str());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(list: &[&str]) -> Vec<String> {
        list.iter().map(|word| word.to_string()).collect()
    }

    /// The argument list of a command that reaches anna@server.
    fn sample() -> Vec<String> {
        words(&[
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            "anna@server",
            "cd '/srv/p' && 'dossier'",
        ])
    }

    #[test]
    fn quote_wraps_a_plain_word_in_single_quotes() {
        assert_eq!(quote("dossier"), "'dossier'");
    }

    #[test]
    fn quote_keeps_a_space() {
        assert_eq!(quote("my project"), "'my project'");
    }

    #[test]
    fn quote_turns_each_single_quote_into_quote_backslash_quote_quote() {
        assert_eq!(quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn quote_keeps_a_dollar_sign_and_a_backtick() {
        assert_eq!(quote("$HOME `date`"), "'$HOME `date`'");
    }

    #[test]
    fn quote_of_an_empty_argument_is_two_single_quotes() {
        assert_eq!(quote(""), "''");
    }

    #[test]
    fn quote_keeps_non_ascii_and_double_quotes() {
        assert_eq!(quote("Grüße \"x\""), "'Grüße \"x\"'");
    }

    #[test]
    fn argv_without_agent_and_without_terminal() {
        let list = argv(
            "anna@server",
            "/srv/my project",
            None,
            "dossier",
            &words(&["add", "decision"]),
            false,
            10,
        );
        assert_eq!(
            list,
            words(&[
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=10",
                "anna@server",
                "cd '/srv/my project' && 'dossier' 'add' 'decision'",
            ])
        );
    }

    #[test]
    fn argv_with_agent_and_without_terminal() {
        let list = argv(
            "anna@server",
            "/srv/p",
            Some("anna-claude"),
            "dossier",
            &words(&["approve", "DEC-001"]),
            false,
            10,
        );
        assert_eq!(
            list,
            words(&[
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=10",
                "anna@server",
                "cd '/srv/p' && DOSSIER_AGENT='anna-claude' 'dossier' 'approve' 'DEC-001'",
            ])
        );
    }

    #[test]
    fn argv_without_agent_and_with_terminal() {
        let list = argv(
            "anna@server",
            "/srv/p",
            None,
            "dossier",
            &words(&["remove", "DEC-001"]),
            true,
            10,
        );
        assert_eq!(
            list,
            words(&[
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=10",
                "-t",
                "anna@server",
                "cd '/srv/p' && 'dossier' 'remove' 'DEC-001'",
            ])
        );
    }

    #[test]
    fn argv_with_agent_and_with_terminal() {
        let list = argv(
            "anna@server",
            "/srv/p",
            Some("anna-claude"),
            "dossier",
            &words(&["approve", "DEC-001"]),
            true,
            10,
        );
        assert_eq!(
            list,
            words(&[
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=10",
                "-t",
                "anna@server",
                "cd '/srv/p' && DOSSIER_AGENT='anna-claude' 'dossier' 'approve' 'DEC-001'",
            ])
        );
    }

    #[test]
    fn text_with_a_line_break_stays_one_argument() {
        let text = "first line\nsecond 'line'\n";
        let list = argv(
            "anna@server",
            "/srv/p",
            None,
            "dossier",
            &words(&["add", text]),
            false,
            10,
        );
        assert_eq!(list.len(), 6);
        assert_eq!(
            list[5],
            "cd '/srv/p' && 'dossier' 'add' 'first line\nsecond '\\''line'\\''\n'"
        );
    }

    #[test]
    fn exit_codes_from_0_to_254_pass_through() {
        assert_eq!(outcome(&sample(), Some(0)).unwrap(), 0);
        assert_eq!(outcome(&sample(), Some(1)).unwrap(), 1);
        assert_eq!(outcome(&sample(), Some(254)).unwrap(), 254);
    }

    #[test]
    fn exit_code_255_is_an_ssh_error_that_names_the_server() {
        let error = outcome(&sample(), Some(255)).unwrap_err();
        assert_eq!(error.code, Code::Ssh);
        assert_eq!(
            error.message,
            "could not reach the server; sign in once by hand with: ssh anna@server"
        );
    }

    #[test]
    fn a_program_ended_by_a_signal_is_an_ssh_error() {
        let error = outcome(&sample(), None).unwrap_err();
        assert_eq!(error.code, Code::Ssh);
        assert_eq!(error.message, "ssh ended without an exit code");
    }

    #[test]
    fn a_code_that_cannot_be_passed_on_is_an_ssh_error() {
        let error = outcome(&sample(), Some(256)).unwrap_err();
        assert_eq!(error.code, Code::Ssh);
        assert_eq!(error.message, "ssh ended with exit code 256");
    }

    #[test]
    fn server_is_the_first_argument_after_the_options() {
        let list = words(&["-o", "BatchMode=yes", "-t", "anna@server", "cd '/p'"]);
        assert_eq!(server_of(&list), Some("anna@server"));
    }

    #[test]
    fn a_missing_program_is_an_ssh_error() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let missing = std::env::temp_dir().join("dossier-no-such-ssh-program");
        let before = std::env::var_os("DOSSIER_SSH");
        // SAFETY: ENV_LOCK is held, so no other test in this process reads or changes the
        // environment while this one does.
        unsafe { std::env::set_var("DOSSIER_SSH", &missing) };
        let result = run(&sample());
        // SAFETY: ENV_LOCK is still held here.
        unsafe {
            match before {
                Some(value) => std::env::set_var("DOSSIER_SSH", value),
                None => std::env::remove_var("DOSSIER_SSH"),
            }
        }
        let error = result.unwrap_err();
        assert_eq!(error.code, Code::Ssh);
        assert_eq!(error.message, "ssh is not installed or not on PATH");
    }
}
