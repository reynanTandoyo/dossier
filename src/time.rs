//! Time stamps: the format of format.md §7, and the day arithmetic the checks need.

/// The time now in the format of format.md §7. `DOSSIER_NOW` replaces it when it parses.
pub fn now() -> String {
    String::new()
}

/// Whether `stamp` is a time stamp in the format of format.md §7.
pub fn valid(stamp: &str) -> bool {
    false
}

/// The date part of a time stamp: its first 10 characters.
pub fn date(stamp: &str) -> &str {
    ""
}

/// Whole days from `earlier` to `later`; negative when `later` is before `earlier`.
pub fn days_between(earlier: &str, later: &str) -> i64 {
    0
}
