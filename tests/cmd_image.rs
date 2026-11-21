//! Runs the built program for `image`: `image add` copies a small picture into DOSSIER/images/<ID>/
//! and adds its line, with the caption, to the Pictures section; `image remove` takes the line out
//! and deletes the file. Each test runs in its own temporary project folder with the clock fixed by
//! `common::Project` (2026-11-20T10:00:00+01:00). The pictures and texts are invented.

mod common;

use common::Project;
use dossier::approval::{self, Mark};
use dossier::entry::{Entry, Status};
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in history lines.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// Invented texts for a finding and a lesson, and the caption of a picture.
const OBSERVATION: &str = "The mass flow still oscillated by 0.4 % with half the time step.";
const EVIDENCE: &str = "Runs 12 and 13, monitor plots in results.";
const STATEMENT: &str = "Steady mixing-plane runs overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const LESSON_EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";
const CAPTION: &str = "Contour of the rod; look at the hot spot";

/// A small PNG: the 8 signature bytes and a few more.
fn png_bytes() -> Vec<u8> {
    let mut bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    bytes.extend_from_slice(b"invented picture data");
    bytes
}

/// Writes bytes to a file relative to the project folder, creating its folders as needed.
fn write_bytes(project: &Project, relative: &str, bytes: &[u8]) {
    let path = project.root().join(relative);
    std::fs::create_dir_all(path.parent().expect("a folder")).expect("the folder");
    std::fs::write(&path, bytes).expect("the file");
}

/// The text of the entry file with `id`.
fn text_of(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the entry file can be read")
}

/// The entry with `id`, parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    Entry::parse(&text_of(project, id)).expect("the file is an entry")
}

/// The picture file of an entry in the dossier.
fn picture_file(project: &Project, id: &str, name: &str) -> std::path::PathBuf {
    project
        .root()
        .join("DOSSIER")
        .join("images")
        .join(id)
        .join(name)
}

/// A finding written by anna, a person: approved at once (FND-001).
fn finding(project: &Project) {
    project.stdout(
        "anna",
        &[
            "found",
            OBSERVATION,
            "--evidence",
            EVIDENCE,
            "--machine",
            "axial compressor",
        ],
    );
}

/// A lesson written by anna, a person: approved at once (LES-001).
fn lesson(project: &Project) {
    project.stdout(
        "anna",
        &[
            "learn",
            STATEMENT,
            "--valid-for",
            VALID_FOR,
            "--evidence",
            LESSON_EVIDENCE,
            "--confidence",
            "simulated",
        ],
    );
}

/// The approval record of an entry, read from the state folder.
fn approval_of(project: &Project, id: &str, text: &str) -> Mark {
    let state = project.root().join("DOSSIER").join(".state");
    approval::mark(&state, id, text.as_bytes())
}

#[test]
fn a_picture_is_copied_into_the_dossier_and_its_line_is_in_the_pictures_section() {
    let project = Project::new();
    finding(&project);
    let before = text_of(&project, "FND-001");
    let picture = png_bytes();
    write_bytes(&project, "plots/Contour Plot.PNG", &picture);

    project
        .run(
            "anna",
            &[
                "image",
                "add",
                "FND-001",
                "plots/Contour Plot.PNG",
                "--caption",
                CAPTION,
            ],
        )
        .success()
        .stdout("FND-001: picture added: contour-plot.png\n");

    assert_eq!(
        std::fs::read(picture_file(&project, "FND-001", "contour-plot.png")).expect("the copy"),
        picture
    );
    let after = text_of(&project, "FND-001");
    let history = format!("- history: {WHEN} anna: picture — added contour-plot.png\n");
    let section = format!("\n## Pictures\n\n![{CAPTION}](../images/FND-001/contour-plot.png)\n");
    assert_eq!(after.matches(&history).count(), 1);
    assert!(after.ends_with(&section));
    // Without the history line and the Pictures section the entry is as it was.
    assert_eq!(
        after.replacen(&history, "", 1).replacen(&section, "", 1),
        before
    );
}

#[test]
fn show_prints_the_picture_line_and_the_journal_names_who_and_when() {
    let project = Project::new();
    finding(&project);
    write_bytes(&project, "plots/contour-plot.png", &png_bytes());
    project.stdout(
        "anna",
        &[
            "image",
            "add",
            "FND-001",
            "plots/contour-plot.png",
            "--caption",
            CAPTION,
        ],
    );

    let shown = project.stdout("anna", &["show", "FND-001"]);
    assert!(shown.contains(&format!("![{CAPTION}](../images/FND-001/contour-plot.png)")));

    let records: Vec<_> = project
        .entry_journal()
        .into_iter()
        .filter(|record| record.event == "picture")
        .collect();
    assert!(!records.is_empty());
    for record in &records {
        assert_eq!(record.id, "FND-001");
        assert_eq!(record.name, "anna");
        assert_eq!(record.time, WHEN);
        assert_eq!(record.detail, "added contour-plot.png");
    }
}

#[test]
fn remove_takes_the_line_and_the_file_out_and_an_empty_section_goes() {
    let project = Project::new();
    finding(&project);
    let before = text_of(&project, "FND-001");
    write_bytes(&project, "plots/contour-plot.png", &png_bytes());
    project.stdout(
        "anna",
        &[
            "image",
            "add",
            "FND-001",
            "plots/contour-plot.png",
            "--caption",
            CAPTION,
        ],
    );

    project
        .run("anna", &["image", "remove", "FND-001", "contour-plot.png"])
        .success()
        .stdout("FND-001: picture removed: contour-plot.png\n");

    assert!(!picture_file(&project, "FND-001", "contour-plot.png").exists());
    let after = text_of(&project, "FND-001");
    let added = format!("- history: {WHEN} anna: picture — added contour-plot.png\n");
    let removed = format!("- history: {WHEN} anna: picture — removed contour-plot.png\n");
    assert!(!after.contains("## Pictures"));
    assert_eq!(
        after.replacen(&added, "", 1).replacen(&removed, "", 1),
        before
    );
}

#[test]
fn remove_keeps_the_other_pictures_of_the_entry() {
    let project = Project::new();
    finding(&project);
    write_bytes(&project, "plots/first.png", &png_bytes());
    write_bytes(&project, "plots/second.png", &png_bytes());
    project.stdout(
        "anna",
        &[
            "image",
            "add",
            "FND-001",
            "plots/first.png",
            "--caption",
            "First plot",
        ],
    );
    project.stdout(
        "anna",
        &[
            "image",
            "add",
            "FND-001",
            "plots/second.png",
            "--caption",
            "Second plot",
        ],
    );

    project.stdout("anna", &["image", "remove", "FND-001", "first.png"]);

    let after = text_of(&project, "FND-001");
    // The history line that says "added first.png" stays; only the picture line goes.
    assert!(!after.contains("](../images/FND-001/first.png)"));
    assert!(after.contains("## Pictures\n\n![Second plot](../images/FND-001/second.png)\n"));
    assert!(!picture_file(&project, "FND-001", "first.png").exists());
    assert!(picture_file(&project, "FND-001", "second.png").exists());
}

#[test]
fn a_text_file_is_refused_and_nothing_is_written() {
    let project = Project::new();
    finding(&project);
    let before = text_of(&project, "FND-001");
    write_bytes(
        &project,
        "plots/notes.png",
        b"This is a note, not a picture.\n",
    );

    project
        .run(
            "anna",
            &[
                "image",
                "add",
                "FND-001",
                "plots/notes.png",
                "--caption",
                CAPTION,
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("a picture is PNG, JPEG or SVG"));

    assert_eq!(text_of(&project, "FND-001"), before);
    assert!(!project.root().join("DOSSIER/images").exists());
}

#[test]
fn a_picture_above_2_mb_is_refused() {
    let project = Project::new();
    finding(&project);
    let mut bytes = png_bytes();
    bytes.resize(2 * 1024 * 1024 + 1, b'x');
    write_bytes(&project, "plots/big.png", &bytes);

    project
        .run(
            "anna",
            &[
                "image",
                "add",
                "FND-001",
                "plots/big.png",
                "--caption",
                CAPTION,
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("at most 2097152 bytes"));

    assert!(!project.root().join("DOSSIER/images").exists());
}

#[test]
fn an_eleventh_picture_is_refused() {
    let project = Project::new();
    finding(&project);
    for number in 1..=10 {
        let relative = format!("plots/plot-{number}.png");
        write_bytes(&project, &relative, &png_bytes());
        project.stdout(
            "anna",
            &[
                "image",
                "add",
                "FND-001",
                relative.as_str(),
                "--caption",
                "A plot",
            ],
        );
    }
    let before = text_of(&project, "FND-001");
    write_bytes(&project, "plots/plot-11.png", &png_bytes());

    project
        .run(
            "anna",
            &[
                "image",
                "add",
                "FND-001",
                "plots/plot-11.png",
                "--caption",
                "A plot",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("at most 10 pictures"));

    assert_eq!(text_of(&project, "FND-001"), before);
    assert!(!picture_file(&project, "FND-001", "plot-11.png").exists());
}

#[test]
fn a_blank_caption_or_one_with_a_line_break_is_refused() {
    let project = Project::new();
    finding(&project);
    write_bytes(&project, "plots/plot.png", &png_bytes());

    project
        .run(
            "anna",
            &[
                "image",
                "add",
                "FND-001",
                "plots/plot.png",
                "--caption",
                "   ",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("needs a caption"));
    project
        .run(
            "anna",
            &[
                "image",
                "add",
                "FND-001",
                "plots/plot.png",
                "--caption",
                "first line\nsecond line",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("one line"));

    assert!(!project.root().join("DOSSIER/images").exists());
    assert!(!text_of(&project, "FND-001").contains("## Pictures"));
}

#[test]
fn a_picture_of_the_same_name_gets_a_number_and_a_folder_with_letters_works() {
    let project = Project::new();
    finding(&project);
    write_bytes(&project, "Bilder für Tests/Kontur Plot.PNG", &png_bytes());
    write_bytes(
        &project,
        "Bilder für Tests/copy/Kontur Plot.PNG",
        &png_bytes(),
    );

    project
        .run(
            "anna",
            &[
                "image",
                "add",
                "FND-001",
                "Bilder für Tests/Kontur Plot.PNG",
                "--caption",
                "First",
            ],
        )
        .success()
        .stdout("FND-001: picture added: kontur-plot.png\n");
    project
        .run(
            "anna",
            &[
                "image",
                "add",
                "FND-001",
                "Bilder für Tests/copy/Kontur Plot.PNG",
                "--caption",
                "Second",
            ],
        )
        .success()
        .stdout("FND-001: picture added: kontur-plot-2.png\n");

    assert!(picture_file(&project, "FND-001", "kontur-plot-2.png").exists());
    assert!(
        text_of(&project, "FND-001").contains("![Second](../images/FND-001/kontur-plot-2.png)")
    );
}

#[test]
fn removing_a_picture_whose_file_is_gone_removes_only_its_line() {
    let project = Project::new();
    finding(&project);
    write_bytes(&project, "plots/plot.png", &png_bytes());
    project.stdout(
        "anna",
        &[
            "image",
            "add",
            "FND-001",
            "plots/plot.png",
            "--caption",
            CAPTION,
        ],
    );
    std::fs::remove_file(picture_file(&project, "FND-001", "plot.png")).expect("the file goes");

    project
        .run("anna", &["image", "remove", "FND-001", "plot.png"])
        .success()
        .stdout("FND-001: picture removed: plot.png (its file was already gone)\n");

    assert!(!text_of(&project, "FND-001").contains("## Pictures"));
}

#[test]
fn a_picture_that_the_entry_does_not_have_is_not_found() {
    let project = Project::new();
    finding(&project);
    let before = text_of(&project, "FND-001");

    project
        .run("anna", &["image", "remove", "FND-001", "missing.png"])
        .failure()
        .code(1)
        .stderr(contains("has no picture missing.png"));

    assert_eq!(text_of(&project, "FND-001"), before);
}

#[test]
fn a_file_that_is_not_there_is_not_found() {
    let project = Project::new();
    finding(&project);

    project
        .run(
            "anna",
            &[
                "image",
                "add",
                "FND-001",
                "plots/nothing.png",
                "--caption",
                CAPTION,
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("no file at"));
}

#[test]
fn an_entry_that_does_not_exist_is_not_found() {
    let project = Project::new();
    write_bytes(&project, "plots/plot.png", &png_bytes());

    project
        .run(
            "anna",
            &[
                "image",
                "add",
                "FND-009",
                "plots/plot.png",
                "--caption",
                CAPTION,
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("no entry FND-009"));
}

#[test]
fn without_a_name_the_command_is_refused() {
    let project = Project::new();
    finding(&project);
    write_bytes(&project, "plots/plot.png", &png_bytes());

    project
        .run(
            "",
            &[
                "image",
                "add",
                "FND-001",
                "plots/plot.png",
                "--caption",
                CAPTION,
            ],
        )
        .failure()
        .code(2);
}

#[test]
fn an_llm_adds_a_picture_as_itself_and_an_approved_lesson_goes_back_to_draft() {
    let project = Project::new();
    lesson(&project);
    assert!(matches!(
        approval_of(&project, "LES-001", &text_of(&project, "LES-001")),
        Mark::Unchanged
    ));
    write_bytes(&project, "plots/plot.png", &png_bytes());

    project
        .run(
            "anna-claude",
            &[
                "image",
                "add",
                "LES-001",
                "plots/plot.png",
                "--caption",
                CAPTION,
            ],
        )
        .success()
        .stdout("LES-001: picture added: plot.png (back to draft, waiting for approval)\n");

    let after = text_of(&project, "LES-001");
    assert_eq!(entry_of(&project, "LES-001").status(), Some(Status::Draft));
    assert!(after.contains(&format!(
        "- history: {WHEN} anna via anna-claude: picture — added plot.png, back to draft\n"
    )));
    assert!(after.contains(&format!("![{CAPTION}](../images/LES-001/plot.png)")));
    assert!(matches!(
        approval_of(&project, "LES-001", &after),
        Mark::NotRecorded
    ));
}

#[test]
fn a_person_adds_a_picture_to_an_approved_lesson_and_it_stays_approved() {
    let project = Project::new();
    lesson(&project);
    write_bytes(&project, "plots/plot.png", &png_bytes());

    project.stdout(
        "anna",
        &[
            "image",
            "add",
            "LES-001",
            "plots/plot.png",
            "--caption",
            CAPTION,
        ],
    );

    let after = text_of(&project, "LES-001");
    assert_eq!(
        entry_of(&project, "LES-001").status(),
        Some(Status::Approved)
    );
    assert!(matches!(
        approval_of(&project, "LES-001", &after),
        Mark::Unchanged
    ));
}
