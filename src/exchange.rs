//! The exchange with the customer or other parties (format.md §17, exchange.md): one block per
//! received (RCV) or delivered (DLV) record.

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};
use std::path::Path;

/// The file of a dossier that holds the exchange, `exchange.md`.
pub const FILE: &str = "exchange.md";

/// The item name of the exchange file in snapshots and the journal.
pub const ITEM: &str = "EXCHANGE";

/// The exchange file of a project and its bytes as read; a missing file is
/// BlockFile::empty("Exchange of <code>"), None.
pub fn load(_dossier: &Path, _code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    Err(Error::new(Code::NotImplemented, "exchange::load"))
}

/// The highest number of RCV or DLV keys with this prefix, 0 when there is none.
pub fn highest(_file: &BlockFile, _prefix: &str) -> u64 {
    0
}
