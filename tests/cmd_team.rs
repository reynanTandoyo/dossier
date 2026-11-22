//! Runs the built program for `team`: the team settings are shown, `team use` links a project to
//! its team dossier (R-PLACE-2) and refuses a folder that is not one, `team member add|remove`
//! changes who may write to the team dossier (R-TEAM-7), and `team name add|remove` keeps the plain
//! names of people who are not users (D34). Each test runs in its own project folder
//! with the clock fixed by `common::Project` (2026-11-20T10:00:00+01:00). The team dossier is made
//! in a second folder whose name has a space and an umlaut.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// A team dossier made by anna in a folder of its own, with anna as its member. `dir` keeps the
/// folder alive; `dossier` is its DOSSIER folder.
struct Team {
    dir: tempfile::TempDir,
    dossier: PathBuf,
}

/// Makes a team dossier with `init --team-dossier --code TEAM` run as anna in a second folder,
/// then adds anna to its members (the team's settings are set by the tests, not by `team`).
fn team_dossier(project: &Project) -> Team {
    let dir = tempfile::Builder::new()
        .prefix("Team Äpfel ")
        .tempdir()
        .expect("a temporary team folder");
    project
        .command("anna", NOW)
        .current_dir(dir.path())
        .args(["init", "--team-dossier", "--code", "TEAM"])
        .assert()
        .success();
    let dossier = dir.path().join("DOSSIER");
    dossier::config::set_list(&dossier, "members", &["anna".to_string()])
        .expect("anna is a member of the team");
    Team { dir, dossier }
}

/// The path as dossier.toml stores it: with / separators.
fn slashed(path: &Path) -> String {
    path.to_str().expect("a UTF-8 path").replace('\\', "/")
}

/// The records of the project's journal, oldest first, settings events included.
fn journal(project: &Project) -> Vec<dossier::journal::Record> {
    dossier::journal::read(&project.root().join("DOSSIER").join(".state"))
        .expect("the journal reads")
}

/// The settings of the project, as text.
fn settings(project: &Project) -> String {
    project.read("DOSSIER/dossier.toml")
}

#[test]
fn team_use_links_the_project_and_team_shows_it() {
    let project = Project::new();
    let team = team_dossier(&project);
    let path = team.dossier.display().to_string();
    project
        .run("anna", &["team", "use", &path])
        .success()
        .stdout(format!("linked to the team dossier {path}\n"));

    let stored = slashed(&team.dossier);
    assert!(
        settings(&project).contains(&format!("team = \"{stored}\"\n")),
        "dossier.toml names the team: {}",
        settings(&project)
    );
    let cfg = dossier::config::load(&project.root().join("DOSSIER")).expect("settings load");
    assert_eq!(cfg.team.as_deref(), Some(stored.as_str()));

    // The team's member is now a member of the project, and the team dossier is shown.
    project
        .run("anna", &["team"])
        .success()
        .stdout(format!(
            "team dossier: {path}\nmembers: anna\nnames: none\n"
        ))
        .stderr("");
}

#[test]
fn team_use_is_recorded_in_the_journal_with_who_and_when() {
    let project = Project::new();
    let team = team_dossier(&project);
    let path = team.dossier.display().to_string();
    let before = journal(&project).len();
    project.run("anna", &["team", "use", &path]).success();

    let records = journal(&project);
    assert_eq!(records.len(), before + 1);
    let record = records.last().expect("the link is in the journal");
    assert_eq!(record.id, "SETTINGS");
    assert_eq!(record.event, "changed");
    assert_eq!(record.command, "team");
    assert_eq!(record.detail, format!("team {}", slashed(&team.dossier)));
    assert_eq!(record.name, "anna");
    assert_eq!(record.person, "anna");
    assert!(!record.llm);
    assert_eq!(record.time, NOW);
}

#[test]
fn team_use_accepts_the_folder_that_holds_the_dossier() {
    let project = Project::new();
    let team = team_dossier(&project);
    let holder = team.dir.path().display().to_string();
    project
        .run("anna", &["team", "use", &holder])
        .success()
        .stdout(format!(
            "linked to the team dossier {}\n",
            team.dossier.display()
        ));
    let cfg = dossier::config::load(&project.root().join("DOSSIER")).expect("settings load");
    assert_eq!(cfg.team, Some(slashed(&team.dossier)));
}

#[test]
fn team_use_takes_a_relative_path_from_the_project_folder() {
    let project = Project::new();
    let team = team_dossier(&project);
    // The team folder is next to the project folder: ../<its name>/DOSSIER from the project.
    let name = team
        .dir
        .path()
        .file_name()
        .and_then(|name| name.to_str())
        .expect("the team folder name is text");
    let relative = format!("../{name}/DOSSIER");
    project
        .run("anna", &["team", "use", &relative])
        .success()
        .stdout(format!(
            "linked to the team dossier {}\n",
            team.dossier.display()
        ));
    let cfg = dossier::config::load(&project.root().join("DOSSIER")).expect("settings load");
    assert_eq!(cfg.team, Some(slashed(&team.dossier)));
}

#[test]
fn a_folder_that_is_not_a_team_dossier_is_refused_and_nothing_changes() {
    let project = Project::new();
    let other = Project::new();
    let before = journal(&project).len();
    let settings_before = settings(&project);

    // Another project's DOSSIER, in its folder.
    project
        .run(
            "anna",
            &["team", "use", &other.root().display().to_string()],
        )
        .failure()
        .code(1)
        .stderr(contains("is not a team dossier"));
    // A folder with no dossier at all.
    let empty = tempfile::tempdir().expect("an empty folder");
    project
        .run(
            "anna",
            &["team", "use", &empty.path().display().to_string()],
        )
        .failure()
        .code(4)
        .stderr(contains("no team dossier here"));

    assert_eq!(settings(&project), settings_before);
    assert_eq!(journal(&project).len(), before);
}

#[test]
fn a_refused_team_use_as_json_gives_the_bad_value_name() {
    let project = Project::new();
    let other = Project::new();
    let out = project
        .command("anna", NOW)
        .args(["--json", "team", "use", &other.root().display().to_string()])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let line: Value = serde_json::from_slice(&out).expect("one JSON line");
    assert_eq!(line["schema"], "dossier.team.v1");
    assert_eq!(line["ok"], false);
    assert_eq!(line["error"]["code"], "BAD_VALUE");
}

#[test]
fn team_use_with_dry_run_writes_nothing() {
    let project = Project::new();
    let team = team_dossier(&project);
    let before = journal(&project).len();
    let settings_before = settings(&project);
    project
        .command("anna", NOW)
        .args([
            "--dry-run",
            "team",
            "use",
            &team.dossier.display().to_string(),
        ])
        .assert()
        .success()
        .stdout(contains("linked to the team dossier"))
        .stderr(contains("dry run: would write"));
    assert_eq!(settings(&project), settings_before);
    assert_eq!(journal(&project).len(), before);
}

#[test]
fn team_use_inside_a_team_dossier_is_refused() {
    let project = Project::new();
    let team = team_dossier(&project);
    // Run from the team folder: the team dossier is the context, and it is not linked to a team.
    project
        .command("anna", NOW)
        .current_dir(team.dir.path())
        .args(["team", "use", &project.root().display().to_string()])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("run team use in the project folder"));
    assert!(!settings_of(&team.dossier).contains("team ="));
}

/// The settings of a DOSSIER folder, as text.
fn settings_of(dossier: &Path) -> String {
    std::fs::read_to_string(dossier.join("dossier.toml")).expect("the settings read")
}

#[test]
fn team_without_a_link_says_there_is_no_team_dossier() {
    let project = Project::new();
    project
        .run("anna", &["team"])
        .success()
        .stdout("team dossier: none\nmembers: none\nnames: none\n")
        .stderr("");
}

#[test]
fn team_as_json_gives_the_team_dossier_members_and_names() {
    let project = Project::new();
    let team = team_dossier(&project);
    project
        .run(
            "anna",
            &["team", "use", &team.dossier.display().to_string()],
        )
        .success();
    let out = project.stdout("anna", &["--json", "team"]);
    let line: Value = serde_json::from_str(out.trim_end()).expect("one JSON line");
    assert_eq!(line["schema"], "dossier.team.v1");
    assert_eq!(line["ok"], true);
    assert_eq!(line["data"]["team"], slashed(&team.dossier));
    assert_eq!(line["data"]["members"], serde_json::json!(["anna"]));
    assert_eq!(line["data"]["names"], serde_json::json!([]));
    assert_eq!(line["warnings"], serde_json::json!([]));
}

#[test]
fn a_team_dossier_that_cannot_be_reached_is_a_warning_when_shown() {
    let project = Project::new();
    let team = team_dossier(&project);
    let path = team.dossier.display().to_string();
    project.run("anna", &["team", "use", &path]).success();
    std::fs::remove_dir_all(&team.dossier).expect("remove the team dossier");

    project
        .run("anna", &["team"])
        .success()
        .stdout(format!(
            "team dossier: {path}\nmembers: none\nnames: none\n"
        ))
        .stderr(contains("warning: the team dossier cannot be reached"));
}

/// A list of strings, as the settings hold it.
fn owned(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

/// The members of a DOSSIER folder's settings.
fn members_of(dossier: &Path) -> Vec<String> {
    dossier::config::load(dossier)
        .expect("the settings load")
        .members
}

/// The plain names of a DOSSIER folder's settings.
fn names_of(dossier: &Path) -> Vec<String> {
    dossier::config::load(dossier)
        .expect("the settings load")
        .names
}

/// The text of a decision with `id` in the project, parsed.
fn decision_of(project: &Project, id: &str) -> dossier::entry::Entry {
    let text = std::fs::read_to_string(project.entry_file(id)).expect("the decision reads");
    dossier::entry::Entry::parse(&text).expect("the decision parses")
}

/// The records of the team dossier's journal, oldest first.
fn team_journal(team: &Team) -> Vec<dossier::journal::Record> {
    dossier::journal::read(&team.dossier.join(".state")).expect("the team's journal reads")
}

/// Whether git runs here. Without git the git test does nothing, as the tests of tx do.
fn git_runs() -> bool {
    Command::new("git")
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Runs git in `dir` with `args`, and returns its standard output without the line end. Panics
/// when git fails.
fn git_out(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("git's output is UTF-8")
        .trim_end()
        .to_string()
}

/// The project's team dossier, linked as anna. Returns the team dossier with its members.
fn linked_team(project: &Project) -> Team {
    let team = team_dossier(project);
    project
        .run(
            "anna",
            &["team", "use", &team.dossier.display().to_string()],
        )
        .success();
    team
}

#[test]
fn member_add_adds_the_username_to_the_team_and_journals_it_with_who_and_when() {
    let project = Project::new();
    let team = linked_team(&project);
    let before = team_journal(&team).len();

    project
        .run("anna", &["team", "member", "add", "tony"])
        .success()
        .stdout("added member tony\n")
        .stderr("");
    assert_eq!(members_of(&team.dossier), owned(&["anna", "tony"]));
    assert!(
        settings_of(&team.dossier).contains("members = [\"anna\", \"tony\"]"),
        "dossier.toml lists both members: {}",
        settings_of(&team.dossier)
    );

    let records = team_journal(&team);
    assert_eq!(records.len(), before + 1);
    let record = records.last().expect("the change is in the team's journal");
    assert_eq!(record.id, "SETTINGS");
    assert_eq!(record.event, "changed");
    assert_eq!(record.command, "team");
    assert_eq!(record.detail, "member tony added");
    assert_eq!(record.name, "anna");
    assert_eq!(record.person, "anna");
    assert!(!record.llm);
    assert_eq!(record.time, NOW);
}

#[test]
fn member_add_as_json_gives_the_member_and_the_members() {
    let project = Project::new();
    let _team = linked_team(&project);
    let out = project.stdout("anna", &["--json", "team", "member", "add", "tony"]);
    let line: Value = serde_json::from_str(out.trim_end()).expect("one JSON line");
    assert_eq!(line["schema"], "dossier.team.v1");
    assert_eq!(line["ok"], true);
    assert_eq!(line["data"]["member"], "tony");
    assert_eq!(line["data"]["members"], serde_json::json!(["anna", "tony"]));
    assert_eq!(line["warnings"], serde_json::json!([]));
}

#[test]
fn a_member_added_to_the_team_can_write_in_the_project() {
    let project = Project::new();
    let team = linked_team(&project);
    let observation = "An invented observation for the team test.";

    // Tony is not a member yet, so the project does not know him as a writer.
    project
        .run("tony", &["found", observation])
        .failure()
        .code(1);

    project
        .run("anna", &["team", "member", "add", "tony"])
        .success();
    project.run("tony", &["found", observation]).success();
    let last = project
        .entry_journal()
        .last()
        .cloned()
        .expect("tony's entry is in the journal");
    assert_eq!(last.name, "tony");
    assert_eq!(members_of(&team.dossier), owned(&["anna", "tony"]));
}

#[test]
fn member_add_refuses_an_existing_member_or_a_bad_name_and_changes_nothing() {
    let project = Project::new();
    let team = linked_team(&project);
    let before = team_journal(&team).len();
    let settings_before = settings_of(&team.dossier);

    for (args, code) in [
        (vec!["team", "member", "add", "Anna"], "EXISTS"),
        (vec!["team", "member", "add", "tony smith"], "BAD_VALUE"),
        (vec!["team", "member", "add", ""], "BAD_VALUE"),
    ] {
        let out = project
            .command("anna", NOW)
            .arg("--json")
            .args(&args)
            .assert()
            .failure()
            .code(1)
            .get_output()
            .stdout
            .clone();
        let line: Value = serde_json::from_slice(&out).expect("one JSON line");
        assert_eq!(line["ok"], false);
        assert_eq!(line["error"]["code"], code, "for {args:?}");
    }
    assert_eq!(settings_of(&team.dossier), settings_before);
    assert_eq!(team_journal(&team).len(), before);
}

#[test]
fn member_remove_needs_a_person_at_a_terminal() {
    let project = Project::new();
    let team = linked_team(&project);
    project
        .run("anna", &["team", "member", "add", "tony"])
        .success();
    let before = team_journal(&team).len();

    // Run by a test, the command has no terminal: a person is refused.
    project
        .command("anna", NOW)
        .args(["team", "member", "remove", "tony"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("must be run by a person at a terminal"));
    // An LLM is refused even at a terminal (DOSSIER_ANSWERS stands in for one).
    project
        .command("anna-claude", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["team", "member", "remove", "tony"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("is for a person; anna-claude is an LLM"));

    assert_eq!(members_of(&team.dossier), owned(&["anna", "tony"]));
    assert_eq!(team_journal(&team).len(), before);
}

#[test]
fn member_remove_by_a_person_at_a_terminal_removes_the_member_and_journals_it() {
    let project = Project::new();
    let team = linked_team(&project);
    project
        .run("anna", &["team", "member", "add", "tony"])
        .success();
    let before = team_journal(&team).len();

    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["team", "member", "remove", "TONY"])
        .assert()
        .success()
        .stdout("removed member tony\n")
        .stderr("");
    assert_eq!(members_of(&team.dossier), owned(&["anna"]));

    let records = team_journal(&team);
    assert_eq!(records.len(), before + 1);
    let record = records
        .last()
        .expect("the removal is in the team's journal");
    assert_eq!(record.id, "SETTINGS");
    assert_eq!(record.event, "changed");
    assert_eq!(record.command, "team");
    assert_eq!(record.detail, "member tony removed");
    assert_eq!(record.name, "anna");
    assert_eq!(record.time, NOW);

    // A name that is not a member is not found, and nothing changes.
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["team", "member", "remove", "nobody"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("nobody is not a member"));
    assert_eq!(members_of(&team.dossier), owned(&["anna"]));
    assert_eq!(team_journal(&team).len(), before + 1);
}

#[test]
fn the_last_member_is_refused_and_nothing_is_written() {
    let project = Project::new();
    let team = linked_team(&project);
    // anna is the only member of the team; a person at a terminal (DOSSIER_ANSWERS stands in).
    let before = team_journal(&team).len();
    let settings_before = settings_of(&team.dossier);

    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["team", "member", "remove", "anna"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains(
            "anna is the last member; add another member first: dossier team member add <username>",
        ));

    assert_eq!(members_of(&team.dossier), owned(&["anna"]));
    assert_eq!(settings_of(&team.dossier), settings_before);
    assert_eq!(team_journal(&team).len(), before);
}

#[test]
fn a_project_person_who_is_not_a_team_member_cannot_change_the_members() {
    let project = Project::new();
    let team = linked_team(&project);
    // Ben is a person of the project, but not a member of the team.
    let out = project
        .command("ben", NOW)
        .args(["--json", "team", "member", "add", "carl"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let line: Value = serde_json::from_slice(&out).expect("one JSON line");
    assert_eq!(line["error"]["code"], "UNKNOWN_NAME");
    assert_eq!(members_of(&team.dossier), owned(&["anna"]));
}

#[test]
fn member_add_with_dry_run_writes_nothing() {
    let project = Project::new();
    let team = linked_team(&project);
    let before = team_journal(&team).len();
    let settings_before = settings_of(&team.dossier);

    project
        .command("anna", NOW)
        .args(["--dry-run", "team", "member", "add", "tony"])
        .assert()
        .success()
        .stdout(contains("added member tony"))
        .stderr(contains("dry run: would write"));
    assert_eq!(settings_of(&team.dossier), settings_before);
    assert_eq!(team_journal(&team).len(), before);
}

#[test]
fn without_a_team_the_members_change_in_the_project() {
    let project = Project::new();
    project
        .run("anna", &["team", "member", "add", "tony"])
        .success()
        .stdout("added member tony\n");
    let dossier = project.root().join("DOSSIER");
    assert_eq!(members_of(&dossier), owned(&["tony"]));

    let records = journal(&project);
    let record = records
        .last()
        .expect("the change is in the project's journal");
    assert_eq!(record.id, "SETTINGS");
    assert_eq!(record.command, "team");
    assert_eq!(record.detail, "member tony added");
    assert_eq!(record.name, "anna");
}

#[test]
fn a_team_that_cannot_be_reached_refuses_a_member_change() {
    let project = Project::new();
    let team = linked_team(&project);
    std::fs::remove_dir_all(&team.dossier).expect("remove the team dossier");

    project
        .command("anna", NOW)
        .args(["team", "member", "add", "tony"])
        .assert()
        .failure()
        .code(4)
        .stderr(contains("the team dossier cannot be reached"));
}

#[test]
fn a_member_change_in_a_team_repository_is_committed_with_its_subject() {
    if !git_runs() {
        return;
    }
    let project = Project::new();
    let team = linked_team(&project);
    let folder = team.dir.path();
    git_out(folder, &["init", "-q"]);
    git_out(folder, &["config", "user.name", "Test Person"]);
    git_out(
        folder,
        &["config", "user.email", "test.person@example.invalid"],
    );
    git_out(folder, &["config", "commit.gpgsign", "false"]);

    project
        .command("anna", NOW)
        .args(["team", "member", "add", "tony"])
        .assert()
        .success()
        .stderr("");
    assert_eq!(
        git_out(folder, &["log", "-1", "--format=%s"]),
        "team: member tony added"
    );
    assert_eq!(git_out(folder, &["status", "--porcelain"]), "");
}

#[test]
fn name_add_keeps_a_plain_name_with_spaces_and_journals_it_with_who_and_when() {
    let project = Project::new();
    let team = linked_team(&project);
    let before = team_journal(&team).len();

    project
        .run("anna", &["team", "name", "add", "Dr. Weber"])
        .success()
        .stdout("added name Dr. Weber\n")
        .stderr("");
    assert_eq!(names_of(&team.dossier), owned(&["Dr. Weber"]));
    assert!(
        settings_of(&team.dossier).contains("names = [\"Dr. Weber\"]"),
        "dossier.toml lists the name: {}",
        settings_of(&team.dossier)
    );

    let records = team_journal(&team);
    assert_eq!(records.len(), before + 1);
    let record = records.last().expect("the change is in the team's journal");
    assert_eq!(record.id, "SETTINGS");
    assert_eq!(record.event, "changed");
    assert_eq!(record.command, "team");
    assert_eq!(record.detail, "name Dr. Weber added");
    assert_eq!(record.name, "anna");
    assert_eq!(record.person, "anna");
    assert!(!record.llm);
    assert_eq!(record.time, NOW);
}

#[test]
fn name_add_as_json_gives_the_name_and_the_names() {
    let project = Project::new();
    let _team = linked_team(&project);
    let out = project.stdout("anna", &["--json", "team", "name", "add", "Dr. Weber"]);
    let line: Value = serde_json::from_str(out.trim_end()).expect("one JSON line");
    assert_eq!(line["schema"], "dossier.team.v1");
    assert_eq!(line["ok"], true);
    assert_eq!(line["data"]["name"], "Dr. Weber");
    assert_eq!(line["data"]["names"], serde_json::json!(["Dr. Weber"]));
    assert_eq!(line["warnings"], serde_json::json!([]));
}

#[test]
fn a_plain_name_is_the_author_of_a_decision_and_cannot_write() {
    let project = Project::new();
    let team = linked_team(&project);
    project
        .run("anna", &["team", "name", "add", "Dr. Weber"])
        .success();

    // The project knows the team's plain names, so anna-claude may name Dr. Weber as the author.
    project
        .run(
            "anna-claude",
            &[
                "decide",
                "Use the transient runs for the last three points",
                "--reason",
                "The customer asked for transient results",
                "--said-by",
                "Dr. Weber",
            ],
        )
        .success();
    let decision = decision_of(&project, "DEC-001");
    assert_eq!(decision.field("author").as_deref(), Some("Dr. Weber"));
    assert_eq!(decision.field("via").as_deref(), Some("anna-claude"));
    assert_eq!(decision.status(), Some(dossier::entry::Status::Approved));

    // Named as the writer, the plain name is refused, and nothing is written.
    let journal_before = project.entry_journal().len();
    let out = project
        .command("Dr. Weber", NOW)
        .args([
            "--json",
            "found",
            "An invented observation for the team test.",
        ])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let line: Value = serde_json::from_slice(&out).expect("one JSON line");
    assert_eq!(line["error"]["code"], "UNKNOWN_NAME");
    assert_eq!(project.entry_journal().len(), journal_before);
    assert_eq!(names_of(&team.dossier), owned(&["Dr. Weber"]));
}

#[test]
fn name_add_without_a_team_changes_the_project_settings() {
    let project = Project::new();
    project
        .run("anna", &["team", "name", "add", "Dr. Weber"])
        .success()
        .stdout("added name Dr. Weber\n");
    let dossier = project.root().join("DOSSIER");
    assert_eq!(names_of(&dossier), owned(&["Dr. Weber"]));

    let records = journal(&project);
    let record = records
        .last()
        .expect("the change is in the project's journal");
    assert_eq!(record.id, "SETTINGS");
    assert_eq!(record.command, "team");
    assert_eq!(record.detail, "name Dr. Weber added");
    assert_eq!(record.name, "anna");
}

#[test]
fn a_project_plain_name_is_not_copied_into_the_team_dossier() {
    let project = Project::new();
    project
        .run("anna", &["team", "name", "add", "Carl Local"])
        .success();
    let team = linked_team(&project);

    // The project's plain name is known in the project, but the team's list is the team's own.
    project
        .run("anna", &["team", "name", "add", "Dr. Weber"])
        .success();
    assert_eq!(names_of(&team.dossier), owned(&["Dr. Weber"]));
    assert_eq!(
        names_of(&project.root().join("DOSSIER")),
        owned(&["Carl Local"])
    );

    // The team does not list Carl Local, so the team cannot remove it either.
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["team", "name", "remove", "Carl Local"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("Carl Local is not a plain name"));
    assert_eq!(
        names_of(&project.root().join("DOSSIER")),
        owned(&["Carl Local"])
    );
}

#[test]
fn name_add_refuses_a_name_already_listed_or_a_bad_name_and_changes_nothing() {
    let project = Project::new();
    let team = linked_team(&project);
    project
        .run("anna", &["team", "name", "add", "Dr. Weber"])
        .success();
    let before = team_journal(&team).len();
    let settings_before = settings_of(&team.dossier);

    for (args, code) in [
        (vec!["team", "name", "add", "dr. weber"], "EXISTS"),
        (vec!["team", "name", "add", "   "], "BAD_VALUE"),
        (vec!["team", "name", "add", "Dr.\tWeber"], "BAD_VALUE"),
    ] {
        let out = project
            .command("anna", NOW)
            .arg("--json")
            .args(&args)
            .assert()
            .failure()
            .code(1)
            .get_output()
            .stdout
            .clone();
        let line: Value = serde_json::from_slice(&out).expect("one JSON line");
        assert_eq!(line["ok"], false);
        assert_eq!(line["error"]["code"], code, "for {args:?}");
    }
    assert_eq!(settings_of(&team.dossier), settings_before);
    assert_eq!(team_journal(&team).len(), before);
}

#[test]
fn name_remove_needs_a_person_at_a_terminal() {
    let project = Project::new();
    let team = linked_team(&project);
    project
        .run("anna", &["team", "name", "add", "Dr. Weber"])
        .success();
    let before = team_journal(&team).len();

    // Run by a test, the command has no terminal: a person is refused.
    project
        .command("anna", NOW)
        .args(["team", "name", "remove", "Dr. Weber"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("must be run by a person at a terminal"));
    // An LLM is refused even at a terminal (DOSSIER_ANSWERS stands in for one).
    project
        .command("anna-claude", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["team", "name", "remove", "Dr. Weber"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("is for a person; anna-claude is an LLM"));

    assert_eq!(names_of(&team.dossier), owned(&["Dr. Weber"]));
    assert_eq!(team_journal(&team).len(), before);
}

#[test]
fn name_remove_by_a_person_at_a_terminal_removes_the_name_and_journals_it() {
    let project = Project::new();
    let team = linked_team(&project);
    project
        .run("anna", &["team", "name", "add", "Dr. Weber"])
        .success();
    let before = team_journal(&team).len();

    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["team", "name", "remove", "dr. weber"])
        .assert()
        .success()
        .stdout("removed name Dr. Weber\n")
        .stderr("");
    assert_eq!(names_of(&team.dossier), owned(&[]));

    let records = team_journal(&team);
    assert_eq!(records.len(), before + 1);
    let record = records
        .last()
        .expect("the removal is in the team's journal");
    assert_eq!(record.id, "SETTINGS");
    assert_eq!(record.event, "changed");
    assert_eq!(record.command, "team");
    assert_eq!(record.detail, "name Dr. Weber removed");
    assert_eq!(record.name, "anna");
    assert_eq!(record.time, NOW);

    // The removed name is no longer known, so it cannot be the author of a decision.
    project
        .run(
            "anna-claude",
            &[
                "decide",
                "Use the transient runs for the last three points",
                "--reason",
                "The customer asked for transient results",
                "--said-by",
                "Dr. Weber",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("no person Dr. Weber in dossier.toml"));

    // A name that is not listed is not found, and nothing changes.
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["team", "name", "remove", "nobody"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("nobody is not a plain name"));
    assert_eq!(names_of(&team.dossier), owned(&[]));
    assert_eq!(team_journal(&team).len(), before + 1);
}

#[test]
fn a_project_person_who_is_not_a_team_member_cannot_add_a_name() {
    let project = Project::new();
    let team = linked_team(&project);
    let out = project
        .command("ben", NOW)
        .args(["--json", "team", "name", "add", "Carl"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let line: Value = serde_json::from_slice(&out).expect("one JSON line");
    assert_eq!(line["error"]["code"], "UNKNOWN_NAME");
    assert_eq!(names_of(&team.dossier), owned(&[]));
}

#[test]
fn name_add_with_dry_run_writes_nothing() {
    let project = Project::new();
    let team = linked_team(&project);
    let before = team_journal(&team).len();
    let settings_before = settings_of(&team.dossier);

    project
        .command("anna", NOW)
        .args(["--dry-run", "team", "name", "add", "Dr. Weber"])
        .assert()
        .success()
        .stdout(contains("added name Dr. Weber"))
        .stderr(contains("dry run: would write"));
    assert_eq!(settings_of(&team.dossier), settings_before);
    assert_eq!(team_journal(&team).len(), before);
}

#[test]
fn a_name_change_in_a_team_repository_is_committed_with_its_subject() {
    if !git_runs() {
        return;
    }
    let project = Project::new();
    let team = linked_team(&project);
    let folder = team.dir.path();
    git_out(folder, &["init", "-q"]);
    git_out(folder, &["config", "user.name", "Test Person"]);
    git_out(
        folder,
        &["config", "user.email", "test.person@example.invalid"],
    );
    git_out(folder, &["config", "commit.gpgsign", "false"]);

    project
        .command("anna", NOW)
        .args(["team", "name", "add", "Dr. Weber"])
        .assert()
        .success()
        .stderr("");
    assert_eq!(
        git_out(folder, &["log", "-1", "--format=%s"]),
        "team: name Dr. Weber added"
    );
    assert_eq!(git_out(folder, &["status", "--porcelain"]), "");
}
