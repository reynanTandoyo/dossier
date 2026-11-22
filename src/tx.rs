//! Transactions: the one way an entry file is written. Takes the lock, saves the snapshot,
//! replaces the file, and appends the journal line.

use crate::atomic::{self, Seen};
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::gitcommit;
use crate::hash;
use crate::journal;
use crate::lock;
use crate::names::Who;
use crate::output::Output;
use crate::snapshot;
use crate::store::Item;
use crate::time;
use std::path::{Path, PathBuf};

/// A transaction: the changes queued by a command, written by `write` once the command has
/// run.
pub struct Tx<'a> {
    ctx: &'a Ctx,
    time: String,
    queued: Vec<Queued>,
}

/// What one change says about itself: the entry, the event and a detail for the journal.
pub struct Change {
    pub id: String,
    pub event: String,
    pub detail: String,
}

/// A queued change, kept until `write` applies it.
struct Queued {
    action: Action,
    /// The bytes of the file as read; None for a new file.
    old: Option<Vec<u8>>,
    change: Change,
}

/// What a queued change does to the files.
enum Action {
    Put {
        path: PathBuf,
        text: String,
    },
    Delete {
        path: PathBuf,
    },
    Rename {
        from: PathBuf,
        to: PathBuf,
        text: String,
    },
    /// A file that is not an entry: its new bytes.
    PutRaw {
        path: PathBuf,
        bytes: Vec<u8>,
    },
    /// A file that is not an entry.
    DeleteRaw {
        path: PathBuf,
    },
}

/// The message of the refusal for a dossier on a network drive (R-LOCK-3).
const NETWORK_DRIVE: &str =
    "the dossier is on a network drive; run the command on the server, see dossier connect";

/// The program argument that lets one command write to a network drive anyway (R-LOCK-3).
const ALLOW_NETWORK: &str = "--allow-network-drive";

impl<'a> Tx<'a> {
    /// The context of the command.
    pub fn ctx(&self) -> &Ctx {
        self.ctx
    }

    /// The time of the whole command.
    pub fn time(&self) -> &str {
        &self.time
    }

    /// Queues writing `text` to `path`. `old` is the item as read (None for a new file).
    pub fn put(&mut self, path: PathBuf, old: Option<&Item>, text: String, change: Change) {
        self.queued.push(Queued {
            action: Action::Put { path, text },
            old: old.map(|item| item.raw.clone()),
            change,
        });
    }

    /// Queues deleting the file of `old`.
    pub fn delete(&mut self, old: &Item, change: Change) {
        self.queued.push(Queued {
            action: Action::Delete {
                path: old.path.clone(),
            },
            old: Some(old.raw.clone()),
            change,
        });
    }

    /// Queues moving `old` to `new_path`, with `text` as its new content.
    pub fn rename(&mut self, old: &Item, new_path: PathBuf, text: String, change: Change) {
        self.queued.push(Queued {
            action: Action::Rename {
                from: old.path.clone(),
                to: new_path,
                text,
            },
            old: Some(old.raw.clone()),
            change,
        });
    }

    /// Queue writing `bytes` to `path` (any file inside the dossier). `old` is the file as read
    /// (None for a new file). The old bytes are snapshotted under change.id when the path ends
    /// in `.md`.
    pub fn put_raw(&mut self, path: PathBuf, old: Option<Vec<u8>>, bytes: Vec<u8>, change: Change) {
        self.queued.push(Queued {
            action: Action::PutRaw { path, bytes },
            old,
            change,
        });
    }

    /// Queue removing a file that is still `old`.
    pub fn delete_raw(&mut self, path: PathBuf, old: Vec<u8>, change: Change) {
        self.queued.push(Queued {
            action: Action::DeleteRaw { path },
            old: Some(old),
            change,
        });
    }
}

/// Refuses a network drive, takes the lock, runs `f`, then for each queued change: saves a
/// snapshot of the old file, replaces the file with atomic::replace (expecting the old bytes),
/// appends the journal record. With ctx.dry_run nothing is written and the Output gets one
/// warning per change: `dry run: would write <path>`. An error in `f` writes nothing.
pub fn write<F>(ctx: &Ctx, f: F) -> Result<Output>
where
    F: FnOnce(&mut Tx) -> Result<Output>,
{
    let who = ctx.writer()?;
    if lock::is_network(&ctx.dossier) && !allowed_on_network() {
        return Err(Error::new(Code::NetworkDrive, NETWORK_DRIVE));
    }
    // Held until the last change is written. A dry run writes nothing, so it takes no lock.
    let _guard = if ctx.dry_run {
        None
    } else {
        Some(lock::acquire(
            &ctx.state,
            ctx.cfg.lock_wait_seconds,
            &who.name,
            &ctx.command,
        )?)
    };

    let mut tx = Tx {
        ctx,
        time: time::now(),
        queued: Vec::new(),
    };
    let output = f(&mut tx)?;
    if ctx.dry_run {
        return Ok(tx.queued.iter().fold(output, |output, queued| {
            output.warn(dry_run_warning(&ctx.root, queued))
        }));
    }
    apply(ctx, who, &tx.time, &tx.queued)?;
    Ok(commit_team(ctx, &tx.queued, output))
}

/// Commits the changes of a team dossier to git (D36, R-TEAM-6): one commit for the command,
/// its subject the command and the change IDs. Only a team dossier, and only when a change was
/// written. A failure is a warning on `output` (the change is written already), never an error.
fn commit_team(ctx: &Ctx, queued: &[Queued], output: Output) -> Output {
    if ctx.cfg.kind != "team" || queued.is_empty() {
        return output;
    }
    let message = format!("{}: {}", ctx.command, change_ids(queued).join(", "));
    match gitcommit::commit(&ctx.root, &ctx.dossier, &message) {
        Ok(_) => output,
        Err(error) => output.warn(format!("git: {}", error.message)),
    }
}

/// The IDs of the queued changes, each once, in the order they were queued.
fn change_ids(queued: &[Queued]) -> Vec<&str> {
    let mut ids: Vec<&str> = Vec::new();
    for op in queued {
        let id = op.change.id.as_str();
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

/// Whether the program was started with `--allow-network-drive`. The arguments are read as they
/// are, so that one that is not UTF-8 does not stop the command.
fn allowed_on_network() -> bool {
    std::env::args_os().any(|arg| arg == ALLOW_NETWORK)
}

/// Writes the queued changes in order. Every change is first checked against the file as it was
/// read, so that a file changed by someone else stops the command before anything is written.
/// Then, for each change: the old bytes are saved as a snapshot, the file is replaced or removed,
/// and the journal line is appended.
fn apply(ctx: &Ctx, who: &Who, time: &str, queued: &[Queued]) -> Result<()> {
    for op in queued {
        check(op)?;
    }
    for op in queued {
        if let Some(old) = snapshot_of(op) {
            snapshot::save(
                &ctx.state,
                &op.change.id,
                old,
                time,
                &who.name,
                &ctx.command,
            )?;
        }
        write_one(op)?;
        journal::append(&ctx.state, &record(time, who, &ctx.command, &op.change))?;
    }
    Ok(())
}

/// The old bytes that a change keeps as a snapshot under its ID: every change of an entry, and
/// a change of another file only when that file is a `.md` file (pictures and values files of
/// other kinds keep no snapshot). None for a new file.
fn snapshot_of(op: &Queued) -> Option<&[u8]> {
    let keeps = match &op.action {
        Action::Put { .. } | Action::Delete { .. } | Action::Rename { .. } => true,
        Action::PutRaw { path, .. } | Action::DeleteRaw { path } => is_md(path),
    };
    if keeps { op.old.as_deref() } else { None }
}

/// Whether the path ends in `.md`.
fn is_md(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "md")
}

/// What must hold before anything is written: each changed file is still the file as it was
/// read, and the new file of a rename does not exist yet.
fn check(op: &Queued) -> Result<()> {
    match &op.action {
        Action::Put { path, .. }
        | Action::Delete { path }
        | Action::PutRaw { path, .. }
        | Action::DeleteRaw { path } => unchanged(path, op.old.as_deref()),
        Action::Rename { from, to, .. } => {
            unchanged(from, op.old.as_deref())?;
            if to == from {
                Ok(())
            } else {
                unchanged(to, None)
            }
        }
    }
}

/// Makes one change on the files. `check` has passed for it already.
fn write_one(op: &Queued) -> Result<()> {
    match &op.action {
        Action::Put { path, text } => replace(path, text.as_bytes(), op.old.as_deref()),
        Action::PutRaw { path, bytes } => replace(path, bytes, op.old.as_deref()),
        Action::Delete { path } | Action::DeleteRaw { path } => remove(path),
        // Renamed onto its own path: the same file, replaced in place.
        Action::Rename { from, to, text } if from == to => {
            replace(to, text.as_bytes(), op.old.as_deref())
        }
        Action::Rename { from, to, text } => {
            replace(to, text.as_bytes(), None)?;
            remove(from)
        }
    }
}

/// Replaces the file at `path` with `bytes`, expecting it to be `old` (None: a new file).
fn replace(path: &Path, bytes: &[u8], old: Option<&[u8]>) -> Result<()> {
    let expected = old.map(seen_of);
    atomic::replace(path, bytes, Some(&expected))
}

/// Removes the file at `path`.
fn remove(path: &Path) -> Result<()> {
    std::fs::remove_file(path)
        .map_err(|error| Error::new(Code::Io, format!("{}: {error}", path.display())))
}

/// Code::Changed unless the file at `path` is `old` (None: the file does not exist).
fn unchanged(path: &Path, old: Option<&[u8]>) -> Result<()> {
    if atomic::seen(path)? == old.map(seen_of) {
        Ok(())
    } else {
        Err(changed(path))
    }
}

/// The length and checksum that `atomic::seen` gives for a file holding `bytes`.
fn seen_of(bytes: &[u8]) -> Seen {
    Seen {
        len: bytes.len() as u64,
        hash: hash::sha256_hex(bytes),
    }
}

/// The refusal for a file changed since it was read. Same words as `atomic::replace`.
fn changed(path: &Path) -> Error {
    Error::new(
        Code::Changed,
        format!(
            "{} was changed by someone else; run the command again",
            path.display()
        ),
    )
}

/// The journal line for one change: who, which command, when, and the change itself.
fn record(time: &str, who: &Who, command: &str, change: &Change) -> journal::Record {
    journal::Record {
        time: time.to_string(),
        name: who.name.clone(),
        person: who.person.clone(),
        llm: who.is_llm,
        command: command.to_string(),
        id: change.id.clone(),
        event: change.event.clone(),
        detail: change.detail.clone(),
    }
}

/// The warning that a dry run gives for one change.
fn dry_run_warning(root: &Path, queued: &Queued) -> String {
    match &queued.action {
        Action::Put { path, .. } | Action::PutRaw { path, .. } => {
            format!("dry run: would write {}", shown(root, path))
        }
        Action::Delete { path } | Action::DeleteRaw { path } => {
            format!("dry run: would delete {}", shown(root, path))
        }
        Action::Rename { from, to, .. } => format!(
            "dry run: would rename {} to {}",
            shown(root, from),
            shown(root, to)
        ),
    }
}

/// A path for the people: relative to the project folder, with `/`. A path outside it is shown
/// whole.
fn shown(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::snapshot;
    use crate::store;
    use std::cell::Cell;
    use std::fs;
    use std::path::Path;
    use std::process::{Command, Stdio};

    /// The time that DOSSIER_NOW is set to during the tests.
    const NOW: &str = "2026-11-21T10:15:00+01:00";

    /// Holds ENV_LOCK and sets DOSSIER_NOW for as long as it lives, so that every time of the
    /// tests is NOW.
    struct FixedClock {
        _env: std::sync::MutexGuard<'static, ()>,
    }

    impl FixedClock {
        fn new() -> FixedClock {
            let guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            // SAFETY: ENV_LOCK is held until this value is dropped, so no other test in this
            // process reads or changes the environment meanwhile.
            unsafe { std::env::set_var("DOSSIER_NOW", NOW) };
            FixedClock { _env: guard }
        }
    }

    impl Drop for FixedClock {
        fn drop(&mut self) {
            // SAFETY: ENV_LOCK is still held here (the guard field is dropped after this).
            unsafe { std::env::remove_var("DOSSIER_NOW") };
        }
    }

    /// Anna's LLM, which acts in the tests.
    fn anna_claude() -> Who {
        Who {
            name: "anna-claude".to_string(),
            person: "Anna".to_string(),
            is_llm: true,
        }
    }

    /// A project folder with its DOSSIER folder (settings and state folder) and a context for
    /// it that acts as anna-claude. The folder name has a space and a non-ASCII letter. The
    /// temporary folder is returned so that it lives as long as the test.
    fn project() -> (tempfile::TempDir, Ctx) {
        let dir = tempfile::Builder::new()
            .prefix("Projekt Übersicht ")
            .tempdir()
            .expect("temporary folder");
        let dossier = dir.path().join("DOSSIER");
        fs::create_dir_all(dossier.join(".state")).expect("the state folder");
        fs::write(dossier.join("dossier.toml"), config::default_text("P-001"))
            .expect("dossier.toml");
        let cfg = config::load(&dossier).expect("the settings load");
        let ctx = Ctx {
            root: dir.path().to_path_buf(),
            state: dossier.join(".state"),
            dossier,
            cfg,
            who: Some(anna_claude()),
            json: false,
            dry_run: false,
            command: "found".to_string(),
            team: None,
            from_document: None,
            project: None,
        };
        (dir, ctx)
    }

    /// A decision file as format.md §1 has it, with an invented statement.
    fn entry_text(heading: &str) -> String {
        format!(
            "<!-- dossier-format: 1 -->\n# {heading}\n\n- kind: decision\n- status: approved\n- created: 2026-11-20T09:00:00+01:00\n- author: Anna\n\n## Decision\n\nAn invented decision for a test.\n"
        )
    }

    /// Writes an entry file at `relative` (under DOSSIER) and reads it back as an item.
    fn existing(ctx: &Ctx, relative: &str, text: &str, id: &str) -> Item {
        let path = ctx.dossier.join(relative);
        fs::create_dir_all(path.parent().expect("a folder")).expect("the kind folder");
        fs::write(&path, text).expect("write the entry");
        store::get(&ctx.dossier, id).expect("the entry is read")
    }

    fn change(id: &str, event: &str, detail: &str) -> Change {
        Change {
            id: id.to_string(),
            event: event.to_string(),
            detail: detail.to_string(),
        }
    }

    #[test]
    fn a_new_file_is_written_with_one_journal_line() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let path = ctx.dossier.join("decisions").join("DEC-001-first.md");
        let text = entry_text("DEC-001 · First");
        let output = write(&ctx, |tx| {
            tx.put(
                path.clone(),
                None,
                text.clone(),
                change("DEC-001", "created", ""),
            );
            Ok(Output::text("DEC-001: added"))
        })
        .expect("the change is written");

        assert_eq!(output.text, "DEC-001: added");
        assert!(output.warnings.is_empty());
        assert_eq!(fs::read_to_string(&path).expect("the file"), text);
        let records = journal::read(&ctx.state).expect("the journal");
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.time, NOW);
        assert_eq!(record.name, "anna-claude");
        assert_eq!(record.person, "Anna");
        assert!(record.llm);
        assert_eq!(record.command, "found");
        assert_eq!(record.id, "DEC-001");
        assert_eq!(record.event, "created");
        assert_eq!(record.detail, "");
        assert!(
            snapshot::list(&ctx.state, "DEC-001")
                .expect("the snapshots")
                .is_empty()
        );
        assert!(
            !ctx.state.join("holder.json").exists(),
            "the lock is released"
        );
    }

    #[test]
    fn a_put_over_an_existing_file_saves_the_old_bytes_as_snapshot_1() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let old_text = entry_text("DEC-002 · Second");
        let item = existing(&ctx, "decisions/DEC-002-second.md", &old_text, "DEC-002");
        let new_text = old_text.replace("An invented decision", "A changed decision");
        write(&ctx, |tx| {
            tx.put(
                item.path.clone(),
                Some(&item),
                new_text.clone(),
                change("DEC-002", "changed", "field decision"),
            );
            Ok(Output::text("DEC-002: changed"))
        })
        .expect("the change is written");

        assert_eq!(fs::read_to_string(&item.path).expect("the file"), new_text);
        let snaps = snapshot::list(&ctx.state, "DEC-002").expect("the snapshots");
        assert_eq!(snaps.len(), 1);
        assert_eq!(snaps[0].n, 1);
        assert_eq!(snaps[0].time, NOW);
        assert_eq!(snaps[0].name, "anna-claude");
        assert_eq!(snaps[0].command, "found");
        assert_eq!(
            snapshot::load(&ctx.state, "DEC-002", 1).expect("snapshot 1"),
            old_text.as_bytes()
        );
        let records = journal::read(&ctx.state).expect("the journal");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].event, "changed");
        assert_eq!(records[0].detail, "field decision");
    }

    #[test]
    fn an_error_in_the_command_writes_nothing() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let old_text = entry_text("DEC-003 · Third");
        let item = existing(&ctx, "decisions/DEC-003-third.md", &old_text, "DEC-003");
        let new_path = ctx.dossier.join("decisions").join("DEC-004-fourth.md");
        let error = write(&ctx, |tx| {
            tx.put(
                item.path.clone(),
                Some(&item),
                "changed text".to_string(),
                change("DEC-003", "changed", ""),
            );
            tx.put(
                new_path.clone(),
                None,
                entry_text("DEC-004 · Fourth"),
                change("DEC-004", "created", ""),
            );
            Err(Error::new(Code::Refused, "the reason is not clear"))
        })
        .err()
        .expect("the command fails");

        assert_eq!(error.code, Code::Refused);
        assert_eq!(error.message, "the reason is not clear");
        assert_eq!(fs::read_to_string(&item.path).expect("the file"), old_text);
        assert!(!new_path.exists());
        assert!(journal::read(&ctx.state).expect("the journal").is_empty());
        assert!(
            snapshot::list(&ctx.state, "DEC-003")
                .expect("the snapshots")
                .is_empty()
        );
    }

    #[test]
    fn a_dry_run_writes_nothing_and_warns_once_per_change() {
        let _clock = FixedClock::new();
        let (_dir, mut ctx) = project();
        ctx.dry_run = true;
        let old_text = entry_text("DEC-005 · Fifth");
        let item = existing(&ctx, "decisions/DEC-005-fifth.md", &old_text, "DEC-005");
        let new_path = ctx.dossier.join("decisions").join("DEC-006-sixth.md");
        let renamed = ctx
            .dossier
            .join("decisions")
            .join("DEC-005-fifth-renamed.md");
        let output = write(&ctx, |tx| {
            tx.put(
                new_path.clone(),
                None,
                entry_text("DEC-006 · Sixth"),
                change("DEC-006", "created", ""),
            );
            tx.rename(
                &item,
                renamed.clone(),
                old_text.clone(),
                change("DEC-005", "changed", "title"),
            );
            Ok(Output::text("two changes"))
        })
        .expect("the dry run");

        assert_eq!(output.text, "two changes");
        assert_eq!(
            output.warnings,
            [
                "dry run: would write DOSSIER/decisions/DEC-006-sixth.md",
                "dry run: would rename DOSSIER/decisions/DEC-005-fifth.md to DOSSIER/decisions/DEC-005-fifth-renamed.md",
            ]
        );
        assert!(!new_path.exists());
        assert!(!renamed.exists());
        assert_eq!(fs::read_to_string(&item.path).expect("the file"), old_text);
        assert!(journal::read(&ctx.state).expect("the journal").is_empty());
        assert!(
            !ctx.state.join("write.lock").exists(),
            "a dry run takes no lock"
        );
    }

    #[test]
    fn a_command_without_a_name_is_a_usage_error_and_runs_nothing() {
        let _clock = FixedClock::new();
        let (_dir, mut ctx) = project();
        ctx.who = None;
        let ran = Cell::new(false);
        let error = write(&ctx, |_tx| {
            ran.set(true);
            Ok(Output::text("never"))
        })
        .err()
        .expect("a name is needed");

        assert_eq!(error.code, Code::Usage);
        assert!(!ran.get(), "the command does not run without a name");
        assert!(!ctx.state.join("write.lock").exists());
    }

    #[test]
    fn a_file_changed_by_someone_else_is_refused_and_their_change_is_kept() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let old_text = entry_text("DEC-007 · Seventh");
        let item = existing(&ctx, "decisions/DEC-007-seventh.md", &old_text, "DEC-007");
        let error = write(&ctx, |tx| {
            tx.put(
                item.path.clone(),
                Some(&item),
                "changed text".to_string(),
                change("DEC-007", "changed", ""),
            );
            // Someone else changes the file while the command is running.
            fs::write(&item.path, "written by someone else").expect("the outside change");
            Ok(Output::text("done"))
        })
        .err()
        .expect("the change is refused");

        assert_eq!(error.code, Code::Changed);
        assert_eq!(error.exit(), 3);
        assert_eq!(
            fs::read_to_string(&item.path).expect("the file"),
            "written by someone else"
        );
        assert!(
            snapshot::list(&ctx.state, "DEC-007")
                .expect("the snapshots")
                .is_empty()
        );
        assert!(journal::read(&ctx.state).expect("the journal").is_empty());
    }

    #[test]
    fn a_new_file_over_an_existing_one_is_refused() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let taken = existing(
            &ctx,
            "decisions/DEC-008-eighth.md",
            &entry_text("DEC-008 · Eighth"),
            "DEC-008",
        );
        let error = write(&ctx, |tx| {
            tx.put(
                taken.path.clone(),
                None,
                "a new text".to_string(),
                change("DEC-008", "created", ""),
            );
            Ok(Output::text("done"))
        })
        .err()
        .expect("the file exists");

        assert_eq!(error.code, Code::Changed);
        assert_eq!(
            fs::read_to_string(&taken.path).expect("the file"),
            entry_text("DEC-008 · Eighth")
        );
        assert!(journal::read(&ctx.state).expect("the journal").is_empty());
    }

    #[test]
    fn a_delete_saves_a_snapshot_and_then_removes_the_file() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let old_text = entry_text("DEC-009 · Ninth");
        let item = existing(&ctx, "decisions/DEC-009-ninth.md", &old_text, "DEC-009");
        write(&ctx, |tx| {
            tx.delete(&item, change("DEC-009", "removed", "no longer needed"));
            Ok(Output::text("DEC-009: removed"))
        })
        .expect("the change is written");

        assert!(!item.path.exists());
        assert_eq!(
            snapshot::load(&ctx.state, "DEC-009", 1).expect("snapshot 1"),
            old_text.as_bytes()
        );
        let records = journal::read(&ctx.state).expect("the journal");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].event, "removed");
        assert_eq!(records[0].detail, "no longer needed");
    }

    #[test]
    fn a_rename_moves_the_file_and_keeps_the_old_bytes_in_a_snapshot() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let old_text = entry_text("DEC-010 · Tenth");
        let item = existing(&ctx, "decisions/DEC-010-tenth.md", &old_text, "DEC-010");
        let new_path = ctx
            .dossier
            .join("decisions")
            .join("DEC-010-tenth-renamed.md");
        let new_text = entry_text("DEC-010 · Tenth renamed");
        write(&ctx, |tx| {
            tx.rename(
                &item,
                new_path.clone(),
                new_text.clone(),
                change("DEC-010", "changed", "title"),
            );
            Ok(Output::text("DEC-010: renamed"))
        })
        .expect("the change is written");

        assert!(!item.path.exists());
        assert_eq!(fs::read_to_string(&new_path).expect("the file"), new_text);
        assert_eq!(
            snapshot::load(&ctx.state, "DEC-010", 1).expect("snapshot 1"),
            old_text.as_bytes()
        );
        assert_eq!(journal::read(&ctx.state).expect("the journal").len(), 1);
    }

    #[test]
    fn a_rename_onto_an_existing_file_is_refused_and_the_old_file_stays() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let old_text = entry_text("DEC-011 · Eleventh");
        let item = existing(&ctx, "decisions/DEC-011-eleventh.md", &old_text, "DEC-011");
        let taken = ctx.dossier.join("decisions").join("DEC-011-taken.md");
        fs::write(&taken, "another entry").expect("the other file");
        let error = write(&ctx, |tx| {
            tx.rename(
                &item,
                taken.clone(),
                old_text.clone(),
                change("DEC-011", "changed", "title"),
            );
            Ok(Output::text("done"))
        })
        .err()
        .expect("the new file exists");

        assert_eq!(error.code, Code::Changed);
        assert_eq!(fs::read_to_string(&item.path).expect("the file"), old_text);
        assert_eq!(
            fs::read_to_string(&taken).expect("the other file"),
            "another entry"
        );
        assert!(
            snapshot::list(&ctx.state, "DEC-011")
                .expect("the snapshots")
                .is_empty()
        );
        assert!(journal::read(&ctx.state).expect("the journal").is_empty());
    }

    #[test]
    fn a_rename_to_the_same_path_is_a_replacement_of_that_file() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let old_text = entry_text("DEC-012 · Twelfth");
        let item = existing(&ctx, "decisions/DEC-012-twelfth.md", &old_text, "DEC-012");
        let new_text = old_text.replace("invented", "reworded");
        write(&ctx, |tx| {
            tx.rename(
                &item,
                item.path.clone(),
                new_text.clone(),
                change("DEC-012", "changed", ""),
            );
            Ok(Output::text("done"))
        })
        .expect("the change is written");

        assert_eq!(fs::read_to_string(&item.path).expect("the file"), new_text);
        assert_eq!(
            snapshot::load(&ctx.state, "DEC-012", 1).expect("snapshot 1"),
            old_text.as_bytes()
        );
    }

    #[test]
    fn several_changes_are_written_in_the_order_they_were_queued() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let item = existing(
            &ctx,
            "decisions/DEC-013-thirteenth.md",
            &entry_text("DEC-013 · Thirteenth"),
            "DEC-013",
        );
        let new_path = ctx.dossier.join("decisions").join("DEC-014-fourteenth.md");
        write(&ctx, |tx| {
            tx.delete(&item, change("DEC-013", "removed", ""));
            tx.put(
                new_path.clone(),
                None,
                entry_text("DEC-014 · Fourteenth"),
                change("DEC-014", "created", ""),
            );
            Ok(Output::text("done"))
        })
        .expect("the changes are written");

        assert!(!item.path.exists());
        assert!(new_path.exists());
        let records = journal::read(&ctx.state).expect("the journal");
        let order: Vec<(&str, &str)> = records
            .iter()
            .map(|record| (record.id.as_str(), record.event.as_str()))
            .collect();
        assert_eq!(order, [("DEC-013", "removed"), ("DEC-014", "created")]);
    }

    /// A block file as format.md §17 has it, with an invented block for `key`.
    fn block_text(key: &str, note: &str) -> Vec<u8> {
        format!(
            "<!-- dossier-format: 1 -->\n# Files of T-001\n\n## {key}\n\n- role: other\n- note: {note}\n\n"
        )
        .into_bytes()
    }

    /// The bytes of a PNG picture: its signature and some invented data after it.
    fn picture_bytes() -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        bytes.extend_from_slice(b"invented picture data");
        bytes
    }

    #[test]
    fn a_new_block_file_is_written_with_one_journal_line() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let path = ctx.dossier.join("files").join("T-001.md");
        let bytes = block_text("models/wing-v3.step", "first note");
        let output = write(&ctx, |tx| {
            tx.put_raw(
                path.clone(),
                None,
                bytes.clone(),
                change("FILES-T-001", "attached", "models/wing-v3.step"),
            );
            Ok(Output::text("T-001: attached"))
        })
        .expect("the change is written");

        assert_eq!(output.text, "T-001: attached");
        assert!(output.warnings.is_empty());
        assert_eq!(fs::read(&path).expect("the file"), bytes);
        let records = journal::read(&ctx.state).expect("the journal");
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.time, NOW);
        assert_eq!(record.name, "anna-claude");
        assert_eq!(record.person, "Anna");
        assert!(record.llm);
        assert_eq!(record.command, "found");
        assert_eq!(record.id, "FILES-T-001");
        assert_eq!(record.event, "attached");
        assert_eq!(record.detail, "models/wing-v3.step");
        assert!(
            snapshot::list(&ctx.state, "FILES-T-001")
                .expect("the snapshots")
                .is_empty()
        );
    }

    #[test]
    fn a_second_put_raw_over_a_block_file_saves_its_old_bytes_as_snapshot_1() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let path = ctx.dossier.join("files").join("T-001.md");
        let first = block_text("models/wing-v3.step", "first note");
        write(&ctx, |tx| {
            tx.put_raw(
                path.clone(),
                None,
                first.clone(),
                change("FILES-T-001", "attached", "models/wing-v3.step"),
            );
            Ok(Output::text("T-001: attached"))
        })
        .expect("the first change is written");

        let second = block_text("models/wing-v3.step", "second note");
        write(&ctx, |tx| {
            tx.put_raw(
                path.clone(),
                Some(first.clone()),
                second.clone(),
                change("FILES-T-001", "changed", "note"),
            );
            Ok(Output::text("T-001: changed"))
        })
        .expect("the second change is written");

        assert_eq!(fs::read(&path).expect("the file"), second);
        let snaps = snapshot::list(&ctx.state, "FILES-T-001").expect("the snapshots");
        assert_eq!(snaps.len(), 1);
        assert_eq!(snaps[0].n, 1);
        assert_eq!(
            snapshot::load(&ctx.state, "FILES-T-001", 1).expect("snapshot 1"),
            first
        );
        let records = journal::read(&ctx.state).expect("the journal");
        assert_eq!(records.len(), 2);
        assert_eq!(records[1].event, "changed");
    }

    #[test]
    fn a_picture_is_written_and_its_deletion_keeps_no_snapshot() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let path = ctx
            .dossier
            .join("images")
            .join("DEC-015")
            .join("speedline.png");
        let bytes = picture_bytes();
        write(&ctx, |tx| {
            tx.put_raw(
                path.clone(),
                None,
                bytes.clone(),
                change("DEC-015", "picture added", "speedline.png"),
            );
            Ok(Output::text("DEC-015: picture added"))
        })
        .expect("the picture is written");
        assert_eq!(fs::read(&path).expect("the picture"), bytes);

        write(&ctx, |tx| {
            tx.delete_raw(
                path.clone(),
                bytes.clone(),
                change("DEC-015", "picture removed", "speedline.png"),
            );
            Ok(Output::text("DEC-015: picture removed"))
        })
        .expect("the picture is removed");

        assert!(!path.exists());
        assert!(
            snapshot::list(&ctx.state, "DEC-015")
                .expect("the snapshots")
                .is_empty()
        );
        let records = journal::read(&ctx.state).expect("the journal");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].event, "picture added");
        assert_eq!(records[1].event, "picture removed");
    }

    #[test]
    fn a_deleted_md_file_keeps_its_old_bytes_as_snapshot_1() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let path = ctx.dossier.join("files").join("T-002.md");
        fs::create_dir_all(path.parent().expect("a folder")).expect("the folder");
        let bytes = block_text("data/old-export.csv", "to be removed");
        fs::write(&path, &bytes).expect("the block file");
        write(&ctx, |tx| {
            tx.delete_raw(
                path.clone(),
                bytes.clone(),
                change("FILES-T-002", "detached", "data/old-export.csv"),
            );
            Ok(Output::text("T-002: detached"))
        })
        .expect("the file is removed");

        assert!(!path.exists());
        assert_eq!(
            snapshot::load(&ctx.state, "FILES-T-002", 1).expect("snapshot 1"),
            bytes
        );
    }

    #[test]
    fn a_raw_file_changed_by_someone_else_gives_changed_and_is_kept() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let path = ctx.dossier.join("files").join("T-003.md");
        fs::create_dir_all(path.parent().expect("a folder")).expect("the folder");
        let read_as = block_text("notes/review.txt", "as read");
        fs::write(&path, b"written by someone else").expect("the outside change");
        let error = write(&ctx, |tx| {
            tx.put_raw(
                path.clone(),
                Some(read_as.clone()),
                block_text("notes/review.txt", "new"),
                change("FILES-T-003", "changed", "note"),
            );
            Ok(Output::text("done"))
        })
        .err()
        .expect("the change is refused");

        assert_eq!(error.code, Code::Changed);
        assert_eq!(error.exit(), 3);
        assert_eq!(
            fs::read(&path).expect("the file"),
            b"written by someone else"
        );
        assert!(
            snapshot::list(&ctx.state, "FILES-T-003")
                .expect("the snapshots")
                .is_empty()
        );
        assert!(journal::read(&ctx.state).expect("the journal").is_empty());
    }

    #[test]
    fn a_raw_file_changed_before_its_delete_gives_changed_and_is_kept() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        let path = ctx
            .dossier
            .join("images")
            .join("DEC-017")
            .join("sketch.png");
        fs::create_dir_all(path.parent().expect("a folder")).expect("the folder");
        let read_as = picture_bytes();
        fs::write(&path, b"a different picture").expect("the outside change");
        let error = write(&ctx, |tx| {
            tx.delete_raw(
                path.clone(),
                read_as.clone(),
                change("DEC-017", "picture removed", "sketch.png"),
            );
            Ok(Output::text("done"))
        })
        .err()
        .expect("the delete is refused");

        assert_eq!(error.code, Code::Changed);
        assert_eq!(fs::read(&path).expect("the file"), b"a different picture");
        assert!(journal::read(&ctx.state).expect("the journal").is_empty());
    }

    #[test]
    fn a_dry_run_of_raw_changes_writes_nothing_and_warns_once_per_change() {
        let _clock = FixedClock::new();
        let (_dir, mut ctx) = project();
        ctx.dry_run = true;
        let picture_path = ctx.dossier.join("images").join("DEC-016").join("old.png");
        fs::create_dir_all(picture_path.parent().expect("a folder")).expect("the folder");
        let picture = picture_bytes();
        fs::write(&picture_path, &picture).expect("the picture");
        let new_path = ctx.dossier.join("files").join("T-004.md");
        let output = write(&ctx, |tx| {
            tx.put_raw(
                new_path.clone(),
                None,
                block_text("models/new.step", "dry"),
                change("FILES-T-004", "attached", "models/new.step"),
            );
            tx.delete_raw(
                picture_path.clone(),
                picture.clone(),
                change("DEC-016", "picture removed", "old.png"),
            );
            Ok(Output::text("two changes"))
        })
        .expect("the dry run");

        assert_eq!(output.text, "two changes");
        assert_eq!(
            output.warnings,
            [
                "dry run: would write DOSSIER/files/T-004.md",
                "dry run: would delete DOSSIER/images/DEC-016/old.png",
            ]
        );
        assert!(!new_path.exists());
        assert_eq!(fs::read(&picture_path).expect("the picture"), picture);
        assert!(journal::read(&ctx.state).expect("the journal").is_empty());
        assert!(
            snapshot::list(&ctx.state, "DEC-016")
                .expect("the snapshots")
                .is_empty()
        );
        assert!(
            !ctx.state.join("write.lock").exists(),
            "a dry run takes no lock"
        );
    }

    /// A team dossier: the project of `project()` with the kind of a team.
    fn team() -> (tempfile::TempDir, Ctx) {
        let (dir, mut ctx) = project();
        ctx.cfg.kind = "team".to_string();
        (dir, ctx)
    }

    /// Whether git runs here. The repository tests return early without it.
    fn git_runs() -> bool {
        Command::new("git")
            .arg("--version")
            .stdin(Stdio::null())
            .output()
            .is_ok_and(|output| output.status.success())
    }

    /// Runs git in `dir` with `args` and returns its standard output.
    fn git_out(dir: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .expect("git runs");
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    /// Makes the project folder of `ctx` a git repository, with an invented identity and no
    /// commit signing, so that the commits work on any machine. It also writes the
    /// `.state/.gitignore` that `init --team-dossier` writes (format.md §20), so that the lock
    /// files stay out of the commits.
    fn git_repository(ctx: &Ctx) {
        fs::write(ctx.state.join(".gitignore"), "write.lock\nholder.json\n")
            .expect("the .gitignore of the state folder");
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.name", "Test Person"],
            vec!["config", "user.email", "test.person@example.invalid"],
            vec!["config", "commit.gpgsign", "false"],
        ] {
            let output = Command::new("git")
                .arg("-C")
                .arg(&ctx.root)
                .args(&args)
                .stdin(Stdio::null())
                .output()
                .expect("git runs");
            assert!(
                output.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    /// The subjects of the commits in `dir`, newest first. Empty when there is no commit.
    fn subjects(dir: &Path) -> Vec<String> {
        git_out(dir, &["log", "--format=%s"])
            .lines()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn a_team_write_outside_a_repository_writes_normally_and_gives_no_warning() {
        let _clock = FixedClock::new();
        let (_dir, ctx) = team();
        let path = ctx.dossier.join("lessons").join("LES-001-first.md");
        let text = "<!-- dossier-format: 1 -->\n# LES-001 · First\n\n- kind: lesson\n".to_string();
        let output = write(&ctx, |tx| {
            tx.put(
                path.clone(),
                None,
                text.clone(),
                change("LES-001", "created", ""),
            );
            Ok(Output::text("LES-001: added"))
        })
        .expect("the change is written");

        assert_eq!(output.text, "LES-001: added");
        assert!(output.warnings.is_empty());
        assert_eq!(fs::read_to_string(&path).expect("the file"), text);
        assert_eq!(journal::read(&ctx.state).expect("the journal").len(), 1);
        assert!(!ctx.root.join(".git").exists());
    }

    #[test]
    fn a_team_write_in_a_repository_is_one_commit_named_by_the_command_and_the_ids() {
        if !git_runs() {
            return;
        }
        let _clock = FixedClock::new();
        let (_dir, ctx) = team();
        git_repository(&ctx);
        let path = ctx.dossier.join("lessons").join("LES-001-first.md");
        let raw = ctx.dossier.join("files").join("T-001.md");
        let output = write(&ctx, |tx| {
            tx.put(
                path.clone(),
                None,
                "lesson text\n".to_string(),
                change("LES-001", "created", ""),
            );
            tx.put_raw(
                raw.clone(),
                None,
                block_text("models/wing-v3.step", "first note"),
                change("FILES-T-001", "attached", "models/wing-v3.step"),
            );
            Ok(Output::text("two changes"))
        })
        .expect("the changes are written");

        assert!(output.warnings.is_empty());
        assert_eq!(subjects(&ctx.root), ["found: LES-001, FILES-T-001"]);
        assert_eq!(git_out(&ctx.root, &["status", "--porcelain"]), "");
    }

    #[test]
    fn a_commit_that_fails_is_a_warning_and_the_change_stays_written() {
        if !git_runs() {
            return;
        }
        let _clock = FixedClock::new();
        let (_dir, ctx) = team();
        git_repository(&ctx);
        // A lock file left behind by another git process makes `git add` fail.
        fs::write(ctx.root.join(".git").join("index.lock"), "").expect("the lock file");
        let path = ctx.dossier.join("lessons").join("LES-001-first.md");
        let output = write(&ctx, |tx| {
            tx.put(
                path.clone(),
                None,
                "lesson text\n".to_string(),
                change("LES-001", "created", ""),
            );
            Ok(Output::text("LES-001: added"))
        })
        .expect("the change is written");

        assert_eq!(output.text, "LES-001: added");
        assert_eq!(output.warnings.len(), 1, "{:?}", output.warnings);
        assert!(
            output.warnings[0].starts_with("git: git add failed"),
            "{:?}",
            output.warnings
        );
        assert_eq!(
            fs::read_to_string(&path).expect("the file"),
            "lesson text\n"
        );
        assert_eq!(journal::read(&ctx.state).expect("the journal").len(), 1);
        assert!(subjects(&ctx.root).is_empty());
    }

    #[test]
    fn a_dry_run_of_a_team_write_makes_no_commit() {
        if !git_runs() {
            return;
        }
        let _clock = FixedClock::new();
        let (_dir, mut ctx) = team();
        git_repository(&ctx);
        ctx.dry_run = true;
        let path = ctx.dossier.join("lessons").join("LES-001-first.md");
        let output = write(&ctx, |tx| {
            tx.put(
                path.clone(),
                None,
                "lesson text\n".to_string(),
                change("LES-001", "created", ""),
            );
            Ok(Output::text("dry"))
        })
        .expect("the dry run");

        assert_eq!(
            output.warnings,
            ["dry run: would write DOSSIER/lessons/LES-001-first.md"]
        );
        assert!(!path.exists());
        assert!(subjects(&ctx.root).is_empty());
    }

    #[test]
    fn a_project_write_is_not_committed_and_a_team_write_without_a_change_is_not_either() {
        if !git_runs() {
            return;
        }
        let _clock = FixedClock::new();
        let (_dir, ctx) = project();
        git_repository(&ctx);
        let path = ctx.dossier.join("decisions").join("DEC-001-first.md");
        let output = write(&ctx, |tx| {
            tx.put(
                path.clone(),
                None,
                entry_text("DEC-001 · First"),
                change("DEC-001", "created", ""),
            );
            Ok(Output::text("DEC-001: added"))
        })
        .expect("the change is written");
        assert!(output.warnings.is_empty());
        assert!(subjects(&ctx.root).is_empty(), "a project is not committed");

        let (_team_dir, team_ctx) = team();
        git_repository(&team_ctx);
        let output = write(&team_ctx, |_tx| Ok(Output::text("nothing to change")))
            .expect("the command runs");
        assert!(output.warnings.is_empty());
        assert!(subjects(&team_ctx.root).is_empty(), "no change, no commit");
    }

    #[test]
    fn the_change_ids_of_a_command_are_each_named_once_in_order() {
        let queued = vec![
            Queued {
                action: Action::Delete {
                    path: PathBuf::from("a.md"),
                },
                old: None,
                change: change("DEC-001", "changed", ""),
            },
            Queued {
                action: Action::Delete {
                    path: PathBuf::from("b.md"),
                },
                old: None,
                change: change("DEC-002", "removed", ""),
            },
            Queued {
                action: Action::Delete {
                    path: PathBuf::from("c.md"),
                },
                old: None,
                change: change("DEC-001", "changed", "again"),
            },
        ];
        assert_eq!(change_ids(&queued), ["DEC-001", "DEC-002"]);
    }
}
