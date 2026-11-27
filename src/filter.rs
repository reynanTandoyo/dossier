//! Filters: the options that choose entries for `list`, `search` and the other commands.

use crate::entry::{Entry, Kind, Status};
use crate::store::Item;
use crate::time;
use chrono::{DateTime, FixedOffset};
use std::cmp::Ordering;

/// The filter options. Without `--status`, `--drafts`, `--all` and `--on`, only approved entries
/// match; `--on` chooses by the dates instead (R-LIFE-9).
#[derive(clap::Args, Debug, Default, Clone)]
pub struct Filter {
    #[arg(long)]
    kind: Option<String>,
    #[arg(long)]
    status: Option<String>,
    #[arg(long)]
    machine: Option<String>,
    #[arg(long)]
    analysis: Option<String>,
    #[arg(long)]
    software: Option<String>,
    #[arg(long)]
    tag: Option<String>,
    /// Only entries that name this topic
    #[arg(long)]
    topic: Option<String>,
    #[arg(long)]
    author: Option<String>,
    #[arg(long)]
    task: Option<String>,
    #[arg(long)]
    state: Option<String>,
    #[arg(long)]
    since: Option<String>,
    /// The day, YYYY-MM-DD: the entries that were in force on it (R-LIFE-9)
    #[arg(long)]
    on: Option<String>,
    #[arg(long = "where")]
    conditions: Vec<String>,
    #[arg(long)]
    drafts: bool,
    #[arg(long)]
    all: bool,
}

impl Filter {
    /// Whether `item` passes the filter: every option that is given must hold.
    pub fn matches(&self, item: &Item) -> bool {
        let entry = &item.entry;
        self.status_holds(entry)
            && self.on_holds(entry)
            && self.kind_holds(entry)
            && self.lists_hold(entry)
            && self.fields_hold(entry)
            && self
                .conditions
                .iter()
                .all(|condition| condition_holds(entry, condition))
    }

    /// The status rule (R-READ-2, R-DRAFT-6). An explicit `--status` decides the status alone.
    /// With `--on` the status rule does not apply: the dates decide. Otherwise `--all` lets every
    /// status through, and `--drafts` adds drafts and proposals to the approved ones (a proposal
    /// is a draft, a declined decision a rejected one).
    fn status_holds(&self, entry: &Entry) -> bool {
        if let Some(text) = &self.status {
            return Status::parse(text.trim()).is_some_and(|wanted| entry.status() == Some(wanted));
        }
        if self.all || self.on.is_some() {
            return true;
        }
        match entry.status() {
            Some(Status::Approved) => true,
            Some(Status::Draft | Status::Proposed) => self.drafts,
            _ => false,
        }
    }

    /// `--kind`, by name or prefix.
    fn kind_holds(&self, entry: &Entry) -> bool {
        self.kind.as_deref().is_none_or(|text| {
            Kind::parse(text.trim()).is_some_and(|wanted| entry.kind() == Some(wanted))
        })
    }

    /// `--machine`, `--analysis`, `--software`, `--tag` (the `tags` field) and `--topic` (the
    /// `topic` field): each value must be an element of the entry's list.
    fn lists_hold(&self, entry: &Entry) -> bool {
        [
            ("machine", &self.machine),
            ("analysis", &self.analysis),
            ("software", &self.software),
            ("tags", &self.tag),
            ("topic", &self.topic),
        ]
        .into_iter()
        .all(|(key, option)| {
            option
                .as_deref()
                .is_none_or(|value| list_has(entry, key, value.trim()))
        })
    }

    /// `--author`, `--task`, `--state` and `--since`.
    fn fields_hold(&self, entry: &Entry) -> bool {
        self.author
            .as_deref()
            .is_none_or(|value| field_is(entry, "author", value.trim()))
            && self
                .task
                .as_deref()
                .is_none_or(|value| field_is(entry, "task", value.trim()))
            && self
                .state
                .as_deref()
                .is_none_or(|value| field_is(entry, "state", value.trim()))
            && since_holds(self.since.as_deref(), entry)
    }

    /// `--on <date>`: the entry was in force on that day (R-LIFE-9), see `in_force_on`.
    fn on_holds(&self, entry: &Entry) -> bool {
        self.on
            .as_deref()
            .is_none_or(|text| in_force_on(entry, text.trim()))
    }
}

/// Whether two texts are the same, ignoring case.
fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// Whether the comma-separated list in `key` has an element equal to `wanted`, ignoring case.
fn list_has(entry: &Entry, key: &str, wanted: &str) -> bool {
    entry.list(key).iter().any(|value| same(value, wanted))
}

/// Whether the field `key` equals `wanted`, ignoring case.
fn field_is(entry: &Entry, key: &str, wanted: &str) -> bool {
    entry.field(key).is_some_and(|value| same(&value, wanted))
}

/// One `--where key=value`: split at the first `=`; the field `key` equals `value`, ignoring case.
/// A condition without `=` holds for no entry.
fn condition_holds(entry: &Entry, condition: &str) -> bool {
    condition
        .split_once('=')
        .is_some_and(|(key, value)| field_is(entry, &key.trim().to_lowercase(), value.trim()))
}

/// `--since`: the created date is not before the date given. Both are written with digits and
/// dashes, such as `2026-11-15` (or its start, `2026-11`); any other text matches nothing.
fn since_holds(since: Option<&str>, entry: &Entry) -> bool {
    let Some(text) = since else {
        return true;
    };
    let wanted = time::date(text.trim());
    let created = entry.field("created").unwrap_or_default();
    let day = time::date(&created);
    is_date_text(wanted) && is_date_text(day) && day >= wanted
}

/// Digits and dashes only, as a date is written in format.md §7 (`2026-11-15`).
fn is_date_text(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_ascii_digit() || c == '-')
}

/// Whether the entry was in force on the day `day` (`2026-11-15`; a time stamp counts by its
/// date): its valid-from date is on or before the day, and its valid-until is empty or its date
/// is after the day. An entry without a valid-from matches nothing; the status does not count.
pub fn in_force_on(entry: &Entry, day: &str) -> bool {
    let day = time::date(day);
    if !is_whole_date(day) {
        return false;
    }
    let stamp = entry.field("valid-from").unwrap_or_default();
    let from = time::date(stamp.trim());
    if !is_whole_date(from) || from > day {
        return false;
    }
    let stamp = entry.field("valid-until").unwrap_or_default();
    let until = stamp.trim();
    if until.is_empty() {
        return true;
    }
    let end = time::date(until);
    is_whole_date(end) && end > day
}

/// A whole date as format.md §7 writes its day, `2026-11-15`: ten characters, digits, with a dash
/// at the fifth and the eighth place.
pub fn is_whole_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            4 | 7 => *byte == b'-',
            _ => byte.is_ascii_digit(),
        })
}

/// The order of entries: newest `created` first, then the higher number first, then the ID in
/// reverse. An entry with no valid `created` comes last.
pub fn newest_first(a: &Entry, b: &Entry) -> Ordering {
    created_at(b)
        .cmp(&created_at(a))
        .then_with(|| number(&b.id()).cmp(&number(&a.id())))
        .then_with(|| b.id().cmp(&a.id()))
}

/// The `created` time stamp of an entry, when it is a valid one.
fn created_at(entry: &Entry) -> Option<DateTime<FixedOffset>> {
    entry
        .field("created")
        .filter(|stamp| time::valid(stamp))
        .and_then(|stamp| DateTime::parse_from_rfc3339(&stamp).ok())
}

/// The number of an ID: 12 for DIG-012 or DEC-012, 0 when there is none.
fn number(id: &str) -> u64 {
    id.rsplit('-')
        .next()
        .and_then(|digits| digits.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::new::{Draft, render};
    use clap::Parser;
    use std::path::PathBuf;

    /// An item as `store::all` gives it: the text `render` makes for these fields, read back.
    fn item(kind: Kind, id: &str, fields: &[(&str, &str)]) -> Item {
        let draft = Draft {
            kind,
            id: id.to_string(),
            title: format!("Invented title of {id}"),
            fields: fields
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            sections: Vec::new(),
        };
        let text = render(&draft);
        Item {
            path: PathBuf::from(format!("{id}-invented-title.md")),
            raw: text.as_bytes().to_vec(),
            entry: Entry::parse(&text).expect("a rendered entry parses"),
        }
    }

    /// Seven invented entries in ID order, one for each status and values that differ per
    /// option. Approved: DEC-001, FND-001, LES-002. Draft: DEC-002. Rejected: LES-001. Retired:
    /// ASM-001. Superseded: FND-002.
    fn sample() -> Vec<Item> {
        vec![
            item(
                Kind::Decision,
                "DEC-001",
                &[
                    ("status", "approved"),
                    ("created", "2026-11-10T09:00:00+01:00"),
                    ("author", "anna"),
                    ("task", "T-042"),
                    ("machine", "axial compressor"),
                    ("analysis", "steady RANS, speedline"),
                    ("software", "ExampleSolver 2406"),
                    ("tags", "validated, mesh"),
                ],
            ),
            item(
                Kind::Decision,
                "DEC-002",
                &[
                    ("status", "draft"),
                    ("created", "2026-11-18T10:00:00+01:00"),
                    ("author", "ben"),
                    ("task", "T-050"),
                    ("machine", "radial turbine"),
                    ("analysis", "modal analysis"),
                    ("software", "ExampleSolver 2406"),
                ],
            ),
            item(
                Kind::Finding,
                "FND-001",
                &[
                    ("status", "approved"),
                    ("created", "2026-11-15T16:40:00+01:00"),
                    ("author", "ben"),
                    ("task", "T-042"),
                    ("machine", "axial compressor"),
                    ("analysis", "steady RANS"),
                    ("software", "ExampleSolver 2406, OtherTool 3.1"),
                    ("tags", "inbox"),
                ],
            ),
            item(
                Kind::Lesson,
                "LES-001",
                &[
                    ("status", "rejected"),
                    ("created", "2026-11-12T08:00:00+01:00"),
                    ("author", "anna"),
                    ("task", "T-042"),
                    ("confidence", "simulated"),
                    ("tags", "mesh"),
                ],
            ),
            item(
                Kind::Assumption,
                "ASM-001",
                &[
                    ("status", "retired"),
                    ("created", "2026-11-01T12:00:00+01:00"),
                    ("author", "anna"),
                    ("task", "T-040"),
                    ("state", "open"),
                ],
            ),
            item(
                Kind::Finding,
                "FND-002",
                &[
                    ("status", "superseded"),
                    ("created", "2026-11-20T11:00:00+01:00"),
                    ("author", "ben"),
                    ("task", "T-050"),
                    ("machine", "Radial Turbine, Axial Compressor"),
                    ("software", "OtherTool 3.1"),
                    ("tags", "Inbox"),
                ],
            ),
            item(
                Kind::Lesson,
                "LES-002",
                &[
                    ("status", "approved"),
                    ("created", "2026-10-30T08:00:00+01:00"),
                    ("author", "ben"),
                    ("task", "T-040"),
                    ("confidence", "measured"),
                    ("machine", "axial compressor"),
                    ("analysis", "modal analysis"),
                ],
            ),
        ]
    }

    /// The IDs of the items that pass the filter, in the order given.
    fn ids(filter: &Filter, items: &[Item]) -> Vec<String> {
        items
            .iter()
            .filter(|item| filter.matches(item))
            .map(|item| item.entry.id())
            .collect()
    }

    #[test]
    fn without_a_status_option_only_approved_entries_match() {
        let items = sample();
        assert_eq!(
            ids(&Filter::default(), &items),
            ["DEC-001", "FND-001", "LES-002"]
        );
    }

    #[test]
    fn drafts_adds_the_drafts_and_nothing_else() {
        let items = sample();
        let filter = Filter {
            drafts: true,
            ..Default::default()
        };
        assert_eq!(
            ids(&filter, &items),
            ["DEC-001", "DEC-002", "FND-001", "LES-002"]
        );
    }

    #[test]
    fn all_matches_every_status() {
        let items = sample();
        let filter = Filter {
            all: true,
            ..Default::default()
        };
        assert_eq!(
            ids(&filter, &items),
            [
                "DEC-001", "DEC-002", "FND-001", "LES-001", "ASM-001", "FND-002", "LES-002"
            ]
        );
    }

    #[test]
    fn status_names_one_status_ignoring_case_and_replaces_the_default() {
        let items = sample();
        let status = |text: &str| Filter {
            status: Some(text.to_string()),
            ..Default::default()
        };
        assert_eq!(ids(&status("draft"), &items), ["DEC-002"]);
        assert_eq!(ids(&status("RETIRED"), &items), ["ASM-001"]);
        assert_eq!(ids(&status("rejected"), &items), ["LES-001"]);
        assert!(ids(&status("pending"), &items).is_empty());
    }

    #[test]
    fn an_explicit_status_decides_alone_even_with_drafts_and_all() {
        let items = sample();
        let filter = Filter {
            status: Some("approved".to_string()),
            drafts: true,
            all: true,
            ..Default::default()
        };
        assert_eq!(ids(&filter, &items), ["DEC-001", "FND-001", "LES-002"]);
    }

    #[test]
    fn an_entry_without_a_status_is_shown_only_by_all() {
        let items = vec![item(
            Kind::Finding,
            "FND-003",
            &[("created", "2026-11-19T09:00:00+01:00")],
        )];
        assert!(ids(&Filter::default(), &items).is_empty());
        let drafts = Filter {
            drafts: true,
            ..Default::default()
        };
        assert!(ids(&drafts, &items).is_empty());
        let all = Filter {
            all: true,
            ..Default::default()
        };
        assert_eq!(ids(&all, &items), ["FND-003"]);
    }

    // The options below are tested one at a time with `all` set, so that the status rule does
    // not hide what the option does.

    #[test]
    fn kind_compares_by_name_or_prefix_in_any_case() {
        let items = sample();
        let kind = |text: &str| Filter {
            kind: Some(text.to_string()),
            all: true,
            ..Default::default()
        };
        assert_eq!(ids(&kind("decision"), &items), ["DEC-001", "DEC-002"]);
        assert_eq!(ids(&kind("FND"), &items), ["FND-001", "FND-002"]);
        assert_eq!(ids(&kind("Lesson"), &items), ["LES-001", "LES-002"]);
        assert!(ids(&kind("procedure"), &items).is_empty());
    }

    #[test]
    fn machine_matches_any_element_of_the_list_ignoring_case() {
        let items = sample();
        let machine = |text: &str| Filter {
            machine: Some(text.to_string()),
            all: true,
            ..Default::default()
        };
        assert_eq!(
            ids(&machine("axial compressor"), &items),
            ["DEC-001", "FND-001", "FND-002", "LES-002"]
        );
        assert_eq!(
            ids(&machine("RADIAL TURBINE"), &items),
            ["DEC-002", "FND-002"]
        );
        assert!(ids(&machine("steam turbine"), &items).is_empty());
    }

    #[test]
    fn analysis_matches_any_element_of_the_list_ignoring_case() {
        let items = sample();
        let analysis = |text: &str| Filter {
            analysis: Some(text.to_string()),
            all: true,
            ..Default::default()
        };
        assert_eq!(
            ids(&analysis("steady RANS"), &items),
            ["DEC-001", "FND-001"]
        );
        assert_eq!(ids(&analysis("speedline"), &items), ["DEC-001"]);
        assert_eq!(
            ids(&analysis("MODAL ANALYSIS"), &items),
            ["DEC-002", "LES-002"]
        );
    }

    #[test]
    fn software_matches_any_element_of_the_list_ignoring_case() {
        let items = sample();
        let software = |text: &str| Filter {
            software: Some(text.to_string()),
            all: true,
            ..Default::default()
        };
        assert_eq!(
            ids(&software("ExampleSolver 2406"), &items),
            ["DEC-001", "DEC-002", "FND-001"]
        );
        assert_eq!(
            ids(&software("othertool 3.1"), &items),
            ["FND-001", "FND-002"]
        );
    }

    #[test]
    fn tag_matches_the_tags_field_only() {
        let items = sample();
        let tag = |text: &str| Filter {
            tag: Some(text.to_string()),
            all: true,
            ..Default::default()
        };
        assert_eq!(ids(&tag("mesh"), &items), ["DEC-001", "LES-001"]);
        assert_eq!(ids(&tag("INBOX"), &items), ["FND-001", "FND-002"]);
        // The machine of DEC-001 is not a tag.
        assert!(ids(&tag("axial compressor"), &items).is_empty());
    }

    #[test]
    fn author_compares_the_field_ignoring_case() {
        let items = sample();
        let author = Filter {
            author: Some("ANNA".to_string()),
            all: true,
            ..Default::default()
        };
        assert_eq!(ids(&author, &items), ["DEC-001", "LES-001", "ASM-001"]);
    }

    #[test]
    fn task_compares_the_field_ignoring_case() {
        let items = sample();
        let task = Filter {
            task: Some("t-042".to_string()),
            all: true,
            ..Default::default()
        };
        assert_eq!(ids(&task, &items), ["DEC-001", "FND-001", "LES-001"]);
    }

    /// Three investigations with invented content, in ID order: INV-001 open, INV-002 given up,
    /// INV-003 solved. All are approved, as an investigation is at once.
    fn investigation_sample() -> Vec<Item> {
        vec![
            item(
                Kind::Investigation,
                "INV-001",
                &[
                    ("status", "approved"),
                    ("created", "2026-11-16T09:00:00+01:00"),
                    ("state", "open"),
                    ("task", "T-042"),
                ],
            ),
            item(
                Kind::Investigation,
                "INV-002",
                &[
                    ("status", "approved"),
                    ("created", "2026-11-17T09:00:00+01:00"),
                    ("state", "given up"),
                    (
                        "given-up-reason",
                        "the invented solver build was not available",
                    ),
                    ("task", "T-042"),
                ],
            ),
            item(
                Kind::Investigation,
                "INV-003",
                &[
                    ("status", "approved"),
                    ("created", "2026-11-18T09:00:00+01:00"),
                    ("state", "solved"),
                    ("task", "T-050"),
                ],
            ),
        ]
    }

    #[test]
    fn state_compares_the_whole_field_ignoring_case_and_spaces() {
        let items = investigation_sample();
        let state = |text: &str| Filter {
            state: Some(text.to_string()),
            all: true,
            ..Default::default()
        };
        assert_eq!(ids(&state("open"), &items), ["INV-001"]);
        assert_eq!(ids(&state("  OPEN "), &items), ["INV-001"]);
        assert_eq!(ids(&state("given up"), &items), ["INV-002"]);
        assert_eq!(ids(&state("Given Up"), &items), ["INV-002"]);
        assert_eq!(ids(&state("solved"), &items), ["INV-003"]);
        // Part of a value is not the value.
        assert!(ids(&state("given"), &items).is_empty());
        assert!(ids(&state("up"), &items).is_empty());
    }

    #[test]
    fn state_keeps_the_status_rule_and_the_other_options() {
        let items = investigation_sample();
        // Without --all the approved investigations still match: --state only narrows.
        let open = Filter {
            state: Some("open".to_string()),
            ..Default::default()
        };
        assert_eq!(ids(&open, &items), ["INV-001"]);
        let open_in_task = Filter {
            state: Some("open".to_string()),
            task: Some("T-050".to_string()),
            all: true,
            ..Default::default()
        };
        assert!(ids(&open_in_task, &items).is_empty());
    }

    #[test]
    fn without_state_the_investigations_match_as_before() {
        let items = investigation_sample();
        let all = Filter {
            all: true,
            ..Default::default()
        };
        assert_eq!(ids(&all, &items), ["INV-001", "INV-002", "INV-003"]);
        assert_eq!(
            ids(&Filter::default(), &items),
            ["INV-001", "INV-002", "INV-003"]
        );
    }

    #[test]
    fn since_keeps_the_entries_created_on_or_after_the_date() {
        let items = sample();
        let since = |text: &str| Filter {
            since: Some(text.to_string()),
            all: true,
            ..Default::default()
        };
        // The same day matches.
        assert_eq!(
            ids(&since("2026-11-15"), &items),
            ["DEC-002", "FND-001", "FND-002"]
        );
        // A full time stamp counts by its date.
        assert_eq!(
            ids(&since("2026-11-15T23:59:59+01:00"), &items),
            ["DEC-002", "FND-001", "FND-002"]
        );
        // The start of a date counts the whole month.
        assert_eq!(
            ids(&since("2026-11"), &items),
            [
                "DEC-001", "DEC-002", "FND-001", "LES-001", "ASM-001", "FND-002"
            ]
        );
        assert!(ids(&since("2026-11-21"), &items).is_empty());
        // Text that is not a date matches nothing.
        assert!(ids(&since("15.11.2026"), &items).is_empty());
    }

    #[test]
    fn where_compares_one_field_ignoring_case() {
        let items = sample();
        let condition = |text: &str| Filter {
            conditions: vec![text.to_string()],
            all: true,
            ..Default::default()
        };
        assert_eq!(
            ids(&condition("author=anna"), &items),
            ["DEC-001", "LES-001", "ASM-001"]
        );
        // The whole field must be equal: FND-002 lists the machine among others.
        assert_eq!(
            ids(&condition("machine=Axial Compressor"), &items),
            ["DEC-001", "FND-001", "LES-002"]
        );
        assert_eq!(ids(&condition("confidence=simulated"), &items), ["LES-001"]);
        assert_eq!(ids(&condition("state=open"), &items), ["ASM-001"]);
    }

    #[test]
    fn where_without_an_equals_sign_matches_nothing() {
        let items = sample();
        let filter = Filter {
            conditions: vec!["author anna".to_string()],
            all: true,
            ..Default::default()
        };
        assert!(ids(&filter, &items).is_empty());
    }

    #[test]
    fn where_splits_at_the_first_equals_sign_and_trims_both_sides() {
        let items = vec![item(
            Kind::Lesson,
            "LES-009",
            &[("status", "approved"), ("note", "ratio=2")],
        )];
        let condition = |text: &str| Filter {
            conditions: vec![text.to_string()],
            all: true,
            ..Default::default()
        };
        assert_eq!(ids(&condition("note=ratio=2"), &items), ["LES-009"]);
        assert_eq!(ids(&condition(" NOTE = ratio=2 "), &items), ["LES-009"]);
        assert!(ids(&condition("note=ratio"), &items).is_empty());
    }

    #[test]
    fn every_where_must_hold() {
        let items = sample();
        let filter = Filter {
            conditions: vec!["author=ben".to_string(), "task=T-050".to_string()],
            all: true,
            ..Default::default()
        };
        assert_eq!(ids(&filter, &items), ["DEC-002", "FND-002"]);
    }

    #[test]
    fn two_options_together_must_both_hold() {
        let items = sample();

        // A kind and a machine, with the default status rule.
        let kind_and_machine = Filter {
            kind: Some("decision".to_string()),
            machine: Some("axial compressor".to_string()),
            ..Default::default()
        };
        assert_eq!(ids(&kind_and_machine, &items), ["DEC-001"]);

        // A task and a date, with --all.
        let task_and_since = Filter {
            task: Some("T-042".to_string()),
            since: Some("2026-11-12".to_string()),
            all: true,
            ..Default::default()
        };
        assert_eq!(ids(&task_and_since, &items), ["FND-001", "LES-001"]);

        // An author with --drafts.
        let author_and_drafts = Filter {
            author: Some("ben".to_string()),
            drafts: true,
            ..Default::default()
        };
        assert_eq!(
            ids(&author_and_drafts, &items),
            ["DEC-002", "FND-001", "LES-002"]
        );

        // A tag with an explicit status.
        let tag_and_status = Filter {
            tag: Some("mesh".to_string()),
            status: Some("rejected".to_string()),
            ..Default::default()
        };
        assert_eq!(ids(&tag_and_status, &items), ["LES-001"]);
    }

    #[test]
    fn the_options_have_the_names_of_the_spec_and_where_repeats() {
        #[derive(clap::Parser, Debug)]
        struct Cli {
            #[command(flatten)]
            filter: Filter,
        }

        let cli = Cli::try_parse_from([
            "dossier",
            "--kind",
            "decision",
            "--status",
            "draft",
            "--machine",
            "axial compressor",
            "--analysis",
            "steady RANS",
            "--software",
            "ExampleSolver 2406",
            "--tag",
            "mesh",
            "--author",
            "anna",
            "--task",
            "T-042",
            "--state",
            "open",
            "--since",
            "2026-11-01",
            "--on",
            "2026-11-22",
            "--where",
            "state=open",
            "--where",
            "note=a=b",
            "--drafts",
            "--all",
        ])
        .expect("the options parse");

        assert_eq!(cli.filter.kind.as_deref(), Some("decision"));
        assert_eq!(cli.filter.status.as_deref(), Some("draft"));
        assert_eq!(cli.filter.machine.as_deref(), Some("axial compressor"));
        assert_eq!(cli.filter.analysis.as_deref(), Some("steady RANS"));
        assert_eq!(cli.filter.software.as_deref(), Some("ExampleSolver 2406"));
        assert_eq!(cli.filter.tag.as_deref(), Some("mesh"));
        assert_eq!(cli.filter.author.as_deref(), Some("anna"));
        assert_eq!(cli.filter.task.as_deref(), Some("T-042"));
        assert_eq!(cli.filter.state.as_deref(), Some("open"));
        assert_eq!(cli.filter.since.as_deref(), Some("2026-11-01"));
        assert_eq!(cli.filter.on.as_deref(), Some("2026-11-22"));
        assert_eq!(cli.filter.conditions, ["state=open", "note=a=b"]);
        assert!(cli.filter.drafts);
        assert!(cli.filter.all);
    }

    #[test]
    fn no_option_given_is_the_default_filter() {
        #[derive(clap::Parser, Debug)]
        struct Cli {
            #[command(flatten)]
            filter: Filter,
        }

        let cli = Cli::try_parse_from(["dossier"]).expect("no options parse");
        let items = sample();
        assert_eq!(ids(&cli.filter, &items), ids(&Filter::default(), &items));
    }

    /// Entries for the `--on` tests, in ID order, with invented dates. DEC-001 was in force from
    /// 2026-11-10 until 2026-11-20 (superseded by DEC-002); ASM-001 from 2026-11-01 until
    /// 2026-11-15 (retired); DEC-002 from 2026-11-20 on. DEC-003 (draft), DEC-004 (proposed),
    /// LES-001 (rejected) and FND-001 (approved) have no valid-from.
    fn in_force_sample() -> Vec<Item> {
        vec![
            item(
                Kind::Decision,
                "DEC-001",
                &[
                    ("status", "superseded"),
                    ("created", "2026-11-10T09:00:00+01:00"),
                    ("valid-from", "2026-11-10T09:00:00+01:00"),
                    ("valid-until", "2026-11-20T10:00:00+01:00"),
                    ("superseded-by", "DEC-002"),
                    ("task", "T-042"),
                ],
            ),
            item(
                Kind::Assumption,
                "ASM-001",
                &[
                    ("status", "retired"),
                    ("created", "2026-11-01T12:00:00+01:00"),
                    ("valid-from", "2026-11-01T12:00:00+01:00"),
                    ("valid-until", "2026-11-15T12:00:00+01:00"),
                    ("state", "open"),
                ],
            ),
            item(
                Kind::Decision,
                "DEC-002",
                &[
                    ("status", "approved"),
                    ("created", "2026-11-20T10:00:00+01:00"),
                    ("valid-from", "2026-11-20T10:00:00+01:00"),
                    ("task", "T-042"),
                ],
            ),
            item(
                Kind::Decision,
                "DEC-003",
                &[
                    ("status", "draft"),
                    ("created", "2026-11-12T09:00:00+01:00"),
                ],
            ),
            item(
                Kind::Decision,
                "DEC-004",
                &[
                    ("status", "proposed"),
                    ("created", "2026-11-12T09:00:00+01:00"),
                ],
            ),
            item(
                Kind::Lesson,
                "LES-001",
                &[
                    ("status", "rejected"),
                    ("created", "2026-11-12T09:00:00+01:00"),
                    ("confidence", "guess"),
                ],
            ),
            item(
                Kind::Finding,
                "FND-001",
                &[
                    ("status", "approved"),
                    ("created", "2026-11-01T09:00:00+01:00"),
                ],
            ),
        ]
    }

    #[test]
    fn on_keeps_the_entries_in_force_on_the_day() {
        let items = in_force_sample();
        let on = |text: &str| Filter {
            on: Some(text.to_string()),
            ..Default::default()
        };
        // Before the first entry began (ASM-001, on the 1st), nothing was in force.
        assert!(ids(&on("2026-10-31"), &items).is_empty());
        assert_eq!(ids(&on("2026-11-09"), &items), ["ASM-001"]);
        // ASM-001 and DEC-001 were in force on the 12th; DEC-002 was not yet.
        assert_eq!(ids(&on("2026-11-12"), &items), ["DEC-001", "ASM-001"]);
        // ASM-001 ended on the 15th, so the day it ends is not in force.
        assert_eq!(ids(&on("2026-11-14"), &items), ["DEC-001", "ASM-001"]);
        assert_eq!(ids(&on("2026-11-15"), &items), ["DEC-001"]);
        // DEC-001 ended on the 20th, the day DEC-002 began.
        assert_eq!(ids(&on("2026-11-20"), &items), ["DEC-002"]);
        // The entries without a valid-from (DEC-003, DEC-004, LES-001, FND-001) never match.
        assert_eq!(ids(&on("2026-11-30"), &items), ["DEC-002"]);
    }

    #[test]
    fn on_ignores_the_status_rule_but_an_explicit_status_still_applies() {
        let items = in_force_sample();
        // The retired and the superseded entries match: the status rule does not apply with --on.
        let on = Filter {
            on: Some("2026-11-12".to_string()),
            ..Default::default()
        };
        assert_eq!(ids(&on, &items), ["DEC-001", "ASM-001"]);
        // An explicit --status still chooses by status.
        let retired = Filter {
            on: Some("2026-11-12".to_string()),
            status: Some("retired".to_string()),
            ..Default::default()
        };
        assert_eq!(ids(&retired, &items), ["ASM-001"]);
        let approved = Filter {
            on: Some("2026-11-12".to_string()),
            status: Some("approved".to_string()),
            ..Default::default()
        };
        assert!(ids(&approved, &items).is_empty());
    }

    #[test]
    fn on_keeps_the_other_options_and_counts_a_time_stamp_by_its_date() {
        let items = in_force_sample();
        let kind_and_task = Filter {
            on: Some("2026-11-12".to_string()),
            kind: Some("decision".to_string()),
            task: Some("T-042".to_string()),
            ..Default::default()
        };
        assert_eq!(ids(&kind_and_task, &items), ["DEC-001"]);
        // Its date is 2026-11-19, the day before DEC-001 ended.
        let stamp = Filter {
            on: Some("2026-11-19T23:00:00+01:00".to_string()),
            ..Default::default()
        };
        assert_eq!(ids(&stamp, &items), ["DEC-001"]);
    }

    #[test]
    fn on_needs_a_whole_date() {
        let items = in_force_sample();
        let on = |text: &str| Filter {
            on: Some(text.to_string()),
            ..Default::default()
        };
        assert!(ids(&on("2026-11"), &items).is_empty());
        assert!(ids(&on("15.11.2026"), &items).is_empty());
        assert!(ids(&on(""), &items).is_empty());
    }

    /// Entries with each status that a proposal or a declined decision can have, in ID order:
    /// DEC-010 approved, DEC-011 draft, DEC-012 proposed, DEC-013 declined, LES-010 rejected.
    fn proposal_sample() -> Vec<Item> {
        vec![
            item(
                Kind::Decision,
                "DEC-010",
                &[
                    ("status", "approved"),
                    ("created", "2026-11-10T09:00:00+01:00"),
                ],
            ),
            item(
                Kind::Decision,
                "DEC-011",
                &[
                    ("status", "draft"),
                    ("created", "2026-11-11T09:00:00+01:00"),
                ],
            ),
            item(
                Kind::Decision,
                "DEC-012",
                &[
                    ("status", "proposed"),
                    ("created", "2026-11-12T09:00:00+01:00"),
                ],
            ),
            item(
                Kind::Decision,
                "DEC-013",
                &[
                    ("status", "declined"),
                    ("created", "2026-11-13T09:00:00+01:00"),
                ],
            ),
            item(
                Kind::Lesson,
                "LES-010",
                &[
                    ("status", "rejected"),
                    ("created", "2026-11-14T09:00:00+01:00"),
                    ("confidence", "guess"),
                ],
            ),
        ]
    }

    #[test]
    fn a_proposal_is_shown_only_by_drafts_like_a_draft() {
        let items = proposal_sample();
        assert_eq!(ids(&Filter::default(), &items), ["DEC-010"]);
        let drafts = Filter {
            drafts: true,
            ..Default::default()
        };
        assert_eq!(ids(&drafts, &items), ["DEC-010", "DEC-011", "DEC-012"]);
        // An explicit status names the proposed entries.
        let proposed = Filter {
            status: Some("proposed".to_string()),
            ..Default::default()
        };
        assert_eq!(ids(&proposed, &items), ["DEC-012"]);
    }

    #[test]
    fn a_declined_decision_is_hidden_like_a_rejected_one() {
        let items = proposal_sample();
        assert_eq!(ids(&Filter::default(), &items), ["DEC-010"]);
        let all = Filter {
            all: true,
            ..Default::default()
        };
        assert_eq!(
            ids(&all, &items),
            ["DEC-010", "DEC-011", "DEC-012", "DEC-013", "LES-010"]
        );
        let declined = Filter {
            status: Some("declined".to_string()),
            ..Default::default()
        };
        assert_eq!(ids(&declined, &items), ["DEC-013"]);
    }
}
