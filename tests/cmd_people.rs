//! Runs the built program for `people`: adding a person and LLMs, listing them, and the refusals.
//! Each test runs in its own temporary folder after `init`, with the clock fixed.

use assert_cmd::Command;
use predicates::str::contains;
use std::path::Path;

/// The program run in `folder` with `args`, not yet checked. The agent names are removed from
/// the environment, and the clock is fixed.
fn dossier(folder: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("dossier")
        .unwrap()
        .current_dir(folder)
        .env("DOSSIER_NOW", "2026-11-21T09:00:00+01:00")
        .env_remove("DOSSIER_AGENT")
        .env_remove("TODO_AGENT")
        .args(args)
        .assert()
}

/// A new temporary folder with `dossier init --code P-001` run in it.
fn project() -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    dossier(folder.path(), &["init", "--code", "P-001"]).success();
    folder
}

/// The text of dossier.toml in the dossier of `folder`.
fn settings(folder: &Path) -> String {
    std::fs::read_to_string(folder.join("DOSSIER").join("dossier.toml")).unwrap()
}

/// The standard output of a successful run, as text.
fn stdout_of(folder: &Path, args: &[&str]) -> String {
    let stdout = dossier(folder, args).success().get_output().stdout.clone();
    String::from_utf8(stdout).unwrap()
}

#[test]
fn add_one_llm_then_a_second_later_lists_one_person_with_both() {
    let folder = project();
    dossier(
        folder.path(),
        &["people", "add", "anna", "--llm", "anna-claude"],
    )
    .success()
    .stdout("added anna with anna-claude\n");
    dossier(
        folder.path(),
        &["people", "add", "anna", "--llm", "anna-codex"],
    )
    .success()
    .stdout("added anna with anna-codex\n");
    assert_eq!(
        stdout_of(folder.path(), &["people", "list"]),
        "anna: anna-claude, anna-codex\n"
    );
    let cfg = dossier::config::load(&folder.path().join("DOSSIER")).unwrap();
    assert_eq!(cfg.people.len(), 1);
    assert_eq!(cfg.people[0].name, "anna");
    assert_eq!(cfg.people[0].llms, ["anna-claude", "anna-codex"]);
}

#[test]
fn several_llm_flags_in_one_add_are_all_kept_in_order() {
    let folder = project();
    dossier(
        folder.path(),
        &[
            "people",
            "add",
            "ben",
            "--llm",
            "ben-codex",
            "--llm",
            "ben-claude",
        ],
    )
    .success()
    .stdout("added ben with ben-codex, ben-claude\n");
    let cfg = dossier::config::load(&folder.path().join("DOSSIER")).unwrap();
    assert_eq!(cfg.people[0].llms, ["ben-codex", "ben-claude"]);
}

#[test]
fn a_person_without_llms_is_listed_with_nothing_after_the_colon() {
    let folder = project();
    dossier(folder.path(), &["people", "add", "ben"])
        .success()
        .stdout("added ben\n");
    assert_eq!(stdout_of(folder.path(), &["people", "list"]), "ben:\n");
}

#[test]
fn people_are_listed_in_the_order_they_were_added() {
    let folder = project();
    dossier(
        folder.path(),
        &["people", "add", "anna", "--llm", "anna-claude"],
    )
    .success();
    dossier(
        folder.path(),
        &["people", "add", "ben", "--llm", "ben-codex"],
    )
    .success();
    assert_eq!(
        stdout_of(folder.path(), &["people", "list"]),
        "anna: anna-claude\nben: ben-codex\n"
    );
}

#[test]
fn list_with_no_people_prints_nothing_and_succeeds() {
    let folder = project();
    assert_eq!(stdout_of(folder.path(), &["people", "list"]), "");
}

#[test]
fn adding_keeps_every_other_line_of_dossier_toml() {
    let folder = project();
    let path = folder.path().join("DOSSIER").join("dossier.toml");
    let custom = "format = 1\n\
                  code = \"P-001\"\n\
                  # the team's own notes stay as they are\n\
                  \n\
                  [tags]\n\
                  machine = [\"axial compressor\"]\n\
                  analysis = []\n\
                  software = []\n\
                  \n\
                  [limits]\n\
                  draft_days = 3\n";
    std::fs::write(&path, custom).unwrap();
    dossier(
        folder.path(),
        &["people", "add", "anna", "--llm", "anna-claude"],
    )
    .success();
    assert_eq!(
        settings(folder.path()),
        format!("{custom}\n[[people]]\nname = \"anna\"\nllms = [\"anna-claude\"]\n")
    );
    let cfg = dossier::config::load(&folder.path().join("DOSSIER")).unwrap();
    assert_eq!(cfg.draft_days, 3);
    assert_eq!(cfg.tags.machine, ["axial compressor"]);
    assert_eq!(cfg.people[0].llms, ["anna-claude"]);
}

#[test]
fn a_person_name_with_a_space_is_refused_and_nothing_is_written() {
    let folder = project();
    let before = settings(folder.path());
    dossier(
        folder.path(),
        &["people", "add", "Anna Berg", "--llm", "anna-claude"],
    )
    .code(1)
    .stderr(contains(
        "error: a person's name must not be empty or contain a space",
    ));
    assert_eq!(settings(folder.path()), before);
}

#[test]
fn a_person_name_with_a_tab_is_refused_too() {
    let folder = project();
    dossier(folder.path(), &["people", "add", "Anna\tBerg"])
        .code(1)
        .stderr(contains("error: a person's name must not be empty"));
}

#[test]
fn an_empty_person_name_is_refused() {
    let folder = project();
    dossier(folder.path(), &["people", "add", ""])
        .code(1)
        .stderr(contains("error: a person's name must not be empty"));
    assert!(!settings(folder.path()).contains("[[people]]"));
}

#[test]
fn an_llm_name_with_a_space_is_refused_and_no_llm_of_that_add_is_written() {
    let folder = project();
    let before = settings(folder.path());
    dossier(
        folder.path(),
        &[
            "people",
            "add",
            "anna",
            "--llm",
            "anna-codex",
            "--llm",
            "anna claude",
        ],
    )
    .code(1)
    .stderr(contains(
        "error: an LLM's name must not be empty or contain a space",
    ));
    assert_eq!(settings(folder.path()), before);
}

#[test]
fn a_person_added_without_a_dossier_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    dossier(folder.path(), &["people", "add", "anna"])
        .code(4)
        .stderr(contains("error: no DOSSIER folder found here or above"));
}

#[test]
fn a_missing_name_or_subcommand_is_a_usage_error() {
    let folder = project();
    dossier(folder.path(), &["people"]).code(2);
    dossier(folder.path(), &["people", "add"]).code(2);
}

#[test]
fn dry_run_writes_nothing_and_says_which_file_it_would_write() {
    let folder = project();
    let before = settings(folder.path());
    dossier(
        folder.path(),
        &["people", "add", "anna", "--llm", "anna-claude", "--dry-run"],
    )
    .success()
    .stdout("added anna with anna-claude\n")
    .stderr(contains("warning: dry run: would write"))
    .stderr(contains("dossier.toml"));
    assert_eq!(settings(folder.path()), before);
}

#[test]
fn add_as_json_names_the_person_and_the_llms() {
    let folder = project();
    dossier(
        folder.path(),
        &["people", "add", "anna", "--llm", "anna-claude", "--json"],
    )
    .success()
    .stdout(contains("\"schema\":\"dossier.people.v1\""))
    .stdout(contains("\"ok\":true"))
    .stdout(contains("\"person\":\"anna\""))
    .stdout(contains("\"llms\":[\"anna-claude\"]"));
}

#[test]
fn list_as_json_is_a_list_of_name_and_llms() {
    let folder = project();
    dossier(
        folder.path(),
        &["people", "add", "anna", "--llm", "anna-claude"],
    )
    .success();
    dossier(folder.path(), &["people", "add", "ben"]).success();
    let out = stdout_of(folder.path(), &["people", "list", "--json"]);
    let value: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(value["schema"], "dossier.people.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"],
        serde_json::json!([
            {"name": "anna", "llms": ["anna-claude"]},
            {"name": "ben", "llms": []}
        ])
    );
}

#[test]
fn a_refused_add_as_json_gives_the_error_name() {
    let folder = project();
    dossier(folder.path(), &["people", "add", "Anna Berg", "--json"])
        .code(1)
        .stdout(contains("\"ok\":false"))
        .stdout(contains("\"code\":\"BAD_VALUE\""));
}

#[test]
fn a_folder_with_spaces_and_umlauts_works() {
    let folder = tempfile::tempdir().unwrap();
    let project_folder = folder.path().join("Projekt Übersicht");
    std::fs::create_dir(&project_folder).unwrap();
    dossier(&project_folder, &["init", "--code", "P-001"]).success();
    dossier(
        &project_folder,
        &["people", "add", "anna", "--llm", "anna-claude"],
    )
    .success();
    assert_eq!(
        stdout_of(&project_folder, &["people", "list"]),
        "anna: anna-claude\n"
    );
}

#[test]
fn a_relative_dossier_option_is_taken_from_the_current_folder() {
    let folder = tempfile::tempdir().unwrap();
    let project_folder = folder.path().join("Projekt");
    std::fs::create_dir(&project_folder).unwrap();
    dossier(&project_folder, &["init", "--code", "P-001"]).success();
    dossier(
        folder.path(),
        &["--dossier", "Projekt/DOSSIER", "people", "add", "anna"],
    )
    .success()
    .stdout("added anna\n");
    let cfg = dossier::config::load(&project_folder.join("DOSSIER")).unwrap();
    assert_eq!(cfg.people[0].name, "anna");
}
