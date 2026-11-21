//! Runs the built program for `found`: a finding by a person and by an LLM is approved at once,
//! `--dead-end` writes its outcome, `--later` makes a draft with the tag inbox, and the title,
//! the IDs, the refusals and the dry run follow the rules. Each test runs in its own temporary
//! project with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use dossier::entry::{Entry, Kind, Status};
use dossier::journal;
use predicates::str::contains;
use std::fs;

/// The text of the entry file with `id`.
fn entry_text(project: &Project, id: &str) -> String {
    fs::read_to_string(project.entry_file(id)).expect("the entry file")
}

/// The entry with `id`, parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    Entry::parse(&entry_text(project, id)).expect("the entry parses")
}

#[test]
fn anna_claude_records_a_finding_that_is_approved_with_via() {
    let project = Project::new();
    let stdout = project.stdout(
        "anna-claude",
        &[
            "found",
            "The mass flow still oscillated by 0.4 %.",
            "--evidence",
            "Runs 12 and 13, monitor plots.",
            "--task",
            "t-042",
            "--machine",
            "  axial compressor  ",
            "--tags",
            "monitor",
        ],
    );
    assert_eq!(stdout, "FND-001: added (approved)\n");

    let entry = entry_of(&project, "FND-001");
    assert_eq!(entry.kind(), Some(Kind::Finding));
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
    assert_eq!(
        entry.field("approved").as_deref(),
        Some("2026-11-20T10:00:00+01:00 by anna")
    );
    assert_eq!(
        entry.field("valid-from").as_deref(),
        Some("2026-11-20T10:00:00+01:00")
    );
    assert_eq!(entry.field("task").as_deref(), Some("T-042"));
    assert_eq!(entry.field("machine").as_deref(), Some("axial compressor"));
    assert_eq!(entry.field("tags").as_deref(), Some("monitor"));
    assert_eq!(entry.field("outcome"), None);
    assert_eq!(entry.title(), "The mass flow still oscillated by 0.4 %.");
    assert_eq!(
        entry.section("Observation").as_deref(),
        Some("The mass flow still oscillated by 0.4 %.")
    );
    assert_eq!(
        entry.section("Evidence").as_deref(),
        Some("Runs 12 and 13, monitor plots.")
    );
    assert_eq!(
        entry.history(),
        [
            "2026-11-20T10:00:00+01:00 anna via anna-claude: created",
            "2026-11-20T10:00:00+01:00 anna via anna-claude: approved",
        ]
    );
}

#[test]
fn a_person_records_a_finding_approved_with_no_via() {
    let project = Project::new();
    let stdout = project.stdout("anna", &["found", "The inlet profile is uniform."]);
    assert_eq!(stdout, "FND-001: added (approved)\n");

    let entry = entry_of(&project, "FND-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via"), None);
    assert_eq!(entry.section("Evidence"), None);
    assert_eq!(
        entry.history(),
        [
            "2026-11-20T10:00:00+01:00 anna: created",
            "2026-11-20T10:00:00+01:00 anna: approved",
        ]
    );
}

#[test]
fn dead_end_writes_outcome_dead_end() {
    let project = Project::new();
    let stdout = project.stdout(
        "anna-claude",
        &[
            "found",
            "--dead-end",
            "Halving the time step did not remove the oscillation.",
        ],
    );
    assert_eq!(stdout, "FND-001: added (approved)\n");

    let text = entry_text(&project, "FND-001");
    assert!(text.contains("\n- outcome: dead end\n"), "{text}");
    let entry = Entry::parse(&text).expect("the entry parses");
    assert_eq!(entry.field("outcome").as_deref(), Some("dead end"));
    assert_eq!(entry.status(), Some(Status::Approved));
}

#[test]
fn later_gives_a_draft_with_the_inbox_tag_added_to_the_tags_given() {
    let project = Project::new();
    let stdout = project.stdout(
        "anna-claude",
        &[
            "found",
            "--later",
            "Check the monitor plots again",
            "--tags",
            "monitor",
        ],
    );
    assert_eq!(stdout, "FND-001: added (draft, sort it out later)\n");

    let entry = entry_of(&project, "FND-001");
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(entry.field("tags").as_deref(), Some("monitor, inbox"));
    assert_eq!(entry.field("approved"), None);
    assert_eq!(entry.field("valid-from"), None);
    assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
    assert_eq!(
        entry.history(),
        ["2026-11-20T10:00:00+01:00 anna via anna-claude: created"]
    );
}

#[test]
fn later_without_tags_gives_the_inbox_tag_alone() {
    let project = Project::new();
    project.stdout("anna", &["found", "--later", "A thought for later"]);

    let entry = entry_of(&project, "FND-001");
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(entry.field("tags").as_deref(), Some("inbox"));
}

#[test]
fn the_default_title_is_the_first_80_characters_with_line_breaks_as_spaces() {
    let project = Project::new();
    let observation = format!("{}\nand a second line", "Ö".repeat(90));
    project.stdout("anna", &["found", &observation]);

    let entry = entry_of(&project, "FND-001");
    assert_eq!(entry.title(), "Ö".repeat(80));
    assert_eq!(entry.section("Observation"), Some(observation));
}

#[test]
fn a_title_with_a_line_break_is_written_on_one_line() {
    let project = Project::new();
    project.stdout(
        "anna",
        &["found", "First line of a note\r\nsecond line that is short"],
    );

    let entry = entry_of(&project, "FND-001");
    assert_eq!(
        entry.title(),
        "First line of a note second line that is short"
    );
}

#[test]
fn title_gives_the_title_and_a_blank_title_gives_the_default() {
    let project = Project::new();
    project.stdout(
        "anna",
        &["found", "--title", "Mass flow", "The mass flow oscillates."],
    );
    project.stdout("anna", &["found", "--title", "   ", "A second finding."]);

    assert_eq!(entry_of(&project, "FND-001").title(), "Mass flow");
    assert_eq!(entry_of(&project, "FND-002").title(), "A second finding.");
}

#[test]
fn task_is_upper_cased_and_blank_tag_values_are_left_out() {
    let project = Project::new();
    project.stdout(
        "anna",
        &[
            "found",
            "A finding.",
            "--task",
            "t-042",
            "--analysis",
            "   ",
            "--software",
            "STAR-CCM+",
        ],
    );

    let entry = entry_of(&project, "FND-001");
    assert_eq!(entry.field("task").as_deref(), Some("T-042"));
    assert_eq!(entry.field("analysis"), None);
    assert_eq!(entry.field("software").as_deref(), Some("STAR-CCM+"));
    assert_eq!(entry.field("tags"), None);
}

#[test]
fn ids_count_up_within_the_findings() {
    let project = Project::new();
    assert_eq!(
        project.stdout("anna", &["found", "First."]),
        "FND-001: added (approved)\n"
    );
    assert_eq!(
        project.stdout("anna", &["found", "Second."]),
        "FND-002: added (approved)\n"
    );
}

#[test]
fn json_gives_the_id_and_the_status() {
    let project = Project::new();
    // The keys of the JSON line are in alphabetical order, so each part is checked on its own.
    project
        .run("anna", &["found", "A finding.", "--json"])
        .success()
        .stdout(contains("\"schema\":\"dossier.found.v1\""))
        .stdout(contains("\"ok\":true"))
        .stdout(contains(
            "\"data\":{\"id\":\"FND-001\",\"status\":\"approved\"}",
        ))
        .stdout(contains("\"warnings\":[]"));

    project
        .run("anna", &["found", "--later", "A note", "--json"])
        .success()
        .stdout(contains("\"id\":\"FND-002\",\"status\":\"draft\""));
}

#[test]
fn the_write_is_in_the_journal_with_who_and_when() {
    let project = Project::new();
    project.stdout("anna-claude", &["found", "A finding."]);

    let state = project.root().join("DOSSIER").join(".state");
    let records = journal::read(&state).expect("the journal");
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.time, "2026-11-20T10:00:00+01:00");
    assert_eq!(record.name, "anna-claude");
    assert_eq!(record.person, "anna");
    assert!(record.llm);
    assert_eq!(record.command, "found");
    assert_eq!(record.id, "FND-001");
    assert_eq!(record.event, "created");
    assert_eq!(record.detail, "approved");
}

#[test]
fn a_finding_needs_a_name() {
    let project = Project::new();
    project
        .run("", &["found", "Something seen"])
        .code(2)
        .stderr(contains("this command needs a name"));
    assert!(
        !project
            .root()
            .join("DOSSIER")
            .join(".state")
            .join("ids.json")
            .exists()
    );
}

#[test]
fn a_name_that_is_not_in_the_settings_is_refused() {
    let project = Project::new();
    project
        .run("carl", &["found", "Something seen"])
        .code(1)
        .stderr(contains("carl"));
}

#[test]
fn a_blank_observation_is_refused_and_nothing_is_written() {
    let project = Project::new();
    project
        .run("anna", &["found", "   "])
        .code(1)
        .stderr(contains("the observation must not be empty"));
    project
        .run("anna", &["found", "--later", ""])
        .code(1)
        .stderr(contains("the observation must not be empty"));
    assert!(
        fs::read_dir(project.root().join("DOSSIER").join("findings"))
            .expect("the findings folder")
            .next()
            .is_none()
    );
}

#[test]
fn a_dry_run_names_the_file_and_writes_nothing() {
    let project = Project::new();
    project
        .run("anna", &["--dry-run", "found", "A finding."])
        .success()
        .stdout("FND-NEW: added (approved)\n")
        .stderr(contains("dry run: would write DOSSIER/findings/FND-NEW-"));
    assert!(
        fs::read_dir(project.root().join("DOSSIER").join("findings"))
            .expect("the findings folder")
            .next()
            .is_none()
    );
    assert!(
        !project
            .root()
            .join("DOSSIER")
            .join(".state")
            .join("ids.json")
            .exists()
    );
}
