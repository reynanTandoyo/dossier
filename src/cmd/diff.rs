//! The `diff` command: what changed in an entry since one of its snapshots (R-HIST-2). Lines only
//! in the snapshot are printed with `- `, lines only in the current file with `+ `, in the order
//! of the files; lines in both are not printed. Nothing is changed.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::snapshot;
use crate::store;
use serde_json::json;
use std::path::{MAIN_SEPARATOR, Path};

/// The arguments of `diff`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the entry, such as DEC-012; any case, with or without a place prefix
    pub id: String,
    /// The snapshot to compare the file with, such as 2 (default: the latest)
    pub n: Option<u32>,
}

/// A line that differs: only in the snapshot (`Removed`) or only in the current file (`Added`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Change<'a> {
    Removed(&'a str),
    Added(&'a str),
}

impl Change<'_> {
    /// The line as it is printed: `- ` or `+ ` in front of the line.
    fn shown(self) -> String {
        match self {
            Change::Removed(line) => format!("- {line}"),
            Change::Added(line) => format!("+ {line}"),
        }
    }
}

/// Runs `diff`: compares snapshot `n` (default: the latest) with the current file, line by line.
/// Problems in other files of the dossier are passed on as warnings.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "diff")?;
    let (_, warnings) = store::all(&ctx.dossier);
    let item = store::get(&ctx.dossier, &args.id)?;
    let id = item.entry.id();
    let n = match args.n {
        Some(n) => n,
        None => latest(&ctx, &id)?,
    };

    let saved = snapshot::load(&ctx.state, &id, n)?;
    let saved = text_of(&saved, &format!("snapshot {n} of {id}"))?;
    let current = text_of(&item.raw, &id)?;
    let changes = changes(&lines(saved), &lines(current));

    let text = if changes.is_empty() {
        "no difference".to_string()
    } else {
        changes
            .iter()
            .map(|change| change.shown())
            .collect::<Vec<String>>()
            .join("\n")
    };
    let removed: Vec<&str> = changes
        .iter()
        .filter_map(|change| match *change {
            Change::Removed(line) => Some(line),
            Change::Added(_) => None,
        })
        .collect();
    let added: Vec<&str> = changes
        .iter()
        .filter_map(|change| match *change {
            Change::Added(line) => Some(line),
            Change::Removed(_) => None,
        })
        .collect();
    let data = json!({"id": id, "n": n, "removed": removed, "added": added});

    let output = Output::with(text, data);
    Ok(warnings.iter().fold(output, |output, warning| {
        output.warn(shown_warning(&ctx.root, warning))
    }))
}

/// The number of the latest snapshot of entry `id`; Code::NotFound when it has none.
fn latest(ctx: &Ctx, id: &str) -> Result<u32> {
    snapshot::list(&ctx.state, id)?
        .iter()
        .map(|snap| snap.n)
        .max()
        .ok_or_else(|| Error::new(Code::NotFound, format!("no snapshots of {id}")))
}

/// The text of `bytes`, which must be UTF-8; otherwise Code::Parse, naming `what`.
fn text_of<'a>(bytes: &'a [u8], what: &str) -> Result<&'a str> {
    std::str::from_utf8(bytes)
        .map_err(|_| Error::new(Code::Parse, format!("{what} is not UTF-8 text")))
}

/// The lines of a text without their line endings.
fn lines(text: &str) -> Vec<&str> {
    text.lines().collect()
}

/// The lines that differ between `old` and `current`, in the order of the files. A longest common
/// subsequence of the two lists decides which lines are in both: those are left out, every other
/// line of `old` is removed and every other line of `current` is added. Where a removed and an
/// added line meet, the removed one comes first.
fn changes<'a>(old: &[&'a str], current: &[&'a str]) -> Vec<Change<'a>> {
    let (n, m) = (old.len(), current.len());
    // `common[i][j]` is the length of the longest common subsequence of old[i..] and current[j..].
    let mut common = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            common[i][j] = if old[i] == current[j] {
                common[i + 1][j + 1] + 1
            } else {
                common[i + 1][j].max(common[i][j + 1])
            };
        }
    }

    let (mut i, mut j) = (0, 0);
    let mut changes = Vec::new();
    while i < n && j < m {
        if old[i] == current[j] {
            i += 1;
            j += 1;
        } else if common[i + 1][j] >= common[i][j + 1] {
            changes.push(Change::Removed(old[i]));
            i += 1;
        } else {
            changes.push(Change::Added(current[j]));
            j += 1;
        }
    }
    changes.extend(old[i..].iter().map(|line| Change::Removed(line)));
    changes.extend(current[j..].iter().map(|line| Change::Added(line)));
    changes
}

/// A warning of `store::all` (`<path>: <message>`) with its path shown relative to the project
/// folder, with `/` between the parts. A warning that does not start in the project folder is
/// returned as it is.
fn shown_warning(root: &Path, warning: &str) -> String {
    let root_text = root.display().to_string();
    let Some(rest) = warning.strip_prefix(root_text.as_str()) else {
        return warning.to_string();
    };
    let rest = rest.strip_prefix(['\\', '/']).unwrap_or(rest);
    match rest.split_once(": ") {
        Some((path, message)) => format!("{}: {message}", path.replace(MAIN_SEPARATOR, "/")),
        None => warning.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The printed changes of two texts, one line each.
    fn shown_changes(old: &str, current: &str) -> Vec<String> {
        let old = lines(old);
        let current = lines(current);
        changes(&old, &current)
            .iter()
            .map(|change| change.shown())
            .collect()
    }

    #[test]
    fn equal_texts_have_no_changes() {
        assert!(shown_changes("a\nb\n", "a\nb\n").is_empty());
    }

    #[test]
    fn an_added_line_is_shown_with_plus() {
        assert_eq!(shown_changes("a\nc\n", "a\nb\nc\n"), ["+ b"]);
    }

    #[test]
    fn a_removed_line_is_shown_with_minus() {
        assert_eq!(shown_changes("a\nb\nc\n", "a\nc\n"), ["- b"]);
    }

    #[test]
    fn a_changed_line_is_removed_then_added_in_file_order() {
        assert_eq!(
            shown_changes("a\nold\nc\n", "a\nnew\nc\n"),
            ["- old", "+ new"]
        );
    }

    #[test]
    fn a_line_that_moves_is_removed_and_added_and_the_common_lines_are_left_out() {
        // The longest common subsequence keeps b and c; a moves from the front to the end.
        assert_eq!(shown_changes("a\nb\nc\n", "b\nc\na\n"), ["- a", "+ a"]);
    }

    #[test]
    fn a_text_without_lines_gives_only_additions() {
        assert_eq!(shown_changes("", "x\ny\n"), ["+ x", "+ y"]);
    }
}
