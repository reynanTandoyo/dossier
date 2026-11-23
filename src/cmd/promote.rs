//! The `promote` command (docs/design.md section 12, R-TEAM-2, R-TEAM-3, R-ENTRY-18, R-INV-8):
//! copies a finding, a lesson, a how-to or a solved investigation of the project into the team
//! dossier as a draft.
//!
//! The order is fixed: the kind and the link are checked, then the confidential terms (the whole
//! text of the entry must be free of them), then a person confirms each picture, then the team
//! copy is written (a draft with `source` `<code>:<ID>`), then its pictures, and last the project
//! entry gets `promoted` `team:<ID>`. A finding becomes a lesson; its Observation becomes the
//! Statement. An investigation becomes the lesson or how-to that `--as` names, and only once it
//! is solved.

use crate::approval::{self, Mark};
use crate::cli::Global;
use crate::conf;
use crate::create::{self, New};
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, rules};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::picture;
use crate::store;
use crate::term;
use crate::tx::{self, Change};
use serde_json::json;
use std::fs;
use std::path::Path;

/// The section that holds the pictures of an entry (format.md §16).
const PICTURES: &str = "Pictures";

/// The state of an investigation that is solved (format.md §28).
const SOLVED: &str = "solved";

/// The fields that are copied to the team copy when the entry has them.
const COPIED_FIELDS: [&str; 5] = ["confidence", "machine", "analysis", "software", "tags"];

/// The arguments of `promote`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the finding, lesson, how-to or solved investigation to copy to the team
    /// dossier, such as LES-004
    pub id: String,
    /// The kind of the team copy: lesson or howto (default: the entry's own kind; a finding
    /// becomes a lesson; an investigation needs it)
    #[arg(long = "as", value_name = "KIND")]
    pub as_kind: Option<String>,
}

/// One picture line of the entry that a person has confirmed.
struct Confirmed {
    file: String,
    caption: String,
    bytes: Vec<u8>,
}

/// What a promotion did.
struct Promoted {
    id: String,
    team_id: String,
    pictures: usize,
    warnings: Vec<String>,
}

/// Runs `promote`: the output is `<ID>: promoted as team:<new ID> (draft, waiting for approval in
/// the team dossier)`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "promote")?;
    let as_kind = match args.as_kind.as_deref() {
        None => None,
        Some(text) => Some(Kind::parse(text).ok_or_else(|| {
            Error::new(
                Code::BadValue,
                format!("--as must be lesson or howto, not {text}"),
            )
        })?),
    };
    let done = promote(&ctx, &args.id, as_kind)?;
    let message = format!(
        "{}: promoted as team:{} (draft, waiting for approval in the team dossier)",
        done.id, done.team_id
    );
    let output = Output::with(
        message,
        json!({
            "id": done.id,
            "team_id": done.team_id,
            "status": "draft",
            "pictures": done.pictures,
        }),
    );
    Ok(done
        .warnings
        .into_iter()
        .fold(output, |output, warning| output.warn(warning)))
}

/// Promotes one entry as `run` does, for the `close` command, and returns the new team ID.
pub fn promote_one(ctx: &Ctx, id: &str, as_kind: Option<Kind>) -> Result<String> {
    promote(ctx, id, as_kind).map(|done| done.team_id)
}

/// The whole promotion, in the order given in the module text.
fn promote(ctx: &Ctx, id: &str, as_kind: Option<Kind>) -> Result<Promoted> {
    let item = store::get(&ctx.dossier, id)?;
    let id = item.entry.id();
    let kind = item
        .entry
        .kind()
        .or_else(|| Kind::of_id(&id))
        .ok_or_else(|| Error::new(Code::Refused, format!("{id} has no valid kind")))?;
    let target = target_kind(&id, kind, as_kind)?;
    if let Some(promoted) = item.entry.field("promoted") {
        return Err(Error::new(
            Code::Refused,
            format!("{id} is already promoted as {promoted}"),
        ));
    }
    // Checked before the team is read: an investigation that is not solved has nothing to copy.
    if kind == Kind::Investigation && item.entry.field("state").as_deref() != Some(SOLVED) {
        return Err(Error::new(
            Code::Refused,
            format!("{id} is not solved yet; close it with dossier solved first"),
        ));
    }
    let who = ctx.writer()?.clone();
    let team = ctx.team_ctx()?;

    // R-TEAM-3: the check comes before anything is written.
    let terms = conf::gate(&ctx.dossier, &ctx.cfg.code)?;
    conf::check(&terms, &item.entry.text())?;

    let confirmed = confirm_pictures(ctx, &id, &item.entry, who.is_llm)?;
    let mut warnings = Vec::new();

    let new = New {
        kind: target,
        title: item.entry.title(),
        fields: copied_fields(&ctx.cfg.code, &id, &item.entry),
        sections: copied_sections(kind, target, &item.entry),
        said_by: None,
        draft: true,
    };
    let mut made = String::new();
    let output = tx::write(&team, |tx| {
        let (team_id, _) = create::create(tx, &new)?;
        made = team_id.clone();
        Ok(Output::text(team_id))
    })?;
    warnings.extend(output.warnings);
    let team_id = made;
    if ctx.dry_run {
        return Ok(Promoted {
            id,
            team_id,
            pictures: confirmed.len(),
            warnings,
        });
    }

    if !confirmed.is_empty() {
        let output = add_pictures(&team, &team_id, &confirmed)?;
        warnings.extend(output.warnings);
    }
    link_back(ctx, &id, &team_id)?;
    Ok(Promoted {
        id,
        team_id,
        pictures: confirmed.len(),
        warnings,
    })
}

/// The kind of the team copy. Only findings, lessons, how-tos and investigations are promoted; a
/// how-to stays a how-to, a finding or lesson becomes a lesson, and an investigation becomes the
/// lesson or how-to that `--as` names (R-INV-8).
fn target_kind(id: &str, kind: Kind, as_kind: Option<Kind>) -> Result<Kind> {
    if kind == Kind::Investigation {
        return match as_kind {
            None => Err(Error::new(
                Code::Usage,
                "an investigation is promoted as a how-to or a lesson: add --as howto or --as lesson",
            )),
            Some(wanted @ (Kind::Lesson | Kind::Howto)) => Ok(wanted),
            Some(_) => Err(Error::new(Code::BadValue, "--as must be lesson or howto")),
        };
    }
    if !matches!(kind, Kind::Finding | Kind::Lesson | Kind::Howto) {
        return Err(Error::new(
            Code::Refused,
            "only findings, lessons and how-tos are promoted",
        ));
    }
    let own = if kind == Kind::Howto {
        Kind::Howto
    } else {
        Kind::Lesson
    };
    match as_kind {
        None => Ok(own),
        Some(wanted) if wanted == own => Ok(own),
        Some(Kind::Lesson | Kind::Howto) => Err(Error::new(
            Code::Refused,
            format!(
                "{id} is a {} and cannot be promoted as a {}",
                kind.name(),
                if own == Kind::Howto {
                    "lesson"
                } else {
                    "howto"
                }
            ),
        )),
        Some(_) => Err(Error::new(
            Code::BadValue,
            "--as must be lesson or howto".to_string(),
        )),
    }
}

/// The fields of the team copy: `source`, then the tag fields (and confidence) the entry has.
fn copied_fields(code: &str, id: &str, entry: &Entry) -> Vec<(String, String)> {
    let mut fields = vec![("source".to_string(), format!("{code}:{id}"))];
    for key in COPIED_FIELDS {
        if let Some(value) = entry.field(key).filter(|value| !value.trim().is_empty()) {
            fields.push((key.to_string(), value));
        }
    }
    fields
}

/// The sections of the team copy, in the order of its kind, without the Pictures section. A
/// finding's Observation becomes the Statement. An investigation is copied like a finding for
/// now: its own sections are not mapped yet.
fn copied_sections(kind: Kind, target: Kind, entry: &Entry) -> Vec<(String, String)> {
    rules::section_order(target)
        .iter()
        .filter_map(|name| {
            let from =
                if matches!(kind, Kind::Finding | Kind::Investigation) && *name == "Statement" {
                    "Observation"
                } else {
                    name
                };
            entry
                .section(from)
                .filter(|body| !body.trim().is_empty())
                .map(|body| (name.to_string(), body))
        })
        .collect()
}

/// The pictures of the entry that a person confirmed (R-ENTRY-18). An LLM or no terminal is
/// Code::Refused; a picture answered with anything but yes is left out; a picture whose file is
/// missing or not a picture is not asked about.
fn confirm_pictures(ctx: &Ctx, id: &str, entry: &Entry, is_llm: bool) -> Result<Vec<Confirmed>> {
    let lines = picture_lines(entry);
    if lines.is_empty() {
        return Ok(Vec::new());
    }
    if is_llm || !term::is_terminal() {
        return Err(Error::new(
            Code::Refused,
            format!(
                "{id} has pictures, which cannot be checked for confidential terms; a person at a terminal must confirm each one"
            ),
        ));
    }
    let mut confirmed = Vec::new();
    for (caption, file) in lines {
        let path = ctx.dossier.join("images").join(id).join(&file);
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        if picture::extension(&bytes).is_none() {
            continue;
        }
        let answer = term::ask(&format!(
            "copy picture {file} ({caption}) to the team? (yes/no) "
        ))?;
        if matches!(answer.to_lowercase().as_str(), "yes" | "y") {
            confirmed.push(Confirmed {
                file,
                caption,
                bytes,
            });
        }
    }
    Ok(confirmed)
}

/// The (caption, file name) of each line of the Pictures section: `![caption](../images/ID/file)`.
fn picture_lines(entry: &Entry) -> Vec<(String, String)> {
    let body = entry.section(PICTURES).unwrap_or_default();
    body.lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("![")?;
            let (caption, target) = rest.split_once("](")?;
            let target = target.strip_suffix(')')?;
            let file = target.rsplit('/').next()?;
            (!file.is_empty()).then(|| (caption.to_string(), file.to_string()))
        })
        .collect()
}

/// Copies the confirmed pictures into the team's `images/<team ID>/` and adds their lines to the
/// Pictures section of the team copy, with a history line each.
fn add_pictures(team: &Ctx, team_id: &str, pictures: &[Confirmed]) -> Result<Output> {
    let who = team.writer()?.clone();
    tx::write(team, |tx| {
        let item = store::get(&tx.ctx().dossier, team_id)?;
        let mut entry = item.entry.clone();
        let folder = tx.ctx().dossier.join("images").join(team_id);
        let mut used: Vec<String> = Vec::new();
        let mut lines = Vec::new();
        for pic in pictures {
            let ext = picture::extension(&pic.bytes).unwrap_or("png");
            let name = unique_name(&folder, &pic.file, ext, &used);
            used.push(name.clone());
            lines.push(picture::line(team_id, &name, &pic.caption));
            entry.add_history(tx.time(), &who, "picture", &format!("added {name}"));
            tx.put_raw(
                folder.join(&name),
                None,
                pic.bytes.clone(),
                Change {
                    id: team_id.to_string(),
                    event: "picture".to_string(),
                    detail: format!("added {name}"),
                },
            );
        }
        entry.set_section(PICTURES, &lines.join("\n"));
        let text = entry.text();
        tx.put(
            item.path.clone(),
            Some(&item),
            text,
            Change {
                id: team_id.to_string(),
                event: "picture".to_string(),
                detail: format!("added {} picture(s)", pictures.len()),
            },
        );
        Ok(Output::text(format!("{team_id}: pictures added")))
    })
}

/// A picture file name that is neither on disk in `folder` nor in `used`.
fn unique_name(folder: &Path, original: &str, ext: &str, used: &[String]) -> String {
    let first = picture::file_name(folder, original, ext);
    let stem = first
        .strip_suffix(&format!(".{ext}"))
        .unwrap_or(&first)
        .to_string();
    let mut name = first;
    let mut number = 1;
    while used.contains(&name) {
        number += 1;
        name = format!("{stem}-{number}.{ext}");
    }
    name
}

/// Sets `promoted` on the project entry, with a history line and a journal line. An approved
/// lesson or how-to whose approval still held keeps it: the link is not a change of content.
fn link_back(ctx: &Ctx, id: &str, team_id: &str) -> Result<()> {
    let who = ctx.writer()?.clone();
    let value = format!("team:{team_id}");
    tx::write(ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, id)?;
        let kind = item.entry.kind().or_else(|| Kind::of_id(id));
        let held = kind.is_some_and(approval::guarded)
            && matches!(
                approval::mark(&tx.ctx().state, id, &item.raw),
                Mark::Unchanged
            );
        let mut entry = item.entry.clone();
        entry.set_field("promoted", &value);
        entry.add_history(tx.time(), &who, "promoted", &value);
        let text = entry.text();
        if held && !tx.ctx().dry_run {
            approval::record(&tx.ctx().state, id, text.as_bytes())?;
        }
        tx.put(
            item.path.clone(),
            Some(&item),
            text,
            Change {
                id: id.to_string(),
                event: "promoted".to_string(),
                detail: value.clone(),
            },
        );
        Ok(Output::text(format!("{id}: promoted")))
    })?;
    Ok(())
}
