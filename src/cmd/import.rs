//! The `import` command (docs/design.md section 12, R-IMPORT-7): `import status` shows how far a
//! finished project has been brought in. It counts the documents found in the folder the settings
//! describe (else the project folder) and their review state in imports.md (format.md §25), and
//! the entries drafted from those documents: how many wait for approval, how many are approved and
//! how many are promoted to the team dossier. Nothing is changed. The imports belong to a project,
//! so `--team` is refused.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Status};
use crate::error::{Code, Error, Result};
use crate::folderscan;
use crate::imports;
use crate::output::Output;
use crate::paths;
use crate::store::{self, Item};
use serde_json::json;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

/// What `--team` gets: the imports are the project's, not the team's.
const TEAM_REFUSED: &str = "import belongs to a project; run without --team";

/// The arguments of `import`.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

/// What `import` does.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Shows how far the finished project has been brought in
    Status,
}

/// Runs `import`. `--team` is refused before anything is loaded, for every subcommand.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    if g.team {
        return Err(Error::new(Code::Usage, TEAM_REFUSED));
    }
    let ctx = Ctx::load(g, "import")?;
    match &args.action {
        Action::Status => status(&ctx),
    }
}

/// The counts of `import status`. The review counts cover the documents found; a document found
/// with no state in imports.md (or with a state other than the three) is `not_reviewed`, so the
/// four document counts add up to `documents`.
struct Counts {
    documents: usize,
    not_reviewed: usize,
    reviewing: usize,
    done: usize,
    nothing: usize,
    drafts: usize,
    approved: usize,
    promoted: usize,
}

/// Counts the documents of the base folder and the imported entries, then prints the counts with
/// the warnings of reading the dossier.
fn status(ctx: &Ctx) -> Result<Output> {
    let documents = folderscan::documents(&base_folder(ctx))?;
    let (imports, _) = imports::load(&ctx.dossier, &ctx.cfg.code)?;
    let states: Vec<Option<String>> = documents
        .iter()
        .map(|doc| imports::state(&imports, &doc.stored))
        .collect();
    let listed = |wanted: &str| {
        states
            .iter()
            .filter(|state| state.as_deref() == Some(wanted))
            .count()
    };
    let reviewing = listed("reviewing");
    let done = listed("done");
    let nothing = listed("nothing");

    let (items, warnings) = store::all(&ctx.dossier);
    let imported: Vec<&Item> = items.iter().filter(|item| imported(&item.entry)).collect();
    let with_status = |wanted: Status| {
        imported
            .iter()
            .filter(|item| item.entry.status() == Some(wanted))
            .count()
    };
    let counts = Counts {
        documents: documents.len(),
        not_reviewed: documents.len() - reviewing - done - nothing,
        reviewing,
        done,
        nothing,
        drafts: with_status(Status::Draft),
        approved: with_status(Status::Approved),
        promoted: imported
            .iter()
            .filter(|item| {
                item.entry
                    .field("promoted")
                    .is_some_and(|value| !value.trim().is_empty())
            })
            .count(),
    };

    let folder = match &ctx.cfg.describes {
        Some(described) => paths::shown(described),
        None => ".".to_string(),
    };
    let lines = [
        format!("Import status of {} (folder {folder})", ctx.cfg.code),
        format!("Documents found: {}", counts.documents),
        format!("  not reviewed: {}", counts.not_reviewed),
        format!("  being reviewed: {}", counts.reviewing),
        format!("  done: {}", counts.done),
        format!("  nothing to take: {}", counts.nothing),
        "Imported entries:".to_string(),
        format!("  drafts waiting for approval: {}", counts.drafts),
        format!("  approved: {}", counts.approved),
        format!("  promoted to the team: {}", counts.promoted),
    ];
    let data = json!({
        "folder": folder,
        "documents": counts.documents,
        "not_reviewed": counts.not_reviewed,
        "reviewing": counts.reviewing,
        "done": counts.done,
        "nothing": counts.nothing,
        "imported_drafts": counts.drafts,
        "imported_approved": counts.approved,
        "imported_promoted": counts.promoted,
    });
    let mut output = Output::with(lines.join("\n"), data);
    for warning in &warnings {
        output = output.warn(shown_warning(&ctx.root, warning));
    }
    Ok(output)
}

/// The folder the settings describe (`describes`, written by `init --in`), else the project
/// folder. The same folder `review-folder` lists.
fn base_folder(ctx: &Ctx) -> PathBuf {
    match &ctx.cfg.describes {
        Some(described) => ctx.root.join(paths::shown(described)),
        None => ctx.root.clone(),
    }
}

/// Whether the entry was drafted from a document: `imported: yes` (format.md §23, R-IMPORT-5).
fn imported(entry: &Entry) -> bool {
    entry
        .field("imported")
        .is_some_and(|value| value.eq_ignore_ascii_case("yes"))
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
