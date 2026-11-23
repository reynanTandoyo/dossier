//! Runs the built program for `connect`: the pointer file and the register entry it writes, the
//! refusals of a bad target or code, and `--check`. Each test runs in its own temporary folder,
//! with the register in that folder too. DOSSIER_SSH points at a program that does not exist,
//! except in the `--check` tests, which point it at a stand-in ssh that answers for the server.
//! So no test runs a real ssh.

use assert_cmd::Command;
use predicates::str::contains;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// A temporary folder with the project folder inside it. The register lies beside it.
struct Sandbox {
    root: tempfile::TempDir,
}

impl Sandbox {
    fn new() -> Sandbox {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("project")).unwrap();
        Sandbox { root }
    }

    /// The project folder, where the program runs unless a test says otherwise.
    fn project(&self) -> PathBuf {
        self.root.path().join("project")
    }

    /// The register file, in a folder that does not exist until the program writes it.
    fn register(&self) -> PathBuf {
        self.root.path().join("register").join("projects.toml")
    }

    /// The program run in `folder`, with the register set and the agent names removed. The ssh
    /// program is one that does not exist.
    fn command_in(&self, folder: &Path) -> Command {
        let mut command = Command::cargo_bin("dossier").unwrap();
        command
            .current_dir(folder)
            .env("DOSSIER_REGISTER", self.register())
            .env("DOSSIER_SSH", self.root.path().join("no-such-ssh"))
            .env_remove("DOSSIER_AGENT")
            .env_remove("TODO_AGENT");
        command
    }

    /// `dossier <args>` run in the project folder, not yet checked.
    fn run(&self, args: &[&str]) -> assert_cmd::assert::Assert {
        self.command_in(&self.project()).args(args).assert()
    }

    /// The text of the register, if the file exists.
    fn register_text(&self) -> Option<String> {
        std::fs::read_to_string(self.register()).ok()
    }
}

/// The text of the pointer file in `folder`, if there is one.
fn pointer_text(folder: &Path) -> Option<String> {
    std::fs::read_to_string(folder.join(".dossier-connect")).ok()
}

#[test]
fn connect_with_a_code_writes_the_pointer_file_and_the_register_entry() {
    let sandbox = Sandbox::new();
    sandbox
        .run(&["connect", "anna@sim01:/projects/P-017", "--code", "P-017"])
        .success()
        .stdout(contains("connected P-017 -> anna@sim01:/projects/P-017"));

    assert_eq!(
        pointer_text(&sandbox.project()).unwrap(),
        "server = \"anna@sim01\"\nfolder = \"/projects/P-017\"\ncode = \"P-017\"\n"
    );
    let register = sandbox.register_text().unwrap();
    assert!(register.contains("code = \"P-017\""), "{register}");
    assert!(register.contains("server = \"anna@sim01\""), "{register}");
    assert!(
        register.contains("folder = \"/projects/P-017\""),
        "{register}"
    );
    assert!(register.contains("local = "), "{register}");
}

#[test]
fn json_output_gives_the_code_the_server_the_folder_and_the_local_folder() {
    let sandbox = Sandbox::new();
    sandbox
        .run(&[
            "connect",
            "anna@sim01:/projects/P-017",
            "--code",
            "P-017",
            "--json",
        ])
        .success()
        .stdout(contains("\"schema\":\"dossier.connect.v1\""))
        .stdout(contains("\"code\":\"P-017\""))
        .stdout(contains("\"server\":\"anna@sim01\""))
        .stdout(contains("\"folder\":\"/projects/P-017\""))
        .stdout(contains("\"local\":\""));
}

#[test]
fn a_target_without_a_colon_fails_with_exit_1_and_writes_nothing() {
    let sandbox = Sandbox::new();
    sandbox
        .run(&["connect", "anna@sim01", "--code", "P-017"])
        .code(1)
        .stderr(contains("error: the target is user@server:folder"));
    assert!(pointer_text(&sandbox.project()).is_none());
    assert!(sandbox.register_text().is_none());
}

#[test]
fn a_bad_target_as_json_names_bad_value() {
    let sandbox = Sandbox::new();
    sandbox
        .run(&["connect", "anna@sim01", "--code", "P-017", "--json"])
        .code(1)
        .stdout(contains("\"ok\":false"))
        .stdout(contains("\"code\":\"BAD_VALUE\""));
}

#[test]
fn a_bad_server_folder_or_code_is_refused_and_writes_nothing() {
    let sandbox = Sandbox::new();
    for target in [
        "anna@sim01:projects/P-017",
        "anna@sim01:",
        ":/projects/P-017",
        "anna sim01:/projects/P-017",
    ] {
        sandbox
            .run(&["connect", target, "--code", "P-017"])
            .code(1)
            .stderr(contains("error: "));
    }
    // A server that starts with '-' comes after `--`, or clap would read it as an option.
    sandbox
        .run(&[
            "connect",
            "--code",
            "P-017",
            "--",
            "-oProxyCommand=x@sim01:/projects/P-017",
        ])
        .code(1)
        .stderr(contains("error: the server is user@server"));
    for code in ["", "P 017"] {
        sandbox
            .run(&["connect", "anna@sim01:/projects/P-017", "--code", code])
            .code(1)
            .stderr(contains("a project code has only letters, digits and -"));
    }
    assert!(pointer_text(&sandbox.project()).is_none());
    assert!(sandbox.register_text().is_none());
}

#[test]
fn no_folder_writes_the_register_but_no_pointer_file() {
    let sandbox = Sandbox::new();
    sandbox
        .run(&[
            "connect",
            "anna@sim01:/projects/P-017",
            "--code",
            "P-017",
            "--no-folder",
        ])
        .success()
        .stdout(contains("connected P-017 -> anna@sim01:/projects/P-017"));

    assert!(!sandbox.project().join(".dossier-connect").exists());
    let register = sandbox.register_text().unwrap();
    assert!(register.contains("code = \"P-017\""), "{register}");
    assert!(register.contains("server = \"anna@sim01\""), "{register}");
    assert!(!register.contains("local = "), "{register}");
}

#[test]
fn a_folder_with_spaces_and_non_ascii_letters_gets_its_pointer_file() {
    let sandbox = Sandbox::new();
    let folder = sandbox
        .project()
        .join("Projekt Übersicht")
        .join("Größe P-017");
    std::fs::create_dir_all(&folder).unwrap();
    sandbox
        .command_in(&folder)
        .args([
            "connect",
            "anna@sim01:/projekte/Größe P-017",
            "--code",
            "P-017",
        ])
        .assert()
        .success();

    let pointer = pointer_text(&folder).unwrap();
    assert!(
        pointer.contains("folder = \"/projekte/Größe P-017\""),
        "{pointer}"
    );
    assert!(sandbox.register_text().unwrap().contains("Größe P-017"));
}

#[test]
fn connecting_the_same_code_again_replaces_the_entry() {
    let sandbox = Sandbox::new();
    sandbox
        .run(&["connect", "anna@sim01:/projects/P-017", "--code", "P-017"])
        .success();
    sandbox
        .run(&["connect", "anna@sim02:/work/P-017", "--code", "P-017"])
        .success();

    let register = sandbox.register_text().unwrap();
    assert_eq!(register.matches("[[project]]").count(), 1, "{register}");
    assert!(register.contains("server = \"anna@sim02\""), "{register}");
    assert!(!register.contains("sim01"), "{register}");
    assert!(
        pointer_text(&sandbox.project())
            .unwrap()
            .contains("server = \"anna@sim02\"")
    );
}

#[test]
fn without_a_code_and_without_ssh_the_command_ends_with_exit_5() {
    let sandbox = Sandbox::new();
    sandbox
        .run(&["connect", "anna@sim01:/projects/P-017"])
        .code(5)
        .stderr(contains(
            "could not read the project code from the server; give it with --code",
        ));
    assert!(pointer_text(&sandbox.project()).is_none());
    assert!(sandbox.register_text().is_none());
}

#[test]
fn check_in_a_folder_that_is_not_connected_says_how_to_connect() {
    let sandbox = Sandbox::new();
    sandbox
        .run(&["connect", "--check"])
        .code(1)
        .stderr(contains(
            "this folder is not connected; run dossier connect <user@server>:<folder>",
        ));
}

#[test]
fn check_of_a_connected_folder_whose_server_cannot_be_reached_ends_with_exit_5() {
    let sandbox = Sandbox::new();
    sandbox
        .run(&["connect", "anna@sim01:/projects/P-017", "--code", "P-017"])
        .success();
    sandbox
        .run(&["connect", "--check"])
        .code(5)
        .stderr(contains(
            "could not reach anna@sim01: ssh is not installed or not on PATH",
        ));
}

#[test]
fn connect_alone_is_a_usage_error_and_check_takes_no_target() {
    let sandbox = Sandbox::new();
    sandbox
        .run(&["connect"])
        .code(2)
        .stderr(contains("give the target as user@server:folder"));
    sandbox
        .run(&["connect", "--check", "anna@sim01:/projects/P-017"])
        .code(2)
        .stderr(contains("--check takes no target"));
    sandbox
        .run(&["connect", "--check", "--no-folder"])
        .code(2)
        .stderr(contains("--check takes no target"));
}

/// The version of this program: the stand-in answers `dossier --version` with it, so that the PC
/// and the server agree.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The answer of `dossier where --json` for project P-017 (invented).
const WHERE_P017: &str =
    r#"{"schema":"dossier.where.v1","ok":true,"data":{"code":"P-017"},"warnings":[]}"#;

/// The answer of `dossier where --json` for project P-018 (invented): another code.
const WHERE_P018: &str =
    r#"{"schema":"dossier.where.v1","ok":true,"data":{"code":"P-018"},"warnings":[]}"#;

/// The answer of `dossier where --json` in a folder without a dossier (invented), with exit 4.
const WHERE_NO_DOSSIER: &str = r#"{"schema":"dossier.where.v1","ok":false,"error":{"code":"NO_DOSSIER","message":"no dossier here"},"warnings":[]}"#;

/// What the stand-in ssh prints for one command, and the exit code it ends with.
#[derive(Clone)]
struct Reply {
    exit: u8,
    line: String,
}

impl Reply {
    fn new(exit: u8, line: &str) -> Reply {
        Reply {
            exit,
            line: line.to_string(),
        }
    }
}

/// The answers of the stand-in ssh to the three commands of `connect --check`.
#[derive(Clone)]
struct Answers {
    /// `dossier --version`
    dossier: Reply,
    /// `todo --version`
    todo: Reply,
    /// `dossier where --json`
    project: Reply,
}

impl Answers {
    /// A server with dossier of this build, todo 0.8.0 and project P-017 in the folder.
    fn all_well() -> Answers {
        Answers {
            dossier: Reply::new(0, &format!("dossier {VERSION}")),
            todo: Reply::new(0, "todo 0.8.0"),
            project: Reply::new(0, WHERE_P017),
        }
    }
}

/// The stand-in ssh on Windows, a .cmd file. The command that the server runs is its 6th argument.
/// The batch file echoes that argument in quotes, where cmd.exe reads its `&&` as text, and looks
/// for `'todo'` and `'where'` in it; a command with neither is `dossier --version`.
fn windows_text(answers: &Answers) -> String {
    let lines = [
        "@echo off".to_string(),
        "echo \"%~6\" | findstr /C:\"'todo'\" >nul && goto todo".to_string(),
        "echo \"%~6\" | findstr /C:\"'where'\" >nul && goto where".to_string(),
        format!("echo {}", answers.dossier.line),
        format!("exit /b {}", answers.dossier.exit),
        ":todo".to_string(),
        format!("echo {}", answers.todo.line),
        format!("exit /b {}", answers.todo.exit),
        ":where".to_string(),
        format!("echo {}", answers.project.line),
        format!("exit /b {}", answers.project.exit),
    ];
    lines.join("\r\n") + "\r\n"
}

/// The stand-in ssh on the other systems: a shell script that answers by its 6th argument, the
/// command that the server runs.
fn unix_text(answers: &Answers) -> String {
    let lines = [
        "#!/bin/sh".to_string(),
        "case \"$6\" in".to_string(),
        format!(
            "  *\"'todo'\"*) printf '%s\\n' '{}'; exit {} ;;",
            answers.todo.line, answers.todo.exit
        ),
        format!(
            "  *\"'where'\"*) printf '%s\\n' '{}'; exit {} ;;",
            answers.project.line, answers.project.exit
        ),
        format!(
            "  *) printf '%s\\n' '{}'; exit {} ;;",
            answers.dossier.line, answers.dossier.exit
        ),
        "esac".to_string(),
    ];
    lines.join("\n") + "\n"
}

/// The stand-in ssh that cannot sign in: it writes `Permission denied` to standard error and exits
/// with 255, as ssh does.
const REFUSED_WINDOWS: &str = "@echo off\r\necho Permission denied 1>&2\r\nexit /b 255\r\n";
const REFUSED_UNIX: &str = "#!/bin/sh\necho 'Permission denied' >&2\nexit 255\n";

/// Writes a stand-in ssh into the sandbox folder: the Windows text as ssh.cmd, the Unix text as
/// an executable script named ssh. Returns its path.
fn write_stand_in(sandbox: &Sandbox, windows: &str, unix: &str) -> PathBuf {
    if cfg!(windows) {
        let path = sandbox.root.path().join("ssh.cmd");
        std::fs::write(&path, windows).expect("the stand-in ssh is written");
        path
    } else {
        let path = sandbox.root.path().join("ssh");
        std::fs::write(&path, unix).expect("the stand-in ssh is written");
        make_executable(&path);
        path
    }
}

/// Writes the stand-in ssh that answers as `answers` says, and returns its path.
fn answers_stand_in(sandbox: &Sandbox, answers: &Answers) -> PathBuf {
    write_stand_in(sandbox, &windows_text(answers), &unix_text(answers))
}

/// Makes the Unix stand-in executable (mode 0o755).
#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .expect("the stand-in ssh is executable");
}

/// Windows runs a .cmd file as it is.
#[cfg(not(unix))]
fn make_executable(_path: &Path) {}

/// `dossier <args>` run in the project folder, with `ssh` as the ssh program.
fn run_with_ssh(sandbox: &Sandbox, ssh: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    sandbox
        .command_in(&sandbox.project())
        .env("DOSSIER_SSH", ssh)
        .args(args)
        .assert()
}

/// Connects the project folder to P-017 on anna@sim01. The code is given, so no ssh runs.
fn connect_p017(sandbox: &Sandbox) {
    sandbox
        .run(&["connect", "anna@sim01:/projects/P-017", "--code", "P-017"])
        .success();
}

#[test]
fn check_prints_four_lines_when_the_server_answers() {
    let sandbox = Sandbox::new();
    connect_p017(&sandbox);
    let ssh = answers_stand_in(&sandbox, &Answers::all_well());
    let stdout = run_with_ssh(&sandbox, &ssh, &["connect", "--check"])
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        String::from_utf8(stdout).unwrap(),
        format!(
            "ssh: ok\ndossier on the server: {VERSION}\ntodo on the server: 0.8.0\ndossier folder: ok (P-017)\n"
        )
    );
}

#[test]
fn check_as_json_gives_the_server_folder_code_and_both_versions() {
    let sandbox = Sandbox::new();
    connect_p017(&sandbox);
    let ssh = answers_stand_in(&sandbox, &Answers::all_well());
    let stdout = run_with_ssh(&sandbox, &ssh, &["connect", "--check", "--json"])
        .success()
        .get_output()
        .stdout
        .clone();
    let answer: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(answer["schema"], "dossier.connect.v1");
    assert_eq!(answer["ok"], true);
    assert_eq!(
        answer["data"],
        json!({
            "server": "anna@sim01",
            "folder": "/projects/P-017",
            "code": "P-017",
            "dossier": VERSION,
            "todo": "0.8.0",
        })
    );
}

#[test]
fn a_server_with_another_dossier_version_is_a_warning_and_the_check_passes() {
    let sandbox = Sandbox::new();
    connect_p017(&sandbox);
    let mut answers = Answers::all_well();
    answers.dossier = Reply::new(0, "dossier 9.9.9");
    let ssh = answers_stand_in(&sandbox, &answers);
    run_with_ssh(&sandbox, &ssh, &["connect", "--check"])
        .success()
        .stdout(contains("dossier on the server: 9.9.9"))
        .stderr(contains(format!(
            "warning: this PC has dossier {VERSION}, the server 9.9.9"
        )));
}

#[test]
fn a_server_that_answers_for_another_code_is_a_warning() {
    let sandbox = Sandbox::new();
    connect_p017(&sandbox);
    let mut answers = Answers::all_well();
    answers.project = Reply::new(0, WHERE_P018);
    let ssh = answers_stand_in(&sandbox, &answers);
    run_with_ssh(&sandbox, &ssh, &["connect", "--check"])
        .success()
        .stdout(contains("dossier folder: ok (P-018)"))
        .stderr(contains(
            "warning: this folder is connected to P-017, but the server answers for P-018",
        ));
}

#[test]
fn a_server_that_refuses_the_sign_in_ends_with_exit_5_and_what_ssh_said() {
    let sandbox = Sandbox::new();
    connect_p017(&sandbox);
    let ssh = write_stand_in(&sandbox, REFUSED_WINDOWS, REFUSED_UNIX);
    run_with_ssh(&sandbox, &ssh, &["connect", "--check"])
        .code(5)
        .stderr(contains(
            "error: could not reach anna@sim01: Permission denied; sign in once by hand with ssh anna@sim01",
        ));
}

#[test]
fn a_server_without_dossier_on_its_path_ends_with_exit_5() {
    let sandbox = Sandbox::new();
    connect_p017(&sandbox);
    let mut answers = Answers::all_well();
    answers.dossier = Reply::new(1, "dossier: command not found");
    let ssh = answers_stand_in(&sandbox, &answers);
    run_with_ssh(&sandbox, &ssh, &["connect", "--check"])
        .code(5)
        .stderr(contains(
            "error: dossier is not installed on anna@sim01, or not on its PATH",
        ));
}

#[test]
fn a_server_without_todo_on_its_path_ends_with_exit_5_and_names_todo() {
    let sandbox = Sandbox::new();
    connect_p017(&sandbox);
    let mut answers = Answers::all_well();
    answers.todo = Reply::new(127, "todo: command not found");
    let ssh = answers_stand_in(&sandbox, &answers);
    run_with_ssh(&sandbox, &ssh, &["connect", "--check"])
        .code(5)
        .stderr(contains(
            "error: todo is not installed on anna@sim01, or not on its PATH",
        ));
}

#[test]
fn a_folder_without_a_dossier_on_the_server_ends_with_exit_4() {
    let sandbox = Sandbox::new();
    connect_p017(&sandbox);
    let mut answers = Answers::all_well();
    answers.project = Reply::new(4, WHERE_NO_DOSSIER);
    let ssh = answers_stand_in(&sandbox, &answers);
    run_with_ssh(&sandbox, &ssh, &["connect", "--check"])
        .code(4)
        .stderr(contains(
            "error: /projects/P-017 on anna@sim01 has no dossier; run dossier init there",
        ));
}
