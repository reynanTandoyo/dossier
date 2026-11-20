//! Finding the DOSSIER folder of a project.

use crate::error::{Code, Error, Result};
use std::path::{Path, PathBuf};

/// The name of the dossier folder (format.md §9).
const FOLDER: &str = "DOSSIER";
/// The settings file that marks a dossier folder (format.md §10).
const SETTINGS: &str = "dossier.toml";

/// Whether `folder` is a dossier folder: a folder that contains dossier.toml.
fn is_dossier(folder: &Path) -> bool {
    folder.is_dir() && folder.join(SETTINGS).is_file()
}

/// The DOSSIER folder: `explicit` if given, else the first `DOSSIER` folder found walking up
/// from `start`. Missing: Code::NoDossier, message names `dossier init`.
pub fn find(start: &Path, explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(folder) = explicit {
        if is_dossier(folder) {
            return Ok(folder.to_path_buf());
        }
        return Err(Error::new(
            Code::NoDossier,
            format!(
                "{} is not a dossier folder (no dossier.toml)",
                folder.display()
            ),
        ));
    }
    start
        .ancestors()
        .map(|folder| folder.join(FOLDER))
        .find(|candidate| is_dossier(candidate))
        .ok_or_else(|| {
            Error::new(
                Code::NoDossier,
                "no DOSSIER folder found here or above; create one with: dossier init --code <code>",
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Makes `parent/DOSSIER` with a settings file in it, and returns the DOSSIER folder.
    fn make_dossier(parent: &Path) -> PathBuf {
        let folder = parent.join(FOLDER);
        fs::create_dir_all(&folder).expect("create the DOSSIER folder");
        fs::write(folder.join(SETTINGS), "format = 1\ncode = \"P-001\"\n")
            .expect("write dossier.toml");
        folder
    }

    /// Makes `path` and all its missing parents, and returns it.
    fn make_dir(path: &Path) -> PathBuf {
        fs::create_dir_all(path).expect("create a folder");
        path.to_path_buf()
    }

    #[test]
    fn finds_the_folder_in_the_start_folder() {
        let dir = tempfile::tempdir().expect("temp folder");
        let expected = make_dossier(dir.path());
        assert_eq!(find(dir.path(), None).expect("found"), expected);
    }

    #[test]
    fn finds_the_folder_two_levels_up() {
        let dir = tempfile::tempdir().expect("temp folder");
        let expected = make_dossier(dir.path());
        let start = make_dir(&dir.path().join("first").join("second"));
        assert_eq!(find(&start, None).expect("found"), expected);
    }

    #[test]
    fn the_nearest_folder_wins() {
        let dir = tempfile::tempdir().expect("temp folder");
        make_dossier(dir.path());
        let nearest = make_dossier(&dir.path().join("first"));
        let start = make_dir(&dir.path().join("first").join("second"));
        assert_eq!(find(&start, None).expect("found"), nearest);
    }

    #[test]
    fn a_dossier_named_folder_without_settings_is_skipped() {
        let dir = tempfile::tempdir().expect("temp folder");
        let expected = make_dossier(dir.path());
        make_dir(&dir.path().join("first").join(FOLDER));
        let start = make_dir(&dir.path().join("first"));
        assert_eq!(find(&start, None).expect("found"), expected);
    }

    #[test]
    fn not_found_says_how_to_create_one() {
        let dir = tempfile::tempdir().expect("temp folder");
        let start = make_dir(&dir.path().join("empty"));
        let error = find(&start, None).expect_err("no dossier here");
        assert_eq!(error.code, Code::NoDossier);
        assert_eq!(
            error.message,
            "no DOSSIER folder found here or above; create one with: dossier init --code <code>"
        );
    }

    #[test]
    fn explicit_folder_with_settings_is_accepted() {
        let dir = tempfile::tempdir().expect("temp folder");
        let start = make_dir(&dir.path().join("start"));
        // The explicit folder need not be called DOSSIER: it only has to hold dossier.toml.
        let explicit = make_dir(&dir.path().join("archive"));
        fs::write(explicit.join(SETTINGS), "format = 1\n").expect("write dossier.toml");
        assert_eq!(find(&start, Some(&explicit)).expect("accepted"), explicit);
    }

    #[test]
    fn explicit_folder_wins_over_a_search() {
        let dir = tempfile::tempdir().expect("temp folder");
        make_dossier(dir.path());
        let other = make_dossier(&dir.path().join("other"));
        let start = make_dir(&dir.path().join("start"));
        assert_eq!(find(&start, Some(&other)).expect("accepted"), other);
    }

    #[test]
    fn explicit_folder_without_settings_is_refused_and_not_searched_for() {
        let dir = tempfile::tempdir().expect("temp folder");
        // A dossier above the start folder must not be used when an explicit path is refused.
        make_dossier(dir.path());
        let plain = make_dir(&dir.path().join("plain"));
        let error = find(dir.path(), Some(&plain)).expect_err("refused");
        assert_eq!(error.code, Code::NoDossier);
        assert_eq!(
            error.message,
            format!(
                "{} is not a dossier folder (no dossier.toml)",
                plain.display()
            )
        );
    }

    #[test]
    fn explicit_path_that_does_not_exist_is_refused() {
        let dir = tempfile::tempdir().expect("temp folder");
        let missing = dir.path().join("missing").join(FOLDER);
        let error = find(dir.path(), Some(&missing)).expect_err("refused");
        assert_eq!(error.code, Code::NoDossier);
        assert_eq!(
            error.message,
            format!(
                "{} is not a dossier folder (no dossier.toml)",
                missing.display()
            )
        );
    }

    #[test]
    fn folder_names_with_a_space_and_an_umlaut_work() {
        let dir = tempfile::tempdir().expect("temp folder");
        let project = make_dir(&dir.path().join("Projekt Äpfel Nord"));
        let expected = make_dossier(&project);
        let start = make_dir(&project.join("Übersicht Teil"));
        assert_eq!(find(&start, None).expect("found"), expected);
        assert_eq!(find(&start, Some(&expected)).expect("accepted"), expected);
    }
}
