//! Copies of large files between a connected PC folder and the server (R-REMOTE-4): the scp
//! arguments, and `take` and `return` in a connected folder.

use crate::error::{Code, Error, Result};
use crate::remote::pointer::Pointer;
use crate::remote::ssh;
use std::ffi::OsString;
use std::io::Write;
use std::path::{Component, Path};

/// The time limit of each connection in seconds (R-REMOTE-11).
const TIMEOUT_SECONDS: u64 = 10;

/// The scp program: DOSSIER_SCP when it is set and not empty, else `scp`.
pub fn program() -> OsString {
    std::env::var_os("DOSSIER_SCP")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| OsString::from("scp"))
}

/// The arguments of scp: -s -o BatchMode=yes -o ConnectTimeout=<n> -p, then source and target.
/// -s forces the SFTP protocol (OpenSSH 8.7 or newer; the default from 9.0 on), so that the remote
/// path is read the same way by every scp.
pub fn argv(source: &str, target: &str, timeout: u64) -> Vec<String> {
    vec![
        "-s".to_string(),
        "-o".to_string(),
        "BatchMode=yes".to_string(),
        "-o".to_string(),
        format!("ConnectTimeout={timeout}"),
        "-p".to_string(),
        source.to_string(),
        target.to_string(),
    ]
}

/// "<server>:<folder/stored>" for scp, not shell-quoted: in SFTP mode the path reaches the server
/// as written, and quotes would become part of the name. Each `\`, `*`, `?`, `[` and `]` gets a
/// backslash in front, so that the SFTP glob matches only this file (tested against OpenSSH 10.2,
/// 2026-11-23: a quoted path is not found, and an unescaped `a*b.sim` also copies `axxb.sim`).
pub fn remote_path(server: &str, folder: &str, stored: &str) -> String {
    let mut escaped = String::new();
    for ch in format!("{folder}/{stored}").chars() {
        if matches!(ch, '\\' | '*' | '?' | '[' | ']') {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    format!("{server}:{escaped}")
}

/// The file arguments of `return`: the words that do not start with `-` and are not the word
/// after --task or --agent. Used by `return --done` (T-373), not yet called.
#[allow(dead_code)]
fn files_of(args: &[String]) -> Vec<String> {
    let mut files = Vec::new();
    let mut value_next = false;
    for arg in args {
        if value_next {
            value_next = false;
        } else if arg == "--task" || arg == "--agent" {
            value_next = true;
        } else if !arg.starts_with('-') {
            files.push(arg.clone());
        }
    }
    files
}

/// Called by forward::maybe for `take` and `return` when the folder is connected; Some(exit code).
/// Only `take` is handled so far; every other command returns None.
pub fn maybe(
    command: &str,
    args: &[String],
    base: &Path,
    pointer: &Pointer,
    agent: Option<&str>,
) -> Option<u8> {
    if command == "take" {
        Some(take(args, base, pointer, agent))
    } else {
        None
    }
}

/// `take` in a connected folder: the server records the take first (`dossier take … --json`), then
/// each file it names is copied down to the same path below `base`. A failed copy ends with exit 5
/// and says that the take is recorded, so running the same command again copies again.
fn take(args: &[String], base: &Path, pointer: &Pointer, agent: Option<&str>) -> u8 {
    let ssh_argv = take_argv(args, pointer, agent);
    let (code, stdout, stderr) = match ssh::capture(&ssh_argv) {
        Ok(captured) => captured,
        Err(error) => return fail(&error),
    };
    if code != 0 {
        let message = failure_message(&stdout, &stderr, code);
        if code == 255 {
            // ssh's own failure: dossier's exit codes stop at 5, so it is reported as Code::Ssh.
            return fail(&Error::new(Code::Ssh, message));
        }
        report(&message);
        return code;
    }
    match copy_all(&stdout, base, pointer) {
        Ok(paths) => {
            for path in &paths {
                let _ = writeln!(std::io::stdout(), "{path}: taken and copied");
            }
            0
        }
        Err(error) => fail(&error),
    }
}

/// The ssh arguments that run `dossier take <args> --json` in the connected folder, with
/// DOSSIER_AGENT when an agent name is given.
fn take_argv(args: &[String], pointer: &Pointer, agent: Option<&str>) -> Vec<String> {
    let env: Vec<(String, String)> = agent
        .map(|name| ("DOSSIER_AGENT".to_string(), name.to_string()))
        .into_iter()
        .collect();
    let mut words = vec!["take".to_string()];
    words.extend(args.iter().cloned());
    words.push("--json".to_string());
    ssh::argv_with(
        &pointer.server,
        &pointer.folder,
        None,
        &env,
        "dossier",
        &words,
        false,
        TIMEOUT_SECONDS,
    )
}

/// The message for a command that failed on the server: `error.message` of its JSON answer on
/// standard output, else the last line of ssh's standard error, else the exit code.
fn failure_message(stdout: &str, stderr: &str, code: u8) -> String {
    let answered = serde_json::from_str::<serde_json::Value>(stdout.trim())
        .ok()
        .and_then(|answer| answer["error"]["message"].as_str().map(str::to_string));
    answered
        .or_else(|| {
            stderr
                .lines()
                .map(str::trim)
                .rfind(|line| !line.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| format!("the command ended with exit code {code}"))
}

/// Copies each file the server took, in the order of its answer. Every path is checked before the
/// first copy, so that a bad answer copies nothing.
fn copy_all(stdout: &str, base: &Path, pointer: &Pointer) -> Result<Vec<String>> {
    let paths = taken_paths(stdout)?;
    for path in &paths {
        copy_down(path, base, pointer)?;
    }
    Ok(paths)
}

/// The paths in the `--json` answer of take: `data.files`, each with its `path`. Each path must be
/// relative and made of plain names, so that it stays below the connected folder.
fn taken_paths(stdout: &str) -> Result<Vec<String>> {
    let answer: serde_json::Value = serde_json::from_str(stdout.trim()).map_err(|error| {
        Error::new(
            Code::Ssh,
            format!("could not read the answer of the server: {error}"),
        )
    })?;
    let files = answer["data"]["files"]
        .as_array()
        .ok_or_else(|| Error::new(Code::Ssh, "the answer of the server names no files"))?;
    let mut paths = Vec::new();
    for file in files {
        let path = file["path"].as_str().ok_or_else(|| {
            Error::new(
                Code::Ssh,
                "the answer of the server has a file without a path",
            )
        })?;
        if !is_below(path) {
            return Err(Error::new(
                Code::Ssh,
                format!("the server named a path outside the connected folder: {path}"),
            ));
        }
        paths.push(path.to_string());
    }
    Ok(paths)
}

/// Whether `path` is relative and made of plain names only, such as `sim/a.sim`: no root, no
/// prefix, no `..`.
fn is_below(path: &str) -> bool {
    let parts: Vec<Component> = Path::new(path).components().collect();
    !parts.is_empty()
        && parts
            .iter()
            .all(|part| matches!(part, Component::Normal(_)))
}

/// Copies one file the server took, from `<folder>/<path>` on the server to `<base>/<path>`,
/// creating the folders above it first.
fn copy_down(path: &str, base: &Path, pointer: &Pointer) -> Result<()> {
    let local = base.join(path);
    if let Some(parent) = local.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            Error::new(
                Code::Ssh,
                format!(
                    "could not copy {path} from the server: cannot create the folder {} ({error}); the take is recorded, run the same command again to copy it",
                    parent.display()
                ),
            )
        })?;
    }
    let source = remote_path(&pointer.server, &pointer.folder, path);
    let target = local.to_string_lossy().into_owned();
    let status = std::process::Command::new(program())
        .args(argv(&source, &target, TIMEOUT_SECONDS))
        .status();
    match status {
        Ok(status) if status.success() => Ok(()),
        _ => Err(Error::new(
            Code::Ssh,
            format!(
                "could not copy {path} from the server; the take is recorded, run the same command again to copy it"
            ),
        )),
    }
}

/// Prints the error and returns its exit code.
fn fail(error: &Error) -> u8 {
    report(&error.message);
    error.exit()
}

/// Writes `error: <message>` to standard error.
fn report(message: &str) {
    let _ = writeln!(std::io::stderr(), "error: {message}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(list: &[&str]) -> Vec<String> {
        list.iter().map(|word| word.to_string()).collect()
    }

    #[test]
    fn argv_forces_sftp_then_has_batch_mode_the_timeout_and_keeps_the_times() {
        assert_eq!(
            argv("anna@server:/srv/my project/a.sim", "a.sim", 10),
            words(&[
                "-s",
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=10",
                "-p",
                "anna@server:/srv/my project/a.sim",
                "a.sim",
            ])
        );
    }

    #[test]
    fn remote_path_is_not_quoted_for_a_folder_and_a_stored_name_with_a_space() {
        assert_eq!(
            remote_path("anna@server", "/srv/my project", "sim/run 1.sim"),
            "anna@server:/srv/my project/sim/run 1.sim"
        );
    }

    #[test]
    fn remote_path_escapes_glob_characters_and_keeps_quotes_and_non_ascii() {
        assert_eq!(
            remote_path("anna@server", "/srv/p", "it's Grüße [2]*?.sim"),
            "anna@server:/srv/p/it's Grüße \\[2\\]\\*\\?.sim"
        );
    }

    #[test]
    fn program_is_scp_unless_dossier_scp_names_another_program() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let before = std::env::var_os("DOSSIER_SCP");
        // SAFETY: ENV_LOCK is held, so no other test in this process reads or changes the
        // environment while this one does.
        unsafe { std::env::remove_var("DOSSIER_SCP") };
        let unset = program();
        // SAFETY: ENV_LOCK is still held here.
        unsafe { std::env::set_var("DOSSIER_SCP", "") };
        let empty = program();
        // SAFETY: ENV_LOCK is still held here.
        unsafe { std::env::set_var("DOSSIER_SCP", "/opt/scp-stand-in") };
        let named = program();
        // SAFETY: ENV_LOCK is still held here.
        unsafe {
            match before {
                Some(value) => std::env::set_var("DOSSIER_SCP", value),
                None => std::env::remove_var("DOSSIER_SCP"),
            }
        }
        assert_eq!(unset, OsString::from("scp"));
        assert_eq!(empty, OsString::from("scp"));
        assert_eq!(named, OsString::from("/opt/scp-stand-in"));
    }

    #[test]
    fn files_of_skips_the_options_and_the_value_of_task() {
        assert_eq!(
            files_of(&words(&["a.sim", "--task", "T-1", "sub/b.sim", "--force"])),
            words(&["a.sim", "sub/b.sim"])
        );
    }

    #[test]
    fn files_of_skips_the_value_of_agent() {
        assert_eq!(
            files_of(&words(&["--agent", "anna-claude", "c.sim", "--done"])),
            words(&["c.sim"])
        );
    }

    fn pointer() -> Pointer {
        Pointer {
            server: "anna@server".to_string(),
            folder: "/projects/p1".to_string(),
            code: "P-001".to_string(),
        }
    }

    #[test]
    fn maybe_leaves_files_and_the_checks_and_abandon_of_return_to_the_caller() {
        let base = tempfile::tempdir().unwrap();
        let pointer = pointer();
        assert_eq!(maybe("files", &[], base.path(), &pointer, None), None);
        assert_eq!(
            maybe(
                "return",
                &words(&["sim/a.sim", "--check"]),
                base.path(),
                &pointer,
                None
            ),
            None
        );
        assert_eq!(
            maybe(
                "return",
                &words(&["sim/a.sim", "--abandon"]),
                base.path(),
                &pointer,
                None
            ),
            None
        );
    }

    #[test]
    fn maybe_answers_take_with_exit_5_when_ssh_cannot_be_started() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let base = tempfile::tempdir().unwrap();
        let before = std::env::var_os("DOSSIER_SSH");
        // SAFETY: ENV_LOCK is held, so no other test in this process reads or changes the
        // environment while this one does.
        unsafe { std::env::set_var("DOSSIER_SSH", base.path().join("no-ssh-here")) };
        let code = maybe(
            "take",
            &words(&["sim/a.sim", "--task", "T-1"]),
            base.path(),
            &pointer(),
            Some("anna-claude"),
        );
        // SAFETY: ENV_LOCK is still held here.
        unsafe {
            match before {
                Some(value) => std::env::set_var("DOSSIER_SSH", value),
                None => std::env::remove_var("DOSSIER_SSH"),
            }
        }
        assert_eq!(code, Some(5));
    }

    #[test]
    fn take_argv_runs_take_with_json_in_the_folder_with_the_agent() {
        let argv = take_argv(
            &words(&["sim/a.sim", "--task", "T-1"]),
            &pointer(),
            Some("anna-claude"),
        );
        assert_eq!(
            argv.last().map(String::as_str),
            Some(
                "cd '/projects/p1' && DOSSIER_AGENT='anna-claude' 'dossier' 'take' 'sim/a.sim' '--task' 'T-1' '--json'"
            )
        );
        let without = take_argv(&words(&["a.sim", "--task", "T-1"]), &pointer(), None);
        assert_eq!(
            without.last().map(String::as_str),
            Some("cd '/projects/p1' && 'dossier' 'take' 'a.sim' '--task' 'T-1' '--json'")
        );
    }

    #[test]
    fn taken_paths_reads_each_path_of_the_answer_in_order() {
        let answer = r#"{"schema":"dossier.take.v1","ok":true,"data":{"task":"T-1","files":[{"path":"sim/a.sim"},{"path":"sim/run 1/b é.sim"}]},"warnings":[]}"#;
        assert_eq!(
            taken_paths(answer).ok(),
            Some(words(&["sim/a.sim", "sim/run 1/b é.sim"]))
        );
    }

    #[test]
    fn taken_paths_refuses_a_path_that_leaves_the_folder_before_anything_is_copied() {
        for path in ["../a.sim", "/etc/a.sim", "sim/../../a.sim", ""] {
            let answer = format!(
                r#"{{"ok":true,"data":{{"task":"T-1","files":[{{"path":"ok.sim"}},{{"path":"{path}"}}]}}}}"#
            );
            let error = taken_paths(&answer).err();
            assert_eq!(error.map(|error| error.exit()), Some(5), "path {path:?}");
        }
    }

    #[test]
    fn failure_message_is_the_error_of_the_answer_else_the_last_line_of_stderr() {
        let answer = r#"{"schema":"dossier.take.v1","ok":false,"error":{"code":"REFUSED","message":"anna holds sim/a.sim"},"warnings":[]}"#;
        assert_eq!(
            failure_message(answer, "", 1),
            "anna holds sim/a.sim".to_string()
        );
        assert_eq!(
            failure_message(
                "",
                "Warning: noise\nssh: connect to host server port 22: Connection refused\n",
                255
            ),
            "ssh: connect to host server port 22: Connection refused".to_string()
        );
        assert_eq!(
            failure_message("", "", 4),
            "the command ended with exit code 4".to_string()
        );
    }
}
