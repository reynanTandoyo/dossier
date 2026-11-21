//! Transactions: the one way an entry file is written. Takes the lock, saves the snapshot,
//! replaces the file, and appends the journal line.

use crate::atomic::{self, Seen};
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
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
    pub fn put_raw(
        &mut self,
        _path: PathBuf,
        _old: Option<Vec<u8>>,
        _bytes: Vec<u8>,
        _change: Change,
    ) {
    }

    /// Queue removing a file that is still `old`.
    pub fn delete_raw(&mut self, _path: PathBuf, _old: Vec<u8>, _change: Change) {}
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
    Ok(output)
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
        if let Some(old) = &op.old {
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

/// What must hold before anything is written: each changed file is still the file as it was
/// read, and the new file of a rename does not exist yet.
fn check(op: &Queued) -> Result<()> {
    match &op.action {
        Action::Put { path, .. } | Action::Delete { path } => unchanged(path, op.old.as_deref()),
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
        Action::Put { path, text } => replace(path, text, op.old.as_deref()),
        Action::Delete { path } => remove(path),
        // Renamed onto its own path: the same file, replaced in place.
        Action::Rename { from, to, text } if from == to => replace(to, text, op.old.as_deref()),
        Action::Rename { from, to, text } => {
            replace(to, text, None)?;
            remove(from)
        }
    }
}

/// Replaces the file at `path` with `text`, expecting it to be `old` (None: a new file).
fn replace(path: &Path, text: &str, old: Option<&[u8]>) -> Result<()> {
    let expected = old.map(seen_of);
    atomic::replace(path, text.as_bytes(), Some(&expected))
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
        Action::Put { path, .. } => format!("dry run: would write {}", shown(root, path)),
        Action::Delete { path } => format!("dry run: would delete {}", shown(root, path)),
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
}
