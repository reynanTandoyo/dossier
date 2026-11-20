//! Time stamps: the format of format.md §7, and the day arithmetic the checks need.

use chrono::{DateTime, Local};

/// The time now in the format of format.md §7. `DOSSIER_NOW` replaces it when it parses.
pub fn now() -> String {
    match std::env::var("DOSSIER_NOW") {
        Ok(stamp) if valid(&stamp) => stamp,
        _ => Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string(),
    }
}

/// Whether `stamp` is a time stamp in the format of format.md §7.
pub fn valid(stamp: &str) -> bool {
    stamp.chars().count() == 25 && DateTime::parse_from_rfc3339(stamp).is_ok()
}

/// The date part of a time stamp: its first 10 characters.
pub fn date(stamp: &str) -> &str {
    let end = stamp
        .char_indices()
        .nth(10)
        .map_or(stamp.len(), |(index, _)| index);
    &stamp[..end]
}

/// Whole days from `earlier` to `later`; negative when `later` is before `earlier`.
pub fn days_between(earlier: &str, later: &str) -> i64 {
    match (
        DateTime::parse_from_rfc3339(earlier),
        DateTime::parse_from_rfc3339(later),
    ) {
        (Ok(earlier), Ok(later)) => later.signed_duration_since(earlier).num_days(),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_accepts_a_stamp_in_the_format_of_section_7() {
        assert!(valid("2026-11-20T14:03:27+01:00"));
        assert!(valid("2026-11-20T14:03:27-05:30"));
    }

    #[test]
    fn valid_refuses_a_date_without_time() {
        assert!(!valid("2026-11-20"));
    }

    #[test]
    fn valid_refuses_a_stamp_that_is_not_25_characters() {
        // Parses as RFC 3339, but has fractional seconds or a Z instead of an offset.
        assert!(!valid("2026-11-20T14:03:27.500+01:00"));
        assert!(!valid("2026-11-20T14:03:27Z"));
    }

    #[test]
    fn valid_refuses_text_that_is_not_a_time() {
        assert!(!valid(""));
        assert!(!valid("yesterday at noon, roughly"));
        assert!(!valid("2026-13-40T14:03:27+01:00"));
    }

    #[test]
    fn now_returns_dossier_now_when_it_is_set() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: ENV_LOCK is held, so no other test in this process reads or changes the
        // environment while this one does.
        unsafe { std::env::set_var("DOSSIER_NOW", "2026-11-20T14:03:27+01:00") };
        let fixed = now();
        unsafe { std::env::remove_var("DOSSIER_NOW") };
        assert_eq!(fixed, "2026-11-20T14:03:27+01:00");
    }

    #[test]
    fn now_uses_the_clock_when_dossier_now_is_invalid_or_missing() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: ENV_LOCK is held, so no other test in this process reads or changes the
        // environment while this one does.
        unsafe { std::env::set_var("DOSSIER_NOW", "tomorrow") };
        let invalid = now();
        unsafe { std::env::remove_var("DOSSIER_NOW") };
        let missing = now();
        for stamp in [invalid, missing] {
            assert!(valid(&stamp), "not a time stamp: {stamp}");
            assert_eq!(stamp.get(10..11), Some("T"), "no T separator: {stamp}");
        }
    }

    #[test]
    fn date_is_the_first_ten_characters() {
        assert_eq!(date("2026-11-20T14:03:27+01:00"), "2026-11-20");
        assert_eq!(date("2026-11-20"), "2026-11-20");
    }

    #[test]
    fn date_returns_a_shorter_string_whole() {
        assert_eq!(date("2026-11"), "2026-11");
        assert_eq!(date(""), "");
    }

    #[test]
    fn date_counts_characters_not_bytes() {
        // Each é is two bytes: slicing by bytes at 10 would split a character and panic.
        assert_eq!(date("ééééééééééééé"), "éééééééééé");
    }

    #[test]
    fn days_between_counts_whole_days() {
        assert_eq!(
            days_between("2026-11-20T14:03:27+01:00", "2026-12-04T14:03:27+01:00"),
            14
        );
        assert_eq!(
            days_between("2026-11-20T14:03:27+01:00", "2026-12-04T13:59:59+01:00"),
            13
        );
        assert_eq!(
            days_between("2026-11-20T14:03:27+01:00", "2026-11-21T13:00:00+01:00"),
            0
        );
    }

    #[test]
    fn days_between_is_negative_when_later_is_before_earlier() {
        assert_eq!(
            days_between("2026-12-04T14:03:27+01:00", "2026-11-20T14:03:27+01:00"),
            -14
        );
    }

    #[test]
    fn days_between_compares_instants_across_offsets() {
        // 2026-11-20T12:00 UTC and 2026-11-21T13:00+01:00 (12:00 UTC) are exactly one day apart.
        assert_eq!(
            days_between("2026-11-20T12:00:00+00:00", "2026-11-21T13:00:00+01:00"),
            1
        );
    }

    #[test]
    fn days_between_is_zero_when_a_stamp_does_not_parse() {
        assert_eq!(days_between("2026-11-20", "2026-12-04T14:03:27+01:00"), 0);
        assert_eq!(days_between("2026-11-20T14:03:27+01:00", "not a time"), 0);
    }
}
