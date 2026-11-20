//! Paths: the slug of a title, names that Windows accepts, and stored and shown paths.

use crate::error::{Code, Error, Result};
use std::path::Path;

/// The file-name part of a title (format.md §4).
pub fn slug(title: &str) -> String {
    let mut out = String::new();
    for c in title.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    // The result is ASCII, so 50 characters are 50 bytes.
    let cut: String = out.chars().take(50).collect();
    let trimmed = cut.trim_end_matches('-');
    if trimmed.is_empty() {
        "entry".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Whether `name` is allowed on Windows (R-XOS-4, R-XOS-5).
pub fn windows_safe(name: &str) -> bool {
    if name.is_empty() || name.ends_with('.') || name.ends_with(' ') {
        return false;
    }
    if name
        .chars()
        .any(|c| u32::from(c) < 32 || "<>:\"|?*".contains(c))
    {
        return false;
    }
    let stem = name.split('.').next().unwrap_or("").to_lowercase();
    !reserved(&stem)
}

/// Whether a lower-case stem is a device name Windows reserves: con, prn, aux, nul, com1 to
/// com9, lpt1 to lpt9.
fn reserved(stem: &str) -> bool {
    if matches!(stem, "con" | "prn" | "aux" | "nul") {
        return true;
    }
    ["com", "lpt"].iter().any(|prefix| {
        stem.strip_prefix(prefix)
            .is_some_and(|digit| matches!(digit.as_bytes(), [b'1'..=b'9']))
    })
}

/// A typed path (with / or \, relative to cwd or absolute) as a path relative to root, with /.
/// Outside root: Code::BadValue.
pub fn stored(root: &Path, cwd: &Path, typed: &str) -> Result<String> {
    let typed = typed.replace('\\', "/");
    let base = slashed(cwd);
    let full = if is_absolute(&typed) || base.is_empty() {
        typed
    } else if base.ends_with('/') {
        format!("{base}{typed}")
    } else {
        format!("{base}/{typed}")
    };
    let root = split(&slashed(root));
    let path = split(&full);
    let inside = same(&path.anchor, &root.anchor)
        && path.segments.len() >= root.segments.len()
        && root
            .segments
            .iter()
            .zip(&path.segments)
            .all(|(a, b)| same(a, b));
    if !inside {
        return Err(Error::new(
            Code::BadValue,
            "the path is outside the project folder",
        ));
    }
    Ok(path
        .segments
        .iter()
        .skip(root.segments.len())
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("/"))
}

/// A stored path with the separator of this system.
pub fn shown(stored: &str) -> String {
    if cfg!(windows) {
        stored.replace('/', "\\")
    } else {
        stored.to_string()
    }
}

/// The text of a path with / for every \.
fn slashed(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// A path that has been split into its anchor and its segments, with `.` and `..` resolved.
/// Nothing touches the disk.
struct Split {
    /// `/`, `C:/`, `//server/share`, or empty for a relative path.
    anchor: String,
    segments: Vec<String>,
}

fn split(path: &str) -> Split {
    let (anchor, rest) = anchor_of(path);
    let mut segments: Vec<String> = Vec::new();
    for part in rest.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if segments.last().is_some_and(|last| last.as_str() != "..") {
                    segments.pop();
                } else if anchor.is_empty() {
                    segments.push("..".to_string());
                }
            }
            _ => segments.push(part.to_string()),
        }
    }
    Split { anchor, segments }
}

/// The anchor of a path (`//server/share`, `C:/`, `/`, or empty when relative) and the rest.
fn anchor_of(path: &str) -> (String, &str) {
    if let Some(tail) = path.strip_prefix("//") {
        let mut parts = tail.splitn(3, '/');
        let server = parts.next().unwrap_or("");
        let share = parts.next();
        let rest = parts.next().unwrap_or("");
        let anchor = match share {
            Some(share) => format!("//{server}/{share}"),
            None => format!("//{server}"),
        };
        return (anchor, rest);
    }
    let bytes = path.as_bytes();
    if bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/' {
        return (format!("{}/", &path[..2]), &path[3..]);
    }
    if let Some(rest) = path.strip_prefix('/') {
        return ("/".to_string(), rest);
    }
    (String::new(), path)
}

/// Whether a path starts with an anchor (a share, a drive with `/`, or `/`).
fn is_absolute(path: &str) -> bool {
    !anchor_of(path).0.is_empty()
}

/// Two parts of a path are the same: ignoring ASCII case on Windows, exactly elsewhere.
fn same(a: &str, b: &str) -> bool {
    if cfg!(windows) {
        a.eq_ignore_ascii_case(b)
    } else {
        a == b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: &str = "/work/proj";

    fn at(cwd: &str, typed: &str) -> Result<String> {
        stored(Path::new(ROOT), Path::new(cwd), typed)
    }

    #[test]
    fn slug_turns_umlauts_into_single_dashes() {
        assert_eq!(slug("Größe"), "gr-e");
        assert_eq!(slug("Über alles"), "ber-alles");
    }

    #[test]
    fn slug_turns_punctuation_into_single_dashes() {
        assert_eq!(
            slug("Use transient runs (for the last 3 points)!"),
            "use-transient-runs-for-the-last-3-points"
        );
        assert_eq!(
            slug("  --Plot: 2/3 of the  data--  "),
            "plot-2-3-of-the-data"
        );
    }

    #[test]
    fn slug_is_cut_to_fifty_characters_without_a_trailing_dash() {
        assert_eq!(slug(&"a".repeat(80)), "a".repeat(50));
        let title = format!("{} b", "a".repeat(49));
        assert_eq!(slug(&title), "a".repeat(49));
        assert_eq!(
            slug(&"word ".repeat(16)),
            format!("{}word", "word-".repeat(9))
        );
    }

    #[test]
    fn slug_of_an_empty_title_is_entry() {
        assert_eq!(slug(""), "entry");
        assert_eq!(slug("   "), "entry");
        assert_eq!(slug("!? ö"), "entry");
    }

    #[test]
    fn windows_refuses_reserved_names() {
        assert!(!windows_safe("con.md"));
        assert!(!windows_safe("CON.md"));
        assert!(!windows_safe("com1.txt"));
        assert!(!windows_safe("lpt9"));
        assert!(windows_safe("console.md"));
        assert!(windows_safe("com10.md"));
    }

    #[test]
    fn windows_refuses_a_colon_a_trailing_dot_and_empty_names() {
        assert!(!windows_safe("time 10:30.md"));
        assert!(!windows_safe("notes."));
        assert!(!windows_safe("notes "));
        assert!(!windows_safe(""));
        assert!(!windows_safe("a|b.md"));
        assert!(!windows_safe("a\u{1}b.md"));
    }

    #[test]
    fn windows_accepts_a_normal_name() {
        assert!(windows_safe("DEC-001-use-transient-runs.md"));
    }

    #[test]
    fn stored_of_a_relative_path_is_relative_to_the_project() {
        assert_eq!(
            at(ROOT, "decisions/DEC-001.md").unwrap(),
            "decisions/DEC-001.md"
        );
        assert_eq!(
            at("/work/proj/findings", "../decisions/DEC-001.md").unwrap(),
            "decisions/DEC-001.md"
        );
        assert_eq!(at(ROOT, ".").unwrap(), "");
        assert_eq!(at(ROOT, "./lessons/").unwrap(), "lessons");
    }

    #[test]
    fn stored_of_a_path_with_backslashes_is_the_same_as_with_slashes() {
        let with_slashes = at("/work/proj/decisions", "../findings/FND-002.md").unwrap();
        let with_backslashes = at("\\work\\proj\\decisions", "..\\findings\\FND-002.md").unwrap();
        assert_eq!(with_slashes, "findings/FND-002.md");
        assert_eq!(with_backslashes, with_slashes);
        assert_eq!(
            at(ROOT, "decisions\\DEC-001.md").unwrap(),
            "decisions/DEC-001.md"
        );
        assert_eq!(
            at(ROOT, "\\work\\proj\\decisions\\DEC-001.md").unwrap(),
            "decisions/DEC-001.md"
        );
    }

    #[test]
    fn stored_resolves_dot_dot_parts_that_stay_inside() {
        assert_eq!(
            at(ROOT, "decisions/../findings/FND-002.md").unwrap(),
            "findings/FND-002.md"
        );
        assert_eq!(
            at("/work/proj/a/b", "../../decisions/x.md").unwrap(),
            "decisions/x.md"
        );
        assert_eq!(at(ROOT, "/work/proj/../proj/x.md").unwrap(), "x.md");
    }

    #[test]
    fn stored_refuses_a_path_that_leaves_the_project() {
        for typed in [
            "../outside.md",
            "decisions/../../x.md",
            "/work/other/x.md",
            "/work/proj-other/x.md",
        ] {
            let error = at(ROOT, typed).unwrap_err();
            assert_eq!(error.code, Code::BadValue, "{typed}");
            assert_eq!(
                error.message, "the path is outside the project folder",
                "{typed}"
            );
        }
    }

    #[test]
    fn stored_treats_a_drive_path_as_absolute_on_every_system() {
        let root = Path::new("C:/work/proj");
        assert_eq!(
            stored(root, root, "C:/work/proj/decisions/DEC-001.md").unwrap(),
            "decisions/DEC-001.md"
        );
        assert_eq!(
            stored(root, root, "C:\\work\\proj\\decisions\\DEC-001.md").unwrap(),
            "decisions/DEC-001.md"
        );
        assert!(stored(root, root, "D:/work/proj/x.md").is_err());
    }

    #[test]
    fn stored_treats_a_share_path_as_absolute_on_every_system() {
        let root = Path::new("//server/share/proj");
        assert_eq!(
            stored(root, root, "//server/share/proj/a/b.md").unwrap(),
            "a/b.md"
        );
        assert_eq!(
            stored(root, root, "\\\\server\\share\\proj\\a\\b.md").unwrap(),
            "a/b.md"
        );
        assert!(stored(root, root, "//server/other/proj/x.md").is_err());
    }

    #[test]
    fn stored_compares_drive_letters_and_folder_names_by_case_only_on_windows() {
        let root = Path::new("C:/Work/Proj");
        let typed = "C:/work/proj/x.md";
        if cfg!(windows) {
            assert_eq!(stored(root, root, typed).unwrap(), "x.md");
        } else {
            assert!(stored(root, root, typed).is_err());
        }
    }

    #[test]
    fn shown_uses_the_separator_of_this_system() {
        let expected = if cfg!(windows) {
            "decisions\\DEC-001.md"
        } else {
            "decisions/DEC-001.md"
        };
        assert_eq!(shown("decisions/DEC-001.md"), expected);
        assert_eq!(shown(""), "");
    }
}
