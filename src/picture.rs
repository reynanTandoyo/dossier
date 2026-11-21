//! Pictures of an entry (format.md §16): the check of a picture's bytes and its file name and
//! line in the entry's Pictures section.

use std::path::Path;

/// The largest picture allowed, 2 MB (format.md §16).
pub const MAX_BYTES: u64 = 2 * 1024 * 1024;

/// The most pictures of one entry (format.md §16).
pub const MAX_PER_ENTRY: usize = 10;

/// "png", "jpg" or "svg" by content, recognised by the first bytes; None for anything else.
pub fn extension(_bytes: &[u8]) -> Option<&'static str> {
    None
}

/// The file name for a new picture in `folder` (format.md §16).
pub fn file_name(_folder: &Path, _original: &str, _ext: &str) -> String {
    String::new()
}

/// The line of a picture in the Pictures section: `![<caption>](../images/<ID>/<file>)`.
pub fn line(_id: &str, _file: &str, _caption: &str) -> String {
    String::new()
}
