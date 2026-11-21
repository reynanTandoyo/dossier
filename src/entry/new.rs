//! The text of a new entry file.

use super::Kind;

/// The first line of every entry file (format.md §1, §3).
const FORMAT_LINE: &str = "<!-- dossier-format: 1 -->";

/// The name of the section that holds the pictures of an entry; it always comes last (format.md
/// §16).
const PICTURES: &str = "Pictures";

/// The known fields in the order of format.md §5, with the fields of §14 and §15 after
/// `rejected-reason` and after `software`. `history` is not listed: it is always written last.
/// `kind` is listed first, but `render` writes it from `Draft::kind`.
const FIELD_ORDER: [&str; 41] = [
    "kind",
    "status",
    "created",
    "author",
    "via",
    "approved",
    "valid-from",
    "valid-until",
    "state",
    "confidence",
    "outcome",
    "task",
    "machine",
    "analysis",
    "software",
    "version",
    "model",
    "model-checksum",
    "howto",
    "tags",
    "related",
    "supersedes",
    "superseded-by",
    "retired-reason",
    "rejected-reason",
    "declined-reason",
    "confirmed",
    "refuted",
    "rests-on",
    "code",
    "source",
    "promoted",
    "promote",
    "imported",
    "source-document",
    "review",
    "reviewed",
    "type",
    "location",
    "language",
    "checksum",
];

/// What a new entry file holds before it is rendered.
pub struct Draft {
    pub kind: Kind,
    pub id: String,
    pub title: String,
    pub fields: Vec<(String, String)>,
    pub sections: Vec<(String, String)>,
}

/// The text of a new entry file exactly as format.md §1, fields in the order of §5 (unknown
/// keys after the known ones, in the order given), sections in the order of §8.
///
/// The text is canonical: a field value or title is written on one line (its line breaks become
/// spaces, its ends are trimmed), an empty value is left out, and a key or section name given
/// twice is written once, with its first non-empty value. The history lines come last, one per
/// value, in the order given. A section whose body is blank is left out; the other sections are
/// the kind's sections of format.md §8 (§15 for usage) in that order, then the others in the order
/// given, and `Pictures` last of all (§16). Every line ends with LF and the text ends with exactly
/// one LF after its last line. A body line that starts with `## ` would start a section and is not
/// checked here.
pub fn render(draft: &Draft) -> String {
    let mut lines = vec![
        FORMAT_LINE.to_string(),
        format!("# {} · {}", one_line(&draft.id), one_line(&draft.title))
            .trim_end()
            .to_string(),
        String::new(),
        format!("- kind: {}", draft.kind.name()),
    ];
    for (key, value) in fields(draft) {
        lines.push(format!("- {key}: {value}"));
    }
    for value in history(draft) {
        lines.push(format!("- history: {value}"));
    }
    let sections = sections(draft);
    if !sections.is_empty() {
        lines.push(String::new());
        for (number, (name, body)) in sections.into_iter().enumerate() {
            if number > 0 {
                lines.push(String::new());
            }
            lines.push(format!("## {name}"));
            lines.push(String::new());
            lines.extend(body);
        }
    }
    lines.iter().map(|line| format!("{line}\n")).collect()
}

/// The fields other than `kind` and `history`: each key once, with its first non-empty value,
/// in the order of format.md §5 and then in the order given.
fn fields(draft: &Draft) -> Vec<(String, String)> {
    let mut kept: Vec<(String, String)> = Vec::new();
    for (key, value) in &draft.fields {
        let value = one_line(value);
        let skipped = key == "kind"
            || key == "history"
            || value.is_empty()
            || kept.iter().any(|(kept_key, _)| kept_key == key);
        if !skipped {
            kept.push((key.clone(), value));
        }
    }
    kept.sort_by_key(|(key, _)| {
        FIELD_ORDER
            .iter()
            .position(|known| *known == key.as_str())
            .unwrap_or(FIELD_ORDER.len())
    });
    kept
}

/// The values of the history lines, in the order given; empty values are left out.
fn history(draft: &Draft) -> Vec<String> {
    draft
        .fields
        .iter()
        .filter(|(key, _)| key == "history")
        .map(|(_, value)| one_line(value))
        .filter(|value| !value.is_empty())
        .collect()
}

/// The sections with a non-blank body, each name once with its first non-blank body, as the
/// lines of the body. The kind's sections of format.md §8 (§15 for usage) come first, in that
/// order; the others follow in the order given, and `Pictures` comes last of all.
fn sections(draft: &Draft) -> Vec<(String, Vec<String>)> {
    let mut kept: Vec<(String, Vec<String>)> = Vec::new();
    for (name, body) in &draft.sections {
        let name = one_line(name);
        let body = body_lines(body);
        let skipped = body.is_empty() || kept.iter().any(|(kept_name, _)| *kept_name == name);
        if !skipped {
            kept.push((name, body));
        }
    }
    let order = known_sections(draft.kind);
    kept.sort_by_key(|(name, _)| section_rank(order, name));
    kept
}

/// The sections of each kind, in the order of format.md §8, and of a usage record in §15.
fn known_sections(kind: Kind) -> &'static [&'static str] {
    match kind {
        Kind::Decision => &["Decision", "Reason", "Rejected"],
        Kind::Assumption => &["Assumption", "Reason", "Affects", "Confirmation"],
        Kind::Finding => &["Observation", "Evidence"],
        Kind::Lesson => &["Statement", "Valid for", "Evidence"],
        Kind::Usage => &["Parameters", "Outputs", "Deviations", "Notes"],
        Kind::Howto => &["Purpose", "Steps", "Settings", "Traps", "Tested with"],
        Kind::Tool => &[
            "Purpose",
            "How to run",
            "Inputs",
            "Outputs",
            "Traps",
            "Tested with",
        ],
    }
}

/// Where a section goes: among the kind's sections (`order`), the others after them in the order
/// given, and `Pictures` after all of them.
fn section_rank(order: &[&str], name: &str) -> usize {
    if name == PICTURES {
        return usize::MAX;
    }
    order
        .iter()
        .position(|known| *known == name)
        .unwrap_or(order.len())
}

/// The lines of a section body, each without a CR at its end, without the blank lines at its
/// start and its end. Empty when the body is blank.
fn body_lines(body: &str) -> Vec<String> {
    let lines: Vec<&str> = body
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();
    let first = lines.iter().position(|line| !line.trim().is_empty());
    let last = lines.iter().rposition(|line| !line.trim().is_empty());
    match (first, last) {
        (Some(first), Some(last)) => lines[first..=last]
            .iter()
            .map(|line| line.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

/// The text on one line: each line trimmed, the non-empty lines joined with a space. A title,
/// a field value or a section name must not break the lines of the file.
fn one_line(text: &str) -> String {
    text.split(['\r', '\n'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::Entry;

    /// format.md section 1, the first example, copied exactly.
    const DECISION: &str = r#"<!-- dossier-format: 1 -->
# DEC-012 · Use transient runs for the last three speedline points

- kind: decision
- status: approved
- created: 2026-11-20T14:03:27+01:00
- author: anna
- via: anna-claude
- approved: 2026-11-20T14:03:27+01:00 by anna
- valid-from: 2026-11-20T14:03:27+01:00
- task: T-042
- machine: axial compressor
- analysis: steady RANS, speedline
- history: 2026-11-20T14:03:27+01:00 anna via anna-claude: created
- history: 2026-11-20T14:03:27+01:00 anna via anna-claude: approved

## Decision

The last three points of every speedline are run transient.

## Reason

Steady runs converged at mass flows where the transient run already showed rotating stall.

## Rejected

Steady runs with a finer mesh: same result, three times the cost.
"#;

    /// format.md section 1, the second example (an assumption), copied exactly.
    const ASSUMPTION: &str = r#"<!-- dossier-format: 1 -->
# ASM-003 · Inlet profile is uniform

- kind: assumption
- status: approved
- created: 2026-11-20T09:12:05+01:00
- author: ben
- approved: 2026-11-20T09:12:05+01:00 by ben
- valid-from: 2026-11-20T09:12:05+01:00
- state: open
- task: T-040
- history: 2026-11-20T09:12:05+01:00 ben: created
- history: 2026-11-20T09:12:05+01:00 ben: approved

## Assumption

The total pressure and temperature at the inlet are uniform over the span.

## Reason

No measured profile was delivered.

## Affects

T-042, T-043
"#;

    /// format.md section 1, the third example (a finding that is a dead end), copied exactly.
    const FINDING: &str = r#"<!-- dossier-format: 1 -->
# FND-007 · Halving the time step did not remove the oscillation

- kind: finding
- status: approved
- created: 2026-11-21T16:40:11+01:00
- author: anna
- via: anna-claude
- approved: 2026-11-21T16:40:11+01:00 by anna
- valid-from: 2026-11-21T16:40:11+01:00
- outcome: dead end
- task: T-042
- history: 2026-11-21T16:40:11+01:00 anna via anna-claude: created
- history: 2026-11-21T16:40:11+01:00 anna via anna-claude: approved

## Observation

The mass flow still oscillated by 0.4 % with half the time step. The cause is not the time step.

## Evidence

Runs 12 and 13, monitor plots in results/monitors/.
"#;

    /// format.md section 1, the fourth example (a lesson drafted by an LLM), copied exactly.
    const LESSON: &str = r#"<!-- dossier-format: 1 -->
# LES-004 · Steady mixing-plane runs overpredict stall margin

- kind: lesson
- status: draft
- created: 2026-11-21T17:02:44+01:00
- author: anna
- via: anna-claude
- confidence: simulated
- task: T-042
- machine: axial compressor
- analysis: steady RANS, speedline
- history: 2026-11-21T17:02:44+01:00 anna via anna-claude: created

## Statement

Near the stability limit, steady runs with a mixing-plane interface converged at mass flows
where the transient run already showed rotating stall.

## Valid for

Transonic front stages, tip Mach number above 1.1. Not checked for subsonic rear stages.

## Evidence

Comparison of steady and transient speedlines, report section 4.2.
"#;

    /// Key and value pairs as the draft holds them.
    fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
        items
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    fn decision_draft() -> Draft {
        Draft {
            kind: Kind::Decision,
            id: "DEC-012".to_string(),
            title: "Use transient runs for the last three speedline points".to_string(),
            fields: pairs(&[
                ("status", "approved"),
                ("created", "2026-11-20T14:03:27+01:00"),
                ("author", "anna"),
                ("via", "anna-claude"),
                ("approved", "2026-11-20T14:03:27+01:00 by anna"),
                ("valid-from", "2026-11-20T14:03:27+01:00"),
                ("task", "T-042"),
                ("machine", "axial compressor"),
                ("analysis", "steady RANS, speedline"),
                (
                    "history",
                    "2026-11-20T14:03:27+01:00 anna via anna-claude: created",
                ),
                (
                    "history",
                    "2026-11-20T14:03:27+01:00 anna via anna-claude: approved",
                ),
            ]),
            sections: pairs(&[
                (
                    "Decision",
                    "The last three points of every speedline are run transient.",
                ),
                (
                    "Reason",
                    "Steady runs converged at mass flows where the transient run already showed rotating stall.",
                ),
                (
                    "Rejected",
                    "Steady runs with a finer mesh: same result, three times the cost.",
                ),
            ]),
        }
    }

    fn assumption_draft() -> Draft {
        Draft {
            kind: Kind::Assumption,
            id: "ASM-003".to_string(),
            title: "Inlet profile is uniform".to_string(),
            fields: pairs(&[
                ("status", "approved"),
                ("created", "2026-11-20T09:12:05+01:00"),
                ("author", "ben"),
                ("approved", "2026-11-20T09:12:05+01:00 by ben"),
                ("valid-from", "2026-11-20T09:12:05+01:00"),
                ("state", "open"),
                ("task", "T-040"),
                ("history", "2026-11-20T09:12:05+01:00 ben: created"),
                ("history", "2026-11-20T09:12:05+01:00 ben: approved"),
            ]),
            sections: pairs(&[
                (
                    "Assumption",
                    "The total pressure and temperature at the inlet are uniform over the span.",
                ),
                ("Reason", "No measured profile was delivered."),
                ("Affects", "T-042, T-043"),
            ]),
        }
    }

    fn finding_draft() -> Draft {
        Draft {
            kind: Kind::Finding,
            id: "FND-007".to_string(),
            title: "Halving the time step did not remove the oscillation".to_string(),
            fields: pairs(&[
                ("status", "approved"),
                ("created", "2026-11-21T16:40:11+01:00"),
                ("author", "anna"),
                ("via", "anna-claude"),
                ("approved", "2026-11-21T16:40:11+01:00 by anna"),
                ("valid-from", "2026-11-21T16:40:11+01:00"),
                ("outcome", "dead end"),
                ("task", "T-042"),
                (
                    "history",
                    "2026-11-21T16:40:11+01:00 anna via anna-claude: created",
                ),
                (
                    "history",
                    "2026-11-21T16:40:11+01:00 anna via anna-claude: approved",
                ),
            ]),
            sections: pairs(&[
                (
                    "Observation",
                    "The mass flow still oscillated by 0.4 % with half the time step. The cause is not the time step.",
                ),
                (
                    "Evidence",
                    "Runs 12 and 13, monitor plots in results/monitors/.",
                ),
            ]),
        }
    }

    fn lesson_draft() -> Draft {
        Draft {
            kind: Kind::Lesson,
            id: "LES-004".to_string(),
            title: "Steady mixing-plane runs overpredict stall margin".to_string(),
            fields: pairs(&[
                ("status", "draft"),
                ("created", "2026-11-21T17:02:44+01:00"),
                ("author", "anna"),
                ("via", "anna-claude"),
                ("confidence", "simulated"),
                ("task", "T-042"),
                ("machine", "axial compressor"),
                ("analysis", "steady RANS, speedline"),
                (
                    "history",
                    "2026-11-21T17:02:44+01:00 anna via anna-claude: created",
                ),
            ]),
            sections: pairs(&[
                (
                    "Statement",
                    "Near the stability limit, steady runs with a mixing-plane interface converged at mass flows\nwhere the transient run already showed rotating stall.",
                ),
                (
                    "Valid for",
                    "Transonic front stages, tip Mach number above 1.1. Not checked for subsonic rear stages.",
                ),
                (
                    "Evidence",
                    "Comparison of steady and transient speedlines, report section 4.2.",
                ),
            ]),
        }
    }

    #[test]
    fn decision_renders_to_the_text_of_format_section_1() {
        assert_eq!(render(&decision_draft()), DECISION);
    }

    #[test]
    fn assumption_finding_and_lesson_render_to_their_examples_of_format_section_1() {
        assert_eq!(render(&assumption_draft()), ASSUMPTION);
        assert_eq!(render(&finding_draft()), FINDING);
        assert_eq!(render(&lesson_draft()), LESSON);
    }

    #[test]
    fn every_rendered_example_parses_back_to_the_same_fields_and_sections() {
        let cases = [
            (decision_draft(), DECISION),
            (assumption_draft(), ASSUMPTION),
            (finding_draft(), FINDING),
            (lesson_draft(), LESSON),
        ];
        for (draft, expected) in cases {
            let text = render(&draft);
            let entry = Entry::parse(&text).expect("a rendered entry parses");
            assert_eq!(entry.text(), text);
            assert_eq!(entry.id(), draft.id);
            assert_eq!(entry.title(), draft.title);
            assert_eq!(entry.kind(), Some(draft.kind));
            let mut fields = vec![("kind".to_string(), draft.kind.name().to_string())];
            fields.extend(draft.fields.iter().cloned());
            assert_eq!(entry.fields(), fields);
            assert_eq!(entry.sections(), draft.sections);
            assert_eq!(text, expected);
        }
    }

    #[test]
    fn fields_and_sections_are_written_in_format_order_and_unknown_ones_follow() {
        let draft = Draft {
            kind: Kind::Decision,
            id: "DEC-013".to_string(),
            title: "Order of the parts".to_string(),
            fields: pairs(&[
                ("history", "2026-11-20T10:00:00+01:00 anna: created"),
                ("task", "T-042"),
                ("custom-note", "kept as given"),
                ("kind", "finding"),
                ("status", "draft"),
                ("machine", ""),
                ("created", "2026-11-20T10:00:00+01:00"),
                ("author", "anna"),
                (
                    "history",
                    "2026-11-20T10:05:00+01:00 anna: changed — field task",
                ),
            ]),
            sections: pairs(&[
                ("Rejected", "Second choice."),
                ("Extra", "Written after the known sections."),
                ("Reason", "Because."),
                ("Decision", "Use it."),
            ]),
        };
        let expected = r#"<!-- dossier-format: 1 -->
# DEC-013 · Order of the parts

- kind: decision
- status: draft
- created: 2026-11-20T10:00:00+01:00
- author: anna
- task: T-042
- custom-note: kept as given
- history: 2026-11-20T10:00:00+01:00 anna: created
- history: 2026-11-20T10:05:00+01:00 anna: changed — field task

## Decision

Use it.

## Reason

Because.

## Rejected

Second choice.

## Extra

Written after the known sections.
"#;
        let text = render(&draft);
        assert_eq!(text, expected);

        let entry = Entry::parse(&text).expect("parses");
        let keys: Vec<String> = entry.fields().into_iter().map(|(key, _)| key).collect();
        assert_eq!(
            keys,
            [
                "kind",
                "status",
                "created",
                "author",
                "task",
                "custom-note",
                "history",
                "history",
            ]
        );
        let names: Vec<String> = entry.sections().into_iter().map(|(name, _)| name).collect();
        assert_eq!(names, ["Decision", "Reason", "Rejected", "Extra"]);
    }

    #[test]
    fn a_section_with_an_empty_body_is_left_out() {
        let draft = Draft {
            kind: Kind::Decision,
            id: "DEC-014".to_string(),
            title: "Blank bodies".to_string(),
            fields: pairs(&[("status", "draft")]),
            sections: pairs(&[
                ("Decision", "Keep the old mesh."),
                ("Reason", ""),
                ("Rejected", "  \n\r\n \n"),
            ]),
        };
        let text = render(&draft);
        assert_eq!(
            text,
            "<!-- dossier-format: 1 -->\n# DEC-014 · Blank bodies\n\n- kind: decision\n- status: draft\n\n## Decision\n\nKeep the old mesh.\n"
        );
        let entry = Entry::parse(&text).expect("parses");
        assert_eq!(entry.section("Reason"), None);
        assert_eq!(entry.section("Rejected"), None);
    }

    #[test]
    fn with_no_section_the_text_ends_after_the_field_block_with_one_line_break() {
        let draft = Draft {
            kind: Kind::Finding,
            id: "FND-008".to_string(),
            title: "No body yet".to_string(),
            fields: pairs(&[
                ("status", "draft"),
                ("history", "2026-11-21T09:00:00+01:00 anna: created"),
            ]),
            sections: pairs(&[("Observation", " \n ")]),
        };
        assert_eq!(
            render(&draft),
            "<!-- dossier-format: 1 -->\n# FND-008 · No body yet\n\n- kind: finding\n- status: draft\n- history: 2026-11-21T09:00:00+01:00 anna: created\n"
        );
    }

    #[test]
    fn empty_values_are_skipped_and_a_key_given_twice_is_written_once() {
        let draft = Draft {
            kind: Kind::Finding,
            id: "FND-009".to_string(),
            title: "Repeated keys".to_string(),
            fields: pairs(&[
                ("machine", ""),
                ("tags", "   "),
                ("analysis", "a"),
                ("analysis", "b"),
                ("history", ""),
                ("outcome", ""),
                ("outcome", "dead end"),
            ]),
            sections: pairs(&[("Observation", "Seen once.")]),
        };
        let text = render(&draft);
        assert_eq!(
            text,
            "<!-- dossier-format: 1 -->\n# FND-009 · Repeated keys\n\n- kind: finding\n- outcome: dead end\n- analysis: a\n\n## Observation\n\nSeen once.\n"
        );
    }

    #[test]
    fn titles_and_values_stay_on_one_line_and_the_text_has_only_lf() {
        let draft = Draft {
            kind: Kind::Lesson,
            id: "LES-005".to_string(),
            title: "  Two\r\nlines  ".to_string(),
            fields: pairs(&[("task", "  T-042  "), ("analysis", "one\nmore")]),
            sections: pairs(&[
                ("Statement", "first\r\n\r\nsecond\r\n"),
                ("Valid for", "\r\n"),
                ("Evidence", "only"),
            ]),
        };
        let text = render(&draft);
        assert_eq!(
            text,
            "<!-- dossier-format: 1 -->\n# LES-005 · Two lines\n\n- kind: lesson\n- task: T-042\n- analysis: one more\n\n## Statement\n\nfirst\n\nsecond\n\n## Evidence\n\nonly\n"
        );
        assert!(!text.contains('\r'));
        assert!(text.ends_with("only\n"));
        assert!(!text.ends_with("\n\n"));
    }

    #[test]
    fn the_fields_of_format_sections_14_and_15_follow_the_known_order() {
        let draft = Draft {
            kind: Kind::Usage,
            id: "USE-005".to_string(),
            title: "Field order".to_string(),
            fields: pairs(&[
                ("code", "src/solver.rs::run"),
                ("rests-on", "ASM-003"),
                ("refuted", "2026-11-20T10:00:00+01:00 by anna"),
                ("confirmed", "2026-11-20T09:00:00+01:00 by anna"),
                ("declined-reason", "Not needed"),
                ("rejected-reason", "Too slow"),
                ("howto", "DEC-002"),
                ("model-checksum", "sha256:9f2c"),
                ("model", "sim/stage1.sim"),
                ("version", "2406"),
                ("software", "ExampleSolver"),
                ("task", "T-042"),
            ]),
            sections: Vec::new(),
        };
        let text = render(&draft);
        let entry = Entry::parse(&text).expect("parses");
        let keys: Vec<String> = entry.fields().into_iter().map(|(key, _)| key).collect();
        assert_eq!(
            keys,
            [
                "kind",
                "task",
                "software",
                "version",
                "model",
                "model-checksum",
                "howto",
                "rejected-reason",
                "declined-reason",
                "confirmed",
                "refuted",
                "rests-on",
                "code",
            ]
        );
    }

    #[test]
    fn a_usage_record_renders_its_sections_in_format_order_with_pictures_last() {
        let draft = Draft {
            kind: Kind::Usage,
            id: "USE-003".to_string(),
            title: "ExampleSolver 2406 run of speedline point 7".to_string(),
            fields: pairs(&[
                ("status", "approved"),
                ("created", "2026-12-02T10:15:00+01:00"),
                ("author", "anna"),
                ("via", "anna-claude"),
                ("approved", "2026-12-02T10:15:00+01:00 by anna"),
                ("valid-from", "2026-12-02T10:15:00+01:00"),
                ("task", "T-042"),
                ("software", "ExampleSolver"),
                ("version", "2406"),
                ("model", "sim/stage1.sim"),
                ("model-checksum", "sha256:9f2c"),
                (
                    "history",
                    "2026-12-02T10:15:00+01:00 anna via anna-claude: created",
                ),
                (
                    "history",
                    "2026-12-02T10:15:00+01:00 anna via anna-claude: approved",
                ),
            ]),
            sections: pairs(&[
                ("Pictures", "![Speedline](../images/USE-003/speedline.png)"),
                ("Notes", "Run on the cluster."),
                ("Outputs", "results/point07/"),
                (
                    "Parameters",
                    "backpressure = 1.42 bar\nmassflow = {design.massflow}",
                ),
            ]),
        };
        let expected = r#"<!-- dossier-format: 1 -->
# USE-003 · ExampleSolver 2406 run of speedline point 7

- kind: usage
- status: approved
- created: 2026-12-02T10:15:00+01:00
- author: anna
- via: anna-claude
- approved: 2026-12-02T10:15:00+01:00 by anna
- valid-from: 2026-12-02T10:15:00+01:00
- task: T-042
- software: ExampleSolver
- version: 2406
- model: sim/stage1.sim
- model-checksum: sha256:9f2c
- history: 2026-12-02T10:15:00+01:00 anna via anna-claude: created
- history: 2026-12-02T10:15:00+01:00 anna via anna-claude: approved

## Parameters

backpressure = 1.42 bar
massflow = {design.massflow}

## Outputs

results/point07/

## Notes

Run on the cluster.

## Pictures

![Speedline](../images/USE-003/speedline.png)
"#;
        assert_eq!(render(&draft), expected);
    }

    #[test]
    fn pictures_come_after_every_other_section_for_every_kind() {
        for kind in Kind::all() {
            let draft = Draft {
                kind,
                id: "XXX-001".to_string(),
                title: "Pictures last".to_string(),
                fields: Vec::new(),
                sections: pairs(&[
                    ("Pictures", "![A plot](../images/XXX-001/plot.png)"),
                    ("Extra", "Kept after the known sections."),
                    ("Observation", "Seen."),
                ]),
            };
            let entry = Entry::parse(&render(&draft)).expect("parses");
            let names: Vec<String> = entry.sections().into_iter().map(|(name, _)| name).collect();
            assert_eq!(
                names.last().map(String::as_str),
                Some("Pictures"),
                "{kind:?}"
            );
            assert_eq!(names.iter().filter(|name| *name == "Pictures").count(), 1);
            let extra = names.iter().position(|name| name == "Extra");
            let pictures = names.iter().position(|name| name == "Pictures");
            assert!(extra < pictures, "{kind:?}: {names:?}");
        }
    }

    #[test]
    fn the_kind_comes_from_the_draft_even_when_no_field_names_it() {
        let draft = Draft {
            kind: Kind::Assumption,
            id: "ASM-004".to_string(),
            title: "Kind from the draft".to_string(),
            fields: pairs(&[("state", "open")]),
            sections: pairs(&[("Assumption", "It holds.")]),
        };
        let text = render(&draft);
        assert!(text.starts_with(
            "<!-- dossier-format: 1 -->\n# ASM-004 · Kind from the draft\n\n- kind: assumption\n- state: open\n"
        ));
        let entry = Entry::parse(&text).expect("parses");
        assert_eq!(entry.kind(), Some(Kind::Assumption));
    }

    #[test]
    fn the_fields_of_format_section_23_follow_code_in_their_order() {
        let draft = Draft {
            kind: Kind::Tool,
            id: "TOOL-001".to_string(),
            title: "Field order of a tool".to_string(),
            fields: pairs(&[
                ("checksum", "sha256:9f2c"),
                ("language", "python"),
                ("location", "scripts/run.py"),
                ("type", "script"),
                ("reviewed", "2026-11-20T10:00:00+01:00 by anna"),
                ("review", "2027-11-20"),
                ("source-document", "docs/a.pdf p. 3"),
                ("imported", "yes"),
                ("promote", "no"),
                ("promoted", "team:TOOL-001"),
                ("source", "P-017:TOOL-001"),
                ("code", "run.py"),
                ("software", "Invented solver"),
            ]),
            sections: Vec::new(),
        };
        let text = render(&draft);
        let entry = Entry::parse(&text).expect("parses");
        let keys: Vec<String> = entry.fields().into_iter().map(|(key, _)| key).collect();
        assert_eq!(
            keys,
            [
                "kind",
                "software",
                "code",
                "source",
                "promoted",
                "promote",
                "imported",
                "source-document",
                "review",
                "reviewed",
                "type",
                "location",
                "language",
                "checksum",
            ]
        );
    }

    #[test]
    fn howto_and_tool_sections_render_in_the_order_of_format_section_22() {
        let howto = Draft {
            kind: Kind::Howto,
            id: "HOW-001".to_string(),
            title: "Invented how-to".to_string(),
            fields: Vec::new(),
            sections: pairs(&[
                ("Tested with", "Version 1."),
                ("Traps", "None known."),
                ("Steps", "1. Go."),
                ("Purpose", "Why."),
                ("Settings", "a = 1"),
            ]),
        };
        let text = render(&howto);
        let headings: Vec<&str> = text.lines().filter(|l| l.starts_with("## ")).collect();
        assert_eq!(
            headings,
            [
                "## Purpose",
                "## Steps",
                "## Settings",
                "## Traps",
                "## Tested with"
            ]
        );
        let tool = Draft {
            kind: Kind::Tool,
            id: "TOOL-001".to_string(),
            title: "Invented tool".to_string(),
            fields: Vec::new(),
            sections: pairs(&[
                ("Tested with", "x"),
                ("Outputs", "x"),
                ("Traps", "x"),
                ("Inputs", "x"),
                ("How to run", "x"),
                ("Purpose", "x"),
            ]),
        };
        let text = render(&tool);
        let headings: Vec<&str> = text.lines().filter(|l| l.starts_with("## ")).collect();
        assert_eq!(
            headings,
            [
                "## Purpose",
                "## How to run",
                "## Inputs",
                "## Outputs",
                "## Traps",
                "## Tested with"
            ]
        );
    }
}
