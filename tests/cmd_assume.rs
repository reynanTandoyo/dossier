//! Runs the built program for `assume`: an assumption is approved at once, whoever writes it, with
//! state `open`, and its file has the sections of format.md §8. Each test runs in its own
//! temporary project folder with the clock fixed.

mod common;

use common::Project;
use dossier::entry::{Entry, Status};
use predicates::str::contains;

/// The text of an assumption used by the tests (invented).
const TEXT: &str = "The inlet profile is uniform over the span";

/// The text of the entry file with `id`.
fn entry_text(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the entry file can be read")
}

/// The entry with `id`, parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    Entry::parse(&entry_text(project, id)).expect("the file is an entry")
}

/// Whether the folder of assumptions has no files in it.
fn no_assumption_written(project: &Project) -> bool {
    let folder = project.root().join("DOSSIER").join("assumptions");
    std::fs::read_dir(folder)
        .expect("the assumptions folder")
        .next()
        .is_none()
}

#[test]
fn anna_claude_records_an_assumption_approved_with_state_open() {
    let project = Project::new();
    let stdout = project.stdout(
        "anna-claude",
        &[
            "assume",
            TEXT,
            "--reason",
            "No measured profile was delivered.",
            "--affects",
            "T-042, T-043",
        ],
    );
    assert_eq!(stdout, "ASM-001: added (approved, open)\n");

    let file = project.entry_file("ASM-001");
    assert_eq!(
        file.file_name().and_then(|name| name.to_str()),
        Some("ASM-001-the-inlet-profile-is-uniform-over-the-span.md")
    );
    assert_eq!(
        entry_text(&project, "ASM-001"),
        "<!-- dossier-format: 1 -->\n\
         # ASM-001 · The inlet profile is uniform over the span\n\
         \n\
         - kind: assumption\n\
         - status: approved\n\
         - created: 2026-11-20T10:00:00+01:00\n\
         - author: anna\n\
         - via: anna-claude\n\
         - approved: 2026-11-20T10:00:00+01:00 by anna\n\
         - valid-from: 2026-11-20T10:00:00+01:00\n\
         - state: open\n\
         - history: 2026-11-20T10:00:00+01:00 anna via anna-claude: created\n\
         - history: 2026-11-20T10:00:00+01:00 anna via anna-claude: approved\n\
         \n\
         ## Assumption\n\
         \n\
         The inlet profile is uniform over the span\n\
         \n\
         ## Reason\n\
         \n\
         No measured profile was delivered.\n\
         \n\
         ## Affects\n\
         \n\
         T-042, T-043\n"
    );
}

#[test]
fn a_person_writing_an_assumption_has_no_via() {
    let project = Project::new();
    let stdout = project.stdout(
        "anna",
        &["assume", TEXT, "--reason", "No profile was sent."],
    );
    assert_eq!(stdout, "ASM-001: added (approved, open)\n");

    let entry = entry_of(&project, "ASM-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via"), None);
    assert_eq!(entry.field("state").as_deref(), Some("open"));
}

#[test]
fn without_a_title_the_first_80_characters_are_the_title() {
    let project = Project::new();
    // The umlauts are two bytes each: the title counts characters, so 80 of them are taken.
    let text = format!("{} tail that is cut off", "ä".repeat(90));
    project.stdout("anna-claude", &["assume", &text, "--reason", "No profile."]);

    assert_eq!(entry_of(&project, "ASM-001").title(), "ä".repeat(80));
}

#[test]
fn without_a_title_line_breaks_become_spaces() {
    let project = Project::new();
    project.stdout(
        "anna-claude",
        &[
            "assume",
            "Pump speed\nis constant",
            "--reason",
            "No profile.",
        ],
    );

    assert_eq!(
        entry_of(&project, "ASM-001").title(),
        "Pump speed is constant"
    );
}

#[test]
fn a_given_title_replaces_the_default_and_the_file_name_follows_it() {
    let project = Project::new();
    project.stdout(
        "anna-claude",
        &[
            "assume",
            TEXT,
            "--reason",
            "No profile.",
            "--title",
            "Inlet is uniform",
        ],
    );

    assert_eq!(entry_of(&project, "ASM-001").title(), "Inlet is uniform");
    assert_eq!(
        project
            .entry_file("ASM-001")
            .file_name()
            .and_then(|name| name.to_str()),
        Some("ASM-001-inlet-is-uniform.md")
    );
}

#[test]
fn the_task_is_upper_cased_and_the_tag_lists_are_written_as_given() {
    let project = Project::new();
    project.stdout(
        "anna-claude",
        &[
            "assume",
            TEXT,
            "--reason",
            "No profile.",
            "--task",
            "t-042",
            "--machine",
            "axial compressor, fan",
            "--analysis",
            "steady RANS",
            "--software",
            "",
            "--tags",
            "speedline",
        ],
    );

    let entry = entry_of(&project, "ASM-001");
    assert_eq!(entry.field("task").as_deref(), Some("T-042"));
    assert_eq!(
        entry.field("machine").as_deref(),
        Some("axial compressor, fan")
    );
    assert_eq!(entry.list("machine"), ["axial compressor", "fan"]);
    assert_eq!(entry.field("analysis").as_deref(), Some("steady RANS"));
    assert_eq!(entry.field("tags").as_deref(), Some("speedline"));
    // An empty tag option is left out.
    assert_eq!(entry.field("software"), None);
}

#[test]
fn without_affects_there_is_no_affects_section() {
    let project = Project::new();
    project.stdout("anna-claude", &["assume", TEXT, "--reason", "No profile."]);

    let names: Vec<String> = entry_of(&project, "ASM-001")
        .sections()
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(names, ["Assumption", "Reason"]);
}

#[test]
fn as_json_gives_the_id_and_the_status() {
    let project = Project::new();
    project
        .run(
            "anna-claude",
            &["assume", TEXT, "--reason", "No profile.", "--json"],
        )
        .success()
        .stdout(contains("\"schema\":\"dossier.assume.v1\""))
        .stdout(contains("\"ok\":true"))
        .stdout(contains("\"id\":\"ASM-001\""))
        .stdout(contains("\"status\":\"approved\""));
}

#[test]
fn a_dry_run_prints_the_id_and_writes_nothing() {
    let project = Project::new();
    project
        .run(
            "anna-claude",
            &["assume", TEXT, "--reason", "No profile.", "--dry-run"],
        )
        .success()
        .stdout("ASM-NEW: added (approved, open)\n")
        .stderr(contains("warning: dry run: would write"));
    assert!(no_assumption_written(&project));
}

#[test]
fn a_missing_reason_is_a_usage_error_and_nothing_is_written() {
    let project = Project::new();
    project
        .run("anna-claude", &["assume", TEXT])
        .code(2)
        .stderr(contains("--reason"));
    assert!(no_assumption_written(&project));
}

#[test]
fn without_a_name_nothing_is_written() {
    let project = Project::new();
    project
        .run("", &["assume", TEXT, "--reason", "No profile."])
        .code(2)
        .stderr(contains("this command needs a name"));
    assert!(no_assumption_written(&project));
}

#[test]
fn a_blank_title_is_refused_and_nothing_is_written() {
    let project = Project::new();
    project
        .run(
            "anna-claude",
            &["assume", TEXT, "--reason", "No profile.", "--title", "  "],
        )
        .code(1)
        .stderr(contains("error: the title must not be empty"));
    assert!(no_assumption_written(&project));
}

#[test]
fn an_empty_assumption_cannot_be_approved_and_nothing_is_written() {
    let project = Project::new();
    project
        .run("anna-claude", &["assume", "", "--reason", "No profile."])
        .code(1)
        .stderr(contains("missing section Assumption"));
    assert!(no_assumption_written(&project));
}
