//! Runs the built program for `howto`: a how-to a person writes is approved and recorded for its
//! checksum, one an LLM writes is a draft that is not recorded, the body comes from a file or from
//! standard input, the review date, task and tags are written, `--team` writes it into the team
//! dossier after the confidential check, and the refusals. Each test runs in its own temporary
//! project folder, with the clock fixed by common::Project (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::approval::{self, Mark};
use dossier::config;
use dossier::entry::{Entry, Kind, Status};
use predicates::str::contains;
use std::fs;
use std::path::{Path, PathBuf};

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for a complete how-to.
const TITLE: &str = "Run the speedline of an invented compressor";
const SOFTWARE: &str = "ExampleSolver 2406";
const PURPOSE: &str = "Shows the speedline of an invented compressor at one operating point.";
const STEPS: &str = "1. Open the invented case file.\n2. Set the invented rotational speed.\n3. Run to convergence.";

/// The sections as the `--section` options give them.
const PURPOSE_SECTION: &str =
    "Purpose=Shows the speedline of an invented compressor at one operating point.";
const STEPS_SECTION: &str = "Steps=1. Open the invented case file.\n2. Set the invented rotational speed.\n3. Run to convergence.";

/// `howto` for a complete how-to of ExampleSolver 2406, followed by `extra`.
fn complete<'a>(extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "howto",
        "--software",
        SOFTWARE,
        "--title",
        TITLE,
        "--section",
        PURPOSE_SECTION,
        "--section",
        STEPS_SECTION,
    ];
    args.extend_from_slice(extra);
    args
}

/// The DOSSIER folder of the project.
fn dossier_of(project: &Project) -> PathBuf {
    project.root().join("DOSSIER")
}

/// The file in `folder` whose name starts with `<id>-`. Panics when there is none.
fn file_in(folder: &Path, id: &str) -> PathBuf {
    let prefix = format!("{id}-");
    fs::read_dir(folder)
        .expect("the folder is read")
        .map(|entry| entry.expect("a folder entry").path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&prefix))
        })
        .unwrap_or_else(|| panic!("no file for {id} in {}", folder.display()))
}

/// The how-to file with `id` in `folder`, read and parsed.
fn how_to_in(folder: &Path, id: &str) -> Entry {
    let text = fs::read_to_string(file_in(folder, id)).expect("the file is read");
    Entry::parse(&text).expect("the file is an entry")
}

/// Whether a folder has no files in it.
fn folder_is_empty(folder: &Path) -> bool {
    fs::read_dir(folder)
        .expect("the folder is read")
        .next()
        .is_none()
}

/// A team dossier made by `init --team-dossier --code TEAM` in its own temporary folder, run as
/// anna, who is made a member. Returns the folder (it keeps the folder alive) and its DOSSIER.
fn team_dossier(project: &Project) -> (tempfile::TempDir, PathBuf) {
    let folder = tempfile::Builder::new()
        .prefix("Team Äpfel ")
        .tempdir()
        .expect("a temporary team folder");
    project
        .command("anna", NOW)
        .current_dir(folder.path())
        .args(["init", "--code", "TEAM", "--team-dossier"])
        .assert()
        .success();
    let dossier = folder.path().join("DOSSIER");
    config::set_list(&dossier, "members", &["anna".to_string()])
        .expect("anna is a member of the team");
    (folder, dossier)
}

/// Links the project to the team dossier at `team`, as `team use` does.
fn link(project: &Project, team: &Path) {
    let path = team.display().to_string();
    project.stdout("anna", &["team", "use", path.as_str()]);
}

/// Writes `topics = ["pumps", "rig"]` as the first line of dossier.toml, before its first table,
/// so that the project has a list of topics (R-TOPIC-1).
fn with_topics(project: &Project) {
    let path = "DOSSIER/dossier.toml";
    let text = project.read(path);
    project.write(path, &format!("topics = [\"pumps\", \"rig\"]\n{text}"));
}

/// The number of files in `folder`.
fn file_count(folder: &Path) -> usize {
    fs::read_dir(folder).expect("the folder is read").count()
}

#[test]
fn a_persons_how_to_is_approved_and_recorded_for_its_checksum() {
    let project = Project::new();
    project
        .run("anna", &complete(&[]))
        .success()
        .stdout("HOW-001: added (approved)\n");

    let path = project.entry_file("HOW-001");
    assert!(path.starts_with(dossier_of(&project).join("howtos")));
    let text = fs::read_to_string(&path).expect("the file is read");
    let entry = Entry::parse(&text).expect("the file is an entry");
    assert_eq!(entry.kind(), Some(Kind::Howto));
    assert_eq!(entry.title(), TITLE);
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("software").as_deref(), Some(SOFTWARE));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via"), None);
    assert!(entry.field("approved").is_some());
    assert_eq!(entry.section("Purpose").as_deref(), Some(PURPOSE));
    assert_eq!(entry.section("Steps").as_deref(), Some(STEPS));
    assert_eq!(entry.history().len(), 2);

    // A person's how-to is recorded for its checksum, so a later hand change is marked.
    let state = dossier_of(&project).join(".state");
    assert!(matches!(
        approval::mark(&state, "HOW-001", text.as_bytes()),
        Mark::Unchanged
    ));
}

#[test]
fn an_llm_how_to_is_a_draft_that_waits_for_approval_and_is_not_recorded() {
    let project = Project::new();
    // A draft may lack the sections that approval needs.
    project
        .run(
            "anna-claude",
            &["howto", "--software", SOFTWARE, "--title", TITLE],
        )
        .success()
        .stdout("HOW-001: added (draft, waiting for approval)\n");

    let entry = how_to_in(&dossier_of(&project).join("howtos"), "HOW-001");
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
    assert_eq!(entry.field("approved"), None);
    assert_eq!(entry.field("valid-from"), None);
    assert!(
        !dossier_of(&project)
            .join(".state")
            .join("approved.json")
            .exists()
    );
}

#[test]
fn a_how_to_without_software_is_a_usage_error_and_nothing_is_written() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "howto",
                "--title",
                TITLE,
                "--section",
                PURPOSE_SECTION,
                "--section",
                STEPS_SECTION,
            ],
        )
        .code(2)
        .stderr(contains("--software"));
    assert!(folder_is_empty(&dossier_of(&project).join("howtos")));

    // A blank software is refused the same way, with its exit code 1.
    project
        .run("anna", &["howto", "--software", "  ", "--title", TITLE])
        .failure()
        .code(1)
        .stderr(contains("--software is empty"));
    assert!(folder_is_empty(&dossier_of(&project).join("howtos")));
}

#[test]
fn a_blank_title_and_a_review_date_that_is_not_a_date_are_refused() {
    let project = Project::new();
    project
        .run("anna", &["howto", "--software", SOFTWARE, "--title", "   "])
        .failure()
        .code(1)
        .stderr(contains("the title is empty"));
    project
        .run("anna", &complete(&["--review", "2027-13-01"]))
        .failure()
        .code(1)
        .stderr(contains("--review needs a date such as 2027-06-01"));
    // Refused before an ID is given out: nothing is written.
    assert!(
        !dossier_of(&project)
            .join(".state")
            .join("ids.json")
            .exists()
    );
    assert!(folder_is_empty(&dossier_of(&project).join("howtos")));
}

#[test]
fn the_body_comes_from_a_file_or_from_standard_input_with_its_sections() {
    let project = Project::new();
    project.write(
        "notes/solver.md",
        &format!(
            "Notes before the first heading are ignored.\n\n## Purpose\n\n{PURPOSE}\n\n## Steps\n\n{STEPS}\n\n## Traps\n\nA trap for a test.\n"
        ),
    );
    project
        .run(
            "anna",
            &[
                "howto",
                "--software",
                SOFTWARE,
                "--title",
                TITLE,
                "--from",
                "notes/solver.md",
            ],
        )
        .success()
        .stdout("HOW-001: added (approved)\n");
    let entry = how_to_in(&dossier_of(&project).join("howtos"), "HOW-001");
    assert_eq!(entry.section("Purpose").as_deref(), Some(PURPOSE));
    assert_eq!(entry.section("Steps").as_deref(), Some(STEPS));
    assert_eq!(
        entry.section("Traps").as_deref(),
        Some("A trap for a test.")
    );

    let body = format!("## Purpose\n\n{PURPOSE}\n\n## Steps\n\n{STEPS}\n");
    project
        .command("anna", NOW)
        .args([
            "howto",
            "--software",
            SOFTWARE,
            "--title",
            "Second run of the speedline",
            "--stdin",
        ])
        .write_stdin(body)
        .assert()
        .success()
        .stdout(contains("HOW-002: added (approved)"));
    let entry = how_to_in(&dossier_of(&project).join("howtos"), "HOW-002");
    assert_eq!(entry.section("Steps").as_deref(), Some(STEPS));
}

#[test]
fn the_review_date_task_and_tags_are_written_as_fields() {
    let project = Project::new();
    project
        .run(
            "anna",
            &complete(&[
                "--review",
                "2027-06-01",
                "--task",
                "t-042",
                "--machine",
                "axial compressor",
                "--analysis",
                "steady RANS, speedline",
                "--tags",
                "tested",
            ]),
        )
        .success()
        .stdout("HOW-001: added (approved)\n");

    let entry = how_to_in(&dossier_of(&project).join("howtos"), "HOW-001");
    assert_eq!(entry.field("review").as_deref(), Some("2027-06-01"));
    // The task is upper-cased, as `add` does with --task.
    assert_eq!(entry.field("task").as_deref(), Some("T-042"));
    assert_eq!(entry.field("machine").as_deref(), Some("axial compressor"));
    assert_eq!(
        entry.field("analysis").as_deref(),
        Some("steady RANS, speedline")
    );
    assert_eq!(entry.field("tags").as_deref(), Some("tested"));
}

#[test]
fn json_gives_the_id_the_status_and_whether_it_went_to_the_team() {
    let project = Project::new();
    project
        .run("anna", &complete(&["--json"]))
        .success()
        .stdout(contains("\"schema\":\"dossier.howto.v1\""))
        .stdout(contains("\"id\":\"HOW-001\""))
        .stdout(contains("\"status\":\"approved\""))
        .stdout(contains("\"team\":false"));
}

#[test]
fn with_team_a_how_to_is_written_into_the_team_dossier_and_not_the_project() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);
    // The project states its confidential terms first: a team write needs them (R-CONF-5).
    project.stdout("anna", &["confidential", "add", "NT-999"]);

    project
        .run("anna", &complete(&["--team"]))
        .success()
        .stdout("HOW-001: added (approved)\n");

    // The team's howtos folder has it, approved and recorded in the team's own state folder.
    let entry = how_to_in(&team.join("howtos"), "HOW-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("software").as_deref(), Some(SOFTWARE));
    assert_eq!(entry.section("Purpose").as_deref(), Some(PURPOSE));
    let text = fs::read_to_string(file_in(&team.join("howtos"), "HOW-001")).expect("the file");
    assert!(matches!(
        approval::mark(&team.join(".state"), "HOW-001", text.as_bytes()),
        Mark::Unchanged
    ));
    // The project's own howtos folder stays empty.
    assert!(folder_is_empty(&dossier_of(&project).join("howtos")));
}

#[test]
fn a_team_how_to_from_a_project_without_terms_or_a_statement_is_refused() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);

    project
        .run("anna", &complete(&["--team"]))
        .failure()
        .code(1)
        .stderr(contains("this project has no confidential terms yet"));
    assert!(folder_is_empty(&team.join("howtos")));
}

#[test]
fn a_team_how_to_with_a_confidential_term_in_it_is_refused() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);
    project.stdout("anna", &["confidential", "add", "Nordwind"]);

    // The term is in the steps, so the how-to is refused whoever writes it (R-WRITE-6, R-CONF-2).
    let steps = "Steps=1. Open the Nordwind case file.\n2. Run to convergence.";
    project
        .run(
            "anna",
            &[
                "--team",
                "howto",
                "--software",
                SOFTWARE,
                "--title",
                TITLE,
                "--section",
                PURPOSE_SECTION,
                "--section",
                steps,
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("confidential terms found: Nordwind"));
    assert!(folder_is_empty(&team.join("howtos")));
}

#[test]
fn with_topics_an_llms_how_to_must_name_one_and_a_persons_need_not() {
    let project = Project::new();
    with_topics(&project);
    let howtos = dossier_of(&project).join("howtos");

    // An LLM's how-to without --topic is refused, and the message names both topics.
    project
        .run(
            "anna-claude",
            &["howto", "--software", SOFTWARE, "--title", TITLE],
        )
        .failure()
        .code(1)
        .stderr(contains(
            "an entry written by an LLM names its topic: --topic <one of pumps, rig>",
        ));
    assert!(folder_is_empty(&howtos));

    // With --topic the draft gets the field topic.
    project
        .run(
            "anna-claude",
            &[
                "howto",
                "--software",
                SOFTWARE,
                "--title",
                TITLE,
                "--topic",
                "pumps",
            ],
        )
        .success()
        .stdout("HOW-001: added (draft, waiting for approval)\n");
    let entry = how_to_in(&howtos, "HOW-001");
    assert_eq!(entry.field("topic").as_deref(), Some("pumps"));
    let text = fs::read_to_string(file_in(&howtos, "HOW-001")).expect("the file is read");
    assert!(text.lines().any(|line| line == "- topic: pumps"));

    // A topic that is not on the list is a usage error, and nothing is written.
    project
        .run(
            "anna-claude",
            &[
                "howto",
                "--software",
                SOFTWARE,
                "--title",
                TITLE,
                "--topic",
                "valves",
            ],
        )
        .failure()
        .code(2)
        .stderr(contains(
            "unknown topic valves; the topics of this project: pumps, rig (dossier topics)",
        ));
    assert_eq!(file_count(&howtos), 1);

    // A person's how-to without --topic is written.
    project
        .run("anna", &complete(&[]))
        .success()
        .stdout("HOW-002: added (approved)\n");
    assert_eq!(how_to_in(&howtos, "HOW-002").field("topic"), None);
}

#[test]
fn without_topics_in_the_settings_an_llms_how_to_without_topic_is_written() {
    let project = Project::new();
    project
        .run(
            "anna-claude",
            &["howto", "--software", SOFTWARE, "--title", TITLE],
        )
        .success()
        .stdout("HOW-001: added (draft, waiting for approval)\n");
    let entry = how_to_in(&dossier_of(&project).join("howtos"), "HOW-001");
    assert_eq!(entry.field("topic"), None);
}
