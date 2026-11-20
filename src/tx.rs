//! Transactions: the one way an entry file is written. Takes the lock, saves the snapshot,
//! replaces the file, and appends the journal line.

use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::store::Item;
use std::path::PathBuf;

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
    pub fn put(&mut self, path: PathBuf, old: Option<&Item>, text: String, change: Change) {}

    /// Queues deleting the file of `old`.
    pub fn delete(&mut self, old: &Item, change: Change) {}

    /// Queues moving `old` to `new_path`, with `text` as its new content.
    pub fn rename(&mut self, old: &Item, new_path: PathBuf, text: String, change: Change) {}
}

/// Refuses a network drive, takes the lock, runs `f`, then for each queued change: saves a
/// snapshot of the old file, replaces the file with atomic::replace (expecting the old bytes),
/// appends the journal record. With ctx.dry_run nothing is written and the Output gets one
/// warning per change: `dry run: would write <path>`. An error in `f` writes nothing.
pub fn write<F>(ctx: &Ctx, f: F) -> Result<Output>
where
    F: FnOnce(&mut Tx) -> Result<Output>,
{
    Err(Error::new(Code::NotImplemented, "tx::write"))
}
