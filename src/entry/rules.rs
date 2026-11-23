//! What an entry needs before it can be approved.

use super::{Entry, Kind};

/// One thing that is missing or wrong.
pub struct Problem {
    pub what: String,
}

/// The values of `confidence` of a lesson (format.md §5).
const CONFIDENCE: [&str; 4] = ["measured", "simulated", "experience", "guess"];

/// The values of `state` of an assumption (format.md §5).
const STATES: [&str; 3] = ["open", "confirmed", "wrong"];

/// The values of `state` of an investigation (format.md §28).
const INVESTIGATION_STATES: [&str; 3] = ["open", "solved", "given up"];

/// The values of `outcome` of a finding (format.md §5).
const OUTCOMES: [&str; 1] = ["dead end"];

/// The sections that every entry of `kind` must have.
pub fn required_sections(kind: Kind) -> &'static [&'static str] {
    match kind {
        Kind::Decision => &["Decision", "Reason"],
        Kind::Assumption => &["Assumption", "Reason"],
        Kind::Finding => &["Observation"],
        Kind::Lesson => &["Statement", "Valid for", "Evidence"],
        Kind::Usage => &[],
        Kind::Howto => &["Purpose", "Steps"],
        Kind::Tool => &["Purpose"],
        Kind::Investigation => &["Problem"],
    }
}

/// The order of the sections of an entry of `kind` (format.md §8, and §15 for a usage record).
pub fn section_order(kind: Kind) -> &'static [&'static str] {
    match kind {
        Kind::Decision => &["Decision", "Reason", "Rejected"],
        Kind::Assumption => &["Assumption", "Reason", "Affects", "Confirmation"],
        Kind::Finding => &["Observation", "Evidence"],
        Kind::Lesson => &["Statement", "Valid for", "Evidence"],
        Kind::Usage => &["Parameters", "Outputs", "Deviations", "Notes"],
        Kind::Howto => &[
            "Purpose",
            "Steps",
            "Settings",
            "Traps",
            "Tested with",
            "Open remarks",
        ],
        Kind::Tool => &[
            "Purpose",
            "How to run",
            "Inputs",
            "Outputs",
            "Traps",
            "Tested with",
        ],
        Kind::Investigation => &["Problem", "Attempts", "Solution"],
    }
}

/// What is missing for approval: required sections, `confidence` for a lesson with one of the
/// four values, a valid `state` for an assumption or an investigation, a valid `outcome` if
/// present, and the fields `software`, `version` and `task` of a usage record (format.md §15).
///
/// The problems come in this order: the required sections of the kind, then the field of the
/// kind. A section with no text counts as missing. A field with an empty value counts as
/// missing, except `outcome`, where an empty value counts as absent. Values are compared without
/// regard to case, as `Kind::parse` and `Status::parse` compare. An entry without a `kind`, or
/// with an unknown one, gives only the problem about the kind.
pub fn problems(entry: &Entry) -> Vec<Problem> {
    let Some(kind) = entry.kind() else {
        let what = match filled_field(entry, "kind") {
            Some(_) => "kind must be decision, assumption, finding or lesson",
            None => "missing field kind",
        };
        return vec![problem(what)];
    };
    let mut found = Vec::new();
    for name in required_sections(kind) {
        if !entry
            .section(name)
            .is_some_and(|body| !body.trim().is_empty())
        {
            found.push(problem(format!("missing section {name}")));
        }
    }
    match kind {
        Kind::Decision => {}
        Kind::Assumption => {
            if !filled_field(entry, "state").is_some_and(|state| is_one_of(&state, &STATES)) {
                found.push(problem("state must be open, confirmed or wrong"));
            }
        }
        Kind::Finding => {
            if filled_field(entry, "outcome").is_some_and(|outcome| !is_one_of(&outcome, &OUTCOMES))
            {
                found.push(problem("outcome must be dead end"));
            }
        }
        Kind::Lesson => match filled_field(entry, "confidence") {
            None => found.push(problem("missing field confidence")),
            Some(confidence) if !is_one_of(&confidence, &CONFIDENCE) => found.push(problem(
                "confidence must be measured, simulated, experience or guess",
            )),
            Some(_) => {}
        },
        Kind::Usage => {
            for key in ["software", "version", "task"] {
                if filled_field(entry, key).is_none() {
                    found.push(problem(format!("missing field {key}")));
                }
            }
        }
        Kind::Howto => {
            if filled_field(entry, "software").is_none() {
                found.push(problem("missing field software"));
            }
        }
        Kind::Tool => {}
        Kind::Investigation => {
            if !filled_field(entry, "state")
                .is_some_and(|state| is_one_of(&state, &INVESTIGATION_STATES))
            {
                found.push(problem("state must be open, solved or given up"));
            }
        }
    }
    found
}

/// A problem with the text `what`.
fn problem(what: impl Into<String>) -> Problem {
    Problem { what: what.into() }
}

/// The value of a field, or None when the field is missing or its value is empty.
fn filled_field(entry: &Entry, key: &str) -> Option<String> {
    entry.field(key).filter(|value| !value.is_empty())
}

/// Whether `value` is one of `allowed`, without regard to case.
fn is_one_of(value: &str, allowed: &[&str]) -> bool {
    allowed
        .iter()
        .any(|option| value.eq_ignore_ascii_case(option))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::new::{Draft, render};

    /// The messages of the problems of an entry text.
    fn whats(text: &str) -> Vec<String> {
        let entry = Entry::parse(text).expect("the test entry parses");
        problems(&entry)
            .into_iter()
            .map(|found| found.what)
            .collect()
    }

    /// `text` with `old` replaced by `new`. `old` must be in the text, so that a test never
    /// passes on an edit that did not happen.
    fn replacing(text: &str, old: &str, new: &str) -> String {
        assert!(text.contains(old), "{old:?} is not in the test entry");
        text.replace(old, new)
    }

    /// A complete decision with its optional Rejected section. Invented content.
    const DECISION: &str = r#"<!-- dossier-format: 1 -->
# DEC-101 · Invented decision

- kind: decision
- status: draft

## Decision

The first setup is used.

## Reason

It was the one that was checked.

## Rejected

The second setup was slower.
"#;

    /// A complete assumption. Invented content.
    const ASSUMPTION: &str = r#"<!-- dossier-format: 1 -->
# ASM-101 · Invented assumption

- kind: assumption
- status: draft
- state: open

## Assumption

The inlet is quiet.

## Reason

Nobody measured it.

## Affects

T-101
"#;

    /// A complete finding that is a dead end. Invented content.
    const FINDING: &str = r#"<!-- dossier-format: 1 -->
# FND-101 · Invented finding

- kind: finding
- status: draft
- outcome: dead end

## Observation

The first try did not converge.

## Evidence

Runs 1 and 2.
"#;

    /// A complete lesson. Invented content.
    const LESSON: &str = r#"<!-- dossier-format: 1 -->
# LES-101 · Invented lesson

- kind: lesson
- status: draft
- confidence: simulated

## Statement

The short setup is stable below ten steps.

## Valid for

Invented geometry only.

## Evidence

Runs 3 to 5.
"#;

    #[test]
    fn a_complete_entry_of_each_kind_has_no_problems() {
        for text in [DECISION, ASSUMPTION, FINDING, LESSON] {
            let found = whats(text);
            assert!(found.is_empty(), "{found:?}");
        }
    }

    #[test]
    fn a_decision_needs_its_decision_and_reason_but_not_rejected() {
        let without_rejected = replacing(
            DECISION,
            "## Rejected\n\nThe second setup was slower.\n",
            "",
        );
        assert!(whats(&without_rejected).is_empty());
        let nothing = "<!-- dossier-format: 1 -->\n# DEC-102 · Invented decision\n\n- kind: decision\n- status: draft\n";
        assert_eq!(
            whats(nothing),
            ["missing section Decision", "missing section Reason"]
        );
    }

    #[test]
    fn a_lesson_without_valid_for_lists_the_missing_section() {
        let text = replacing(LESSON, "## Valid for\n\nInvented geometry only.\n\n", "");
        assert_eq!(whats(&text), ["missing section Valid for"]);
    }

    #[test]
    fn a_lesson_without_confidence_lists_the_missing_field() {
        let text = replacing(LESSON, "- confidence: simulated\n", "");
        assert_eq!(whats(&text), ["missing field confidence"]);
    }

    #[test]
    fn a_lesson_with_another_confidence_lists_the_allowed_values() {
        let text = replacing(
            LESSON,
            "- confidence: simulated\n",
            "- confidence: certain\n",
        );
        assert_eq!(
            whats(&text),
            ["confidence must be measured, simulated, experience or guess"]
        );
    }

    #[test]
    fn every_confidence_value_of_format_section_5_is_accepted() {
        for value in ["measured", "simulated", "experience", "guess"] {
            let text = replacing(
                LESSON,
                "- confidence: simulated\n",
                &format!("- confidence: {value}\n"),
            );
            let found = whats(&text);
            assert!(found.is_empty(), "{value}: {found:?}");
        }
    }

    #[test]
    fn a_lesson_with_nothing_lists_each_problem_in_order() {
        let text = "<!-- dossier-format: 1 -->\n# LES-102 · Invented lesson\n\n- kind: lesson\n- status: draft\n";
        assert_eq!(
            whats(text),
            [
                "missing section Statement",
                "missing section Valid for",
                "missing section Evidence",
                "missing field confidence",
            ]
        );
    }

    #[test]
    fn an_assumption_without_state_lists_the_state_rule() {
        let text = replacing(ASSUMPTION, "- state: open\n", "");
        assert_eq!(whats(&text), ["state must be open, confirmed or wrong"]);
    }

    #[test]
    fn an_assumption_with_another_state_lists_the_state_rule() {
        let text = replacing(ASSUMPTION, "- state: open\n", "- state: maybe\n");
        assert_eq!(whats(&text), ["state must be open, confirmed or wrong"]);
    }

    #[test]
    fn every_state_of_format_section_5_is_accepted() {
        for value in ["open", "confirmed", "wrong"] {
            let text = replacing(
                ASSUMPTION,
                "- state: open\n",
                &format!("- state: {value}\n"),
            );
            let found = whats(&text);
            assert!(found.is_empty(), "{value}: {found:?}");
        }
    }

    #[test]
    fn a_finding_with_another_outcome_lists_the_outcome_rule() {
        let text = replacing(FINDING, "- outcome: dead end\n", "- outcome: success\n");
        assert_eq!(whats(&text), ["outcome must be dead end"]);
    }

    #[test]
    fn a_finding_without_an_outcome_needs_only_its_observation() {
        let text = replacing(FINDING, "- outcome: dead end\n", "");
        assert!(whats(&text).is_empty());
    }

    #[test]
    fn a_finding_without_observation_lists_the_missing_section() {
        let text = replacing(
            FINDING,
            "## Observation\n\nThe first try did not converge.\n\n",
            "",
        );
        assert_eq!(whats(&text), ["missing section Observation"]);
    }

    #[test]
    fn a_section_with_no_text_counts_as_missing() {
        let text = replacing(DECISION, "It was the one that was checked.\n", "");
        assert_eq!(whats(&text), ["missing section Reason"]);
    }

    #[test]
    fn an_entry_without_kind_gives_one_problem_only() {
        let text = replacing(DECISION, "- kind: decision\n", "");
        assert_eq!(whats(&text), ["missing field kind"]);
        let empty = replacing(DECISION, "- kind: decision\n", "- kind:\n");
        assert_eq!(whats(&empty), ["missing field kind"]);
        let bare = "<!-- dossier-format: 1 -->\n# DEC-103 · Invented decision\n\n- status: draft\n";
        assert_eq!(whats(bare), ["missing field kind"]);
    }

    #[test]
    fn an_unknown_kind_gives_one_problem_only() {
        let text = replacing(DECISION, "- kind: decision\n", "- kind: note\n");
        assert_eq!(
            whats(&text),
            ["kind must be decision, assumption, finding or lesson"]
        );
    }

    #[test]
    fn values_are_compared_without_regard_to_case() {
        let lesson = replacing(
            LESSON,
            "- confidence: simulated\n",
            "- confidence: Measured\n",
        );
        assert!(whats(&lesson).is_empty());
        let finding = replacing(FINDING, "- outcome: dead end\n", "- outcome: Dead End\n");
        assert!(whats(&finding).is_empty());
    }

    /// A complete usage record (format.md section 15). Invented content.
    const USAGE: &str = r#"<!-- dossier-format: 1 -->
# USE-101 · Invented run

- kind: usage
- status: approved
- software: Invented solver
- version: 1.0
- task: T-101

## Parameters

pressure = 1.4 bar

## Outputs

results/run-1/
"#;

    #[test]
    fn a_complete_usage_record_has_no_problems_and_needs_no_sections() {
        assert!(whats(USAGE).is_empty());
        let without_sections = "<!-- dossier-format: 1 -->\n# USE-102 · Invented run\n\n- kind: usage\n- status: approved\n- software: Invented solver\n- version: 1.0\n- task: T-101\n";
        assert!(whats(without_sections).is_empty());
    }

    #[test]
    fn a_usage_record_without_software_version_or_task_lists_the_missing_field() {
        let text = replacing(USAGE, "- software: Invented solver\n", "");
        assert_eq!(whats(&text), ["missing field software"]);
        let text = replacing(USAGE, "- version: 1.0\n", "");
        assert_eq!(whats(&text), ["missing field version"]);
        let text = replacing(USAGE, "- task: T-101\n", "");
        assert_eq!(whats(&text), ["missing field task"]);
    }

    #[test]
    fn a_usage_record_with_none_of_the_three_fields_lists_them_in_order() {
        let text = replacing(
            USAGE,
            "- software: Invented solver\n- version: 1.0\n- task: T-101\n",
            "",
        );
        assert_eq!(
            whats(&text),
            [
                "missing field software",
                "missing field version",
                "missing field task",
            ]
        );
    }

    #[test]
    fn an_empty_usage_field_counts_as_missing() {
        let text = replacing(USAGE, "- version: 1.0\n", "- version:\n");
        assert_eq!(whats(&text), ["missing field version"]);
    }

    #[test]
    fn required_sections_are_those_of_format_section_8_in_their_order() {
        let expected: [(Kind, &[&str], &[&str]); 8] = [
            (
                Kind::Decision,
                &["Decision", "Reason"],
                &["Decision", "Reason", "Rejected"],
            ),
            (
                Kind::Assumption,
                &["Assumption", "Reason"],
                &["Assumption", "Reason", "Affects", "Confirmation"],
            ),
            (
                Kind::Finding,
                &["Observation"],
                &["Observation", "Evidence"],
            ),
            (
                Kind::Lesson,
                &["Statement", "Valid for", "Evidence"],
                &["Statement", "Valid for", "Evidence"],
            ),
            (
                Kind::Usage,
                &[],
                &["Parameters", "Outputs", "Deviations", "Notes"],
            ),
            (
                Kind::Howto,
                &["Purpose", "Steps"],
                &[
                    "Purpose",
                    "Steps",
                    "Settings",
                    "Traps",
                    "Tested with",
                    "Open remarks",
                ],
            ),
            (
                Kind::Tool,
                &["Purpose"],
                &[
                    "Purpose",
                    "How to run",
                    "Inputs",
                    "Outputs",
                    "Traps",
                    "Tested with",
                ],
            ),
            (
                Kind::Investigation,
                &["Problem"],
                &["Problem", "Attempts", "Solution"],
            ),
        ];
        for (kind, required, order) in expected {
            assert_eq!(required_sections(kind), required, "{}", kind.name());
            assert_eq!(section_order(kind), order, "{}", kind.name());
        }
    }

    /// A complete how-to. Invented content.
    const HOWTO: &str = "<!-- dossier-format: 1 -->
# HOW-101 · Invented how-to

- kind: howto
- status: draft
- software: Invented solver

## Purpose

Run the invented case.

## Steps

1. Open it.
";

    #[test]
    fn a_complete_howto_has_no_problems() {
        assert!(whats(HOWTO).is_empty());
    }

    #[test]
    fn a_howto_with_open_remarks_after_tested_with_has_no_problems() {
        let text = format!(
            "{HOWTO}
## Tested with

Version 1.2 on an invented case.

## Open remarks

- P-017 2026-11-20: Invented deviation.
"
        );
        let found = whats(&text);
        assert!(found.is_empty(), "{found:?}");
        assert_eq!(section_order(Kind::Howto).last(), Some(&"Open remarks"));
    }

    #[test]
    fn a_howto_lists_a_missing_section_before_a_missing_software_field() {
        let text = replacing(
            HOWTO,
            "## Steps

1. Open it.
",
            "",
        );
        assert_eq!(whats(&text), ["missing section Steps"]);
        let text = replacing(
            HOWTO,
            "- software: Invented solver
",
            "",
        );
        assert_eq!(whats(&text), ["missing field software"]);
        let text = replacing(
            &text,
            "## Steps

1. Open it.
",
            "",
        );
        assert_eq!(
            whats(&text),
            ["missing section Steps", "missing field software"]
        );
    }

    #[test]
    fn a_tool_needs_only_the_purpose_section() {
        let tool = "<!-- dossier-format: 1 -->
# TOOL-101 · Invented tool

- kind: tool
- status: draft

## Purpose

Converts files.
";
        assert!(whats(tool).is_empty());
        let text = replacing(
            tool,
            "## Purpose

Converts files.
",
            "",
        );
        assert_eq!(whats(&text), ["missing section Purpose"]);
    }

    /// A complete investigation, as `render` writes it, with the given `state`. Invented content.
    fn investigation(state: &str) -> String {
        render(&Draft {
            kind: Kind::Investigation,
            id: "INV-101".to_string(),
            title: "Invented investigation".to_string(),
            fields: vec![
                ("status".to_string(), "approved".to_string()),
                ("state".to_string(), state.to_string()),
            ],
            sections: vec![(
                "Problem".to_string(),
                "The invented run stops early.".to_string(),
            )],
        })
    }

    #[test]
    fn an_investigation_with_a_state_and_a_problem_has_no_problems() {
        let found = whats(&investigation("open"));
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn every_state_of_an_investigation_is_accepted_in_any_case() {
        for value in ["open", "solved", "given up", "Given Up"] {
            let found = whats(&investigation(value));
            assert!(found.is_empty(), "{value}: {found:?}");
        }
    }

    #[test]
    fn an_investigation_with_another_state_lists_the_state_rule() {
        let text = investigation("closed");
        assert_eq!(whats(&text), ["state must be open, solved or given up"]);
    }

    #[test]
    fn an_investigation_without_state_lists_the_state_rule() {
        let text = replacing(&investigation("open"), "- state: open\n", "");
        assert_eq!(whats(&text), ["state must be open, solved or given up"]);
    }

    #[test]
    fn an_investigation_without_problem_lists_the_section_before_the_state_rule() {
        let text = replacing(
            &replacing(&investigation("open"), "- state: open\n", ""),
            "## Problem\n\nThe invented run stops early.\n",
            "",
        );
        assert_eq!(
            whats(&text),
            [
                "missing section Problem",
                "state must be open, solved or given up",
            ]
        );
    }
}
