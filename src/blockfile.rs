//! Block files (format.md §17): the format line, a heading, and blocks of `## <key>` with field
//! lines. Kept as lines, so that every byte not changed stays.

use crate::error::{Code, Error, Result};
use crate::names::Who;

/// A block file (format.md §17), kept as its lines so that every byte not changed stays.
#[derive(Debug, Clone)]
pub struct BlockFile {
    pub lines: Vec<String>,
}

impl BlockFile {
    /// TooNew above format 1; Parse: no heading.
    pub fn parse(_text: &str) -> Result<BlockFile> {
        Err(Error::new(Code::NotImplemented, "blockfile::parse"))
    }

    /// Format line, "# <title>", one blank line.
    pub fn empty(_title: &str) -> BlockFile {
        BlockFile { lines: Vec::new() }
    }

    /// Equals the parsed text.
    pub fn text(&self) -> String {
        String::new()
    }

    /// `## ` keys in file order.
    pub fn keys(&self) -> Vec<String> {
        Vec::new()
    }

    /// The value of one field of a block.
    pub fn field(&self, _key: &str, _name: &str) -> Option<String> {
        None
    }

    /// The fields of a block, history included.
    pub fn fields(&self, _key: &str) -> Vec<(String, String)> {
        Vec::new()
    }

    /// The values of the history lines of a block.
    pub fn history(&self, _key: &str) -> Vec<String> {
        Vec::new()
    }

    /// Appends `## <key>`, a blank line, the fields (history pairs last), a blank line.
    pub fn add_block(&mut self, _key: &str, _fields: &[(String, String)]) {}

    /// Replaces the field or adds it before the block's first history line; an empty value
    /// removes it; `history` changes nothing; a missing block changes nothing.
    pub fn set_field(&mut self, _key: &str, _name: &str, _value: &str) {}

    /// Adds one history line to a block.
    pub fn add_history(
        &mut self,
        _key: &str,
        _time: &str,
        _who: &Who,
        _event: &str,
        _detail: &str,
    ) {
    }
}
