//! Runs the built program for `confirm`: a person confirms an open, approved assumption, with
//! `--by` naming who and how; an LLM does so with `--said-by <person>` (D57). The entry gets state
//! `confirmed`, the `confirmed` field, the Confirmation section and a history line. An LLM without
//! `--said-by`, for another person or in a worker session, a draft, an assumption that is not
//! open, an entry of another kind and a blank, multi-line or heading-like `--by` are refused, and
//! nothing is written in those cases. Each test runs in its own temporary project folder, with the
//! clock fixed by `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in the fields and history lines.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for an assumption, a decision, its reasons and the `--by` text.
const ASSUMPTION: &str = "The invented inlet profile is uniform over the span";
const REASON: &str = "An invented reason: no measured profile was delivered.";
const DECISION: &str = "Run the invented test rig with two inlet guide vanes.";
const BY: &str = "measured on the invented rig, report 3";

/// The text of the entry file with `id`.
fn entry_text(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the entry file can be read")
}

/// The bytes of the entry file with `id`.
fn bytes_of(project: &Project, id: &str) -> Vec<u8> {
    std::fs::read(project.entry_file(id)).expect("the entry file can be read")
}

/// Anna-claude records an approved assumption with state open: ASM-001.
fn claude_assumes(project: &Project) {
    project.stdout("anna-claude", &["assume", ASSUMPTION, "--reason", REASON]);
}

#[test]
fn ben_confirms_an_open_assumption_with_who_when_and_how() {
    let project = Project::new();
    claude_assumes(&project);
    let path_before = project.entry_file("ASM-001");

    project
        .run("ben", &["confirm", "ASM-001", "--by", BY])
        .success()
        .stdout("ASM-001: confirmed\n");

    // The file stays where it was. Its state changes, `confirmed` names the time and the person
    // who confirmed it, the Confirmation section holds the --by text, and the history line
    // names who and how. Every other line keeps its bytes.
    assert_eq!(project.entry_file("ASM-001"), path_before);
    assert_eq!(
        entry_text(&project, "ASM-001"),
        "<!-- dossier-format: 1 -->\n\
         # ASM-001 · The invented inlet profile is uniform over the span\n\
         \n\
         - kind: assumption\n\
         - status: approved\n\
         - created: 2026-11-20T10:00:00+01:00\n\
         - author: anna\n\
         - via: anna-claude\n\
         - approved: 2026-11-20T10:00:00+01:00 by anna\n\
         - valid-from: 2026-11-20T10:00:00+01:00\n\
         - state: confirmed\n\
         - confirmed: 2026-11-20T10:00:00+01:00 by ben\n\
         - history: 2026-11-20T10:00:00+01:00 anna via anna-claude: created\n\
         - history: 2026-11-20T10:00:00+01:00 anna via anna-claude: approved\n\
         - history: 2026-11-20T10:00:00+01:00 ben: confirmed — measured on the invented rig, report 3\n\
         \n\
         ## Assumption\n\
         \n\
         The invented inlet profile is uniform over the span\n\
         \n\
         ## Reason\n\
         \n\
         An invented reason: no measured profile was delivered.\n\
         \n\
         ## Confirmation\n\
         \n\
         measured on the invented rig, report 3\n"
    );

    // The journal names the person who confirmed and the --by text, and the file as it was is
    // kept as a snapshot.
    let record = project.entry_journal().pop().expect("a journal record");
    assert_eq!(record.id, "ASM-001");
    assert_eq!(record.event, "confirmed");
    assert_eq!(record.name, "ben");
    assert!(!record.llm);
    assert_eq!(record.detail, BY);
    assert!(
        project
            .root()
            .join("DOSSIER")
            .join(".state")
            .join("history")
            .join("ASM-001")
            .join("000001.md")
            .exists()
    );
}

#[test]
fn confirming_an_assumption_again_is_refused_and_writes_nothing() {
    let project = Project::new();
    claude_assumes(&project);
    project
        .run("ben", &["confirm", "ASM-001", "--by", BY])
        .success();
    let before = bytes_of(&project, "ASM-001");
    let journal_before = project.entry_journal().len();

    project
        .run("ben", &["confirm", "ASM-001", "--by", "again"])
        .failure()
        .code(1)
        .stderr(contains(
            "error: only an open assumption can be confirmed; ASM-001 is confirmed",
        ));
    assert_eq!(bytes_of(&project, "ASM-001"), before);
    assert_eq!(project.entry_journal().len(), journal_before);
}

#[test]
fn anna_claude_cannot_confirm_and_nothing_is_written() {
    let project = Project::new();
    claude_assumes(&project);
    let before = bytes_of(&project, "ASM-001");

    project
        .run("anna-claude", &["confirm", "ASM-001", "--by", BY])
        .failure()
        .code(1)
        .stderr(contains("--said-by anna"));
    assert_eq!(bytes_of(&project, "ASM-001"), before);
}

#[test]
fn an_llm_without_said_by_is_refused_and_nothing_is_written() {
    let project = Project::new();
    claude_assumes(&project);
    let before = bytes_of(&project, "ASM-001");

    project
        .run("anna-claude", &["confirm", "ASM-001", "--by", BY])
        .failure()
        .code(1)
        .stderr(contains("--said-by anna"));
    assert_eq!(bytes_of(&project, "ASM-001"), before);
}

#[test]
fn an_llm_with_said_by_confirms_its_assumption_without_a_terminal_or_a_question() {
    let project = Project::new();
    claude_assumes(&project);
    // No answers are prepared and no terminal is attached, so nothing can be asked: the
    // assumption is confirmed as the person decided (D57).
    project
        .command("anna-claude", NOW)
        .args(["confirm", "ASM-001", "--by", BY, "--said-by", "anna"])
        .assert()
        .success()
        .stdout("ASM-001: confirmed\n")
        .stderr("");

    let text = entry_text(&project, "ASM-001");
    assert!(text.contains(&format!("- state: confirmed\n- confirmed: {NOW} by anna\n")));
    let newest_history = text.lines().rfind(|line| line.starts_with("- history: "));
    assert_eq!(
        newest_history,
        Some(format!("- history: {NOW} anna via anna-claude: confirmed — {BY}").as_str())
    );
    let journal = project.entry_journal();
    let newest = journal.last().expect("the confirmation is journalled");
    assert_eq!(newest.id, "ASM-001");
    assert_eq!(newest.event, "confirmed");
    assert_eq!(newest.person, "anna");
    assert_eq!(newest.name, "anna-claude");
    assert!(newest.llm);
}

#[test]
fn an_llm_with_said_by_for_another_person_is_refused_and_nothing_is_written() {
    let project = Project::new();
    claude_assumes(&project);
    let before = bytes_of(&project, "ASM-001");

    project
        .run(
            "anna-claude",
            &["confirm", "ASM-001", "--by", BY, "--said-by", "ben"],
        )
        .failure()
        .code(1)
        .stderr(contains("acts for anna only"));
    assert_eq!(bytes_of(&project, "ASM-001"), before);
}

#[test]
fn a_worker_session_decides_nothing_even_with_said_by() {
    let project = Project::new();
    claude_assumes(&project);
    let before = bytes_of(&project, "ASM-001");

    project
        .command("anna-claude", NOW)
        .env("DOSSIER_ROLE", "worker")
        .args(["confirm", "ASM-001", "--by", BY, "--said-by", "anna"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("a worker decides nothing"));
    assert_eq!(bytes_of(&project, "ASM-001"), before);
}

#[test]
fn a_draft_assumption_cannot_be_confirmed() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "add",
                "assumption",
                "--title",
                "A draft assumption",
                "--section",
                &format!("Assumption={ASSUMPTION}"),
                "--section",
                &format!("Reason={REASON}"),
                "--draft",
            ],
        )
        .success()
        .stdout(contains("ASM-001: added (draft"));
    let before = bytes_of(&project, "ASM-001");

    project
        .run("ben", &["confirm", "ASM-001", "--by", BY])
        .failure()
        .code(1)
        .stderr(contains(
            "error: only an approved assumption can be confirmed; ASM-001 is draft",
        ));
    assert_eq!(bytes_of(&project, "ASM-001"), before);
}

#[test]
fn an_assumption_whose_state_is_not_open_is_refused() {
    let project = Project::new();
    claude_assumes(&project);
    project.stdout("anna", &["set", "ASM-001", "--field", "state=wrong"]);
    let before = bytes_of(&project, "ASM-001");

    project
        .run("ben", &["confirm", "ASM-001", "--by", BY])
        .failure()
        .code(1)
        .stderr(contains(
            "error: only an open assumption can be confirmed; ASM-001 is wrong",
        ));
    assert_eq!(bytes_of(&project, "ASM-001"), before);
}

#[test]
fn only_an_assumption_can_be_confirmed() {
    let project = Project::new();
    project.stdout("anna", &["decide", DECISION, "--reason", REASON]);
    let before = bytes_of(&project, "DEC-001");

    project
        .run("ben", &["confirm", "DEC-001", "--by", BY])
        .failure()
        .code(1)
        .stderr(contains(
            "error: only an assumption can be confirmed; DEC-001 is not one",
        ));
    assert_eq!(bytes_of(&project, "DEC-001"), before);
}

#[test]
fn the_by_text_must_be_one_line_and_not_blank() {
    let project = Project::new();
    claude_assumes(&project);
    let before = bytes_of(&project, "ASM-001");

    project
        .run("ben", &["confirm", "ASM-001", "--by", "   "])
        .failure()
        .code(1)
        .stderr(contains("error: the --by text must not be empty"));
    project
        .run(
            "ben",
            &["confirm", "ASM-001", "--by", "first line\nsecond line"],
        )
        .failure()
        .code(1)
        .stderr(contains("error: the --by text is one line"));
    project
        .run("ben", &["confirm", "ASM-001", "--by", "## A heading"])
        .failure()
        .code(1)
        .stderr(contains("cannot start with"));
    assert_eq!(bytes_of(&project, "ASM-001"), before);
}

#[test]
fn by_is_required() {
    let project = Project::new();
    claude_assumes(&project);
    let before = bytes_of(&project, "ASM-001");

    project
        .run("ben", &["confirm", "ASM-001"])
        .failure()
        .code(2);
    assert_eq!(bytes_of(&project, "ASM-001"), before);
}

#[test]
fn json_output_names_the_id_and_the_state() {
    let project = Project::new();
    claude_assumes(&project);

    let stdout = project.stdout("ben", &["confirm", "ASM-001", "--by", BY, "--json"]);
    let value: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("the output is one JSON object");
    assert_eq!(value["schema"], "dossier.confirm.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["id"], "ASM-001");
    assert_eq!(value["data"]["state"], "confirmed");
    assert_eq!(value["warnings"], serde_json::json!([]));
    // The time of the fixed clock is what the confirmation records.
    assert!(entry_text(&project, "ASM-001").contains(&format!("- confirmed: {NOW} by ben\n")));
}
