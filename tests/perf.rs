//! The speed limits of NFR-3, measured on the built program with 300 entries in the project
//! dossier: `brief` and `search` must take under 300 ms, and a write (`found`) under 200 ms.
//! Each command runs once as a warm-up and then five times; the median of the five timed runs is
//! compared with the limit. The entries are written straight into the dossier folders as entry
//! files, with the canonical text of format.md §1; the timed runs are the real program.
//!
//! A debug build is slow, so its limits are ten times larger. When the variable CI is set
//! (shared runners are slow), every limit is three times larger. The measured values are printed;
//! run with `-- --nocapture` to see them when the test passes.

mod common;

use common::Project;
use dossier::entry::Kind;
use dossier::entry::new::{Draft, render};
use dossier::store::path_for;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

/// The fixed clock of every run (25 characters, with a T), as in the other command tests.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The entries of the project for this test: 150 findings, 100 decisions and 50 lessons.
const FINDINGS: usize = 150;
const DECISIONS: usize = 100;
const LESSONS: usize = 50;

/// The timed runs of each command, after one warm-up run.
const RUNS: usize = 5;

/// The NFR-3 limits in milliseconds.
const BRIEF_MS: u64 = 300;
const SEARCH_MS: u64 = 300;
const WRITE_MS: u64 = 200;

/// The topics the entries take their wording from, so that titles and paragraphs differ.
const TOPICS: [&str; 5] = [
    "speedline point",
    "mixing-plane interface",
    "time step",
    "inlet profile",
    "tip clearance",
];

/// The confidence values of lessons, cycled through (format.md §8).
const CONFIDENCE: [&str; 4] = ["measured", "simulated", "experience", "guess"];

#[test]
fn brief_search_and_write_meet_the_limits_with_300_entries() {
    let project = Project::new();
    write_entries(&project);
    println!(
        "build: {}; CI variable: {}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        if std::env::var_os("CI").is_some() {
            "set"
        } else {
            "not set"
        }
    );

    // One untimed run of search checks that it finds all 300 entries before anything is timed.
    let found = project.stdout("", &["search", "compressor"]);
    assert_eq!(
        found.lines().last(),
        Some("300 matches"),
        "search finds every fixture entry"
    );

    let brief = median_time(|_| command_of(&project, "", &["brief"]));
    let search = median_time(|_| command_of(&project, "", &["search", "compressor"]));
    let write = median_time(|run| {
        let text = format!("Perf write {run}: the compressor speedline check passed");
        command_of(&project, "anna", &["found", text.as_str()])
    });

    // Every timed write must have added a finding: the warm-up and the five timed runs.
    let findings = project.root().join("DOSSIER").join("findings");
    let count = fs::read_dir(&findings)
        .expect("the findings folder")
        .count();
    assert_eq!(count, FINDINGS + RUNS + 1, "every write added one finding");

    let measured = [
        ("brief", brief, BRIEF_MS),
        ("search", search, SEARCH_MS),
        ("write", write, WRITE_MS),
    ];
    for (name, median, base_ms) in measured {
        println!(
            "{name}: median {} ms, limit {} ms",
            millis(median),
            millis(limit(base_ms))
        );
    }
    for (name, median, base_ms) in measured {
        assert!(
            median < limit(base_ms),
            "{name} median {} ms is not under the limit {} ms",
            millis(median),
            millis(limit(base_ms))
        );
    }
}

/// The limit for one kind of run: its NFR-3 limit, ten times larger in a debug build and three
/// times larger when the variable CI is set.
fn limit(base_ms: u64) -> Duration {
    let mut factor = 1;
    if cfg!(debug_assertions) {
        factor *= 10;
    }
    if std::env::var_os("CI").is_some() {
        factor *= 3;
    }
    Duration::from_millis(base_ms * factor)
}

/// Runs the command that `build` gives for each run (0 is the warm-up, not measured, and 1 to
/// `RUNS` are timed) and returns the median of the timed runs. Every run must succeed.
fn median_time(mut build: impl FnMut(usize) -> assert_cmd::Command) -> Duration {
    let _warm_up = timed(build(0));
    let mut times: Vec<Duration> = (1..=RUNS).map(|run| timed(build(run))).collect();
    times.sort();
    times[times.len() / 2]
}

/// Runs the built command and returns its wall-clock time from start to exit.
fn timed(mut command: assert_cmd::Command) -> Duration {
    let started = Instant::now();
    let output = command.output().expect("the dossier program starts");
    let elapsed = started.elapsed();
    assert!(
        output.status.success(),
        "the run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    elapsed
}

/// The program run in the project folder as `agent`, with `args`, not yet started.
fn command_of(project: &Project, agent: &str, args: &[&str]) -> assert_cmd::Command {
    let mut command = project.command(agent, NOW);
    command.args(args);
    command
}

/// A duration in milliseconds with one decimal, for the printed values.
fn millis(time: Duration) -> String {
    format!("{:.1}", time.as_secs_f64() * 1000.0)
}

/// Writes the fixture entries into the project's dossier folders: 150 findings, 100 decisions and
/// 50 lessons, each with its own ID, title and paragraph.
fn write_entries(project: &Project) {
    let dossier = project.root().join("DOSSIER");
    for number in 1..=DECISIONS {
        write(&dossier, &decision(number));
    }
    for number in 1..=FINDINGS {
        write(&dossier, &finding(number));
    }
    for number in 1..=LESSONS {
        write(&dossier, &lesson(number));
    }
}

/// Renders one fixture entry and writes it to its file under `dossier`, where the store finds it.
fn write(dossier: &Path, draft: &Draft) {
    let path = path_for(dossier, draft.kind, &draft.id, &draft.title);
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder).expect("the folder of the entry");
    }
    fs::write(&path, render(draft)).expect("the entry file");
}

/// The creation time of the `number`-th entry: September 2026, spread over the days and hours so
/// that the newest-first order has something to sort.
fn stamp(number: usize) -> String {
    format!(
        "2026-09-{:02}T{:02}:{:02}:00+01:00",
        1 + number % 28,
        (number / 28) % 24,
        number % 60
    )
}

/// The `todo` task of the `number`-th entry, T-001 to T-100.
fn task(number: usize) -> String {
    format!("T-{:03}", number % 100 + 1)
}

fn pair(key: &str, value: &str) -> (String, String) {
    (key.to_string(), value.to_string())
}

/// The fields every fixture entry has: approved, written by anna through anna-claude, with the
/// time of creation and of approval and the two history lines (format.md §1).
fn approved_fields(stamp: &str) -> Vec<(String, String)> {
    vec![
        pair("status", "approved"),
        pair("created", stamp),
        pair("author", "anna"),
        pair("via", "anna-claude"),
        pair("approved", &format!("{stamp} by anna")),
        pair("valid-from", stamp),
        pair("history", &format!("{stamp} anna via anna-claude: created")),
        pair(
            "history",
            &format!("{stamp} anna via anna-claude: approved"),
        ),
    ]
}

/// A fixture decision: the Decision, Reason and Rejected sections of format.md §1.
fn decision(number: usize) -> Draft {
    let topic = TOPICS[number % TOPICS.len()];
    let stamp = stamp(number);
    let mut fields = approved_fields(&stamp);
    fields.push(pair("task", &task(number)));
    fields.push(pair("machine", "axial compressor"));
    fields.push(pair("analysis", "steady RANS, speedline"));
    Draft {
        kind: Kind::Decision,
        id: format!("DEC-{number:03}"),
        title: format!("Compressor case {number}: keep the {topic} setting"),
        fields,
        sections: vec![
            pair(
                "Decision",
                &format!(
                    "In compressor case {number} the {topic} is run transient, and the steady \
                     result is kept only as a reference. The mass flow and the pressure ratio of \
                     every run are compared with the rig data at the same speed."
                ),
            ),
            pair(
                "Reason",
                &format!(
                    "Compressor case {number} showed rotating stall in the transient run where \
                     the steady run had converged, so the steady {topic} cannot be trusted near \
                     the stability limit."
                ),
            ),
            pair(
                "Rejected",
                "Steady runs with a finer mesh: same result, three times the cost.",
            ),
        ],
    }
}

/// A fixture finding. Every third one is a dead end (format.md §1, FND-007).
fn finding(number: usize) -> Draft {
    let topic = TOPICS[number % TOPICS.len()];
    let stamp = stamp(number);
    let mut fields = approved_fields(&stamp);
    let dead_end = number.is_multiple_of(3);
    if dead_end {
        fields.push(pair("outcome", "dead end"));
    }
    fields.push(pair("task", &task(number)));
    fields.push(pair("machine", "axial compressor"));
    let observation = if dead_end {
        format!(
            "Refining the {topic} of compressor case {number} did not remove the oscillation. \
             The cause is not the {topic}."
        )
    } else {
        format!(
            "In compressor case {number} the {topic} changed the stall margin by a few percent \
             between two runs at the same speed. Nothing else in the setup was changed."
        )
    };
    Draft {
        kind: Kind::Finding,
        id: format!("FND-{number:03}"),
        title: format!("Compressor case {number}: the {topic} moves the stall margin"),
        fields,
        sections: vec![
            pair("Observation", &observation),
            pair(
                "Evidence",
                &format!(
                    "Runs {} and {} of compressor case {number}, monitor plots in \
                     results/case-{number:03}/.",
                    2 * number,
                    2 * number + 1
                ),
            ),
        ],
    }
}

/// A fixture lesson, approved, with the sections of format.md §8 and a confidence value.
fn lesson(number: usize) -> Draft {
    let topic = TOPICS[number % TOPICS.len()];
    let stamp = stamp(number);
    let mut fields = approved_fields(&stamp);
    fields.push(pair("confidence", CONFIDENCE[number % CONFIDENCE.len()]));
    fields.push(pair("task", &task(number)));
    fields.push(pair("machine", "axial compressor"));
    fields.push(pair("analysis", "steady RANS, speedline"));
    Draft {
        kind: Kind::Lesson,
        id: format!("LES-{number:03}"),
        title: format!("Compressor lesson {number}: the {topic} holds only near stall"),
        fields,
        sections: vec![
            pair(
                "Statement",
                &format!(
                    "Near the stability limit of a compressor, the {topic} of steady runs gives a \
                     stall margin that the transient runs do not confirm (case {number})."
                ),
            ),
            pair(
                "Valid for",
                "Transonic front stages, tip Mach number above 1.1. Not checked for subsonic rear \
                 stages.",
            ),
            pair(
                "Evidence",
                &format!(
                    "Comparison of steady and transient speedlines, compressor case {number}, \
                     report section 4.2."
                ),
            ),
        ],
    }
}
