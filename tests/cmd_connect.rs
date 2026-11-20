//! Runs the built program for `connect`: the pointer file and the register entry it writes, the
//! refusals of a bad target or code, and `--check`. Each test runs in its own temporary folder,
//! with the register in that folder too. DOSSIER_SSH always points at a program that does not
//! exist, so no test runs a real ssh.

use assert_cmd::Command;
use predicates::str::contains;
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
        .stderr(contains("could not read the project code from the server"));
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
