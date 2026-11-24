//! Forwarding a command to the server that keeps the dossier.
//!
//! A command typed in a connected folder (a pointer file, R-REMOTE-1) or with `--project <code>`
//! (R-REMOTE-10) runs on the server through ssh. The arguments go on as they were typed, so that
//! they arrive unchanged (R-REMOTE-2); the exit code and the output come back unchanged.

use crate::error::{Code, Error, Result};
use crate::remote::{pointer, register, ssh, transfer, version};
use crate::term;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The global options that take their value as the next argument.
const VALUE_OPTIONS: [&str; 3] = ["--dossier", "--agent", "--project"];

/// The commands that stay on the PC: `connect` and `projects` manage the PC's files, `help`
/// prints the text of the program on the PC, and `hooks`, `guard` and `skill` work with the
/// Claude Code on the PC. `after` reads the payload of the Claude Code on the PC.
/// `review-session`, `session` and `session-end` read files of this machine, which the server
/// could not read.
const LOCAL_COMMANDS: [&str; 10] = [
    "connect",
    "projects",
    "help",
    "hooks",
    "guard",
    "skill",
    "after",
    "review-session",
    "session",
    "session-end",
];

/// The commands that ask ssh for a terminal when the PC has one (R-REMOTE-3): the approvals and
/// changes of a person, and the decisions of a decider.
const TERMINAL_COMMANDS: [&str; 12] = [
    "approve", "remove", "restore", "edit", "accept", "decline", "confirm", "refute", "cleared",
    "value", "close", "reviewed",
];

/// The option that lets a person at a terminal write text that looks like a credential (R-CONF-7).
/// A command with it asks ssh for a terminal too.
const ALLOW_CREDENTIAL: &str = "--allow-credential-like";

/// The time limit of the connection in seconds (R-REMOTE-11).
const TIMEOUT_SECONDS: u64 = 10;

/// The exit code when the command could not be run on the server.
const FORWARD_FAILED: u8 = 5;

/// Where a command runs: the server and the folder of the project on it, and the folder below it
/// that the command runs in when it exists.
struct Target {
    server: String,
    folder: String,
    /// The current folder below the folder of the pointer file, written with `/` (R-XOS-3).
    /// None for the folder itself and for `--project`.
    sub: Option<String>,
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
/// is not one of `LOCAL_COMMANDS`. None: run locally. `take` and `return --done` copy files, so in
/// a connected folder transfer runs them (R-REMOTE-4); with `--project` they are refused.
pub fn maybe(args: &[String]) -> Option<u8> {
    // The program name comes first; everything after it is the command line.
    let (_, args) = args.split_first()?;
    let command_at = command_index(args)?;
    let command = args[command_at].as_str();
    if LOCAL_COMMANDS.contains(&command) {
        return None;
    }

    let mut forwarded = args.to_vec();
    let (target, connected) = match take_project(&mut forwarded) {
        ProjectOption::NoValue => return None,
        ProjectOption::Absent => {
            let (target, base, pointer) = pointer_target()?;
            (target, Some((base, pointer)))
        }
        ProjectOption::Code(code) => match project_target(&code) {
            Ok(Some(target)) => (target, None),
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

    let after = after_command(&forwarded);
    let agent = agent_name(&forwarded);
    if let Some((base, pointer)) = &connected {
        if matches!(command, "take" | "return")
            && let Some(code) = transfer::maybe(command, after, base, pointer, agent.as_deref())
        {
            return Some(code);
        }
    } else if copies_files(command, after) {
        let error = Error::new(
            Code::Usage,
            "take and return --done copy files and need a connected folder: dossier connect",
        );
        report(&error.message);
        return Some(error.exit());
    }

    let mut program = "dossier";
    if command == "todo" {
        program = "todo";
        if let Some(at) = command_index(&forwarded) {
            forwarded.remove(at);
        }
    }
    let terminal = (TERMINAL_COMMANDS.contains(&command) || allows_credential(&forwarded))
        && term::is_terminal();
    let env = remote_variables(program, agent.as_deref());
    let argv = ssh::argv_with(
        &target.server,
        &target.folder,
        target.sub.as_deref(),
        &env,
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

/// The words after the command word (`command_index`); none when there is no command word.
fn after_command(args: &[String]) -> &[String] {
    match command_index(args) {
        Some(at) => &args[at + 1..],
        None => &[],
    }
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

/// The variables set for the command on the server: DOSSIER_AGENT when a name is known, and for
/// `dossier` (not `todo`) the program version and format of the PC (R-XOS-9), which the server
/// compares before it runs the command.
pub(crate) fn remote_variables(program: &str, agent: Option<&str>) -> Vec<(String, String)> {
    let mut env = Vec::new();
    if let Some(agent) = agent {
        env.push(("DOSSIER_AGENT".to_string(), agent.to_string()));
    }
    if program == "dossier" {
        env.push((
            version::CLIENT_VERSION.to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        ));
        env.push((
            version::CLIENT_FORMAT.to_string(),
            version::FORMAT.to_string(),
        ));
    }
    env
}

/// The value of an environment variable, when it is set and not empty.
fn non_empty_variable(name: &str) -> Option<String> {
    std::env::var_os(name)
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
}

/// Whether `--allow-credential-like` is among the arguments, anywhere, as the program reads it
/// (tx.rs).
fn allows_credential(args: &[String]) -> bool {
    args.iter().any(|arg| arg == ALLOW_CREDENTIAL)
}

/// The server and folder of the pointer file in the current folder or above, and the current
/// folder below the pointer's folder. Also the folder that holds the pointer file, and the pointer.
fn pointer_target() -> Option<(Target, PathBuf, pointer::Pointer)> {
    let folder = std::env::current_dir().ok()?;
    let (pointer_folder, pointer) = pointer::find(&folder)?;
    let target = Target {
        server: pointer.server.clone(),
        folder: pointer.folder.clone(),
        sub: sub_folder(&folder, &pointer_folder),
    };
    Some((target, pointer_folder, pointer))
}

/// Whether the command copies files, which needs a connected folder (R-REMOTE-4): `take`, and
/// `return` with `--done`. `args` are the words after the command.
fn copies_files(command: &str, args: &[String]) -> bool {
    command == "take" || (command == "return" && args.iter().any(|arg| arg == "--done"))
}

/// The folder `current` below `pointer_folder`, its parts joined with `/`. None when they are
/// the same folder, or when a part of the path is not text.
pub(crate) fn sub_folder(current: &Path, pointer_folder: &Path) -> Option<String> {
    let parts = current
        .strip_prefix(pointer_folder)
        .ok()?
        .components()
        .map(|part| part.as_os_str().to_str())
        .collect::<Option<Vec<&str>>>()?;
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
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
            sub: None,
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
            written.contains(&format!(
                "cd '/srv/P-001' && DOSSIER_CLIENT_VERSION='{}' DOSSIER_CLIENT_FORMAT='1' 'dossier' 'list'",
                env!("CARGO_PKG_VERSION")
            )),
            "{written}"
        );

        // take and return --done copy files, so transfer runs them here: take asks the server for
        // the take with --json. The other returns are forwarded like any other command.
        let (code, written) =
            maybe_with(&args_file, &["dossier", "take", "a.sim", "--task", "T-1"]);
        assert_eq!(code, Some(7));
        assert!(
            written.contains("cd '/srv/P-001' && DOSSIER_CLIENT_VERSION="),
            "{written}"
        );
        assert!(
            written.contains("'dossier' 'take' 'a.sim' '--task' 'T-1' '--json'"),
            "{written}"
        );
        let (code, written) = maybe_with(&args_file, &["dossier", "return", "a.sim", "--done"]);
        assert_eq!(code, Some(7));
        assert!(
            written.contains("'dossier' 'return' 'a.sim' '--check'"),
            "{written}"
        );
        let (code, written) = maybe_with(&args_file, &["dossier", "return", "a.sim", "--check"]);
        assert_eq!(code, Some(7));
        assert!(
            written.contains("'dossier' 'return' 'a.sim' '--check'"),
            "{written}"
        );

        // Commands that stay on the PC, with a pointer present.
        let local: [&[&str]; 9] = [
            &["dossier", "connect", "anna@sim01:/srv/P-001"],
            &["dossier", "projects"],
            &["dossier", "help"],
            &["dossier", "hooks", "install"],
            &["dossier", "guard"],
            &["dossier", "skill", "install"],
            &["dossier", "after"],
            &["dossier", "session-end", "--run", "review-command"],
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
            written.contains("cd '/work/P-002' && DOSSIER_CLIENT_VERSION="),
            "{written}"
        );
        assert!(written.contains("'dossier' 'list'"), "{written}");
        assert!(!written.contains("--project"), "{written}");
        let (code, written) = maybe_with(&args_file, &["dossier", "list", "--project=p-002"]);
        assert_eq!(code, Some(7));
        assert!(
            written.contains("cd '/work/P-002' && DOSSIER_CLIENT_VERSION="),
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

        // take and return --done copy files and need a connected folder: --project refuses them,
        // even with a pointer in this folder. The other returns are forwarded.
        assert_eq!(
            maybe_with(
                &args_file,
                &[
                    "dossier",
                    "--project",
                    "P-002",
                    "take",
                    "a.sim",
                    "--task",
                    "T-1"
                ]
            ),
            (Some(2), String::new())
        );
        assert_eq!(
            maybe_with(
                &args_file,
                &["dossier", "--project", "P-002", "return", "a.sim", "--done"]
            ),
            (Some(2), String::new())
        );
        let (code, written) = maybe_with(
            &args_file,
            &[
                "dossier",
                "--project",
                "P-002",
                "return",
                "a.sim",
                "--check",
            ],
        );
        assert_eq!(code, Some(7));
        assert!(
            written.contains("cd '/work/P-002' && DOSSIER_CLIENT_VERSION="),
            "{written}"
        );

        // ssh cannot be run: the command fails with exit code 5.
        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::set_var("DOSSIER_SSH", root.join("no-such-ssh")) };
        assert_eq!(
            maybe_with(&args_file, &["dossier", "list"]),
            (Some(5), String::new())
        );
    }

    #[test]
    fn allow_credential_like_is_found_anywhere_in_the_arguments() {
        let args =
            |list: &[&str]| -> Vec<String> { list.iter().map(|word| word.to_string()).collect() };
        // The option after the command word, before it, and with other options around it.
        assert!(allows_credential(&args(&[
            "accept",
            "--allow-credential-like",
            "FND-001"
        ])));
        assert!(allows_credential(&args(&[
            "--agent",
            "anna-claude",
            "--allow-credential-like",
            "list"
        ])));
        assert!(allows_credential(&args(&["--allow-credential-like"])));
        // Other options, and a longer word that starts the same way, do not count.
        assert!(!allows_credential(&args(&["accept", "FND-001"])));
        assert!(!allows_credential(&args(&[
            "--allow-network-drive",
            "list"
        ])));
        assert!(!allows_credential(&args(&["--allow-credential", "list"])));
        assert!(!allows_credential(&args(&[
            "--allow-credential-like-x",
            "list"
        ])));
        assert!(!allows_credential(&args(&[])));
    }

    #[test]
    fn take_and_return_done_are_the_commands_that_copy_files() {
        let words =
            |list: &[&str]| -> Vec<String> { list.iter().map(|word| word.to_string()).collect() };
        assert!(copies_files("take", &words(&["a.sim", "--task", "T-1"])));
        assert!(copies_files("return", &words(&["a.sim", "--done"])));
        // The other returns, and the other commands, copy nothing.
        assert!(!copies_files("return", &words(&["a.sim", "--check"])));
        assert!(!copies_files("return", &words(&["a.sim", "--abandon"])));
        assert!(!copies_files("files", &words(&["a.sim", "--taken"])));
        assert!(!copies_files("list", &words(&[])));
    }

    #[test]
    fn sub_folder_is_the_current_folder_below_the_pointer_folder_with_slashes() {
        let pointer = Path::new("/srv/work/p1");
        assert_eq!(sub_folder(pointer, pointer), None);
        assert_eq!(
            sub_folder(&pointer.join("sim").join("run 1"), pointer),
            Some("sim/run 1".to_string())
        );
        assert_eq!(
            sub_folder(&pointer.join("Größe").join("x"), pointer),
            Some("Größe/x".to_string())
        );
        // A folder that is not below the pointer's folder gives no sub folder.
        assert_eq!(sub_folder(Path::new("/elsewhere"), pointer), None);
    }

    #[test]
    fn remote_variables_are_the_agent_and_the_versions_for_dossier_only() {
        let agent = remote_variables("dossier", Some("anna-claude"));
        assert_eq!(
            agent,
            vec![
                ("DOSSIER_AGENT".to_string(), "anna-claude".to_string()),
                (
                    "DOSSIER_CLIENT_VERSION".to_string(),
                    env!("CARGO_PKG_VERSION").to_string()
                ),
                ("DOSSIER_CLIENT_FORMAT".to_string(), "1".to_string()),
            ]
        );
        assert_eq!(
            remote_variables("dossier", None)
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            vec!["DOSSIER_CLIENT_VERSION", "DOSSIER_CLIENT_FORMAT"]
        );
        assert!(remote_variables("todo", None).is_empty());
        assert_eq!(
            remote_variables("todo", Some("anna-claude")),
            vec![("DOSSIER_AGENT".to_string(), "anna-claude".to_string())]
        );
    }
}
