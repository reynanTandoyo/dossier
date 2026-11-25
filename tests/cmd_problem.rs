//! Runs the built program for `problem`: an investigation by a person and by an LLM is approved at
//! once with state `open`, its Problem section holds the problem and the observation, `--task` is
//! upper-cased, the similar investigations are named, a line break or a heading is refused with
//! nothing written, a missing investigations folder is made again, and a team dossier refuses.
//! Each test runs in its own temporary project with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use dossier::entry::{Entry, Kind, Status};
use predicates::str::contains;
use std::fs;

/// The clock of every run in these tests: the clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The text of the entry file with `id`.
fn entry_text(project: &Project, id: &str) -> String {
    fs::read_to_string(project.entry_file(id)).expect("the entry file")
}

/// The entry with `id`, parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    Entry::parse(&entry_text(project, id)).expect("the entry parses")
}

/// Whether the investigations folder of the project has no file in it.
fn no_investigation_written(project: &Project) -> bool {
    fs::read_dir(project.root().join("DOSSIER").join("investigations"))
        .expect("the investigations folder")
        .next()
        .is_none()
}

#[test]
fn anna_claude_opens_an_investigation_approved_with_state_open() {
    let project = Project::new();
    let stdout = project.stdout(
        "anna-claude",
        &[
            "problem",
            "The density residual grows from iteration 40 on.",
        ],
    );
    assert_eq!(stdout, "INV-001: added (open)\n");

    let entry = entry_of(&project, "INV-001");
    assert_eq!(entry.kind(), Some(Kind::Investigation));
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("state").as_deref(), Some("open"));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
    assert_eq!(
        entry.field("approved").as_deref(),
        Some("2026-11-20T10:00:00+01:00 by anna")
    );
    assert_eq!(entry.field("task"), None);
    assert_eq!(
        entry.title(),
        "The density residual grows from iteration 40 on."
    );
    assert_eq!(
        entry.section("Problem").as_deref(),
        Some("The density residual grows from iteration 40 on.")
    );
    assert_eq!(entry.section("Attempts"), None);
    assert_eq!(entry.section("Solution"), None);
    assert_eq!(
        entry.history(),
        [
            "2026-11-20T10:00:00+01:00 anna via anna-claude: created",
            "2026-11-20T10:00:00+01:00 anna via anna-claude: approved",
        ]
    );
    assert!(
        project
            .entry_file("INV-001")
            .starts_with(project.root().join("DOSSIER").join("investigations")),
        "the investigation is in the investigations folder"
    );
}

#[test]
fn a_person_opens_an_investigation_whose_problem_section_holds_the_observation() {
    let project = Project::new();
    let stdout = project.stdout(
        "anna",
        &[
            "problem",
            "The run stops with a negative pressure.",
            "--observed",
            "  Iteration 412 of run 13.  ",
        ],
    );
    assert_eq!(stdout, "INV-001: added (open)\n");

    let entry = entry_of(&project, "INV-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via"), None);
    assert_eq!(
        entry.section("Problem").as_deref(),
        Some("The run stops with a negative pressure.\n\nObserved: Iteration 412 of run 13.")
    );
}

#[test]
fn task_is_upper_cased_and_the_tag_options_are_written_blank_ones_left_out() {
    let project = Project::new();
    project.stdout(
        "anna-claude",
        &[
            "problem",
            "The outlet flux is wrong.",
            "--task",
            "t-12",
            "--machine",
            "  axial compressor  ",
            "--analysis",
            "steady",
            "--software",
            "   ",
            "--tags",
            "  monitor ",
        ],
    );

    let entry = entry_of(&project, "INV-001");
    assert_eq!(entry.field("task").as_deref(), Some("T-12"));
    assert_eq!(entry.field("machine").as_deref(), Some("axial compressor"));
    assert_eq!(entry.field("analysis").as_deref(), Some("steady"));
    assert_eq!(entry.field("software"), None);
    assert_eq!(entry.field("tags").as_deref(), Some("monitor"));
    assert_eq!(entry.field("state").as_deref(), Some("open"));
}

#[test]
fn the_title_is_the_title_given_or_the_first_80_characters_of_the_problem() {
    let project = Project::new();
    let problem = "Ö".repeat(90);
    project.stdout("anna", &["problem", &problem]);
    project.stdout(
        "anna",
        &[
            "problem",
            "A second problem.",
            "--title",
            "  Outlet   flux  ",
        ],
    );

    assert_eq!(entry_of(&project, "INV-001").title(), "Ö".repeat(80));
    assert_eq!(entry_of(&project, "INV-002").title(), "Outlet   flux");
}

#[test]
fn a_problem_like_an_earlier_one_names_it_as_similar() {
    let project = Project::new();
    project.stdout(
        "anna",
        &[
            "problem",
            "The density residual grows from iteration 40 on.",
            "--tags",
            "solver",
        ],
    );
    let stdout = project.stdout(
        "anna",
        &[
            "problem",
            "The density residual grows at iteration 40 again.",
            "--tags",
            "solver",
        ],
    );
    assert_eq!(
        stdout,
        "INV-002: added (open)\nsimilar: INV-001 [open] The density residual grows from iteration 40 on.\n"
    );
}

#[test]
fn a_problem_unlike_the_earlier_ones_names_no_similar_investigation() {
    let project = Project::new();
    project.stdout(
        "anna",
        &[
            "problem",
            "The density residual grows from iteration 40 on.",
        ],
    );
    let stdout = project.stdout("anna", &["problem", "Mesh export fails on Windows."]);
    assert_eq!(stdout, "INV-002: added (open)\n");
}

#[test]
fn a_problem_with_other_forms_of_the_words_of_an_earlier_one_names_it_as_similar() {
    let project = Project::new();
    project.stdout("anna", &["problem", "Transfers fail for paths with spaces"]);
    let stdout = project.stdout(
        "anna",
        &["problem", "A transfer failed for a path with a space"],
    );
    assert_eq!(
        stdout,
        "INV-002: added (open)\nsimilar: INV-001 [open] Transfers fail for paths with spaces\n"
    );
}

#[test]
fn an_earlier_problem_that_shares_only_common_words_is_not_named() {
    let project = Project::new();
    project.stdout("anna", &["problem", "The log for the run with the solver"]);
    let stdout = project.stdout(
        "anna",
        &["problem", "The transfer failed for a path with a space"],
    );
    assert_eq!(stdout, "INV-002: added (open)\n");
}

#[test]
fn json_gives_the_id_the_state_and_the_similar_ids() {
    let project = Project::new();
    // The keys of the JSON data are in alphabetical order, so each part is checked on its own.
    project
        .run("anna", &["problem", "A first problem.", "--json"])
        .success()
        .stdout(contains("\"schema\":\"dossier.problem.v1\""))
        .stdout(contains("\"ok\":true"))
        .stdout(contains(
            "\"data\":{\"id\":\"INV-001\",\"similar\":[],\"state\":\"open\"}",
        ));

    project
        .run("anna", &["problem", "A first problem again.", "--json"])
        .success()
        .stdout(contains("\"similar\":[\"INV-001\"]"))
        .stdout(contains("\"id\":\"INV-002\""));
}

#[test]
fn the_write_is_in_the_journal_with_who_and_when() {
    let project = Project::new();
    project.stdout("anna-claude", &["problem", "A problem."]);

    // The settings event of init is not about an entry, so it is not among these records.
    let records = project.entry_journal();
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.time, NOW);
    assert_eq!(record.name, "anna-claude");
    assert_eq!(record.person, "anna");
    assert!(record.llm);
    assert_eq!(record.command, "problem");
    assert_eq!(record.id, "INV-001");
    assert_eq!(record.event, "created");
    assert_eq!(record.detail, "approved");
}

#[test]
fn a_problem_on_two_lines_is_refused_and_nothing_is_written() {
    let project = Project::new();
    project
        .run("anna", &["problem", "first line\nsecond line"])
        .code(1)
        .stderr(contains("the problem must be one line"));
    project
        .run(
            "anna",
            &["problem", "A problem.", "--observed", "first\r\nsecond"],
        )
        .code(1)
        .stderr(contains("--observed must be one line"));
    assert!(no_investigation_written(&project));
}

#[test]
fn a_problem_or_observation_that_starts_with_a_heading_mark_is_refused() {
    let project = Project::new();
    project
        .run("anna", &["problem", "## Not a heading"])
        .code(1)
        .stderr(contains("must not start with the heading mark ##"));
    project
        .run(
            "anna",
            &["problem", "A problem.", "--observed", "## Not either"],
        )
        .code(1)
        .stderr(contains(
            "--observed must not start with the heading mark ##",
        ));
    assert!(no_investigation_written(&project));
}

#[test]
fn a_blank_problem_is_refused_and_nothing_is_written() {
    let project = Project::new();
    project
        .run("anna", &["problem", "   "])
        .code(1)
        .stderr(contains("the problem must not be empty"));
    assert!(no_investigation_written(&project));
}

#[test]
fn a_dossier_without_an_investigations_folder_still_accepts_the_command() {
    let project = Project::new();
    fs::remove_dir_all(project.root().join("DOSSIER").join("investigations"))
        .expect("the investigations folder of init");

    let stdout = project.stdout("anna", &["problem", "A problem with no folder yet."]);
    assert_eq!(stdout, "INV-001: added (open)\n");
    assert_eq!(
        entry_of(&project, "INV-001").status(),
        Some(Status::Approved)
    );
}

#[test]
fn a_team_dossier_refuses_and_names_promote() {
    let team = Project {
        dir: tempfile::TempDir::new().expect("a temporary folder"),
    };
    team.command("anna", NOW)
        .args(["init", "--code", "P-001", "--team-dossier"])
        .assert()
        .success();

    team.run("anna", &["problem", "A problem."])
        .code(1)
        .stderr(contains("investigations belong to a project"))
        .stderr(contains("dossier promote <ID> --as howto"));
    assert!(
        !team.root().join("DOSSIER").join("investigations").exists(),
        "a refused team write creates no folder"
    );
}
