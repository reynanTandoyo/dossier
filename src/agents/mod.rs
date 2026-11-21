//! The rules block that `dossier agents install` keeps in AGENTS.md.

/// The rules text (agents/rules.md).
pub const RULES: &str = include_str!("rules.md");

/// The line that starts the block.
pub const BEGIN: &str = "<!-- dossier:begin -->";

/// The line that ends the block.
pub const END: &str = "<!-- dossier:end -->";

/// The line that ends todo's block in AGENTS.md.
const TODO_END: &str = "<!-- todo:end -->";

/// The rules text with LF line endings and without its trailing line breaks.
fn rules_lf() -> String {
    RULES
        .replace("\r\n", "\n")
        .trim_end_matches(['\r', '\n'])
        .to_string()
}

/// The block without its final line break: BEGIN, the rules, END.
fn block_body() -> String {
    format!("{BEGIN}\n{}\n{END}", rules_lf())
}

/// The block: BEGIN, the rules, END, each line ending with LF.
pub fn block() -> String {
    format!("{}\n", block_body())
}

/// The text of AGENTS.md with the block added (after todo's block if there is one, else at the
/// end) or replaced; every other line unchanged.
pub fn installed(existing: &str) -> String {
    if let Some((start, end)) = managed_span(existing) {
        // Replace BEGIN through END in place; the line break after END stays as it is.
        let style = style_before(existing, start);
        let body = block_body();
        let mut out = String::with_capacity(existing.len() + body.len());
        out.push_str(&existing[..start]);
        out.push_str(&with_style(&body, style));
        out.push_str(&existing[end..]);
        return out;
    }
    if let Some(after) = after_todo_end(existing) {
        // A blank line, then the block, right after todo's END line. A last line without a line
        // break gets one first.
        let style = style_before(existing, after);
        let mut out = String::with_capacity(existing.len() + block().len() + 4);
        out.push_str(&existing[..after]);
        if !existing[..after].ends_with('\n') {
            out.push_str(style);
        }
        out.push_str(style);
        out.push_str(&with_style(&block(), style));
        out.push_str(&existing[after..]);
        return out;
    }
    if existing.is_empty() {
        return block();
    }
    // Append: a blank line first, unless the text already ends with one.
    let style = style_before(existing, existing.len());
    let mut out = existing.to_string();
    if !ends_with_blank_line(existing) {
        if !existing.ends_with('\n') {
            out.push_str(style);
        }
        out.push_str(style);
    }
    out.push_str(&with_style(&block(), style));
    out
}

/// Whether the block in `existing` is current.
pub fn current(existing: &str) -> bool {
    existing.replace("\r\n", "\n").contains(&block_body())
}

/// The Claude Code skill: a front matter (name: dossier, description: one line) and RULES.
pub fn skill() -> String {
    String::new()
}

/// The byte range of the managed block in `text`: from its BEGIN to the end of its END. An END is
/// paired with the nearest BEGIN before it, so a stray marker is not taken for the block's start.
fn managed_span(text: &str) -> Option<(usize, usize)> {
    let mut search = 0;
    while let Some(found) = text[search..].find(END) {
        let end = search + found;
        if let Some(start) = text[..end].rfind(BEGIN) {
            return Some((start, end + END.len()));
        }
        search = end + END.len();
    }
    None
}

/// The byte index just after the first line that is todo's END line, its line break included.
fn after_todo_end(text: &str) -> Option<usize> {
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        offset += line.len();
        if line.trim() == TODO_END {
            return Some(offset);
        }
    }
    None
}

/// The line break that ends the nearest line before byte `pos`: CRLF, or LF when there is none.
fn style_before(text: &str, pos: usize) -> &'static str {
    let before = text.get(..pos).unwrap_or_default();
    match before.rsplit_once('\n') {
        Some((head, _)) if head.ends_with('\r') => "\r\n",
        _ => "\n",
    }
}

/// `text` with every LF turned into `style`.
fn with_style(text: &str, style: &str) -> String {
    text.replace('\n', style)
}

/// Whether `text` ends with a line break that follows a blank line.
fn ends_with_blank_line(text: &str) -> bool {
    match text.strip_suffix('\n') {
        Some(before) => before.rsplit('\n').next().unwrap_or("").trim().is_empty(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{BEGIN, END, block, current, installed};

    /// The text with every LF turned into CRLF.
    fn crlf(text: &str) -> String {
        text.replace('\n', "\r\n")
    }

    #[test]
    fn block_starts_with_begin_and_ends_with_end_and_a_line_break() {
        let block = block();
        assert!(block.starts_with(&format!("{BEGIN}\n")));
        assert!(block.ends_with(&format!("\n{END}\n")));
    }

    #[test]
    fn installs_into_an_empty_text() {
        assert_eq!(installed(""), block());
        assert!(current(&installed("")));
    }

    #[test]
    fn installs_after_todos_block() {
        let before = "# Project\n\n<!-- todo:begin -->\n# Work\n<!-- todo:end -->\n\nNotes\n";
        let expected = format!(
            "# Project\n\n<!-- todo:begin -->\n# Work\n<!-- todo:end -->\n\n{}\nNotes\n",
            block()
        );
        assert_eq!(installed(before), expected);
    }

    #[test]
    fn installing_twice_gives_the_same_text() {
        let texts = [
            "",
            "# Project\n",
            "# Project\n\n<!-- todo:begin -->\n<!-- todo:end -->\n",
        ];
        for before in texts {
            let once = installed(before);
            assert_eq!(installed(&once), once);
        }
    }

    #[test]
    fn current_is_false_before_and_true_after_installing() {
        let before = "# Project\n\n<!-- todo:begin -->\n<!-- todo:end -->\n";
        assert!(!current(before));
        assert!(current(&installed(before)));
    }

    #[test]
    fn replaces_an_older_block_in_place_and_keeps_the_other_lines() {
        let old = format!("# Project\n\n{BEGIN}\nold rules\n{END}\n\nNotes\n");
        assert!(!current(&old));
        let new = installed(&old);
        assert_eq!(new, format!("# Project\n\n{}\nNotes\n", block()));
        assert!(current(&new));
    }

    #[test]
    fn appends_after_a_text_without_a_todo_block() {
        assert_eq!(
            installed("# Project\n\nNotes\n"),
            format!("# Project\n\nNotes\n\n{}", block())
        );
    }

    #[test]
    fn appends_without_a_second_blank_line_after_a_blank_line() {
        assert_eq!(
            installed("# Project\n\n"),
            format!("# Project\n\n{}", block())
        );
    }

    #[test]
    fn appends_a_line_break_and_a_blank_line_to_a_text_without_a_final_line_break() {
        assert_eq!(installed("# Project"), format!("# Project\n\n{}", block()));
    }

    #[test]
    fn crlf_text_keeps_its_other_lines_unchanged() {
        let before = crlf("# Project\n\n<!-- todo:begin -->\n# Work\n<!-- todo:end -->\n\nNotes\n");
        let expected = format!(
            "{}{}{}",
            crlf("# Project\n\n<!-- todo:begin -->\n# Work\n<!-- todo:end -->\n\n"),
            crlf(&block()),
            crlf("\nNotes\n")
        );
        let after = installed(&before);
        assert_eq!(after, expected);
        assert!(current(&after));
        assert_eq!(installed(&after), after);
    }

    #[test]
    fn crlf_text_without_todo_block_gets_crlf_lines_when_appended() {
        assert_eq!(
            installed("# Project\r\n"),
            format!("# Project\r\n\r\n{}", crlf(&block()))
        );
    }

    #[test]
    fn a_stray_begin_marker_does_not_take_other_lines_with_it() {
        let before = format!("{BEGIN}\nkeep me\n");
        let once = installed(&before);
        let twice = installed(&once);
        assert_eq!(twice, once);
        assert!(twice.contains("keep me"));
    }
}
