//! Code pointers: the `code` field of an entry names a file or a folder, optionally followed by
//! `::` and a symbol (R-CODE-6 to R-CODE-8). Every command matches a pointer to a path with the
//! functions here, so they agree on when a pointer covers a path.

/// The path of a code pointer: the part before its first `::` (all of it when none), trimmed,
/// backslashes as slashes, a leading `./` and trailing slashes removed.
pub fn path_of(pointer: &str) -> String {
    let before = match pointer.find("::") {
        Some(at) => &pointer[..at],
        None => pointer,
    };
    let slashed = before.trim().replace('\\', "/");
    let relative = slashed.strip_prefix("./").unwrap_or(slashed.as_str());
    relative.trim_end_matches('/').to_string()
}

/// True when the pointer's path is the stored path, or a folder that holds it.
pub fn covers(pointer: &str, stored: &str) -> bool {
    let path = path_of(pointer);
    !path.is_empty() && is_or_within(stored, &path)
}

/// True when the pointer's path is the folder, or lies in it.
pub fn lies_in(pointer: &str, folder: &str) -> bool {
    is_or_within(&path_of(pointer), folder)
}

/// True when `path` is `folder` itself or a path below it, separated by a slash.
fn is_or_within(path: &str, folder: &str) -> bool {
    path == folder
        || path
            .strip_prefix(folder)
            .is_some_and(|rest| rest.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::{covers, lies_in, path_of};

    #[test]
    fn path_of_drops_the_symbol_and_surrounding_spaces_and_makes_slashes() {
        assert_eq!(
            path_of(" src\\pump\\seal.rs::leak_model "),
            "src/pump/seal.rs"
        );
    }

    #[test]
    fn path_of_drops_a_leading_dot_slash_and_trailing_slashes() {
        assert_eq!(path_of("./src/pump/"), "src/pump");
        assert_eq!(path_of("src/pump///"), "src/pump");
    }

    #[test]
    fn path_of_cuts_at_the_first_double_colon_only() {
        assert_eq!(
            path_of("src/pump/seal.rs::outer::inner"),
            "src/pump/seal.rs"
        );
    }

    #[test]
    fn path_of_keeps_the_text_when_there_is_no_symbol() {
        assert_eq!(path_of("src/pump/seal.rs"), "src/pump/seal.rs");
        assert_eq!(path_of("C:\\Project\\dossier"), "C:/Project/dossier");
    }

    #[test]
    fn path_of_is_empty_for_an_empty_or_slash_only_pointer() {
        assert_eq!(path_of(""), "");
        assert_eq!(path_of("./"), "");
        assert_eq!(path_of("::leak_model"), "");
    }

    #[test]
    fn path_of_keeps_spaces_and_non_ascii_letters_in_the_path() {
        assert_eq!(
            path_of(" notes\\Ärger plan/café.md::Größe "),
            "notes/Ärger plan/café.md"
        );
    }

    #[test]
    fn covers_a_folder_holds_the_file() {
        assert!(covers("src/pump", "src/pump/seal.rs"));
        assert!(covers("src\\pump\\", "src/pump/seal.rs"));
        assert!(covers("./src/pump::anything", "src/pump/seal.rs"));
    }

    #[test]
    fn covers_is_not_a_text_prefix_match() {
        assert!(!covers("src/pum", "src/pump/seal.rs"));
        assert!(!covers("src/pump", "src/pump2/seal.rs"));
    }

    #[test]
    fn covers_a_file_pointer_with_its_own_path() {
        assert!(covers("src/pump/seal.rs::f", "src/pump/seal.rs"));
        assert!(covers("src/pump/seal.rs", "src/pump/seal.rs"));
        assert!(!covers("src/pump/seal.rs", "src/pump/seal.rs.bak"));
    }

    #[test]
    fn covers_is_false_for_an_empty_pointer_path() {
        assert!(!covers("", "a"));
        assert!(!covers("./", "a"));
        assert!(!covers("::f", "a"));
    }

    #[test]
    fn covers_works_with_spaces_and_non_ascii_letters() {
        assert!(covers("notes/Ärger plan", "notes/Ärger plan/Entwurf 1.md"));
        assert!(covers("notes\\Ärger plan::x", "notes/Ärger plan/café.md"));
        assert!(!covers("notes/Ärger", "notes/Ärger plan/café.md"));
    }

    #[test]
    fn lies_in_a_folder_or_is_the_folder() {
        assert!(lies_in("src/pump/seal.rs", "src/pump"));
        assert!(lies_in("src/pump", "src/pump"));
        assert!(lies_in("src\\pump\\seal.rs::leak_model", "src/pump"));
    }

    #[test]
    fn lies_in_is_not_a_text_prefix_match() {
        assert!(!lies_in("src/pump/seal.rs", "src/pum"));
        assert!(!lies_in("src/pump2/seal.rs", "src/pump"));
        assert!(!lies_in("src/pump", "src/pump/seal.rs"));
    }

    #[test]
    fn lies_in_works_with_spaces_and_non_ascii_letters() {
        assert!(lies_in(
            "notes/Ärger plan/café.md::Größe",
            "notes/Ärger plan"
        ));
        assert!(!lies_in("notes/Ärger plan/café.md", "notes/Ärger"));
    }
}
