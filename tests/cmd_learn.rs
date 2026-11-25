//! Runs the built program for `learn`: a lesson a person writes is approved, one an LLM writes is
//! a draft with the person as author, and the confidence is checked. Each test runs in its own
//! project folder, with the clock fixed by `common::Project`.

mod common;

use common::Project;
use dossier::entry::{Entry, Status};
use predicates::str::contains;
use std::path::Path;

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for the parts of a lesson.
const STATEMENT: &str = "Steady runs with a mixing-plane interface overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";

/// `learn` with a complete lesson, followed by `extra` arguments.
fn learn<'a>(extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "learn",
        STATEMENT,
        "--valid-for",
        VALID_FOR,
        "--evidence",
        EVIDENCE,
    ];
    args.extend_from_slice(extra);
    args
}

/// The entry file with `id`, read and parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    let text = std::fs::read_to_string(project.entry_file(id)).expect("the entry file is read");
    Entry::parse(&text).expect("the entry parses")
}

/// Whether a folder of the project has no files in it.
fn folder_is_empty(folder: &Path) -> bool {
    std::fs::read_dir(folder)
        .expect("the folder is read")
        .next()
        .is_none()
}

#[test]
fn a_persons_lesson_is_approved_and_recorded_for_its_checksum() {
    let project = Project::new();
    project
        .run("anna", &learn(&["--confidence", "simulated"]))
        .success()
        .stdout("LES-001: added (approved)\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via"), None);
    assert_eq!(entry.field("confidence").as_deref(), Some("simulated"));
    assert_eq!(
        entry.field("approved").as_deref(),
        Some(format!("{NOW} by anna").as_str())
    );
    assert_eq!(entry.field("valid-from").as_deref(), Some(NOW));
    assert_eq!(entry.section("Statement").as_deref(), Some(STATEMENT));
    assert_eq!(entry.section("Valid for").as_deref(), Some(VALID_FOR));
    assert_eq!(entry.section("Evidence").as_deref(), Some(EVIDENCE));
    assert_eq!(
        entry.history(),
        [
            format!("{NOW} anna: created"),
            format!("{NOW} anna: approved"),
        ]
    );
    // An approved lesson has its checksum recorded (R-LIFE-6).
    assert!(
        project
            .read("DOSSIER/.state/approved.json")
            .contains("LES-001")
    );
}

#[test]
fn an_llms_lesson_is_a_draft_with_the_person_as_author_and_the_llm_as_via() {
    let project = Project::new();
    project
        .run("anna-claude", &learn(&["--confidence", "guess"]))
        .success()
        .stdout("LES-001: added (draft, waiting for approval)\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
    assert_eq!(entry.field("approved"), None);
    assert_eq!(entry.field("valid-from"), None);
    assert_eq!(
        entry.history(),
        [format!("{NOW} anna via anna-claude: created")]
    );
    // A draft is not approved, so nothing is recorded for its checksum.
    assert!(!project.root().join("DOSSIER/.state/approved.json").exists());
}

#[test]
fn a_wrong_confidence_is_refused_and_nothing_is_written() {
    let project = Project::new();
    project
        .run("anna", &learn(&["--confidence", "sure"]))
        .code(1)
        .stderr(contains(
            "error: confidence must be measured, simulated, experience or guess",
        ));
    project
        .run("anna", &learn(&["--confidence", "sure", "--json"]))
        .code(1)
        .stdout(contains("\"code\":\"BAD_VALUE\""));

    assert!(folder_is_empty(&project.root().join("DOSSIER/lessons")));
    assert!(!project.root().join("DOSSIER/.state/ids.json").exists());
}

#[test]
fn the_confidence_is_accepted_in_any_case_and_kept_in_lower_case() {
    let project = Project::new();
    project
        .run("anna", &learn(&["--confidence", "Measured"]))
        .success();
    assert_eq!(
        entry_of(&project, "LES-001").field("confidence").as_deref(),
        Some("measured")
    );
}

#[test]
fn json_output_gives_the_id_and_the_status() {
    let project = Project::new();
    let out = project.stdout(
        "anna-claude",
        &learn(&["--confidence", "experience", "--json"]),
    );
    let value: serde_json::Value = serde_json::from_str(out.trim()).expect("one JSON line");
    assert_eq!(value["schema"], "dossier.learn.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"],
        serde_json::json!({"id": "LES-001", "status": "draft"})
    );
}

#[test]
fn the_default_title_is_the_first_80_characters_with_line_breaks_as_spaces() {
    let project = Project::new();
    // 50 accented letters, a line break, then 60 sharp s letters: 111 characters in one line.
    let statement = format!("{}\n{}", "é".repeat(50), "ß".repeat(60));
    let args = [
        "learn",
        statement.as_str(),
        "--valid-for",
        VALID_FOR,
        "--evidence",
        EVIDENCE,
        "--confidence",
        "measured",
    ];
    project.run("anna", &args).success();

    let expected = format!("{} {}", "é".repeat(50), "ß".repeat(29));
    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.title(), expected);
    assert_eq!(entry.title().chars().count(), 80);
}

#[test]
fn an_explicit_title_is_used_and_names_the_file() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(&[
                "--confidence",
                "measured",
                "--title",
                "Stall margin is overpredicted",
            ]),
        )
        .success();
    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.title(), "Stall margin is overpredicted");
    assert_eq!(
        project
            .entry_file("LES-001")
            .file_name()
            .and_then(|name| name.to_str()),
        Some("LES-001-stall-margin-is-overpredicted.md")
    );
}

#[test]
fn the_task_is_upper_cased_and_the_tags_become_fields_and_empty_ones_are_left_out() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(&[
                "--confidence",
                "simulated",
                "--task",
                "t-042",
                "--machine",
                "axial compressor",
                "--analysis",
                " steady RANS, speedline ",
                "--software",
                "ExampleSolver 2406",
                "--tags",
                "   ",
            ]),
        )
        .success();

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.field("task").as_deref(), Some("T-042"));
    assert_eq!(entry.field("machine").as_deref(), Some("axial compressor"));
    assert_eq!(
        entry.field("analysis").as_deref(),
        Some("steady RANS, speedline")
    );
    assert_eq!(entry.list("analysis"), ["steady RANS", "speedline"]);
    assert_eq!(
        entry.field("software").as_deref(),
        Some("ExampleSolver 2406")
    );
    assert_eq!(entry.field("tags"), None);
}

#[test]
fn an_llm_with_said_by_names_the_person_and_the_lesson_is_approved() {
    let project = Project::new();
    project
        .run(
            "anna-claude",
            &learn(&["--confidence", "measured", "--said-by", "anna"]),
        )
        .success()
        .stdout("LES-001: added (approved)\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
}

#[test]
fn a_persons_lesson_without_valid_for_is_refused_because_it_cannot_be_approved() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "learn",
                STATEMENT,
                "--valid-for",
                "",
                "--evidence",
                EVIDENCE,
                "--confidence",
                "measured",
            ],
        )
        .code(1)
        .stderr(contains("cannot approve lesson: missing section Valid for"));
    assert!(folder_is_empty(&project.root().join("DOSSIER/lessons")));
}

#[test]
fn a_dry_run_says_what_it_would_write_and_writes_nothing() {
    let project = Project::new();
    project
        .run("anna", &learn(&["--confidence", "measured", "--dry-run"]))
        .success()
        .stdout("LES-NEW: added (approved)\n")
        .stderr(contains("warning: dry run: would write"));
    assert!(folder_is_empty(&project.root().join("DOSSIER/lessons")));
    assert!(!project.root().join("DOSSIER/.state/ids.json").exists());
}

#[test]
fn an_empty_statement_is_refused() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "learn",
                "   ",
                "--valid-for",
                VALID_FOR,
                "--evidence",
                EVIDENCE,
                "--confidence",
                "measured",
            ],
        )
        .code(1)
        .stderr(contains("error: the statement is empty"));
    assert!(folder_is_empty(&project.root().join("DOSSIER/lessons")));
}

#[test]
fn a_missing_required_option_is_a_usage_error() {
    let project = Project::new();
    // No --evidence.
    project
        .run(
            "anna",
            &[
                "learn",
                STATEMENT,
                "--valid-for",
                VALID_FOR,
                "--confidence",
                "measured",
            ],
        )
        .code(2);
}

#[test]
fn without_a_name_the_command_is_refused() {
    let project = Project::new();
    project
        .run("", &learn(&["--confidence", "measured"]))
        .code(2)
        .stderr(contains("this command needs a name"));
    assert!(folder_is_empty(&project.root().join("DOSSIER/lessons")));
}

#[test]
fn a_review_date_is_written_as_the_review_field() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(&["--confidence", "simulated", "--review", "2027-06-01"]),
        )
        .success()
        .stdout("LES-001: added (approved)\n");

    assert_eq!(
        entry_of(&project, "LES-001").field("review").as_deref(),
        Some("2027-06-01")
    );
}

#[test]
fn a_lesson_has_no_review_date_unless_one_is_given() {
    let project = Project::new();
    project
        .run("anna", &learn(&["--confidence", "measured"]))
        .success();

    assert_eq!(entry_of(&project, "LES-001").field("review"), None);
}

#[test]
fn a_review_date_that_is_not_a_date_is_refused_and_nothing_is_written() {
    let project = Project::new();
    // 2027 has no 30th of February.
    project
        .run(
            "anna",
            &learn(&["--confidence", "measured", "--review", "2027-02-30"]),
        )
        .code(1)
        .stderr(contains(
            "error: --review needs a date such as 2027-06-01, not \"2027-02-30\"",
        ));
    project
        .run(
            "anna",
            &learn(&[
                "--confidence",
                "measured",
                "--review",
                "next june",
                "--json",
            ]),
        )
        .code(1)
        .stdout(contains("\"code\":\"BAD_VALUE\""));

    assert!(folder_is_empty(&project.root().join("DOSSIER/lessons")));
    assert!(!project.root().join("DOSSIER/.state/ids.json").exists());
}

#[test]
fn an_update_sets_the_review_date_and_one_without_it_keeps_the_date() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(&["--confidence", "measured", "--review", "2027-06-01"]),
        )
        .success();

    project
        .run(
            "anna",
            &learn(&["--confidence", "measured", "--update", "LES-001"]),
        )
        .success()
        .stdout("LES-001: updated\n");
    assert_eq!(
        entry_of(&project, "LES-001").field("review").as_deref(),
        Some("2027-06-01")
    );

    project
        .run(
            "anna",
            &learn(&[
                "--confidence",
                "measured",
                "--update",
                "LES-001",
                "--review",
                "2027-07-01",
            ]),
        )
        .success()
        .stdout("LES-001: updated\n");
    assert_eq!(
        entry_of(&project, "LES-001").field("review").as_deref(),
        Some("2027-07-01")
    );
}

/// Writes `topics = [...]` as the first line of dossier.toml: a top-level key before any table.
fn project_has_topics(project: &Project, names: &[&str]) {
    let list = names
        .iter()
        .map(|name| format!("\"{name}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let text = project.read("DOSSIER/dossier.toml");
    project.write(
        "DOSSIER/dossier.toml",
        &format!("topics = [{list}]\n{text}"),
    );
}

/// Whether the lessons folder holds no file and the ID counter has not been written.
fn nothing_is_written(project: &Project) -> bool {
    folder_is_empty(&project.root().join("DOSSIER/lessons"))
        && !project.root().join("DOSSIER/.state/ids.json").exists()
}

#[test]
fn an_llms_lesson_without_a_topic_is_refused_when_the_project_has_topics() {
    let project = Project::new();
    project_has_topics(&project, &["pumps", "rig"]);
    project
        .run("anna-claude", &learn(&["--confidence", "guess"]))
        .code(1)
        .stderr(contains("names its topic"))
        .stderr(contains("pumps, rig"));
    assert!(nothing_is_written(&project));
    assert!(project.entry_journal().is_empty());
}

#[test]
fn an_llms_lesson_with_a_topic_on_the_list_gets_the_topic_line() {
    let project = Project::new();
    project_has_topics(&project, &["pumps", "rig"]);
    project
        .run(
            "anna-claude",
            &learn(&["--confidence", "guess", "--topic", "pumps"]),
        )
        .success()
        .stdout("LES-001: added (draft, waiting for approval)\n");

    let text = std::fs::read_to_string(project.entry_file("LES-001")).expect("the entry file");
    assert!(text.contains("\n- topic: pumps\n"), "{text}");
    assert_eq!(
        entry_of(&project, "LES-001").field("topic").as_deref(),
        Some("pumps")
    );
}

#[test]
fn a_topic_off_the_list_is_a_usage_error_for_learn_and_nothing_is_written() {
    let project = Project::new();
    project_has_topics(&project, &["pumps", "rig"]);
    project
        .run(
            "anna-claude",
            &learn(&["--confidence", "guess", "--topic", "valves"]),
        )
        .code(2)
        .stderr(contains("unknown topic valves"));
    assert!(nothing_is_written(&project));
}

#[test]
fn a_persons_lesson_without_a_topic_is_written_when_the_project_has_topics() {
    let project = Project::new();
    project_has_topics(&project, &["pumps", "rig"]);
    project
        .run("anna", &learn(&["--confidence", "measured"]))
        .success()
        .stdout("LES-001: added (approved)\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("topic"), None);
}

#[test]
fn without_topics_in_the_settings_an_llms_lesson_without_a_topic_is_written() {
    let project = Project::new();
    project
        .run("anna-claude", &learn(&["--confidence", "guess"]))
        .success()
        .stdout("LES-001: added (draft, waiting for approval)\n");

    assert_eq!(entry_of(&project, "LES-001").field("topic"), None);
}

#[test]
fn code_gives_the_code_field_with_slashes_and_the_pointers_joined() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(&[
                "--confidence",
                "measured",
                "--title",
                "Seal leak is overpredicted",
                "--code",
                "src\\pump, src/rig/valve.rs::open",
            ]),
        )
        .success()
        .stdout("LES-001: added (approved)\n");

    let text = std::fs::read_to_string(project.entry_file("LES-001")).expect("the entry file");
    assert!(
        text.contains("\n- code: src/pump, src/rig/valve.rs::open\n"),
        "{text}"
    );
    assert_eq!(
        entry_of(&project, "LES-001").field("code").as_deref(),
        Some("src/pump, src/rig/valve.rs::open")
    );
    assert_eq!(
        project.stdout("anna", &["which", "src/pump/seal.rs"]),
        "not attached to any task\nentries: LES-001 Seal leak is overpredicted\n"
    );
}

#[test]
fn without_code_no_code_line_is_written() {
    let project = Project::new();
    project
        .run("anna", &learn(&["--confidence", "measured"]))
        .success();

    let text = std::fs::read_to_string(project.entry_file("LES-001")).expect("the entry file");
    assert!(!text.contains("\n- code:"), "{text}");
    assert_eq!(entry_of(&project, "LES-001").field("code"), None);
}
