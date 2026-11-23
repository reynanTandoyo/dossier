//! Runs the built program for `take` and `return` in a connected folder (R-REMOTE-4, R-FILE-1 to
//! R-FILE-4): the server's answers come from an ssh stand-in and the copies from an scp stand-in,
//! so no test runs a real ssh or scp. Each stand-in appends its arguments to a file in the project
//! folder (ssh-calls.txt, scp-calls.txt), so a test can count the calls and read their arguments.
//! The register lies in each test's temporary folder (DOSSIER_REGISTER), and the clock is fixed.

mod common;

use common::Project;
use predicates::str::contains;

/// The server and folder that the connected project folder points at.
const TARGET: &str = "anna@server:/projects/p1";

/// The file in the project folder where the ssh stand-in writes the arguments of each call.
const SSH_CALLS: &str = "ssh-calls.txt";

/// The file in the project folder where the scp stand-in writes the arguments of each call.
const SCP_CALLS: &str = "scp-calls.txt";

/// The answer of `dossier take --json` for sim/a.sim, as the ssh stand-in prints it.
const TAKE_ANSWER: &str = r#"{"schema":"dossier.take.v1","ok":true,"data":{"task":"T-1","files":[{"path":"sim/a.sim"}]},"warnings":[]}"#;

/// The text of a stand-in program for Windows and for the other systems.
type Texts = (String, String);

/// The ssh stand-in for `take`: it records its arguments, prints TAKE_ANSWER and exits 0.
fn ssh_answers_take() -> Texts {
    (
        format!("@echo off\r\necho %* >> {SSH_CALLS}\r\necho {TAKE_ANSWER}\r\nexit /b 0\r\n"),
        format!("#!/bin/sh\necho \"$*\" >> {SSH_CALLS}\nprintf '%s\\n' '{TAKE_ANSWER}'\nexit 0\n"),
    )
}

/// A stand-in that records its arguments in `calls`, prints nothing and exits with `code`.
fn records_and_exits(calls: &str, code: u8) -> Texts {
    (
        format!("@echo off\r\necho %* >> {calls}\r\nexit /b {code}\r\n"),
        format!("#!/bin/sh\necho \"$*\" >> {calls}\nexit {code}\n"),
    )
}

/// The scp stand-in that copies: it records its arguments, writes `copied` to its last argument
/// (the target) and exits 0.
fn scp_copies() -> Texts {
    (
        format!(
            "@echo off\r\n\
             echo %* >> {SCP_CALLS}\r\n\
             set \"last=\"\r\n\
             :next\r\n\
             if \"%~1\"==\"\" goto write\r\n\
             set \"last=%~1\"\r\n\
             shift\r\n\
             goto next\r\n\
             :write\r\n\
             <nul set /p \"=copied\" > \"%last%\"\r\n\
             exit /b 0\r\n"
        ),
        format!(
            "#!/bin/sh\necho \"$*\" >> {SCP_CALLS}\nfor last; do :; done\nprintf copied > \"$last\"\nexit 0\n"
        ),
    )
}

/// A project folder with its folder connected to TARGET with the code P-001, and the stand-ins
/// ssh and scp written from `ssh` and `scp`. `connect --code` runs no ssh.
fn connected(ssh: Texts, scp: Texts) -> Project {
    let project = Project::new();
    project.write_stand_in("ssh", &ssh.0, &ssh.1);
    project.write_stand_in("scp", &scp.0, &scp.1);
    project
        .run("", &["connect", "--code", "P-001", TARGET])
        .success();
    project
}

/// The lines that a stand-in wrote to `calls`, in order. Rust puts quotes around an argument of a
/// .cmd file and echo leaves a trailing space; neither is part of the arguments, so both are dropped
/// before the comparison (as in cmd_projects.rs).
fn call_lines(project: &Project, calls: &str) -> Vec<String> {
    project
        .read(calls)
        .lines()
        .map(|line| line.replace('"', "").trim().to_string())
        .collect()
}

#[test]
fn take_asks_the_server_and_copies_the_file_through_a_part_file() {
    let project = connected(ssh_answers_take(), scp_copies());
    project
        .run("", &["take", "sim/a.sim", "--task", "T-1"])
        .success()
        .stdout(contains("sim/a.sim: taken and copied"));

    let ssh = call_lines(&project, SSH_CALLS);
    assert_eq!(ssh.len(), 1, "{ssh:?}");
    assert!(ssh[0].contains("take"), "{}", ssh[0]);
    assert!(ssh[0].contains("DOSSIER_CLIENT_VERSION"), "{}", ssh[0]);

    let scp = call_lines(&project, SCP_CALLS);
    assert_eq!(scp.len(), 1, "{scp:?}");
    let line = &scp[0];
    assert!(line.starts_with("-s "), "{line}");
    assert!(
        line.contains("anna@server:") && line.contains("/projects/p1/sim/a.sim"),
        "{line}"
    );
    assert!(line.ends_with("a.sim.dossier-part"), "{line}");

    assert_eq!(project.read("sim/a.sim"), "copied");
    assert!(
        !project
            .root()
            .join("sim")
            .join("a.sim.dossier-part")
            .exists()
    );
}

#[test]
fn take_keeps_a_local_file_that_exists_and_runs_no_scp() {
    let project = connected(ssh_answers_take(), scp_copies());
    project.write("sim/a.sim", "mine");
    project
        .run("", &["take", "sim/a.sim", "--task", "T-1"])
        .success()
        .stdout(contains("kept your local copy"));

    assert!(!project.root().join(SCP_CALLS).exists());
    assert_eq!(project.read("sim/a.sim"), "mine");
}

#[test]
fn take_with_a_failed_copy_exits_5_and_leaves_no_file() {
    let project = connected(ssh_answers_take(), records_and_exits(SCP_CALLS, 1));
    project
        .run("", &["take", "sim/a.sim", "--task", "T-1"])
        .code(5)
        .stderr(contains("run the same command again"));

    assert_eq!(call_lines(&project, SCP_CALLS).len(), 1);
    assert!(!project.root().join("sim").join("a.sim").exists());
    assert!(
        !project
            .root()
            .join("sim")
            .join("a.sim.dossier-part")
            .exists()
    );
}

#[test]
fn return_done_stops_when_the_check_fails_and_copies_nothing() {
    let project = connected(records_and_exits(SSH_CALLS, 1), scp_copies());
    project.write("sim/a.sim", "my edits");
    project.run("", &["return", "sim/a.sim", "--done"]).code(1);

    let ssh = call_lines(&project, SSH_CALLS);
    assert_eq!(ssh.len(), 1, "{ssh:?}");
    assert!(ssh[0].contains("--check"), "{}", ssh[0]);
    assert!(!ssh[0].contains("--done"), "{}", ssh[0]);
    assert!(!project.root().join(SCP_CALLS).exists());
}

#[test]
fn return_check_calls_ssh_once_and_never_scp() {
    let project = connected(records_and_exits(SSH_CALLS, 0), scp_copies());
    project.write("sim/a.sim", "my edits");
    project
        .run("", &["return", "sim/a.sim", "--check"])
        .success();

    let ssh = call_lines(&project, SSH_CALLS);
    assert_eq!(ssh.len(), 1, "{ssh:?}");
    assert!(ssh[0].contains("--check"), "{}", ssh[0]);
    assert!(!project.root().join(SCP_CALLS).exists());
}
