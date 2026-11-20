//! The `connect` command: ties the current folder to a project on a server, and records the
//! project in the per-user register (R-REMOTE-1, R-REMOTE-9). It needs no dossier and no name.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::remote::pointer::{self, Pointer};
use crate::remote::register::{self, Project};
use crate::remote::ssh;
use serde_json::{Value, json};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// How long ssh may take to connect, in seconds (R-REMOTE-11, `ssh_timeout_seconds`).
const SSH_TIMEOUT_SECONDS: u64 = 10;

/// The arguments of `connect`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The server and the project folder on it, for example anna@sim01:/projects/P-017
    pub target: Option<String>,
    /// The project code, such as P-017; without it the code is read from the server
    #[arg(long)]
    pub code: Option<String>,
    /// Records the project in the register only, with no pointer file in this folder
    #[arg(long)]
    pub no_folder: bool,
    /// Tests the connection of this connected folder (takes no target)
    #[arg(long)]
    pub check: bool,
}

/// Runs `connect`: with a target, connects this folder; with `--check`, tests its connection.
pub fn run(args: &Args, _g: &Global) -> Result<Output> {
    if args.check {
        if args.target.is_some() || args.code.is_some() || args.no_folder {
            return Err(Error::new(
                Code::Usage,
                "--check takes no target and no other option",
            ));
        }
        return check(&current_folder()?);
    }
    match &args.target {
        Some(target) => connect(args, target),
        None => Err(Error::new(
            Code::Usage,
            "give the target as user@server:folder, for example anna@sim01:/projects/P-017, or --check in a connected folder",
        )),
    }
}

/// Connects the current folder to `target`: writes the pointer file and the register entry.
fn connect(args: &Args, target: &str) -> Result<Output> {
    let (server, folder) = split_target(target)?;
    let code = match &args.code {
        Some(code) => code.clone(),
        None => read_code(&server, &folder)?,
    };
    let code = checked_code(code)?;
    let here = current_folder()?;
    let local = if args.no_folder {
        None
    } else {
        Some(here.to_string_lossy().into_owned())
    };
    if !args.no_folder {
        pointer::write(
            &here,
            &Pointer {
                server: server.clone(),
                folder: folder.clone(),
                code: code.clone(),
            },
        )?;
    }
    register::put(&Project {
        code: code.clone(),
        server: server.clone(),
        folder: folder.clone(),
        local: local.clone(),
    })?;
    let text = format!("connected {code} -> {server}:{folder}");
    let data = json!({"code": code, "server": server, "folder": folder, "local": local});
    Ok(Output::with(text, data))
}

/// Tests the connection of the folder `here`: its pointer file is found from it, and the server
/// answers for the project. A server answer for another code is a warning.
fn check(here: &Path) -> Result<Output> {
    let (_, saved) = pointer::find(here).ok_or_else(|| {
        Error::new(
            Code::NotFound,
            "this folder is not connected; run dossier connect <user@server>:<folder>",
        )
    })?;
    let code = read_code(&saved.server, &saved.folder)?;
    let text = format!("ok: {code} on {}", saved.server);
    let data = json!({"code": code, "server": saved.server, "folder": saved.folder});
    let output = Output::with(text, data);
    if code == saved.code {
        Ok(output)
    } else {
        Ok(output.warn(format!(
            "this folder is connected to {}, but the server answers for {code}",
            saved.code
        )))
    }
}

/// The server and the folder of `target`, which is `user@server:folder`, split at the first ':'.
/// The server is not empty, does not start with '-' (ssh would read it as an option) and has no
/// spaces; the folder starts with '/'. Anything else is Code::BadValue.
fn split_target(target: &str) -> Result<(String, String)> {
    let Some((server, folder)) = target.split_once(':') else {
        return Err(Error::new(
            Code::BadValue,
            "the target is user@server:folder, for example anna@sim01:/projects/P-017",
        ));
    };
    if server.is_empty() || server.starts_with('-') || server.chars().any(char::is_whitespace) {
        return Err(Error::new(
            Code::BadValue,
            "the server is user@server, for example anna@sim01: it must not be empty, start with - or contain spaces",
        ));
    }
    if !folder.starts_with('/') {
        return Err(Error::new(
            Code::BadValue,
            "the folder on the server must start with /, for example anna@sim01:/projects/P-017",
        ));
    }
    Ok((server.to_string(), folder.to_string()))
}

/// The project code as `init` accepts it, or Code::BadValue.
fn checked_code(code: String) -> Result<String> {
    if valid_code(&code) {
        Ok(code)
    } else {
        Err(Error::new(
            Code::BadValue,
            "a project code has only letters, digits and -, such as P-017",
        ))
    }
}

/// Whether `code` is a project code: not empty, and only ASCII letters, digits and `-`.
fn valid_code(code: &str) -> bool {
    !code.is_empty() && code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// The project code of the dossier in `folder` on `server`: `dossier where --json` run over
/// ssh, and its `data.code`. Every failure is the same Code::Ssh.
fn read_code(server: &str, folder: &str) -> Result<String> {
    let args = ["where".to_string(), "--json".to_string()];
    let argv = ssh::argv(
        server,
        folder,
        None,
        "dossier",
        &args,
        false,
        SSH_TIMEOUT_SECONDS,
    );
    let output = Command::new(ssh_program())
        .args(&argv)
        .stdin(Stdio::null())
        .output()
        .map_err(|_| unreadable())?;
    if !output.status.success() {
        return Err(unreadable());
    }
    code_in(&output.stdout).ok_or_else(unreadable)
}

/// The failure when the project code cannot be read from the server.
fn unreadable() -> Error {
    Error::new(
        Code::Ssh,
        "could not read the project code from the server; give it with --code",
    )
}

/// The ssh program: DOSSIER_SSH when it is set (tests point it at a stand-in), else `ssh`.
fn ssh_program() -> OsString {
    std::env::var_os("DOSSIER_SSH")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| OsString::from("ssh"))
}

/// The project code in the answer of `dossier where --json`. The answer is the last line of
/// standard output (a login banner may come before it); it must say `ok` and carry `data.code`.
fn code_in(stdout: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(stdout);
    let line = text.lines().rev().find(|line| !line.trim().is_empty())?;
    let answer: Value = serde_json::from_str(line).ok()?;
    if answer.get("ok") != Some(&Value::Bool(true)) {
        return None;
    }
    let code = answer.get("data")?.get("code")?.as_str()?;
    (!code.trim().is_empty()).then(|| code.to_string())
}

/// The folder the command runs in.
fn current_folder() -> Result<PathBuf> {
    std::env::current_dir()
        .map_err(|error| Error::new(Code::Io, format!("the current folder: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(server: &str, folder: &str) -> (String, String) {
        (server.into(), folder.into())
    }

    #[test]
    fn the_target_is_split_at_the_first_colon() {
        assert_eq!(
            split_target("anna@sim01:/projects/P-017").unwrap(),
            pair("anna@sim01", "/projects/P-017")
        );
    }

    #[test]
    fn a_folder_with_colons_spaces_and_non_ascii_letters_is_kept_whole() {
        assert_eq!(
            split_target("anna@sim01:/srv/Projekt Ü:2").unwrap(),
            pair("anna@sim01", "/srv/Projekt Ü:2")
        );
    }

    #[test]
    fn a_target_that_is_not_server_and_folder_is_bad_value() {
        for target in [
            "",
            "anna@sim01",
            "anna@sim01:",
            "anna@sim01:projects/P-017",
            ":/projects/P-017",
            "-oProxyCommand=x@sim01:/projects/P-017",
            "anna sim01:/projects/P-017",
            "anna@sim01\t:/projects/P-017",
        ] {
            let error = split_target(target).unwrap_err();
            assert_eq!(error.code, Code::BadValue, "{target:?}");
            assert_eq!(error.exit(), 1, "{target:?}");
        }
    }

    #[test]
    fn the_code_is_read_from_the_last_line_of_a_successful_answer() {
        let stdout = b"Welcome to sim01\n{\"schema\":\"dossier.where.v1\",\"ok\":true,\"data\":{\"code\":\"P-017\"},\"warnings\":[]}\n";
        assert_eq!(code_in(stdout).as_deref(), Some("P-017"));
    }

    #[test]
    fn an_answer_that_failed_or_has_no_code_gives_none() {
        let answers: [&[u8]; 6] = [
            b"",
            b"not json\n",
            b"{\"schema\":\"dossier.where.v1\",\"ok\":false,\"error\":{\"code\":\"NO_DOSSIER\",\"message\":\"x\"},\"warnings\":[]}\n",
            b"{\"ok\":true,\"data\":{\"code\":\"\"},\"warnings\":[]}\n",
            b"{\"ok\":true,\"data\":{},\"warnings\":[]}\n",
            b"{\"ok\":true,\"data\":{\"code\":7},\"warnings\":[]}\n",
        ];
        for stdout in answers {
            assert_eq!(code_in(stdout), None, "{}", String::from_utf8_lossy(stdout));
        }
    }

    #[test]
    fn a_code_is_accepted_as_init_accepts_it() {
        assert_eq!(checked_code("P-017".into()).unwrap(), "P-017");
        for code in ["", "P 017", "P_017", "P/017", "P-017\n"] {
            let error = checked_code(code.into()).unwrap_err();
            assert_eq!(error.code, Code::BadValue, "{code:?}");
        }
    }
}
