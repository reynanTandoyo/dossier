//! Forwarding a command to the server that keeps the dossier.
//!
//! A command typed in a connected folder (a pointer file, R-REMOTE-1) or with `--project <code>`
//! (R-REMOTE-10) runs on the server through ssh. The arguments go on as they were typed, so that
//! they arrive unchanged (R-REMOTE-2); the exit code and the output come back unchanged.

use crate::error::Result;
use crate::remote::{pointer, register, ssh};
use crate::term;
use std::io::Write;

/// The global options that take their value as the next argument.
const VALUE_OPTIONS: [&str; 3] = ["--dossier", "--agent", "--project"];

/// The commands that stay on the PC: `connect` and `projects` manage the PC's files, `help`
/// prints the text of the program on the PC, and `hooks`, `guard` and `skill` work with the
/// Claude Code on the PC.
const LOCAL_COMMANDS: [&str; 6] = ["connect", "projects", "help", "hooks", "guard", "skill"];

/// The commands that ask ssh for a terminal when the PC has one (R-REMOTE-3): the approvals and
/// changes of a person, and the decisions of a decider.
const TERMINAL_COMMANDS: [&str; 12] = [
    "approve", "remove", "restore", "edit", "accept", "decline", "confirm", "refute", "cleared",
    "value", "close", "reviewed",
];

/// The time limit of the connection in seconds (R-REMOTE-11).
const TIMEOUT_SECONDS: u64 = 10;

/// The exit code when the command could not be run on the server.
const FORWARD_FAILED: u8 = 5;

/// Where a command runs: the server and the folder of the project on it.
struct Target {
    server: String,
    folder: String,
}

/// What `--project` says about a command.
enum ProjectOption {
    /// Not given.
    Absent,
    /// Given with this code.
    Code(String),
    /// Given without a value; the program's own parser reports that.
    NoValue,
}

/// Called first by `main_exit`. Some(exit code) when the command was run on a server: the
/// `--project <code>` option is given, or the current folder has a pointer file and the command
/// is not one of `LOCAL_COMMANDS`. None: run locally.
pub fn maybe(args: &[String]) -> Option<u8> {
    // The program name comes first; everything after it is the command line.
    let (_, args) = args.split_first()?;
    let command_at = command_index(args)?;
    let command = args[command_at].as_str();
    if LOCAL_COMMANDS.contains(&command) {
        return None;
    }

    let mut forwarded = args.to_vec();
    let target = match take_project(&mut forwarded) {
        ProjectOption::NoValue => return None,
        ProjectOption::Absent => pointer_target()?,
        ProjectOption::Code(code) => match project_target(&code) {
            Ok(Some(target)) => target,
            Ok(None) => {
                report(&format!(
                    "no project {code} in the register; see dossier projects"
                ));
                return Some(1);
            }
            Err(error) => {
                report(&error.message);
                return Some(error.exit());
            }
        },
    };

    let mut program = "dossier";
    if command == "todo" {
        program = "todo";
        if let Some(at) = command_index(&forwarded) {
            forwarded.remove(at);
        }
    }
    let terminal = TERMINAL_COMMANDS.contains(&command) && term::is_terminal();
    let agent = agent_name(&forwarded);
    let argv = ssh::argv(
        &target.server,
        &target.folder,
        agent.as_deref(),
        program,
        &forwarded,
        terminal,
        TIMEOUT_SECONDS,
    );
    match ssh::run(&argv) {
        Ok(code) => Some(code),
        Err(error) => {
            report(&error.message);
            Some(FORWARD_FAILED)
        }
    }
}

/// The index of the command word: the first argument that does not start with `-` and is not
/// the value of `--dossier`, `--agent` or `--project`. None when there is none.
fn command_index(args: &[String]) -> Option<usize> {
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        if VALUE_OPTIONS.contains(&arg) {
            index += 2;
        } else if arg.starts_with('-') {
            index += 1;
        } else {
            return Some(index);
        }
    }
    None
}

/// Takes `--project <code>` or `--project=<code>` out of `args`, and says what it found.
fn take_project(args: &mut Vec<String>) -> ProjectOption {
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        if let Some(code) = arg.strip_prefix("--project=") {
            let code = code.to_string();
            args.remove(index);
            return ProjectOption::Code(code);
        }
        if arg == "--project" {
            if index + 1 == args.len() {
                return ProjectOption::NoValue;
            }
            let code = args.remove(index + 1);
            args.remove(index);
            return ProjectOption::Code(code);
        }
        index += if VALUE_OPTIONS.contains(&arg) { 2 } else { 1 };
    }
    ProjectOption::Absent
}

/// The value of `--name <value>` or `--name=<value>` in `args`, which stays in `args`.
fn option_value(args: &[String], name: &str) -> Option<String> {
    let mut words = args.iter();
    while let Some(arg) = words.next() {
        if arg == name {
            return words.next().cloned();
        }
        if let Some(value) = arg
            .strip_prefix(name)
            .and_then(|rest| rest.strip_prefix('='))
        {
            return Some(value.to_string());
        }
    }
    None
}

/// The name the command runs as: `--agent`, else DOSSIER_AGENT, else TODO_AGENT. An empty value
/// counts as not given.
fn agent_name(args: &[String]) -> Option<String> {
    option_value(args, "--agent")
        .filter(|name| !name.is_empty())
        .or_else(|| non_empty_variable("DOSSIER_AGENT"))
        .or_else(|| non_empty_variable("TODO_AGENT"))
}

/// The value of an environment variable, when it is set and not empty.
fn non_empty_variable(name: &str) -> Option<String> {
    std::env::var_os(name)
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
}

/// The server and folder of the pointer file in the current folder or above.
fn pointer_target() -> Option<Target> {
    let folder = std::env::current_dir().ok()?;
    let (_, pointer) = pointer::find(&folder)?;
    Some(Target {
        server: pointer.server,
        folder: pointer.folder,
    })
}

/// The server and folder of the project `code` in the register (codes ignore case). Ok(None)
/// when the register has no such project.
fn project_target(code: &str) -> Result<Option<Target>> {
    Ok(register::load()?
        .into_iter()
        .find(|project| project.code.to_lowercase() == code.to_lowercase())
        .map(|project| Target {
            server: project.server,
            folder: project.folder,
        }))
}

/// Writes `error: <message>` to standard error. A write that fails is ignored.
fn report(message: &str) {
    let _ = writeln!(std::io::stderr(), "error: {message}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    /// Puts back the current folder and the named environment variables when it is dropped,
    /// so that a failing test leaves them as they were too.
    struct Restore {
        folder: PathBuf,
        variables: Vec<(&'static str, Option<OsString>)>,
    }

    impl Restore {
        /// Remembers the current folder and the present values of `names`.
        fn new(names: &[&'static str]) -> Restore {
            Restore {
                folder: std::env::current_dir().unwrap(),
                variables: names
                    .iter()
                    .map(|name| (*name, std::env::var_os(name)))
                    .collect(),
            }
        }
    }

    impl Drop for Restore {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.folder);
            for (name, value) in &self.variables {
                // SAFETY: the test holds crate::ENV_LOCK until this is dropped, so no other
                // test in this process reads or changes the environment meanwhile.
                unsafe {
                    match value {
                        Some(value) => std::env::set_var(name, value),
                        None => std::env::remove_var(name),
                    }
                }
            }
        }
    }

    /// The stand-in for ssh in `folder`: it writes its arguments to `args_file` and exits with 7.
    #[cfg(windows)]
    fn stand_in(folder: &Path, args_file: &Path) -> PathBuf {
        let path = folder.join("ssh.cmd");
        let text = format!(
            "@echo off\r\necho %* > \"{}\"\r\nexit /b 7\r\n",
            args_file.display()
        );
        std::fs::write(&path, text).unwrap();
        path
    }

    /// The stand-in for ssh in `folder`: it writes its arguments to `args_file` and exits with 7.
    #[cfg(unix)]
    fn stand_in(folder: &Path, args_file: &Path) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = folder.join("ssh");
        let text = format!(
            "#!/bin/sh\nprintf \"%s\\n\" \"$@\" > \"{}\"\nexit 7\n",
            args_file.display()
        );
        std::fs::write(&path, text).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    /// Runs `maybe` with `words` (the program name first). Returns its result and what the
    /// stand-in wrote, which is empty when the stand-in did not run.
    fn maybe_with(args_file: &Path, words: &[&str]) -> (Option<u8>, String) {
        let _ = std::fs::remove_file(args_file);
        let args: Vec<String> = words.iter().map(|word| word.to_string()).collect();
        let code = maybe(&args);
        let written = std::fs::read_to_string(args_file).unwrap_or_default();
        (code, written)
    }

    /// The whitespace-separated words of what the stand-in wrote.
    fn tokens(text: &str) -> Vec<&str> {
        text.split_whitespace().collect()
    }

    #[test]
    fn commands_run_on_the_server_of_the_connected_folder_or_of_the_project() {
        let _lock = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let _restore = Restore::new(&["DOSSIER_SSH", "DOSSIER_REGISTER", "DOSSIER_ANSWERS"]);
        let root = dir.path();
        let args_file = root.join("args.txt");

        let ssh_folder = root.join(".test");
        std::fs::create_dir_all(&ssh_folder).unwrap();
        let ssh = stand_in(&ssh_folder, &args_file);

        let register = root.join("register").join("projects.toml");
        std::fs::create_dir_all(register.parent().unwrap()).unwrap();
        std::fs::write(
            &register,
            "[[project]]\ncode = \"P-002\"\nserver = \"ben@sim02\"\nfolder = \"/work/P-002\"\n",
        )
        .unwrap();

        // SAFETY: ENV_LOCK is held, so no other test in this process reads or changes the
        // environment while this one does.
        unsafe {
            std::env::set_var("DOSSIER_SSH", &ssh);
            std::env::set_var("DOSSIER_REGISTER", &register);
            std::env::remove_var("DOSSIER_ANSWERS");
        }

        // No pointer and no --project: the command runs here.
        std::env::set_current_dir(root).unwrap();
        assert_eq!(
            maybe_with(&args_file, &["dossier", "list"]),
            (None, String::new())
        );

        // A connected folder: the command runs in the pointer's folder on its server.
        pointer::write(
            root,
            &pointer::Pointer {
                server: "anna@sim01".into(),
                folder: "/srv/P-001".into(),
                code: "P-001".into(),
            },
        )
        .unwrap();
        let (code, written) = maybe_with(&args_file, &["dossier", "list"]);
        assert_eq!(code, Some(7));
        assert!(written.contains("BatchMode=yes"), "{written}");
        assert!(written.contains("anna@sim01"), "{written}");
        assert!(
            written.contains("cd '/srv/P-001' && 'dossier' 'list'"),
            "{written}"
        );

        // Commands that stay on the PC, with a pointer present.
        let local: [&[&str]; 7] = [
            &["dossier", "connect", "anna@sim01:/srv/P-001"],
            &["dossier", "projects"],
            &["dossier", "help"],
            &["dossier", "hooks", "install"],
            &["dossier", "guard"],
            &["dossier", "skill", "install"],
            &["dossier", "--json"],
        ];
        for words in local {
            assert_eq!(
                maybe_with(&args_file, words),
                (None, String::new()),
                "{words:?}"
            );
        }

        // todo runs the todo program in the same folder, without the word todo.
        let (code, written) = maybe_with(&args_file, &["dossier", "todo", "list"]);
        assert_eq!(code, Some(7));
        assert!(
            written.contains("cd '/srv/P-001' && 'todo' 'list'"),
            "{written}"
        );

        // The value of --dossier is not the command word.
        let (code, written) = maybe_with(&args_file, &["dossier", "--dossier", "connect", "list"]);
        assert_eq!(code, Some(7));
        assert!(
            written.contains("'dossier' '--dossier' 'connect' 'list'"),
            "{written}"
        );

        // --agent names the agent on the server; the --agent option itself goes on too.
        let (_, written) = maybe_with(&args_file, &["dossier", "--agent=anna-claude", "list"]);
        assert!(written.contains("DOSSIER_AGENT='anna-claude'"), "{written}");

        // A terminal on the PC gives ssh one for approve, and none for list.
        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::set_var("DOSSIER_ANSWERS", "") };
        let (_, written) = maybe_with(&args_file, &["dossier", "approve", "DEC-001"]);
        assert!(tokens(&written).contains(&"-t"), "{written}");
        let (code, written) = maybe_with(&args_file, &["dossier", "accept", "DEC-001"]);
        assert_eq!(code, Some(7));
        assert!(tokens(&written).contains(&"-t"), "{written}");
        for command in ["close", "reviewed"] {
            let (code, written) = maybe_with(&args_file, &["dossier", command, "DEC-001"]);
            assert_eq!(code, Some(7));
            assert!(tokens(&written).contains(&"-t"), "{written}");
        }
        let (_, written) = maybe_with(&args_file, &["dossier", "list"]);
        assert!(!tokens(&written).contains(&"-t"), "{written}");
        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_ANSWERS") };

        // --project runs the command for the register's project, whatever the folder holds.
        let (code, written) = maybe_with(&args_file, &["dossier", "--project", "P-002", "list"]);
        assert_eq!(code, Some(7));
        assert!(written.contains("ben@sim02"), "{written}");
        assert!(
            written.contains("cd '/work/P-002' && 'dossier' 'list'"),
            "{written}"
        );
        assert!(!written.contains("--project"), "{written}");
        let (code, written) = maybe_with(&args_file, &["dossier", "list", "--project=p-002"]);
        assert_eq!(code, Some(7));
        assert!(
            written.contains("cd '/work/P-002' && 'dossier' 'list'"),
            "{written}"
        );

        // An unknown project and a --project without a value are refused here.
        assert_eq!(
            maybe_with(&args_file, &["dossier", "--project", "P-999", "list"]),
            (Some(1), String::new())
        );
        assert_eq!(
            maybe_with(&args_file, &["dossier", "list", "--project"]),
            (None, String::new())
        );

        // ssh cannot be run: the command fails with exit code 5.
        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::set_var("DOSSIER_SSH", root.join("no-such-ssh")) };
        assert_eq!(
            maybe_with(&args_file, &["dossier", "list"]),
            (Some(5), String::new())
        );
    }
}
