//! Runs the built program for `schema`: the JSON description of the commands, the envelope, the
//! exit codes and the error codes, as text and, with `--json`, in the usual envelope. The command
//! needs no dossier, so each test runs in an empty temporary folder.

mod common;

use common::Project;
use serde_json::{Value, json};

/// A temporary folder with no dossier in it.
fn empty_folder() -> Project {
    Project {
        dir: tempfile::TempDir::new().expect("a temporary folder"),
    }
}

/// The text of `schema`, parsed as JSON.
fn schema_value(project: &Project) -> Value {
    let text = project.stdout("", &["schema"]);
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("not JSON ({error}): {text}"))
}

/// The `commands` array of a description.
fn commands(value: &Value) -> &Vec<Value> {
    value["commands"].as_array().expect("an array of commands")
}

#[test]
fn the_text_is_one_json_object_with_four_parts_and_needs_no_dossier() {
    let project = empty_folder();
    let value = schema_value(&project);
    let mut keys: Vec<&str> = value
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, ["commands", "envelope", "error_codes", "exit_codes"]);
    assert!(!project.root().join("DOSSIER").exists());
}

#[test]
fn commands_follow_the_help_overview_one_for_each_line() {
    let project = empty_folder();
    let value = schema_value(&project);
    let help = project.stdout("", &["help"]);
    let lines: Vec<&str> = help.lines().collect();
    let commands = commands(&value);
    assert_eq!(commands.len(), lines.len());
    for (command, line) in commands.iter().zip(lines) {
        let name = command["name"].as_str().expect("a name");
        let description = command["description"].as_str().expect("a description");
        assert_eq!(line.split_whitespace().next(), Some(name), "{line}");
        assert!(line.ends_with(description), "{line}");
        assert_eq!(command["schema"], format!("dossier.{name}.v1"));
    }
}

#[test]
fn brief_has_the_schema_dossier_brief_v1() {
    let value = schema_value(&empty_folder());
    let brief = commands(&value)
        .iter()
        .find(|command| command["name"] == "brief")
        .expect("brief is listed");
    assert_eq!(brief["schema"], "dossier.brief.v1");
}

#[test]
fn no_dossier_is_an_error_code_with_exit_four() {
    let value = schema_value(&empty_folder());
    let no_dossier = value["error_codes"]
        .as_array()
        .expect("an array of error codes")
        .iter()
        .find(|code| code["name"] == "NO_DOSSIER")
        .expect("NO_DOSSIER is listed");
    assert_eq!(no_dossier["exit"].as_u64(), Some(4));
}

#[test]
fn exit_codes_name_the_six_statuses() {
    let value = schema_value(&empty_folder());
    assert_eq!(
        value["exit_codes"],
        json!({
            "0": "done",
            "1": "refused by a rule, or problems found",
            "2": "wrong usage",
            "3": "busy, try again",
            "4": "file or settings problem",
            "5": "a needed program or connection failed",
        })
    );
}

#[test]
fn envelope_names_the_parts_of_the_json_output() {
    let value = schema_value(&empty_folder());
    assert_eq!(
        value["envelope"],
        json!({
            "ok": "bool",
            "data": "object",
            "warnings": "array of strings",
            "error": {"code": "string", "message": "string"},
        })
    );
}

#[test]
fn json_output_wraps_the_same_description_in_the_envelope() {
    let project = empty_folder();
    let text = schema_value(&project);
    let line = project.stdout("", &["schema", "--json"]);
    assert_eq!(line.matches('\n').count(), 1, "{line}");
    let envelope: Value = serde_json::from_str(&line).expect("one JSON object");
    assert_eq!(envelope["schema"], "dossier.schema.v1");
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["data"], text);
    assert_eq!(envelope["warnings"], json!([]));
}
