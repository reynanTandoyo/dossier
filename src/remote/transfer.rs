//! Copies of large files between a connected PC folder and the server (R-REMOTE-4): the scp
//! arguments, and `take` and `return` in a connected folder.

use crate::remote::pointer::Pointer;
use std::ffi::OsString;
use std::path::Path;

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
pub fn maybe(
    _command: &str,
    _args: &[String],
    _base: &Path,
    _pointer: &Pointer,
    _agent: Option<&str>,
) -> Option<u8> {
    None
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
}
