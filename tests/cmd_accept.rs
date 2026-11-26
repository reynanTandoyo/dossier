//! Runs the built program for `accept`: a decider puts a proposal in force, at a terminal or, for
//! an LLM, with `--said-by <person>` (D57). Only a person named in `deciders` may accept, and only
//! a proposal can be accepted. Each test runs in its own project folder (common::Project), with the
//! clock fixed at 2026-11-20T10:00:00+01:00 unless a test says otherwise.

mod common;

use common::Project;
use dossier::entry::{Entry, Status};
use predicates::str::contains;

/// The fixed clock of `common::Project`: when the proposals are made.
const NOW: &str = "2026-11-20T10:00:00+01:00";
/// The time of the acceptance in the tests that accept, a day later.
const LATER: &str = "2026-11-21T09:30:00+01:00";

/// Writes `deciders = ["anna"]` into dossier.toml, after the `code` line, so that it is a
/// top-level key before the [tags] table (format.md §19).
fn only_anna_decides(project: &Project) {
    let text = project.read("DOSSIER/dossier.toml");
    let mut edited = String::new();
    for line in text.split_inclusive('\n') {
        edited.push_str(line);
        if line.starts_with("code = ") {
            edited.push_str("deciders = [\"anna\"]\n");
        }
    }
    assert!(edited.contains("deciders = [\"anna\"]\n"), "{edited}");
    project.write("DOSSIER/dossier.toml", &edited);
}

/// Ben proposes a decision with `decide`. With deciders named and Ben not one of them, the
/// decision is a proposal (R-WRITE-13). Returns its ID.
fn ben_decides(project: &Project) -> &'static str {
    project.stdout(
        "ben",
        &[
            "decide",
            "Use the transient runs for the last three points.",
            "--reason",
            "Steady runs converged where the transient run already showed rotating stall.",
        ],
    );
    "DEC-001"
}

/// Runs `accept` as `agent` at `now`, answering `answers` (DOSSIER_ANSWERS) to the question.
fn accept_with(
    project: &Project,
    agent: &str,
    now: &str,
    answers: &str,
    id: &str,
) -> assert_cmd::assert::Assert {
    project
        .command(agent, now)
        .env("DOSSIER_ANSWERS", answers)
        .args(["accept", id])
        .assert()
}

/// The entry file with `id`, read and parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    let text = std::fs::read_to_string(project.entry_file(id)).expect("the entry file is read");
    Entry::parse(&text).expect("the entry parses")
}

/// The bytes of the entry file with `id`.
fn bytes_of(project: &Project, id: &str) -> Vec<u8> {
    std::fs::read(project.entry_file(id)).expect("the entry file is read")
}

#[test]
fn a_decider_accepts_a_proposal_and_it_is_in_force_from_the_acceptance() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_decides(&project);
    assert_eq!(entry_of(&project, id).status(), Some(Status::Proposed));

    accept_with(&project, "anna", LATER, "yes", id)
        .success()
        .stdout("DEC-001: accepted\n")
        .stderr(contains("# DEC-001 ·"));

    let entry = entry_of(&project, id);
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(
        entry.field("approved").as_deref(),
        Some(format!("{LATER} by anna").as_str())
    );
    assert_eq!(entry.field("valid-from").as_deref(), Some(LATER));
    // The proposal keeps its author and its creation time; the acceptance is a history line.
    assert_eq!(entry.field("author").as_deref(), Some("ben"));
    assert_eq!(entry.field("created").as_deref(), Some(NOW));
    assert_eq!(
        entry.history(),
        [
            format!("{NOW} ben: created"),
            format!("{NOW} ben: proposed"),
            format!("{LATER} anna: accepted"),
        ]
    );

    // The journal has the acceptance, by the person, with the command that did it.
    let journal = project.entry_journal();
    let last = journal.last().expect("a journal line");
    assert_eq!(last.command, "accept");
    assert_eq!(last.id, "DEC-001");
    assert_eq!(last.event, "accepted");
    assert_eq!(last.person, "anna");
    assert!(!last.llm);

    // It is now an approved decision: listed without a mark, not among the drafts.
    let list = project.stdout("anna", &["list"]);
    assert!(list.contains("DEC-001"), "list shows: {list}");
    assert!(!list.contains("[proposed]"), "list shows: {list}");
}

#[test]
fn an_answer_other_than_yes_changes_nothing() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_decides(&project);
    let before = bytes_of(&project, id);
    let journal_before = project.entry_journal().len();

    accept_with(&project, "anna", LATER, "no", id)
        .success()
        .stdout("DEC-001: nothing changed\n");

    assert_eq!(bytes_of(&project, id), before, "the file is unchanged");
    assert_eq!(project.entry_journal().len(), journal_before);
    assert_eq!(entry_of(&project, id).status(), Some(Status::Proposed));
}

#[test]
fn ben_is_not_a_decider_so_he_is_refused_and_the_proposal_stays() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_decides(&project);
    let before = bytes_of(&project, id);

    accept_with(&project, "ben", LATER, "yes", id)
        .failure()
        .code(1)
        .stderr(contains("ben is not in deciders"));

    assert_eq!(bytes_of(&project, id), before, "the file is unchanged");
    assert_eq!(entry_of(&project, id).status(), Some(Status::Proposed));
}

#[test]
fn anna_claude_is_an_llm_so_it_is_refused_even_though_its_person_decides() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_decides(&project);
    let before = bytes_of(&project, id);

    accept_with(&project, "anna-claude", LATER, "yes", id)
        .failure()
        .code(1)
        .stderr(contains("--said-by anna"));

    assert_eq!(bytes_of(&project, id), before, "the file is unchanged");
    assert_eq!(entry_of(&project, id).status(), Some(Status::Proposed));
}

#[test]
fn without_a_terminal_nothing_is_asked_and_nothing_is_written() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_decides(&project);
    let before = bytes_of(&project, id);

    project
        .command("anna", LATER)
        .env_remove("DOSSIER_ANSWERS")
        .args(["accept", id])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("must be run by a person at a terminal"));

    assert_eq!(bytes_of(&project, id), before, "the file is unchanged");
}

#[test]
fn an_approved_decision_is_refused_because_only_a_proposal_can_be_accepted() {
    let project = Project::new();
    only_anna_decides(&project);
    // Anna is a decider, so her decision is approved at once and is not a proposal.
    project.stdout(
        "anna",
        &[
            "decide",
            "Use the transient runs for the last three points.",
            "--reason",
            "Steady runs converged.",
        ],
    );
    let before = bytes_of(&project, "DEC-001");

    accept_with(&project, "anna", LATER, "yes", "DEC-001")
        .failure()
        .code(1)
        .stderr(contains(
            "only a proposal can be accepted; DEC-001 is approved",
        ));

    assert_eq!(
        bytes_of(&project, "DEC-001"),
        before,
        "the file is unchanged"
    );
}

#[test]
fn an_unknown_id_is_not_found() {
    let project = Project::new();
    only_anna_decides(&project);
    accept_with(&project, "anna", LATER, "yes", "DEC-009")
        .failure()
        .code(1)
        .stderr(contains("no entry DEC-009"));
}

#[test]
fn with_no_deciders_named_anyone_may_accept_a_proposal() {
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
    assert_eq!(
        entry_of(&project, "DEC-001").status(),
        Some(Status::Proposed)
    );

    accept_with(&project, "ben", LATER, "yes", "DEC-001")
        .success()
        .stdout("DEC-001: accepted\n");

    let entry = entry_of(&project, "DEC-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(
        entry.field("approved").as_deref(),
        Some(format!("{LATER} by ben").as_str())
    );
}

#[test]
fn json_gives_the_id_the_status_and_whether_it_was_accepted() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_decides(&project);

    let declined = project
        .command("anna", LATER)
        .env("DOSSIER_ANSWERS", "no")
        .args(["accept", id, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&declined).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.accept.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["id"], "DEC-001");
    assert_eq!(value["data"]["status"], "proposed");
    assert_eq!(value["data"]["accepted"], false);

    let accepted = project
        .command("anna", LATER)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["accept", id, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&accepted).expect("one line of JSON");
    assert_eq!(value["data"]["status"], "approved");
    assert_eq!(value["data"]["accepted"], true);
}

#[test]
fn a_refusal_in_json_names_its_code() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_decides(&project);

    project
        .command("ben", LATER)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["accept", id, "--json"])
        .assert()
        .failure()
        .code(1)
        .stdout(contains("\"code\":\"REFUSED\""));
}

#[test]
fn an_llm_without_said_by_is_refused_and_nothing_is_written() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_decides(&project);
    let before = bytes_of(&project, id);

    accept_with(&project, "anna-claude", LATER, "yes", id)
        .failure()
        .code(1)
        .stderr(contains("--said-by anna"));

    assert_eq!(bytes_of(&project, id), before, "the file is unchanged");
    assert_eq!(entry_of(&project, id).status(), Some(Status::Proposed));
}

#[test]
fn an_llm_with_said_by_accepts_without_a_terminal_or_a_question() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_decides(&project);
    // No answers are prepared and no terminal is attached, so nothing can be asked: the proposal
    // is accepted as the person decided (D57).
    project
        .command("anna-claude", LATER)
        .args(["accept", id, "--said-by", "anna"])
        .assert()
        .success()
        .stdout("DEC-001: accepted\n")
        .stderr("");

    let entry = entry_of(&project, id);
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(
        entry.field("approved").as_deref(),
        Some(format!("{LATER} by anna").as_str())
    );
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some(format!("{LATER} anna via anna-claude: accepted").as_str())
    );

    let journal = project.entry_journal();
    let last = journal.last().expect("a journal line");
    assert_eq!(last.command, "accept");
    assert_eq!(last.id, "DEC-001");
    assert_eq!(last.event, "accepted");
    assert_eq!(last.person, "anna");
    assert_eq!(last.name, "anna-claude");
    assert!(last.llm);
}

#[test]
fn an_llm_with_said_by_for_another_person_is_refused_and_nothing_is_written() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_decides(&project);
    let before = bytes_of(&project, id);

    project
        .command("anna-claude", LATER)
        .args(["accept", id, "--said-by", "ben"])
        .assert()
        .code(1)
        .stderr(contains("acts for anna only"));

    assert_eq!(bytes_of(&project, id), before, "the file is unchanged");
    assert_eq!(entry_of(&project, id).status(), Some(Status::Proposed));
}

#[test]
fn a_worker_session_decides_nothing_even_with_said_by() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_decides(&project);
    let before = bytes_of(&project, id);

    project
        .command("anna-claude", LATER)
        .env("DOSSIER_ROLE", "worker")
        .args(["accept", id, "--said-by", "anna"])
        .assert()
        .code(1)
        .stderr(contains("a worker decides nothing"));

    assert_eq!(bytes_of(&project, id), before, "the file is unchanged");
    assert_eq!(entry_of(&project, id).status(), Some(Status::Proposed));
}

#[test]
fn an_llm_whose_person_is_not_a_decider_is_refused_even_with_said_by() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_decides(&project);
    // Only ben is named in deciders from here on, so anna is not one.
    let text = project
        .read("DOSSIER/dossier.toml")
        .replace("deciders = [\"anna\"]", "deciders = [\"ben\"]");
    project.write("DOSSIER/dossier.toml", &text);
    let before = bytes_of(&project, id);

    project
        .command("anna-claude", LATER)
        .args(["accept", id, "--said-by", "anna"])
        .assert()
        .code(1)
        .stderr(contains("anna is not in deciders"));

    assert_eq!(bytes_of(&project, id), before, "the file is unchanged");
    assert_eq!(entry_of(&project, id).status(), Some(Status::Proposed));
}
