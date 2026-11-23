//! Copies of large files between a connected PC folder and the server (R-REMOTE-4): the scp
//! arguments, and `take` and `return` in a connected folder.

use crate::error::{Code, Error, Result};
use crate::remote::pointer::Pointer;
use crate::remote::ssh;
use std::ffi::OsString;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

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

/// "<server>:<folder/stored>" for the target of an upload (`return --done`): unquoted and not
/// escaped, because SFTP does not glob a target and a backslash would become part of the name
/// (tested 2026-11-23).
pub fn remote_target(server: &str, folder: &str, stored: &str) -> String {
    format!("{server}:{folder}/{stored}")
}

/// The file arguments of `return`: the words that do not start with `-` and are not the word
/// after --task or --agent.
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

/// Called by forward::maybe for `take` and for `return --done` when the folder is connected;
/// Some(exit code). Every other command returns None.
pub fn maybe(
    command: &str,
    args: &[String],
    base: &Path,
    pointer: &Pointer,
    agent: Option<&str>,
) -> Option<u8> {
    if command == "take" {
        Some(take(args, base, pointer, agent))
    } else if command == "return" && args.iter().any(|arg| arg == "--done") {
        Some(give_back(args, base, pointer, agent))
    } else {
        None
    }
}

/// `take` in a connected folder: the server records the take first (`dossier take … --json`), then
/// each file it names is copied down to the same path below `base`, unless a local file already
/// exists there (D46: that copy is kept). A failed copy ends with exit 5 and says that the take is
/// recorded, so running the same command again copies again.
fn take(args: &[String], base: &Path, pointer: &Pointer, agent: Option<&str>) -> u8 {
    let sub = std::env::current_dir()
        .ok()
        .and_then(|current| crate::remote::forward::sub_folder(&current, base));
    let ssh_argv = take_argv(args, sub.as_deref(), pointer, agent);
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
        Ok(files) => {
            for (path, copied) in &files {
                let line = if *copied {
                    format!("{path}: taken and copied")
                } else {
                    format!(
                        "{path}: taken; kept your local copy (delete it and run the same take again to copy the server's version)"
                    )
                };
                let _ = writeln!(std::io::stdout(), "{line}");
            }
            0
        }
        Err(error) => fail(&error),
    }
}

/// The ssh arguments that run `dossier take <args> --json` in the connected folder, in the
/// subfolder `sub` when it exists, with the variables of every forwarded command: DOSSIER_AGENT when
/// an agent name is given, and the PC's versions (§16).
fn take_argv(
    args: &[String],
    sub: Option<&str>,
    pointer: &Pointer,
    agent: Option<&str>,
) -> Vec<String> {
    let env = crate::remote::forward::remote_variables("dossier", agent);
    let mut words = vec!["take".to_string()];
    words.extend(args.iter().cloned());
    words.push("--json".to_string());
    ssh::argv_with(
        &pointer.server,
        &pointer.folder,
        sub,
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

/// Copies each file the server took, in the order of its answer, and returns each path with whether
/// it was copied (false: the local file was kept). Every path is checked before the first copy, so
/// that a bad answer copies nothing.
fn copy_all(stdout: &str, base: &Path, pointer: &Pointer) -> Result<Vec<(String, bool)>> {
    let paths = taken_paths(stdout)?;
    let mut files = Vec::new();
    for path in paths {
        let copied = copy_down(&path, base, pointer)?;
        files.push((path, copied));
    }
    Ok(files)
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

/// The part file of a copy: the same path with `.dossier-part` appended to the file name, so that
/// `sim/a.sim` gives `sim/a.sim.dossier-part`. Built with OsString, so non-ASCII names stay as they
/// are.
fn part_path(local: &Path) -> PathBuf {
    let mut name = local.file_name().unwrap_or_default().to_os_string();
    name.push(".dossier-part");
    local.with_file_name(name)
}

/// The error of a copy that did not finish; running the same take again copies it.
fn copy_failed(path: &str) -> Error {
    Error::new(
        Code::Ssh,
        format!(
            "could not copy {path} from the server; the take is recorded, run the same command again to copy it"
        ),
    )
}

/// Copies one file the server took, from `<folder>/<path>` on the server to `<base>/<path>`,
/// creating the folders above it first. The copy goes to the part file beside it and is renamed
/// over the file only when complete, so a failed copy leaves no half file. A local file that
/// already exists is kept, with no copy: returns false (D46). Returns true when the file was copied.
fn copy_down(path: &str, base: &Path, pointer: &Pointer) -> Result<bool> {
    let local = base.join(path);
    if local.exists() {
        return Ok(false);
    }
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
    let part = part_path(&local);
    match std::fs::remove_file(&part) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(Error::new(
                Code::Ssh,
                format!(
                    "could not copy {path} from the server: cannot remove the part file {} ({error}); the take is recorded, run the same command again to copy it",
                    part.display()
                ),
            ));
        }
    }
    let source = remote_path(&pointer.server, &pointer.folder, path);
    let target = part.to_string_lossy().into_owned();
    let status = std::process::Command::new(program())
        .args(argv(&source, &target, TIMEOUT_SECONDS))
        .status();
    match status {
        Ok(status) if status.success() => {}
        _ => {
            let _ = std::fs::remove_file(&part);
            return Err(copy_failed(path));
        }
    }
    if std::fs::rename(&part, &local).is_err() {
        let _ = std::fs::remove_file(&part);
        return Err(copy_failed(path));
    }
    Ok(true)
}

/// `return --done` in a connected folder, in this order: the server checks that every file is still
/// as taken (`return … --check`) and stops when one is not; every file (the current folder joined
/// with the path as typed) is uploaded to a part file beside its copy on the server; each part file
/// is moved over its file; last, the server records the return (`return … --done`). A failed upload
/// leaves every file on the server as it was, so the take is unchanged and the same command works
/// again. After the first move a rerun's `--check` would refuse the replaced files, so a failure
/// there names the command that finishes the return on the server for the files moved so far, and
/// the command that gives back the rest here again.
fn give_back(args: &[String], base: &Path, pointer: &Pointer, agent: Option<&str>) -> u8 {
    let current = match std::env::current_dir() {
        Ok(current) => current,
        Err(error) => {
            return fail(&Error::new(
                Code::Io,
                format!("cannot read the current folder: {error}"),
            ));
        }
    };
    let sub = crate::remote::forward::sub_folder(&current, base);
    let files = files_of(args);

    let mut check = vec!["return".to_string()];
    check.extend(files.iter().cloned());
    check.push("--check".to_string());
    let code = server_result(remote(pointer, sub.as_deref(), agent, "dossier", &check));
    if code != 0 {
        return code;
    }

    for path in &files {
        if !current.join(path).exists() {
            report(&format!("{path} is not in this folder"));
            return 1;
        }
    }

    let names: Vec<String> = files.iter().map(|path| path.replace('\\', "/")).collect();
    for (path, name) in files.iter().zip(&names) {
        if !upload_part(&current.join(path), name, sub.as_deref(), pointer) {
            return fail(&Error::new(
                Code::Ssh,
                format!(
                    "could not copy {path} to the server; the take is unchanged, run the same command again"
                ),
            ));
        }
    }

    for (index, name) in names.iter().enumerate() {
        if !move_into_place(name, sub.as_deref(), pointer, agent) {
            // The `index` files before this one are replaced on the server; this one and the rest
            // still hold the old version. Nothing was recorded, so when none was replaced yet, the
            // take is unchanged and the same command works again.
            if index == 0 {
                return fail(&Error::new(
                    Code::Ssh,
                    format!(
                        "could not copy {} to the server; the take is unchanged, run the same command again",
                        files[index]
                    ),
                ));
            }
            report(&format!(
                "{}; {}",
                finish_on_server(pointer, sub.as_deref(), &files[..index]),
                rest_message(&files[index..])
            ));
            return 5;
        }
    }

    let mut done = vec!["return".to_string()];
    done.extend(files.iter().cloned());
    done.push("--done".to_string());
    let code = server_result(remote(pointer, sub.as_deref(), agent, "dossier", &done));
    if code != 0 {
        report(&finish_on_server(pointer, sub.as_deref(), &files));
        return 5;
    }
    0
}

/// Uploads one local file to `<sub>/<name>.dossier-part` on the server, beside the file it will
/// replace: scp, run in the connected folder. The file on the server does not change. True when
/// the upload succeeds.
fn upload_part(local: &Path, name: &str, sub: Option<&str>, pointer: &Pointer) -> bool {
    let stored = match sub {
        Some(sub) => format!("{sub}/{name}"),
        None => name.to_string(),
    };
    let target = remote_target(
        &pointer.server,
        &pointer.folder,
        &format!("{stored}.dossier-part"),
    );
    let source = local.to_string_lossy().into_owned();
    matches!(
        std::process::Command::new(program())
            .args(argv(&source, &target, TIMEOUT_SECONDS))
            .status(),
        Ok(status) if status.success()
    )
}

/// Moves `<name>.dossier-part` over `<name>` on the server with `mv -f`, run in `sub` below the
/// connected folder. This is the step that replaces the server's file. True when the move succeeds.
fn move_into_place(name: &str, sub: Option<&str>, pointer: &Pointer, agent: Option<&str>) -> bool {
    let part = format!("{name}.dossier-part");
    let mv = ["-f", "--", part.as_str(), name].map(str::to_string);
    matches!(remote(pointer, sub, agent, "mv", &mv), Ok((0, _, _)))
}

/// A file name as it is printed in a command for a shell: as it is when it holds only letters,
/// digits, `-`, `_`, `.` and `/`, else quoted (a space, a quote or `$` would change the command).
fn shell_word(file: &str) -> String {
    let plain = !file.is_empty()
        && file
            .chars()
            .all(|ch| ch.is_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/'));
    if plain {
        file.to_string()
    } else {
        ssh::quote(file)
    }
}

/// The files as they are printed in a command for a shell, separated by spaces.
fn shell_words(files: &[String]) -> String {
    files
        .iter()
        .map(|file| shell_word(file))
        .collect::<Vec<String>>()
        .join(" ")
}

/// The message for a return whose files are on the server but not recorded: the command that
/// finishes it on the server, in the folder (and subfolder) there. It is not a rerun of the same
/// command, because its `--check` would see the replaced files as changed.
fn finish_on_server(pointer: &Pointer, sub: Option<&str>, files: &[String]) -> String {
    let folder = match sub {
        Some(sub) => format!("{}/{sub}", pointer.folder),
        None => pointer.folder.clone(),
    };
    format!(
        "the files are on the server but the return is not recorded; finish it on the server: ssh {}, then in {folder} run: dossier return {} --done",
        pointer.server,
        shell_words(files)
    )
}

/// The line that gives back the files a failed move left unfinished, run here again. Their files on
/// the server are untouched, so this command passes its `--check`.
fn rest_message(files: &[String]) -> String {
    format!(
        "then run here again: dossier return {} --done",
        shell_words(files)
    )
}

/// Runs `program args` on the server, in `sub` below the connected folder when it exists, with the
/// variables of the forwarded commands. Its standard output and standard error are printed as they
/// are; returns the exit code with both outputs.
fn remote(
    pointer: &Pointer,
    sub: Option<&str>,
    agent: Option<&str>,
    program: &str,
    args: &[String],
) -> Result<(u8, String, String)> {
    let env = crate::remote::forward::remote_variables(program, agent);
    let ssh_argv = ssh::argv_with(
        &pointer.server,
        &pointer.folder,
        sub,
        &env,
        program,
        args,
        false,
        TIMEOUT_SECONDS,
    );
    let (code, stdout, stderr) = ssh::capture(&ssh_argv)?;
    let _ = write!(std::io::stdout(), "{stdout}");
    let _ = write!(std::io::stderr(), "{stderr}");
    Ok((code, stdout, stderr))
}

/// The exit code of a command run on the server. ssh's own failure (255) is reported as Code::Ssh,
/// as in `take`; any other failure to run ssh is reported as it is.
fn server_result(outcome: Result<(u8, String, String)>) -> u8 {
    match outcome {
        Ok((255, stdout, stderr)) => fail(&Error::new(
            Code::Ssh,
            failure_message(&stdout, &stderr, 255),
        )),
        Ok((code, _, _)) => code,
        Err(error) => fail(&error),
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
    fn remote_target_is_not_quoted_and_not_escaped() {
        assert_eq!(
            remote_target("anna@server", "/srv/p", "it's [2]*.sim.dossier-part"),
            "anna@server:/srv/p/it's [2]*.sim.dossier-part"
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
    fn maybe_answers_return_done_with_exit_5_when_ssh_cannot_be_started() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let base = tempfile::tempdir().unwrap();
        let before = std::env::var_os("DOSSIER_SSH");
        // SAFETY: ENV_LOCK is held, so no other test in this process reads or changes the
        // environment while this one does.
        unsafe { std::env::set_var("DOSSIER_SSH", base.path().join("no-ssh-here")) };
        let code = maybe(
            "return",
            &words(&["sim/a.sim", "--done"]),
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
        let version = env!("CARGO_PKG_VERSION");
        let argv = take_argv(
            &words(&["sim/a.sim", "--task", "T-1"]),
            None,
            &pointer(),
            Some("anna-claude"),
        );
        let expected = format!(
            "cd '/projects/p1' && DOSSIER_AGENT='anna-claude' DOSSIER_CLIENT_VERSION='{version}' DOSSIER_CLIENT_FORMAT='1' 'dossier' 'take' 'sim/a.sim' '--task' 'T-1' '--json'"
        );
        assert_eq!(argv.last(), Some(&expected));

        let without = take_argv(&words(&["a.sim", "--task", "T-1"]), None, &pointer(), None);
        let expected = format!(
            "cd '/projects/p1' && DOSSIER_CLIENT_VERSION='{version}' DOSSIER_CLIENT_FORMAT='1' 'dossier' 'take' 'a.sim' '--task' 'T-1' '--json'"
        );
        assert_eq!(without.last(), Some(&expected));

        // A subfolder of the connected folder: the command runs there when it exists, else in the
        // connected folder (argv_with), so `a.sim` means sim/a.sim on the server.
        let in_sub = take_argv(
            &words(&["a.sim", "--task", "T-1"]),
            Some("sim"),
            &pointer(),
            None,
        );
        let expected = format!(
            "{{ cd '/projects/p1/sim' 2>/dev/null || cd '/projects/p1'; }} && DOSSIER_CLIENT_VERSION='{version}' DOSSIER_CLIENT_FORMAT='1' 'dossier' 'take' 'a.sim' '--task' 'T-1' '--json'"
        );
        assert_eq!(in_sub.last(), Some(&expected));
        assert_ne!(in_sub.last(), without.last());
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

    #[test]
    fn part_path_appends_the_part_suffix_to_the_file_name() {
        assert_eq!(
            part_path(Path::new("sim/a.sim")),
            PathBuf::from("sim/a.sim.dossier-part")
        );
        assert_eq!(
            part_path(Path::new("sim/Größe 1.sim")),
            PathBuf::from("sim/Größe 1.sim.dossier-part")
        );
    }

    #[test]
    fn copy_down_keeps_a_local_file_that_exists_and_runs_no_scp() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let base = tempfile::tempdir().unwrap();
        let local = base.path().join("sim").join("a.sim");
        std::fs::create_dir_all(local.parent().unwrap()).unwrap();
        std::fs::write(&local, "my edits").unwrap();
        let before = std::env::var_os("DOSSIER_SCP");
        // SAFETY: ENV_LOCK is held, so no other test in this process reads or changes the
        // environment while this one does. A missing program makes any run of scp fail.
        unsafe { std::env::set_var("DOSSIER_SCP", base.path().join("no-scp-here")) };
        let result = copy_down("sim/a.sim", base.path(), &pointer()).ok();
        // SAFETY: ENV_LOCK is still held here.
        unsafe {
            match before {
                Some(value) => std::env::set_var("DOSSIER_SCP", value),
                None => std::env::remove_var("DOSSIER_SCP"),
            }
        }
        assert_eq!(result, Some(false));
        assert_eq!(std::fs::read_to_string(&local).unwrap(), "my edits");
        assert!(!part_path(&local).exists());
    }

    #[test]
    fn copy_down_removes_an_old_part_file_and_the_part_file_of_a_failed_copy() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let base = tempfile::tempdir().unwrap();
        let local = base.path().join("a.sim");
        let part = part_path(&local);
        std::fs::write(&part, "half copy from before").unwrap();
        let before = std::env::var_os("DOSSIER_SCP");
        // SAFETY: ENV_LOCK is held, so no other test in this process reads or changes the
        // environment while this one does.
        unsafe { std::env::set_var("DOSSIER_SCP", base.path().join("no-scp-here")) };
        let result = copy_down("a.sim", base.path(), &pointer());
        // SAFETY: ENV_LOCK is still held here.
        unsafe {
            match before {
                Some(value) => std::env::set_var("DOSSIER_SCP", value),
                None => std::env::remove_var("DOSSIER_SCP"),
            }
        }
        assert_eq!(result.err().map(|error| error.exit()), Some(5));
        assert!(!part.exists());
        assert!(!local.exists());
    }

    #[test]
    fn finish_on_server_names_the_folder_with_or_without_a_sub_folder_and_the_files() {
        let files = words(&["a.sim", "sim/b.sim"]);
        assert_eq!(
            finish_on_server(&pointer(), None, &files),
            "the files are on the server but the return is not recorded; finish it on the server: ssh anna@server, then in /projects/p1 run: dossier return a.sim sim/b.sim --done"
        );
        assert_eq!(
            finish_on_server(&pointer(), Some("sim"), &files),
            "the files are on the server but the return is not recorded; finish it on the server: ssh anna@server, then in /projects/p1/sim run: dossier return a.sim sim/b.sim --done"
        );
    }

    #[test]
    fn finish_on_server_quotes_a_file_with_a_space_and_keeps_a_plain_one_plain() {
        let files = words(&["a.sim", "run 1.sim"]);
        assert_eq!(
            finish_on_server(&pointer(), None, &files),
            "the files are on the server but the return is not recorded; finish it on the server: ssh anna@server, then in /projects/p1 run: dossier return a.sim 'run 1.sim' --done"
        );
    }

    #[test]
    fn shell_word_keeps_plain_names_and_quotes_what_a_shell_would_change() {
        assert_eq!(shell_word("sim/Größe-1_a.sim"), "sim/Größe-1_a.sim");
        assert_eq!(shell_word("run 1.sim"), "'run 1.sim'");
        assert_eq!(shell_word("it's.sim"), "'it'\\''s.sim'");
        assert_eq!(shell_word("$HOME.sim"), "'$HOME.sim'");
        assert_eq!(shell_word("a\\b.sim"), "'a\\b.sim'");
        assert_eq!(shell_word(""), "''");
    }

    #[test]
    fn rest_message_gives_back_the_files_from_the_failed_one_on_quoted_the_same_way() {
        assert_eq!(
            rest_message(&words(&["sim/b.sim", "run 1.sim"])),
            "then run here again: dossier return sim/b.sim 'run 1.sim' --done"
        );
        assert_eq!(
            rest_message(&words(&["c.sim"])),
            "then run here again: dossier return c.sim --done"
        );
    }
}
