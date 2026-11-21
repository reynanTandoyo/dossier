//! The `image` command (docs/design.md section 9, R-ENTRY-16, R-ENTRY-17): adds or removes a
//! picture of an entry. `image add` copies the picture into `DOSSIER/images/<ID>/` and adds its
//! line, with the caption, to the entry's Pictures section; `image remove` takes the line out and
//! deletes the file. The approved-lesson rule of `set` applies. Every change adds a history line
//! with who and when, and a journal line (D29).

use crate::approval;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::names::Who;
use crate::output::Output;
use crate::picture;
use crate::store;
use crate::tx::{self, Change};
use serde_json::json;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// The section that holds the pictures of an entry (format.md §16).
const PICTURES: &str = "Pictures";

/// The history event and the journal event of a picture change (format.md §14).
const EVENT: &str = "picture";

/// The detail added to the history line when an LLM's change takes an approved lesson back to
/// draft. The same words as `set` uses.
const BACK_TO_DRAFT: &str = "back to draft";

/// The arguments of `image`.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

/// What `image` does.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Adds a picture to an entry: it is copied into the dossier and shown with its caption
    Add {
        /// The ID of the entry, such as FND-004
        id: String,
        /// The picture file: PNG, JPEG or SVG, at most 2 MB
        file: PathBuf,
        /// What the picture shows and what to look at; one line
        #[arg(long)]
        caption: String,
    },
    /// Removes a picture from an entry and deletes its file
    Remove {
        /// The ID of the entry, such as FND-004
        id: String,
        /// The picture's file name as the Pictures section shows it, such as contour-plot.png
        file: PathBuf,
    },
}

/// Runs `image`: adds or removes one picture of an entry in one transaction.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "image")?;
    match &args.action {
        Action::Add { id, file, caption } => add(&ctx, id, file, caption),
        Action::Remove { id, file } => remove(&ctx, id, file),
    }
}

/// `image add`: copies the picture into `images/<ID>/` under a new file name and adds its line to
/// the Pictures section. The caption, the file and the number of pictures are checked before
/// anything is written; a refused picture writes nothing.
fn add(ctx: &Ctx, id: &str, file: &Path, caption: &str) -> Result<Output> {
    let who = ctx.writer()?.clone();
    let caption = caption_text(caption)?;
    let original = file_name_of(file)?;
    let (bytes, ext) = read_picture(file)?;

    tx::write(ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, id)?;
        let id = item.entry.id();
        let mut entry = item.entry.clone();
        let approved_lesson = is_approved_lesson(&entry);

        let body = entry.section(PICTURES).unwrap_or_default();
        let count = body.lines().filter(|line| !line.trim().is_empty()).count();
        if count >= picture::MAX_PER_ENTRY {
            return Err(bad(format!(
                "an entry has at most {} pictures; {id} has {count}",
                picture::MAX_PER_ENTRY
            )));
        }
        let folder = tx.ctx().dossier.join("images").join(&id);
        let name = picture::file_name(&folder, &original, ext);
        let line = picture::line(&id, &name, &caption);
        let next_body = if body.is_empty() {
            line
        } else {
            format!("{body}\n{line}")
        };
        entry.set_section(PICTURES, &next_body);

        let mut detail = format!("added {name}");
        let back = approved_lesson && who.is_llm;
        if back {
            back_to_draft(&mut entry, &mut detail);
        }
        entry.add_history(tx.time(), &who, EVENT, &detail);
        let text = entry.text();
        record_approval(tx.ctx(), &id, &who, approved_lesson, &text)?;

        tx.put_raw(
            folder.join(&name),
            None,
            bytes,
            Change {
                id: id.clone(),
                event: EVENT.to_string(),
                detail: format!("added {name}"),
            },
        );
        let status = status_name(&entry);
        tx.put(
            item.path.clone(),
            Some(&item),
            text,
            Change {
                id: id.clone(),
                event: EVENT.to_string(),
                detail,
            },
        );

        let mut message = format!("{id}: picture added: {name}");
        if back {
            message.push_str(" (back to draft, waiting for approval)");
        }
        Ok(Output::with(
            message,
            json!({
                "id": id,
                "file": name,
                "caption": caption,
                "status": status,
                "back_to_draft": back,
            }),
        ))
    })
}

/// `image remove`: takes the picture's line out of the Pictures section (an empty section is
/// removed) and deletes its file. A picture whose file is already gone loses only its line.
fn remove(ctx: &Ctx, id: &str, file: &Path) -> Result<Output> {
    let who = ctx.writer()?.clone();
    let name = file_name_of(file)?;

    tx::write(ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, id)?;
        let id = item.entry.id();
        let mut entry = item.entry.clone();
        let approved_lesson = is_approved_lesson(&entry);

        let body = entry.section(PICTURES).unwrap_or_default();
        let mut lines: Vec<String> = body.lines().map(str::to_string).collect();
        let Some(index) = lines.iter().position(|line| is_line_of(line, &id, &name)) else {
            return Err(Error::new(
                Code::NotFound,
                format!("{id} has no picture {name} in its Pictures section"),
            ));
        };
        lines.remove(index);
        entry.set_section(PICTURES, &lines.join("\n"));

        let path = tx.ctx().dossier.join("images").join(&id).join(&name);
        let old = read_if_present(&path)?;
        let mut detail = format!("removed {name}");
        let back = approved_lesson && who.is_llm;
        if back {
            back_to_draft(&mut entry, &mut detail);
        }
        entry.add_history(tx.time(), &who, EVENT, &detail);
        let text = entry.text();
        record_approval(tx.ctx(), &id, &who, approved_lesson, &text)?;

        let file_deleted = old.is_some();
        if let Some(old) = old {
            tx.delete_raw(
                path,
                old,
                Change {
                    id: id.clone(),
                    event: EVENT.to_string(),
                    detail: format!("removed {name}"),
                },
            );
        }
        let status = status_name(&entry);
        tx.put(
            item.path.clone(),
            Some(&item),
            text,
            Change {
                id: id.clone(),
                event: EVENT.to_string(),
                detail,
            },
        );

        let mut message = format!("{id}: picture removed: {name}");
        if back {
            message.push_str(" (back to draft, waiting for approval)");
        }
        if !file_deleted {
            message.push_str(" (its file was already gone)");
        }
        Ok(Output::with(
            message,
            json!({
                "id": id,
                "file": name,
                "status": status,
                "back_to_draft": back,
                "file_deleted": file_deleted,
            }),
        ))
    })
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}

/// The caption as given, trimmed. A blank caption and one with a line break are refused: the
/// picture line is one line, and a caption says what the picture shows (R-ENTRY-16).
fn caption_text(caption: &str) -> Result<String> {
    if caption.contains(['\r', '\n']) {
        return Err(bad("a caption is one line"));
    }
    let caption = caption.trim();
    if caption.is_empty() {
        return Err(bad(
            "a picture needs a caption: say what it shows and what to look at",
        ));
    }
    Ok(caption.to_string())
}

/// The file name of a path as text. Code::BadValue when the path has none, such as `..`.
fn file_name_of(file: &Path) -> Result<String> {
    file.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| bad(format!("not a file name: {}", file.display())))
}

/// The bytes of a picture file and its extension (format.md §16). Code::NotFound for a missing
/// file; Code::BadValue for a file that is not a file, is over 2 MB, or is not PNG, JPEG or SVG.
fn read_picture(file: &Path) -> Result<(Vec<u8>, &'static str)> {
    let metadata = match fs::metadata(file) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(Error::new(
                Code::NotFound,
                format!("no file at {}", file.display()),
            ));
        }
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_file() {
        return Err(bad(format!("not a file: {}", file.display())));
    }
    if metadata.len() > picture::MAX_BYTES {
        return Err(bad(format!(
            "a picture is at most {} bytes (2 MB); {} is {} bytes",
            picture::MAX_BYTES,
            file.display(),
            metadata.len()
        )));
    }
    let bytes = fs::read(file)?;
    let Some(ext) = picture::extension(&bytes) else {
        return Err(bad(format!(
            "a picture is PNG, JPEG or SVG: {}",
            file.display()
        )));
    };
    Ok((bytes, ext))
}

/// The bytes of a file, or None when there is no such file.
fn read_if_present(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

/// Whether the line is the picture `name` of entry `id`: a picture line (format.md §16) that ends
/// with the file, `](../images/<ID>/<file>)`.
fn is_line_of(line: &str, id: &str, name: &str) -> bool {
    line.starts_with("![") && line.ends_with(&format!("](../images/{id}/{name})"))
}

/// Whether the entry is an approved lesson, the kind `set` takes back to draft.
fn is_approved_lesson(entry: &Entry) -> bool {
    entry.kind() == Some(Kind::Lesson) && entry.status() == Some(Status::Approved)
}

/// Takes an approved lesson back to draft, as `set` does, and adds `, back to draft` to the
/// history detail.
fn back_to_draft(entry: &mut Entry, detail: &mut String) {
    entry.set_field("status", "draft");
    entry.remove_field("approved");
    entry.remove_field("valid-from");
    detail.push_str(", ");
    detail.push_str(BACK_TO_DRAFT);
}

/// The approval record of an approved lesson that changed, as `set` keeps it: an LLM's change
/// forgets the checksum (the lesson is a draft now), a person's change records the new text.
/// Nothing is recorded in a dry run.
fn record_approval(
    ctx: &Ctx,
    id: &str,
    who: &Who,
    approved_lesson: bool,
    text: &str,
) -> Result<()> {
    if !approved_lesson || ctx.dry_run {
        return Ok(());
    }
    let state = &ctx.state;
    if who.is_llm {
        approval::forget(state, id)
    } else {
        approval::record(state, id, text.as_bytes())
    }
}

/// The status word of an entry for the data, or the empty text when it has none.
fn status_name(entry: &Entry) -> &'static str {
    entry.status().map_or("", Status::name)
}
