//! Runs the built program for `propose`: a decision suggested with its reason, from anyone, with
//! status proposed. It is not in force until a decider accepts it, so `list` leaves it out and
//! `list --drafts` shows it. Each test runs in its own temporary project folder (common::Project),
//! with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use predicates::str::contains;

/// The text of the decision file with `id`.
fn decision_text(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the decision file")
}

/// The history lines of an entry file, in file order.
fn history_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| line.starts_with("- history: "))
        .collect()
}

/// Writes `topics = [...]` into dossier.toml, after the `code` line, so that it is a top-level key
/// before the tables (format.md §35).
fn project_has_topics(project: &Project, names: &[&str]) {
    let list = names
        .iter()
        .map(|name| format!("\"{name}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let text = project.read("DOSSIER/dossier.toml");
    let mut edited = String::new();
    for line in text.split_inclusive('\n') {
        edited.push_str(line);
        if line.starts_with("code = ") {
            edited.push_str(&format!("topics = [{list}]\n"));
        }
    }
    assert!(edited.contains("topics = ["), "{edited}");
    project.write("DOSSIER/dossier.toml", &edited);
}

/// Whether the decisions folder holds no file at all.
fn no_decision_written(project: &Project) -> bool {
    let folder = project.root().join("DOSSIER").join("decisions");
    std::fs::read_dir(folder)
        .expect("the decisions folder")
        .next()
        .is_none()
}

#[test]
fn anna_claude_proposes_without_said_by_and_the_file_is_a_proposal_listed_only_by_drafts() {
    let project = Project::new();
    let output = project.stdout(
        "anna-claude",
        &[
            "propose",
            "Use the transient runs for the last three points.",
            "--reason",
            "Steady runs converged where the transient run already showed rotating stall.",
        ],
    );
    assert_eq!(
        output,
        "DEC-001: proposed; a decider accepts it with: dossier accept DEC-001\n"
    );

    let text = decision_text(&project, "DEC-001");
    assert!(text.starts_with(
        "<!-- dossier-format: 1 -->\n# DEC-001 · Use the transient runs for the last three points.\n"
    ));
    assert!(text.contains("\n- kind: decision\n"));
    assert!(text.contains("\n- status: proposed\n"));
    assert!(text.contains("\n- author: anna\n"));
    assert!(text.contains("\n- via: anna-claude\n"));
    assert!(!text.contains("- approved:"), "a proposal is not approved");
    assert!(
        !text.contains("- valid-from:"),
        "a proposal is not in force"
    );
    assert!(text.contains(
        "\n## Decision\n\nUse the transient runs for the last three points.\n\n## Reason\n\nSteady runs converged where the transient run already showed rotating stall.\n"
    ));
    assert_eq!(
        history_lines(&text),
        [
            "- history: 2026-11-20T10:00:00+01:00 anna via anna-claude: created",
            "- history: 2026-11-20T10:00:00+01:00 anna via anna-claude: proposed",
        ]
    );

    // Not in the list of approved entries; in the list of drafts, with its mark.
    let list = project.stdout("anna", &["list"]);
    assert!(!list.contains("DEC-001"), "list shows: {list}");
    let drafts = project.stdout("anna", &["list", "--drafts"]);
    assert!(drafts.contains("DEC-001"), "list --drafts shows: {drafts}");
    assert!(
        drafts.contains("[proposed]"),
        "list --drafts shows: {drafts}"
    );
}

#[test]
fn a_person_proposal_is_written_with_the_title_task_and_tags_given() {
    let project = Project::new();
    project.stdout(
        "ben",
        &[
            "propose",
            "Use the transient runs.",
            "--reason",
            "Steady runs converged.",
            "--title",
            "Transient runs",
            "--task",
            "t-187",
            "--machine",
            "cluster-b",
            "--software",
            "ExampleSolver",
        ],
    );

    let text = decision_text(&project, "DEC-001");
    assert!(text.starts_with("<!-- dossier-format: 1 -->\n# DEC-001 · Transient runs\n"));
    assert!(text.contains("\n- status: proposed\n"));
    assert!(text.contains("\n- author: ben\n"));
    assert!(!text.contains("- via:"), "a person's proposal has no LLM");
    assert!(text.contains("\n- task: T-187\n"));
    assert!(text.contains("\n- machine: cluster-b\n"));
    assert!(text.contains("\n- software: ExampleSolver\n"));
    assert!(!text.contains("- approved:"));
    assert_eq!(
        history_lines(&text),
        [
            "- history: 2026-11-20T10:00:00+01:00 ben: created",
            "- history: 2026-11-20T10:00:00+01:00 ben: proposed",
        ]
    );
}

#[test]
fn the_default_title_is_the_first_80_characters_of_the_decision() {
    let project = Project::new();
    let decision = "b".repeat(90);
    project.stdout(
        "anna",
        &["propose", &decision, "--reason", "Steady runs converged."],
    );

    let text = decision_text(&project, "DEC-001");
    assert!(text.starts_with(&format!(
        "<!-- dossier-format: 1 -->\n# DEC-001 · {}\n",
        "b".repeat(80)
    )));
}

#[test]
fn a_proposal_gives_the_journal_a_created_line_with_the_proposed_detail() {
    let project = Project::new();
    project.stdout(
        "ben",
        &[
            "propose",
            "Use the transient runs.",
            "--reason",
            "Steady runs converged.",
        ],
    );

    let journal = project.entry_journal();
    assert_eq!(journal.len(), 1, "one entry event: {journal:?}");
    assert_eq!(journal[0].command, "propose");
    assert_eq!(journal[0].id, "DEC-001");
    assert_eq!(journal[0].event, "created");
    assert_eq!(journal[0].detail, "proposed");
    assert_eq!(journal[0].person, "ben");
    assert!(!journal[0].llm);
}

#[test]
fn json_gives_the_id_and_the_status() {
    let project = Project::new();
    let stdout = project.stdout(
        "anna",
        &[
            "propose",
            "Use the transient runs.",
            "--reason",
            "Steady runs converged.",
            "--json",
        ],
    );
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.propose.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["id"], "DEC-001");
    assert_eq!(value["data"]["status"], "proposed");
}

#[test]
fn a_proposal_without_a_reason_is_a_usage_error_and_nothing_is_written() {
    let project = Project::new();
    project
        .run("anna-claude", &["propose", "Use the transient runs."])
        .failure()
        .code(2)
        .stderr(contains("--reason"));
    assert!(no_decision_written(&project));
}

#[test]
fn with_topics_an_llms_proposal_without_a_topic_is_refused_and_nothing_is_written() {
    let project = Project::new();
    project_has_topics(&project, &["pumps", "rig"]);

    project
        .run(
            "anna-claude",
            &[
                "propose",
                "Use the transient runs.",
                "--reason",
                "Steady runs converged.",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains(
            "an entry written by an LLM names its topic: --topic <one of pumps, rig>",
        ));
    assert!(no_decision_written(&project));
}

#[test]
fn with_topics_an_llms_proposal_with_a_listed_topic_gives_the_topic_line() {
    let project = Project::new();
    project_has_topics(&project, &["pumps", "rig"]);

    project.stdout(
        "anna-claude",
        &[
            "propose",
            "Use the transient runs.",
            "--reason",
            "Steady runs converged.",
            "--topic",
            "pumps",
        ],
    );

    let text = decision_text(&project, "DEC-001");
    assert!(text.contains("\n- topic: pumps\n"), "{text}");
    assert!(text.contains("\n- status: proposed\n"));
}

#[test]
fn with_topics_a_proposal_with_an_unknown_topic_is_a_usage_error_and_nothing_is_written() {
    let project = Project::new();
    project_has_topics(&project, &["pumps", "rig"]);

    project
        .run(
            "anna-claude",
            &[
                "propose",
                "Use the transient runs.",
                "--reason",
                "Steady runs converged.",
                "--topic",
                "valves",
            ],
        )
        .failure()
        .code(2)
        .stderr(contains(
            "unknown topic valves; the topics of this project: pumps, rig (dossier topics)",
        ));
    assert!(no_decision_written(&project));
}

#[test]
fn with_topics_a_persons_proposal_without_a_topic_is_written() {
    let project = Project::new();
    project_has_topics(&project, &["pumps", "rig"]);

    project.stdout(
        "ben",
        &[
            "propose",
            "Use the transient runs.",
            "--reason",
            "Steady runs converged.",
        ],
    );

    let text = decision_text(&project, "DEC-001");
    assert!(text.contains("\n- status: proposed\n"));
    assert!(!text.contains("- topic:"), "{text}");
}

#[test]
fn without_topics_in_the_settings_an_llms_proposal_without_a_topic_is_written() {
    let project = Project::new();

    project.stdout(
        "anna-claude",
        &[
            "propose",
            "Use the transient runs.",
            "--reason",
            "Steady runs converged.",
        ],
    );

    let text = decision_text(&project, "DEC-001");
    assert!(text.contains("\n- status: proposed\n"));
    assert!(!text.contains("- topic:"), "{text}");
}
