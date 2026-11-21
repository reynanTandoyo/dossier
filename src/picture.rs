//! Pictures of an entry (format.md §16): the check of a picture's bytes and its file name and
//! line in the entry's Pictures section.

use crate::paths;
use std::fs;
use std::path::Path;

/// The largest picture allowed, 2 MB (format.md §16).
pub const MAX_BYTES: u64 = 2 * 1024 * 1024;

/// The most pictures of one entry (format.md §16).
pub const MAX_PER_ENTRY: usize = 10;

/// The first bytes of a PNG file.
const PNG_START: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// The first bytes of a JPEG file.
const JPEG_START: [u8; 3] = [0xFF, 0xD8, 0xFF];

/// "png", "jpg" or "svg" by content, recognised by the first bytes; None for anything else.
pub fn extension(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&PNG_START) {
        Some("png")
    } else if bytes.starts_with(&JPEG_START) {
        Some("jpg")
    } else if is_svg(bytes) {
        Some("svg")
    } else {
        None
    }
}

/// An SVG is UTF-8 text whose first part starts with `<?xml` or `<svg` and which contains
/// `<svg`. Blanks and a byte order mark before the first part are skipped.
fn is_svg(bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    let first = text.trim_start_matches(|c: char| c == '\u{feff}' || c.is_whitespace());
    (first.starts_with("<?xml") || first.starts_with("<svg")) && text.contains("<svg")
}

/// The file name for a new picture in `folder` (format.md §16): the slug of `original` without
/// its extension, then `.<ext>`. While a file of that name exists in `folder`, `-2`, `-3`, …
/// come before the extension.
pub fn file_name(folder: &Path, original: &str, ext: &str) -> String {
    let stem = Path::new(original)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(original);
    let base = paths::slug(stem);
    let mut number = 1;
    let mut name = format!("{base}.{ext}");
    while fs::symlink_metadata(folder.join(&name)).is_ok() {
        number += 1;
        name = format!("{base}-{number}.{ext}");
    }
    name
}

/// The line of a picture in the Pictures section: `![<caption>](../images/<ID>/<file>)`. The
/// caption must be one line; the caller checks that.
pub fn line(id: &str, file: &str, caption: &str) -> String {
    format!("![{caption}](../images/{id}/{file})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_is_recognised_by_its_signature() {
        let mut bytes = PNG_START.to_vec();
        bytes.extend_from_slice(b"\0\0\0\rIHDR and the rest");
        assert_eq!(extension(&bytes), Some("png"));
        assert_eq!(extension(&PNG_START[..7]), None);
    }

    #[test]
    fn jpeg_is_recognised_by_its_signature() {
        let bytes = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F'];
        assert_eq!(extension(&bytes), Some("jpg"));
        assert_eq!(extension(&[0xFF, 0xD8, 0xFE]), None);
    }

    #[test]
    fn svg_is_utf8_text_that_starts_with_xml_or_svg_and_contains_svg() {
        let with_xml = b"<?xml version=\"1.0\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\"/>";
        assert_eq!(extension(with_xml), Some("svg"));
        let bare = b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";
        assert_eq!(extension(bare), Some("svg"));
        let indented = b"  \r\n\t<svg></svg>";
        assert_eq!(extension(indented), Some("svg"));
        let with_bom = "\u{feff}<?xml version=\"1.0\"?><svg></svg>";
        assert_eq!(extension(with_bom.as_bytes()), Some("svg"));
    }

    #[test]
    fn text_and_other_markup_are_not_pictures() {
        assert_eq!(extension(b"This is a note.\nSecond line."), None);
        assert_eq!(extension(b"%PDF-1.7 not a picture"), None);
        assert_eq!(extension(b""), None);
        assert_eq!(extension(b"<?xml version=\"1.0\"?><html></html>"), None);
        assert_eq!(extension(b"<html><svg></svg></html>"), None);
        assert_eq!(extension(b"Notes: <svg> is a tag"), None);
        assert_eq!(extension(b"<svg>\xFF</svg>"), None);
    }

    #[test]
    fn file_name_is_the_slug_of_the_name_with_the_extension() {
        let folder = tempfile::tempdir().unwrap();
        assert_eq!(
            file_name(folder.path(), "Contour Plot.PNG", "png"),
            "contour-plot.png"
        );
        assert_eq!(
            file_name(folder.path(), "rod (final) v2.final.jpeg", "jpg"),
            "rod-final-v2-final.jpg"
        );
        assert_eq!(file_name(folder.path(), "Größe", "svg"), "gr-e.svg");
        assert_eq!(file_name(folder.path(), "!!!", "png"), "entry.png");
    }

    #[test]
    fn file_name_gets_a_number_when_the_name_exists() {
        let folder = tempfile::tempdir().unwrap();
        fs::write(folder.path().join("plot.png"), b"first").unwrap();
        assert_eq!(file_name(folder.path(), "plot.png", "png"), "plot-2.png");
        fs::write(folder.path().join("plot-2.png"), b"second").unwrap();
        assert_eq!(file_name(folder.path(), "plot.png", "png"), "plot-3.png");
        // The same name with another extension is another file.
        assert_eq!(file_name(folder.path(), "plot.svg", "svg"), "plot.svg");
    }

    #[test]
    fn file_name_works_in_a_folder_with_spaces_and_non_ascii_characters() {
        let root = tempfile::tempdir().unwrap();
        let folder = root.path().join("Ordner für Bilder");
        fs::create_dir(&folder).unwrap();
        fs::write(folder.join("plot.png"), b"first").unwrap();
        assert_eq!(file_name(&folder, "plot.png", "png"), "plot-2.png");
    }

    #[test]
    fn line_is_a_markdown_image_in_the_entry_folder() {
        assert_eq!(
            line(
                "FND-004",
                "contour-plot.png",
                "Contour of the rod; look at the hot spot"
            ),
            "![Contour of the rod; look at the hot spot](../images/FND-004/contour-plot.png)"
        );
    }
}
