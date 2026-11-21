//! Windows and Linux write the same files, with the same times, names, line endings and search
//! order (R-XOS-4, R-XOS-6, R-XOS-7; TEST-11, TEST-3). Each test runs in its own temporary project,
//! with every command at a fixed time given by `run_at`.
//!
//! The three constants (the digest, the file list and the search text) were copied from a run on
//! Windows. The same tests on Linux must give the same values; a difference there is a failure.

mod common;

use common::Project;
use dossier::hash::sha256_hex;
use std::fs;
use std::path::Path;

/// The byte-order mark at the start of a file (format.md §2).
const BYTE_ORDER_MARK: [u8; 3] = [0xEF, 0xBB, 0xBF];

/// The two files of the state folder that describe the command running now. They are left out
/// of the digest: one holds the process ID, the other is the lock itself.
const LOCK_FILES: [&str; 2] = [".state/holder.json", ".state/write.lock"];

/// The folders of the four kinds of entry (format.md §4).
const KIND_FOLDERS: [&str; 4] = ["decisions", "assumptions", "findings", "lessons"];

/// Windows reserved device names (R-XOS-4), compared without their extension.
const RESERVED: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// The fixed times of the commands of `fixed_dossier` and `search_dossier`. Project::new runs
/// its setup at 2026-11-20T10:00:00+01:00.
const AT_1: &str = "2026-11-20T10:01:00+01:00";
const AT_2: &str = "2026-11-20T10:02:00+01:00";
const AT_3: &str = "2026-11-20T10:03:00+01:00";
const AT_4: &str = "2026-11-20T10:04:00+01:00";
const AT_5: &str = "2026-11-20T10:05:00+01:00";

/// The SHA-256 of the files of `fixed_dossier` other than `LOCK_FILES`: for each file, its path
/// with `/`, a line break, its bytes and a line break, in path order. The journal holds init's
/// line naming anna (Project::new runs as anna), so the value does not depend on the user of the
/// machine.
const EXPECTED_DIGEST: &str = "8ef1aa4e8adf1067be22ef292a0b0160a6323680d1452782028c50cddc043729";

/// The paths of the files that the digest covers, in path order.
const EXPECTED_FILES: &[&str] = &[
    ".state/approved.json",
    ".state/history/FND-001/000001.md",
    ".state/history/FND-001/index.jsonl",
    ".state/ids.json",
    ".state/journal.jsonl",
    "assumptions/ASM-001-inlet-profile-is-uniform.md",
    "decisions/DEC-001-use-transient-runs-for-the-last-three-speedline-po.md",
    "dossier.toml",
    "findings/FND-001-the-mass-flow-still-oscillated-by-0-4-with-half-th.md",
    "lessons/LES-001-steady-mixing-plane-runs-overpredict-the-stall-mar.md",
];

/// The exact standard output of `search stall` over `search_dossier`.
const EXPECTED_SEARCH: &str = concat!(
    "LES-001  Steady runs overpredict the stall margin  — # LES-001 · Steady runs overpredict the stall margin\n",
    "FND-001  Stall started at the lowest mass flow  — # FND-001 · Stall started at the lowest mass flow\n",
    "DEC-001  Keep the stall margin check in every speedline  — # DEC-001 · Keep the stall margin check in every speedline\n",
    "FND-002  Mass flow oscillated at the surge line  — - tags: stall margin\n",
    "ASM-001  Inlet flow is uniform  — No measured profile; stall was judged from the uniform inlet\n",
    "5 matches\n",
);

/// A project with a decision, a finding, an assumption and a lesson, and then a change to the
/// finding. Every command is run as anna, at its own fixed time.
fn fixed_dossier() -> Project {
    let project = Project::new();
    project
        .run_at(
            "anna",
            AT_1,
            &[
                "decide",
                "Use transient runs for the last three speedline points",
                "--reason",
                "Steady runs converged where the transient run already showed rotating stall.",
                "--rejected",
                "Steady runs with a finer mesh: same result, three times the cost.",
                "--machine",
                "axial compressor",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            AT_2,
            &[
                "found",
                "The mass flow still oscillated by 0.4 % with half the time step",
                "--evidence",
                "Runs 12 and 13, monitor plots in results/monitors/",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            AT_3,
            &[
                "assume",
                "Inlet profile is uniform",
                "--reason",
                "No measured profile was delivered.",
                "--affects",
                "T-042, T-043",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            AT_4,
            &[
                "learn",
                "Steady mixing-plane runs overpredict the stall margin",
                "--valid-for",
                "Transonic front stages, tip Mach number above 1.1",
                "--evidence",
                "Comparison of steady and transient speedlines, report section 4.2",
                "--confidence",
                "simulated",
                "--machine",
                "axial compressor",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            AT_5,
            &[
                "set",
                "FND-001",
                "--field",
                "machine=axial compressor",
                "--section",
                "Evidence=Runs 12 and 13, monitor plots in results/monitors/",
            ],
        )
        .success();
    project
}

/// A project with four entries whose titles, tags and text hold the word `stall` in different
/// places, created at different times: a title match, a tag match and a text match.
fn search_dossier() -> Project {
    let project = Project::new();
    project
        .run_at(
            "anna",
            AT_1,
            &[
                "decide",
                "Keep the stall margin check in every speedline",
                "--reason",
                "The margin is the design limit.",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            AT_2,
            &["found", "Stall started at the lowest mass flow"],
        )
        .success();
    project
        .run_at(
            "anna",
            AT_3,
            &[
                "found",
                "Mass flow oscillated at the surge line",
                "--tags",
                "stall margin",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            AT_4,
            &[
                "assume",
                "Inlet flow is uniform",
                "--reason",
                "No measured profile; stall was judged from the uniform inlet",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            AT_5,
            &[
                "learn",
                "Steady runs overpredict the stall margin",
                "--valid-for",
                "Transonic front stages",
                "--evidence",
                "Report section 4.2",
                "--confidence",
                "measured",
            ],
        )
        .success();
    project
}

/// The path of `path` under `folder`, with `/` between its parts whatever the system.
fn relative_name(folder: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(folder).expect("a file inside the folder");
    relative
        .components()
        .map(|part| part.as_os_str().to_str().expect("a UTF-8 name").to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// Every file under `folder`, with its path relative to it and its bytes, in path order.
fn files_under(folder: &Path) -> Vec<(String, Vec<u8>)> {
    let mut found = Vec::new();
    let mut pending = vec![folder.to_path_buf()];
    while let Some(current) = pending.pop() {
        let entries = fs::read_dir(&current).expect("a folder of the dossier can be read");
        for entry in entries {
            let path = entry.expect("a folder entry can be read").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let bytes = fs::read(&path).expect("a file of the dossier can be read");
                found.push((relative_name(folder, &path), bytes));
            }
        }
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
}

/// Every file of the dossier of `project`, as `files_under` gives them.
fn dossier_files(project: &Project) -> Vec<(String, Vec<u8>)> {
    files_under(&project.root().join("DOSSIER"))
}

/// The entry files: the Markdown files in the four kind folders, with their text, in path order.
fn entry_files(project: &Project) -> Vec<(String, String)> {
    dossier_files(project)
        .into_iter()
        .filter(|(name, _)| {
            let folder = name.split('/').next().unwrap_or("");
            name.ends_with(".md") && KIND_FOLDERS.contains(&folder)
        })
        .map(|(name, bytes)| {
            let text = String::from_utf8(bytes).expect("an entry file is UTF-8");
            (name, text)
        })
        .collect()
}

/// Whether one part of a path can be created on Windows (R-XOS-4): not a reserved device name,
/// no character Windows forbids, no trailing dot or space, and under 120 characters.
fn windows_safe(part: &str) -> bool {
    let stem = part
        .split('.')
        .next()
        .unwrap_or("")
        .trim_end_matches(' ')
        .to_ascii_lowercase();
    !part.is_empty()
        && part.chars().count() < 120
        && !part.ends_with(['.', ' '])
        && !part
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
        && !RESERVED.contains(&stem.as_str())
}

/// The part of a file name after its ID (format.md §4): `DEC-001-use-title.md` gives
/// `use-title.md`. A name without an ID is returned whole.
fn after_id(name: &str) -> &str {
    let mut parts = name.splitn(3, '-');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(prefix), Some(number), Some(rest))
            if prefix.len() == 3
                && prefix.bytes().all(|byte| byte.is_ascii_uppercase())
                && number.len() >= 3
                && number.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            rest
        }
        _ => name,
    }
}

/// Whether a file name, after its ID, is lower case: ASCII letters in lower case, digits, `-`
/// and `.`.
fn lower_case_after_id(file_name: &str) -> bool {
    let rest = after_id(file_name);
    !rest.is_empty()
        && rest
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
}

/// Whether `text` is a time as format.md §7 writes it: `2026-11-20T14:03:27+01:00`, with a UTC
/// offset of sign, two digits, a colon and two digits.
fn is_stamp(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 25
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit)
        && bytes[10] == b'T'
        && bytes[11..13].iter().all(u8::is_ascii_digit)
        && bytes[13] == b':'
        && bytes[14..16].iter().all(u8::is_ascii_digit)
        && bytes[16] == b':'
        && bytes[17..19].iter().all(u8::is_ascii_digit)
        && matches!(bytes[19], b'+' | b'-')
        && bytes[20..22].iter().all(u8::is_ascii_digit)
        && bytes[22] == b':'
        && bytes[23..25].iter().all(u8::is_ascii_digit)
}

/// Whether `value` starts with a time and then `separator`, as the `approved` and `history`
/// values do.
fn stamp_then(value: &str, separator: &str) -> bool {
    value.get(..25).is_some_and(is_stamp)
        && value
            .get(25..)
            .is_some_and(|rest| rest.starts_with(separator))
}

#[test]
fn the_same_commands_give_the_same_dossier_bytes_on_every_system() {
    let project = fixed_dossier();
    let files: Vec<(String, Vec<u8>)> = dossier_files(&project)
        .into_iter()
        .filter(|(name, _)| !LOCK_FILES.contains(&name.as_str()))
        .collect();

    // No file may hold the place where the project was made.
    let root = project.root().to_string_lossy().into_owned();
    let forward = root.replace('\\', "/");
    let leaf = project
        .root()
        .file_name()
        .and_then(|name| name.to_str())
        .expect("the temporary folder has a name")
        .to_string();
    for (name, bytes) in &files {
        let text = String::from_utf8_lossy(bytes);
        assert!(
            !text.contains(&root) && !text.contains(&forward) && !text.contains(&leaf),
            "{name} holds the path of the temporary folder"
        );
    }

    let mut input = Vec::new();
    for (name, bytes) in &files {
        input.extend_from_slice(name.as_bytes());
        input.push(b'\n');
        input.extend_from_slice(bytes);
        input.push(b'\n');
    }
    let digest = sha256_hex(&input);
    let names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
    println!("files: {names:?}");
    println!("digest: {digest}");
    assert_eq!(names, EXPECTED_FILES);
    assert_eq!(digest, EXPECTED_DIGEST);
}

#[test]
fn a_crlf_entry_with_a_byte_order_mark_keeps_every_other_byte() {
    let project = Project::new();
    project
        .run_at(
            "anna",
            AT_1,
            &[
                "found",
                "The mass flow still oscillated by 0.4 % with half the time step",
                "--evidence",
                "Runs 12 and 13, monitor plots in results/monitors/",
            ],
        )
        .success();

    // The file as a Windows editor saves it: CRLF endings and a byte-order mark.
    let path = project.entry_file("FND-001");
    let lf = fs::read_to_string(&path).expect("the finding file");
    let mut written = BYTE_ORDER_MARK.to_vec();
    written.extend_from_slice(lf.replace('\n', "\r\n").as_bytes());
    fs::write(&path, &written).expect("the CRLF file with a byte-order mark is written");

    project
        .run_at("anna", AT_2, &["set", "FND-001", "--field", "machine=x"])
        .success();

    let after = fs::read(&path).expect("the finding file after set");
    assert!(
        after.starts_with(&BYTE_ORDER_MARK),
        "the byte-order mark is kept"
    );
    let before_text = std::str::from_utf8(&written[BYTE_ORDER_MARK.len()..]).expect("UTF-8");
    let after_text = std::str::from_utf8(&after[BYTE_ORDER_MARK.len()..]).expect("UTF-8");
    let before_lines: Vec<&str> = before_text.split_inclusive('\n').collect();
    let after_lines: Vec<&str> = after_text.split_inclusive('\n').collect();

    assert!(
        after_lines.iter().all(|line| line.ends_with("\r\n")),
        "every line ends with CRLF: {after_text:?}"
    );
    // Only the two lines the change adds are new; every other line is the same bytes.
    let added = [
        "- machine: x\r\n",
        "- history: 2026-11-20T10:02:00+01:00 anna: changed — field machine\r\n",
    ];
    let kept: Vec<&str> = after_lines
        .iter()
        .copied()
        .filter(|line| !added.contains(line))
        .collect();
    assert_eq!(kept, before_lines, "every other line is kept byte for byte");
    assert_eq!(after_lines.len(), before_lines.len() + added.len());

    let shown = project.stdout("", &["show", "FND-001"]);
    assert!(
        shown.contains("- machine: x"),
        "show reads the file: {shown}"
    );
}

#[test]
fn every_file_name_is_lower_case_after_its_id_and_safe_on_windows() {
    let project = fixed_dossier();
    let files = dossier_files(&project);
    assert!(files.len() >= 8, "the dossier has its files: {files:?}");

    let mut lower = Vec::new();
    for (name, _) in &files {
        for part in name.split('/') {
            assert!(
                windows_safe(part),
                "{name}: {part} is not a name Windows accepts"
            );
        }
        let file_name = name.rsplit('/').next().unwrap_or(name);
        assert!(
            lower_case_after_id(file_name),
            "{name}: the name after its ID is not lower case"
        );
        lower.push(name.to_lowercase());
    }
    // Two files never differ only in upper and lower case (R-XOS-4).
    lower.sort();
    lower.dedup();
    assert_eq!(lower.len(), files.len(), "no two names differ only in case");
}

#[test]
fn the_name_checks_refuse_what_windows_refuses() {
    assert!(windows_safe("DEC-001-use-transient-runs.md"));
    assert!(!windows_safe("con.md"));
    assert!(!windows_safe("NUL"));
    assert!(!windows_safe("lpt1.txt"));
    assert!(!windows_safe("a:b.md"));
    assert!(!windows_safe("name?.md"));
    assert!(!windows_safe("report."));
    assert!(!windows_safe("report "));
    assert!(lower_case_after_id("DEC-001-use-transient-runs.md"));
    assert!(lower_case_after_id("dossier.toml"));
    assert!(!lower_case_after_id("DEC-001-Use-Transient.md"));
    assert!(!lower_case_after_id("DEC-001-use transient.md"));
}

#[test]
fn every_created_approved_and_history_time_has_a_utc_offset() {
    let project = fixed_dossier();
    let mut checked = 0;
    for (name, text) in entry_files(&project) {
        for line in text.lines().map(|line| line.trim_end_matches('\r')) {
            let Some((key, value)) = line
                .strip_prefix("- ")
                .and_then(|rest| rest.split_once(": "))
            else {
                continue;
            };
            let well_formed = match key {
                "created" | "valid-from" => is_stamp(value),
                "approved" => stamp_then(value, " by "),
                "history" => stamp_then(value, " "),
                _ => continue,
            };
            assert!(
                well_formed,
                "{name}: `{line}` does not hold a time with a UTC offset"
            );
            checked += 1;
        }
    }
    // Four entries: each has created, valid-from and approved, and their history lines.
    assert_eq!(checked, 21, "every time in the entries was checked");
}

#[test]
fn the_time_check_refuses_a_time_without_an_offset() {
    assert!(is_stamp("2026-11-20T10:00:00+01:00"));
    assert!(!is_stamp("2026-11-20 10:00:00+01:00"));
    assert!(!is_stamp("2026-11-20T10:00:00"));
    assert!(!is_stamp("2026-11-20T10:00:00Z"));
    assert!(stamp_then("2026-11-20T10:00:00+01:00 anna: created", " "));
    assert!(!stamp_then("2026-11-20T10:00:00+01:00anna", " "));
}

#[test]
fn search_gives_the_same_text_and_order_on_every_system() {
    let project = search_dossier();
    let stdout = project.stdout("", &["search", "stall"]);
    println!("search:\n{stdout}");
    assert_eq!(stdout, EXPECTED_SEARCH);
}
