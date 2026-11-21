//! Commits the changes of a team dossier to git (D36).

use crate::error::{Code, Error, Result};
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// Runs `git add -- <path>` and `git commit -m <message> -- <path>` in `repo`. Ok(false) when
/// `repo` is not a git work tree or there is nothing to commit; Code::Io when git fails.
pub fn commit(repo: &Path, path: &Path, message: &str) -> Result<bool> {
    if !inside_work_tree(repo) {
        return Ok(false);
    }
    run(git(repo).arg("add").arg("--").arg(path), "git add")?;
    if !has_staged_change(repo, path)? {
        return Ok(false);
    }
    run(
        git(repo)
            .args(["commit", "-q", "-m"])
            .arg(message)
            .arg("--")
            .arg(path),
        "git commit",
    )?;
    Ok(true)
}

/// A `git` command that runs in `repo` with its standard input closed.
fn git(repo: &Path) -> Command {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo).stdin(Stdio::null());
    command
}

/// Whether `repo` is a git work tree. A missing git, or a folder git does not know, is not one.
fn inside_work_tree(repo: &Path) -> bool {
    git(repo)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .is_ok_and(|output| output.status.success() && output.stdout.trim_ascii() == b"true")
}

/// Whether `path` has a staged change: `git diff --cached --quiet` exits 0 when nothing is
/// staged there and 1 when something is. Any other exit is Code::Io.
fn has_staged_change(repo: &Path, path: &Path) -> Result<bool> {
    let output = git(repo)
        .args(["diff", "--cached", "--quiet", "--"])
        .arg(path)
        .output()
        .map_err(|error| Error::new(Code::Io, format!("git diff could not run: {error}")))?;
    match output.status.code() {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err(failure("git diff", &output)),
    }
}

/// Runs `command` and returns its output when it succeeds. A command that cannot start or that
/// fails is Code::Io, and the message carries git's own words.
fn run(command: &mut Command, what: &str) -> Result<Output> {
    let output = command
        .output()
        .map_err(|error| Error::new(Code::Io, format!("{what} could not run: {error}")))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(failure(what, &output))
    }
}

/// Code::Io for a failed `what`, with git's error text (or its exit status when git said nothing).
fn failure(what: &str, output: &Output) -> Error {
    let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let detail = if detail.is_empty() {
        output.status.to_string()
    } else {
        detail
    };
    Error::new(Code::Io, format!("{what} failed: {detail}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Whether git runs here. The repository tests return early without it.
    fn git_installed() -> bool {
        Command::new("git")
            .arg("--version")
            .stdin(Stdio::null())
            .output()
            .is_ok_and(|output| output.status.success())
    }

    /// Makes `dir` a repository with an invented identity and no commit signing, so the
    /// commits work on any machine.
    fn init_repository(dir: &Path) {
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.name", "Test Person"],
            vec!["config", "user.email", "test.person@example.invalid"],
            vec!["config", "commit.gpgsign", "false"],
        ] {
            let output = git(dir).args(&args).output().expect("git runs");
            assert!(
                output.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    /// The subjects of the commits in `dir`, newest first, one per line.
    fn subjects(dir: &Path) -> Vec<String> {
        let output = git(dir)
            .args(["log", "--format=%s"])
            .output()
            .expect("git log runs");
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn outside_a_repository_nothing_happens() {
        let folder = tempfile::tempdir().expect("temporary folder");
        let dossier = folder.path().join("DOSSIER");
        std::fs::create_dir_all(&dossier).expect("folder");
        std::fs::write(dossier.join("lesson.md"), "text\n").expect("file");
        assert!(!commit(folder.path(), &dossier, "T-219 test: nothing").expect("commit"));
        assert!(!folder.path().join(".git").exists());
    }

    #[test]
    fn a_folder_that_does_not_exist_is_not_a_repository() {
        let folder = tempfile::tempdir().expect("temporary folder");
        let missing = folder.path().join("missing");
        assert!(!commit(&missing, &missing, "T-219 test: nothing").expect("commit"));
    }

    #[test]
    fn changes_in_a_work_tree_are_committed_with_their_message() {
        if !git_installed() {
            return;
        }
        let folder = tempfile::tempdir().expect("temporary folder");
        init_repository(folder.path());
        // A name with a space and non-ASCII letters must work.
        let dossier = folder.path().join("Team dossier äö");
        std::fs::create_dir_all(&dossier).expect("folder");
        std::fs::write(dossier.join("lesson.md"), "first\n").expect("file");
        std::fs::write(folder.path().join("other.txt"), "not dossier\n").expect("file");

        assert!(commit(folder.path(), &dossier, "T-219 test: first change").expect("commit"));
        assert_eq!(subjects(folder.path()), ["T-219 test: first change"]);
        let files = git(folder.path())
            .args(["show", "--name-only", "--format=", "HEAD"])
            .output()
            .expect("git show runs");
        let files = String::from_utf8_lossy(&files.stdout);
        assert!(files.contains("lesson.md"), "{files}");
        assert!(!files.contains("other.txt"), "{files}");
    }

    #[test]
    fn nothing_new_gives_false_and_no_commit() {
        if !git_installed() {
            return;
        }
        let folder = tempfile::tempdir().expect("temporary folder");
        init_repository(folder.path());
        let dossier = folder.path().join("DOSSIER");
        std::fs::create_dir_all(&dossier).expect("folder");
        std::fs::write(dossier.join("lesson.md"), "first\n").expect("file");
        assert!(commit(folder.path(), &dossier, "T-219 test: first change").expect("commit"));

        assert!(!commit(folder.path(), &dossier, "T-219 test: again").expect("commit"));
        assert_eq!(subjects(folder.path()), ["T-219 test: first change"]);

        std::fs::write(dossier.join("lesson.md"), "second\n").expect("rewrite");
        assert!(commit(folder.path(), &dossier, "T-219 test: second change").expect("commit"));
        assert_eq!(
            subjects(folder.path()),
            ["T-219 test: second change", "T-219 test: first change"]
        );
    }

    #[test]
    fn git_refusing_the_path_is_an_io_error_with_its_message() {
        if !git_installed() {
            return;
        }
        let repo = tempfile::tempdir().expect("temporary folder");
        init_repository(repo.path());
        let elsewhere = tempfile::tempdir().expect("temporary folder");
        let outside = elsewhere.path().join("DOSSIER");
        std::fs::create_dir_all(&outside).expect("folder");
        std::fs::write(outside.join("lesson.md"), "text\n").expect("file");

        let error = commit(repo.path(), &outside, "T-219 test: outside")
            .expect_err("git refuses a path outside the repository");
        assert_eq!(error.code, Code::Io);
        assert!(
            error.message.starts_with("git add failed: "),
            "{}",
            error.message
        );
        assert!(subjects(repo.path()).is_empty());
    }
}
