//! A minimal tar archive: ustar, regular files and folders, no compression.
//!
//! Every file is a 512-byte header followed by its contents, padded with zeros to a whole block.
//! Folders have no header of their own: unpacking creates the folders a file needs. The archive
//! ends with two zero blocks. Packing writes names in byte order at every level and sets the same
//! mode and time on every file, so one folder gives one archive on every system.

use crate::error::{Code, Error, Result};
use std::fs::File;
use std::io::{BufWriter, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// The size of a block: a header, or the contents of a file padded to whole blocks.
const BLOCK: usize = 512;
/// The longest name in the name field of a header.
const NAME_LEN: usize = 100;
/// The longest prefix in the prefix field of a header.
const PREFIX_LEN: usize = 155;
/// The largest size an octal size field holds: eleven octal digits, 8 GiB - 1.
const MAX_SIZE: u64 = 0o77_777_777_777;
/// Zeros: padding, and the two blocks that end an archive.
const ZEROS: [u8; BLOCK] = [0; BLOCK];

/// Writes the files and folders of `folder` into the archive `out`, which is replaced if it
/// exists. Names are relative to `folder`, with '/', and are written in byte order at every
/// level. Symbolic links and other special files are skipped, and so is `out` when it lies
/// inside `folder`. A name that does not fit a header (over 100 bytes, or over 155 bytes before
/// a '/'), that is not UTF-8 or that has a backslash is Code::BadValue, and then nothing is
/// written. Returns the number of files.
pub fn pack(folder: &Path, out: &Path) -> Result<u32> {
    if !folder.is_dir() {
        return Err(Error::new(
            Code::NotFound,
            format!("{}: not a folder", folder.display()),
        ));
    }
    // The canonical path of `out` exists only once `out` does, so the first pack has nothing to
    // skip.
    let skip = std::fs::canonicalize(out).ok();
    let mut sources = Vec::new();
    gather(folder, "", skip.as_deref(), &mut sources)?;
    let count = u32::try_from(sources.len())
        .map_err(|_| Error::new(Code::BadValue, "more files than an archive can count"))?;
    write_archive(&sources, out)?;
    Ok(count)
}

/// Restores the archive `archive` into `into`, which must be empty or missing (Code::Exists
/// otherwise). The whole archive is checked before anything is written, so a damaged archive,
/// or a name that would leave the folder (Code::BadValue), leaves `into` as it was. Returns the
/// number of files.
pub fn unpack(archive: &Path, into: &Path) -> Result<u32> {
    refuse_non_empty(into)?;
    let mut file = File::open(archive).map_err(|error| {
        if error.kind() == ErrorKind::NotFound {
            Error::new(
                Code::NotFound,
                format!("{}: no such archive", archive.display()),
            )
        } else {
            io_error(archive, error)
        }
    })?;
    let len = file
        .metadata()
        .map_err(|error| io_error(archive, error))?
        .len();
    let entries = index(&mut file, len, archive)?;
    let files = entries
        .iter()
        .filter(|entry| entry.kind == Kind::File)
        .count();
    let count = u32::try_from(files)
        .map_err(|_| Error::new(Code::BadValue, "more files than an archive can count"))?;

    std::fs::create_dir_all(into).map_err(|error| io_error(into, error))?;
    for entry in &entries {
        let mut target = into.to_path_buf();
        for part in &entry.parts {
            target.push(part);
        }
        match entry.kind {
            Kind::Folder => {
                std::fs::create_dir_all(&target).map_err(|error| io_error(&target, error))?
            }
            Kind::File => extract(&mut file, archive, entry, &target)?,
        }
    }
    Ok(count)
}

/// A regular file to pack: the prefix and name of its header, and where its contents are.
struct Source {
    prefix: String,
    name: String,
    path: PathBuf,
    size: u64,
}

/// Adds the regular files below `dir` to `sources`, depth first, names in byte order. `relative`
/// is the folder's name from the top folder, with '/' ("" for the top folder itself).
fn gather(
    dir: &Path,
    relative: &str,
    skip: Option<&Path>,
    sources: &mut Vec<Source>,
) -> Result<()> {
    let mut names = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|error| io_error(dir, error))? {
        let entry = entry.map_err(|error| io_error(dir, error))?;
        let name = entry.file_name().into_string().map_err(|bad| {
            Error::new(
                Code::BadValue,
                format!("{}: the name is not UTF-8", dir.join(bad).display()),
            )
        })?;
        names.push(name);
    }
    names.sort();

    for name in names {
        let path = dir.join(&name);
        // symlink_metadata: a symbolic link is never followed, and is neither a file nor a folder.
        let meta = std::fs::symlink_metadata(&path).map_err(|error| io_error(&path, error))?;
        let file_type = meta.file_type();
        let inside = if relative.is_empty() {
            name
        } else {
            format!("{relative}/{name}")
        };
        if file_type.is_dir() {
            gather(&path, &inside, skip, sources)?;
        } else if file_type.is_file() && !is_same_file(&path, skip) {
            if inside.contains('\\') {
                return Err(Error::new(
                    Code::BadValue,
                    format!("{inside}: a backslash is not allowed in an archive name"),
                ));
            }
            let (prefix, name) = split_name(&inside)?;
            if meta.len() > MAX_SIZE {
                return Err(Error::new(
                    Code::BadValue,
                    format!("{inside}: the file is too big for the archive format"),
                ));
            }
            sources.push(Source {
                prefix: prefix.to_string(),
                name: name.to_string(),
                path,
                size: meta.len(),
            });
        }
    }
    Ok(())
}

/// Whether `path` is the archive being written (`skip` is its canonical path).
fn is_same_file(path: &Path, skip: Option<&Path>) -> bool {
    skip.is_some_and(|skip| std::fs::canonicalize(path).is_ok_and(|full| full.as_path() == skip))
}

/// The ustar prefix and name of a relative name. A name of at most 100 bytes has no prefix; a
/// longer one is split at the last '/' that leaves a prefix of at most 155 bytes. Code::BadValue
/// when no such split leaves a name of at most 100 bytes.
fn split_name(name: &str) -> Result<(&str, &str)> {
    if name.len() <= NAME_LEN {
        return Ok(("", name));
    }
    let slash = (0..=PREFIX_LEN.min(name.len() - 1))
        .rev()
        .find(|&at| name.as_bytes()[at] == b'/');
    match slash {
        Some(at) if name.len() - at - 1 <= NAME_LEN => Ok((&name[..at], &name[at + 1..])),
        _ => Err(Error::new(
            Code::BadValue,
            format!(
                "{name}: the name is too long for the archive (over 100 bytes, or over 155 bytes before a '/')"
            ),
        )),
    }
}

/// Creates `out` and writes the archive into it. A partial archive is removed on failure.
fn write_archive(sources: &[Source], out: &Path) -> Result<()> {
    let file = File::create(out).map_err(|error| io_error(out, error))?;
    let written = write_members(sources, file, out);
    if written.is_err() {
        // A partial archive is litter; the error says why it failed.
        let _ = std::fs::remove_file(out);
    }
    written
}

/// Writes the header, the contents and the padding of each file, then the two zero blocks.
fn write_members(sources: &[Source], file: File, out: &Path) -> Result<()> {
    let mut writer = BufWriter::new(file);
    for source in sources {
        writer
            .write_all(&header(&source.name, &source.prefix, source.size))
            .map_err(|error| io_error(out, error))?;
        let reader = File::open(&source.path).map_err(|error| io_error(&source.path, error))?;
        // At most `size` bytes: a file that grew since it was listed is cut at the size written.
        let copied = std::io::copy(&mut reader.take(source.size), &mut writer)
            .map_err(|error| io_error(&source.path, error))?;
        if copied != source.size {
            return Err(Error::new(
                Code::Changed,
                format!(
                    "{} changed while it was packed; run the command again",
                    source.path.display()
                ),
            ));
        }
        let padding = source.size.next_multiple_of(BLOCK as u64) - source.size;
        writer
            .write_all(&ZEROS[..padding as usize])
            .map_err(|error| io_error(out, error))?;
    }
    writer
        .write_all(&ZEROS)
        .map_err(|error| io_error(out, error))?;
    writer
        .write_all(&ZEROS)
        .map_err(|error| io_error(out, error))?;
    writer.flush().map_err(|error| io_error(out, error))
}

/// The header block of a regular file: mode 0644, no owner, modification time 0, ustar magic.
/// Offsets: name 0..100, mode 100..108, uid 108..116, gid 116..124, size 124..136,
/// mtime 136..148, checksum 148..156, typeflag 156, magic 257..263, version 263..265,
/// devmajor 329..337, devminor 337..345, prefix 345..500.
fn header(name: &str, prefix: &str, size: u64) -> [u8; BLOCK] {
    let mut block = [0u8; BLOCK];
    put(&mut block, 0, NAME_LEN, name.as_bytes());
    put_octal(&mut block, 100, 8, 0o644);
    put_octal(&mut block, 108, 8, 0);
    put_octal(&mut block, 116, 8, 0);
    put_octal(&mut block, 124, 12, size);
    put_octal(&mut block, 136, 12, 0);
    // The checksum is the sum of the header with the checksum field read as spaces.
    block[148..156].fill(b' ');
    block[156] = b'0';
    block[257..263].copy_from_slice(b"ustar\0");
    block[263..265].copy_from_slice(b"00");
    put_octal(&mut block, 329, 8, 0);
    put_octal(&mut block, 337, 8, 0);
    put(&mut block, 345, PREFIX_LEN, prefix.as_bytes());
    let sum: u64 = block.iter().map(|&byte| u64::from(byte)).sum();
    // Six octal digits, a NUL and a space.
    block[148..154].copy_from_slice(format!("{sum:06o}").as_bytes());
    block[154] = 0;
    block[155] = b' ';
    block
}

/// Copies `bytes` into the field at `at` of `field_len` bytes, truncating if it is longer.
fn put(block: &mut [u8; BLOCK], at: usize, field_len: usize, bytes: &[u8]) {
    let n = bytes.len().min(field_len);
    block[at..at + n].copy_from_slice(&bytes[..n]);
}

/// Writes `value` in octal into the field at `at` of `len` bytes: zero-padded digits, then a NUL.
fn put_octal(block: &mut [u8; BLOCK], at: usize, len: usize, value: u64) {
    let digits = format!("{value:0width$o}", width = len - 1);
    put(block, at, len - 1, digits.as_bytes());
    block[at + len - 1] = 0;
}

/// The number in an octal field: digits, possibly padded with spaces, ended by a NUL. An empty
/// field is 0. None when the field holds anything else.
fn octal(field: &[u8]) -> Option<u64> {
    let end = field
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(field.len());
    field[..end]
        .trim_ascii()
        .iter()
        .try_fold(0u64, |value, &byte| {
            if !(b'0'..=b'7').contains(&byte) {
                return None;
            }
            value.checked_mul(8)?.checked_add(u64::from(byte - b'0'))
        })
}

/// The text of a name field up to its first NUL.
fn field_text<'a>(field: &'a [u8], archive: &Path) -> Result<&'a str> {
    let end = field
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(field.len());
    std::str::from_utf8(&field[..end]).map_err(|_| {
        Error::new(
            Code::BadValue,
            format!("{}: a name in the archive is not UTF-8", archive.display()),
        )
    })
}

/// Whether the checksum of a header is right: the sum of its bytes with the checksum field read
/// as spaces (POSIX), or the same sum with signed bytes (older tools).
fn checksum_ok(block: &[u8; BLOCK]) -> bool {
    let Some(stored) = octal(&block[148..156]) else {
        return false;
    };
    let Ok(stored) = i64::try_from(stored) else {
        return false;
    };
    let (mut unsigned, mut signed) = (0i64, 0i64);
    for (at, &byte) in block.iter().enumerate() {
        let byte = if (148..156).contains(&at) { b' ' } else { byte };
        unsigned += i64::from(byte);
        signed += i64::from(byte as i8);
    }
    stored == unsigned || stored == signed
}

/// The kind of an entry that unpack writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    File,
    Folder,
}

/// What a header says: the kind of entry, its size, and its name (prefix and name joined), which
/// is None when the name field is empty.
struct Header {
    kind: Kind,
    size: u64,
    name: Option<String>,
}

/// An entry that unpack writes: its kind, the folders and name it goes to, and where its
/// contents are in the archive.
struct Entry {
    kind: Kind,
    parts: Vec<String>,
    offset: u64,
    size: u64,
}

/// Code::Parse: the archive is not one this module can read.
fn damaged(archive: &Path, what: &str) -> Error {
    Error::new(
        Code::Parse,
        format!("{}: the archive is damaged: {what}", archive.display()),
    )
}

/// Code::Exists when `folder` exists and is not an empty folder.
fn refuse_non_empty(folder: &Path) -> Result<()> {
    match std::fs::metadata(folder) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error(folder, error)),
        Ok(meta) if !meta.is_dir() => Err(Error::new(
            Code::Exists,
            format!("{}: exists and is not a folder", folder.display()),
        )),
        Ok(_) => {
            let mut entries = std::fs::read_dir(folder).map_err(|error| io_error(folder, error))?;
            match entries.next() {
                None => Ok(()),
                Some(Ok(_)) => Err(Error::new(
                    Code::Exists,
                    format!("{}: exists and is not empty", folder.display()),
                )),
                Some(Err(error)) => Err(io_error(folder, error)),
            }
        }
    }
}

/// Reads the whole archive once, block by block, and lists its entries. Contents are skipped,
/// not read. The archive must end with a zero block.
fn index(file: &mut File, len: u64, archive: &Path) -> Result<Vec<Entry>> {
    let mut entries = Vec::new();
    let mut block = [0u8; BLOCK];
    // The offset of the next block.
    let mut at: u64 = 0;
    loop {
        let filled = read_block(file, &mut block, archive)?;
        if filled == 0 {
            return Err(damaged(archive, "it ends without a zero block"));
        }
        if filled < BLOCK {
            return Err(damaged(archive, "it is cut off inside a header"));
        }
        let header_at = at;
        at += BLOCK as u64;
        if block.iter().all(|&byte| byte == 0) {
            return Ok(entries);
        }
        let header = parse_header(&block, header_at, archive)?;
        let contents = at;
        if header.size > len.saturating_sub(contents) {
            return Err(damaged(
                archive,
                "it is cut off inside the contents of an entry",
            ));
        }
        at = contents + header.size.next_multiple_of(BLOCK as u64);
        if let Some(name) = header.name {
            let parts = parts(&name, archive)?;
            // A name with no parts (such as "./") has nothing to write; its contents are skipped.
            if !parts.is_empty() {
                entries.push(Entry {
                    kind: header.kind,
                    parts,
                    offset: contents,
                    size: header.size,
                });
            }
        }
        file.seek(SeekFrom::Start(at))
            .map_err(|error| io_error(archive, error))?;
    }
}

/// Reads the next block. The count is the number of bytes read: 0 at the end, less than a block
/// when the archive is cut off.
fn read_block(file: &mut File, block: &mut [u8; BLOCK], archive: &Path) -> Result<usize> {
    let mut filled = 0;
    while filled < BLOCK {
        match file.read(&mut block[filled..]) {
            Ok(0) => break,
            Ok(n) => filled += n,
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) => return Err(io_error(archive, error)),
        }
    }
    Ok(filled)
}

/// Checks a header and reads what it says. Code::Parse for a bad checksum, a bad size or magic;
/// Code::BadValue for an entry that is neither a file nor a folder.
fn parse_header(block: &[u8; BLOCK], at: u64, archive: &Path) -> Result<Header> {
    if !checksum_ok(block) {
        return Err(damaged(
            archive,
            &format!("the header at byte {at} has a bad checksum"),
        ));
    }
    let size = octal(&block[124..136]).ok_or_else(|| {
        damaged(
            archive,
            &format!("the size in the header at byte {at} is not an octal number"),
        )
    })?;
    let kind = match block[156] {
        b'0' | 0 => Kind::File,
        b'5' => Kind::Folder,
        _ => {
            return Err(Error::new(
                Code::BadValue,
                format!(
                    "{}: the entry at byte {at} is neither a file nor a folder",
                    archive.display()
                ),
            ));
        }
    };
    let magic = &block[257..265];
    if !(magic.starts_with(b"ustar") || magic.iter().all(|&byte| byte == 0)) {
        return Err(damaged(
            archive,
            &format!("the header at byte {at} is not a tar header"),
        ));
    }
    let name = field_text(&block[0..NAME_LEN], archive)?;
    let name = if name.is_empty() {
        None
    } else if &block[257..263] == b"ustar\0" {
        let prefix = field_text(&block[345..345 + PREFIX_LEN], archive)?;
        Some(if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{prefix}/{name}")
        })
    } else {
        Some(name.to_string())
    };
    Ok(Header { kind, size, name })
}

/// The folders and file name of an archive name, relative to the folder being unpacked into.
/// A name that is absolute, has a backslash, starts with a drive letter or has a '..' segment
/// is refused (Code::BadValue). Empty and '.' segments are dropped.
fn parts(name: &str, archive: &Path) -> Result<Vec<String>> {
    let bytes = name.as_bytes();
    let drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    if name.starts_with('/') || name.contains('\\') || drive {
        return Err(refused(name, archive));
    }
    let mut parts = Vec::new();
    for part in name.split('/') {
        match part {
            "" | "." => {}
            ".." => return Err(refused(name, archive)),
            _ => parts.push(part.to_string()),
        }
    }
    Ok(parts)
}

/// Code::BadValue for a name that would write outside the folder.
fn refused(name: &str, archive: &Path) -> Error {
    Error::new(
        Code::BadValue,
        format!(
            "{}: the name {name:?} would leave the folder",
            archive.display()
        ),
    )
}

/// Writes one file of the archive to `target`, creating its folders. Code::Parse when the
/// archive ends inside its contents.
fn extract(source: &mut File, archive: &Path, entry: &Entry, target: &Path) -> Result<()> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
    }
    let mut out = File::create(target).map_err(|error| io_error(target, error))?;
    source
        .seek(SeekFrom::Start(entry.offset))
        .map_err(|error| io_error(archive, error))?;
    let copied = std::io::copy(&mut source.take(entry.size), &mut out)
        .map_err(|error| io_error(target, error))?;
    if copied != entry.size {
        return Err(damaged(
            archive,
            "it is cut off inside the contents of a file",
        ));
    }
    Ok(())
}

/// A failure on `path` as Code::Io, with the path in the message.
fn io_error(path: &Path, error: std::io::Error) -> Error {
    Error::new(Code::Io, format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The files the tests pack: a nested file, an empty file, a file of 1000 bytes and a name
    /// with a space and an umlaut. They are created in the order given, or in reverse order.
    fn sample(folder: &Path, reversed: bool) {
        let mut files: Vec<(&str, Vec<u8>)> = vec![
            ("sub/deeper/note.md", b"nested".to_vec()),
            ("empty.txt", Vec::new()),
            ("big.bin", (0..1000).map(|i| (i % 251) as u8).collect()),
            ("Größe Übersicht.md", "Ü und ß".as_bytes().to_vec()),
        ];
        if reversed {
            files.reverse();
        }
        for (name, bytes) in files {
            let path = folder.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, bytes).unwrap();
        }
    }

    /// Every regular file below `folder`: its name with '/' and its contents, sorted.
    fn contents(folder: &Path) -> Vec<(String, Vec<u8>)> {
        fn walk(root: &Path, dir: &Path, found: &mut Vec<(String, Vec<u8>)>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(root, &path, found);
                } else {
                    let name = path
                        .strip_prefix(root)
                        .unwrap()
                        .components()
                        .map(|part| part.as_os_str().to_string_lossy().into_owned())
                        .collect::<Vec<_>>()
                        .join("/");
                    found.push((name, std::fs::read(&path).unwrap()));
                }
            }
        }
        let mut found = Vec::new();
        walk(folder, folder, &mut found);
        found.sort();
        found
    }

    /// An archive of the given (name, contents) entries, written with `header`.
    fn crafted(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (name, data) in entries {
            bytes.extend_from_slice(&header(name, "", data.len() as u64));
            bytes.extend_from_slice(data);
            bytes.resize(bytes.len().next_multiple_of(BLOCK), 0);
        }
        bytes.extend_from_slice(&ZEROS);
        bytes.extend_from_slice(&ZEROS);
        bytes
    }

    #[cfg(unix)]
    fn make_link(target: &Path, link: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(windows)]
    fn make_link(target: &Path, link: &Path) -> std::io::Result<()> {
        std::os::windows::fs::symlink_file(target, link)
    }

    #[test]
    fn a_folder_packed_and_unpacked_is_identical() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("Projekt Übersicht");
        let source = base.join("source");
        sample(&source, false);
        let archive = base.join("backup.tar");
        assert_eq!(pack(&source, &archive).unwrap(), 4);
        let target = base.join("restored");
        std::fs::create_dir(&target).unwrap();
        assert_eq!(unpack(&archive, &target).unwrap(), 4);
        assert_eq!(contents(&target), contents(&source));
        assert_eq!(contents(&source).len(), 4);
    }

    #[test]
    fn the_same_folder_packs_to_identical_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        sample(&source, false);
        let first = dir.path().join("first.tar");
        let second = dir.path().join("second.tar");
        pack(&source, &first).unwrap();
        pack(&source, &second).unwrap();
        assert_eq!(
            std::fs::read(&first).unwrap(),
            std::fs::read(&second).unwrap()
        );
    }

    #[test]
    fn the_order_of_creation_does_not_change_the_archive() {
        let dir = tempfile::tempdir().unwrap();
        let forward = dir.path().join("forward");
        let backward = dir.path().join("backward");
        sample(&forward, false);
        sample(&backward, true);
        let first = dir.path().join("forward.tar");
        let second = dir.path().join("backward.tar");
        pack(&forward, &first).unwrap();
        pack(&backward, &second).unwrap();
        assert_eq!(
            std::fs::read(&first).unwrap(),
            std::fs::read(&second).unwrap()
        );
    }

    #[test]
    fn unpack_into_a_non_empty_folder_fails_and_keeps_the_folder() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        sample(&source, false);
        let archive = dir.path().join("backup.tar");
        pack(&source, &archive).unwrap();
        let target = dir.path().join("restored");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("keep.txt"), b"mine").unwrap();
        let error = unpack(&archive, &target).unwrap_err();
        assert_eq!(error.code, Code::Exists);
        assert_eq!(error.exit(), 1);
        assert_eq!(std::fs::read(target.join("keep.txt")).unwrap(), b"mine");
        assert_eq!(contents(&target).len(), 1);
    }

    #[test]
    fn headers_follow_the_ustar_layout() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        std::fs::create_dir(&source).unwrap();
        std::fs::write(source.join("a.txt"), b"abc").unwrap();
        let archive = dir.path().join("one.tar");
        assert_eq!(pack(&source, &archive).unwrap(), 1);
        let bytes = std::fs::read(&archive).unwrap();
        // One header, one block of contents, two zero blocks.
        assert_eq!(bytes.len(), 512 + 512 + 1024);
        let header = &bytes[..512];
        assert_eq!(&header[0..5], b"a.txt");
        assert!(header[5..100].iter().all(|&byte| byte == 0));
        assert_eq!(&header[100..108], b"0000644\0");
        assert_eq!(&header[108..116], b"0000000\0");
        assert_eq!(&header[116..124], b"0000000\0");
        assert_eq!(&header[124..136], b"00000000003\0");
        assert_eq!(&header[136..148], b"00000000000\0");
        assert_eq!(header[156], b'0');
        assert_eq!(&header[257..263], b"ustar\0");
        assert_eq!(&header[263..265], b"00");
        let sum: u32 = header
            .iter()
            .enumerate()
            .map(|(at, &byte)| {
                if (148..156).contains(&at) {
                    u32::from(b' ')
                } else {
                    u32::from(byte)
                }
            })
            .sum();
        assert_eq!(&header[148..154], format!("{sum:06o}").as_bytes());
        assert_eq!(header[154], 0);
        assert_eq!(header[155], b' ');
        assert_eq!(&bytes[512..515], b"abc");
        assert!(bytes[515..1024].iter().all(|&byte| byte == 0));
        assert!(bytes[1024..].iter().all(|&byte| byte == 0));
    }

    #[test]
    fn a_long_name_uses_the_prefix_field_and_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let prefix = format!("{}/{}", "a".repeat(60), "b".repeat(60));
        std::fs::create_dir_all(source.join(&prefix)).unwrap();
        std::fs::write(source.join(&prefix).join("c.txt"), b"x").unwrap();
        let archive = dir.path().join("long.tar");
        assert_eq!(pack(&source, &archive).unwrap(), 1);
        let bytes = std::fs::read(&archive).unwrap();
        assert_eq!(&bytes[..5], b"c.txt");
        assert_eq!(&bytes[345..345 + prefix.len()], prefix.as_bytes());
        let target = dir.path().join("restored");
        unpack(&archive, &target).unwrap();
        assert_eq!(
            contents(&target),
            vec![(format!("{prefix}/c.txt"), b"x".to_vec())]
        );
    }

    #[test]
    fn names_too_long_for_the_header_are_refused() {
        let long_names = [
            format!("{}.txt", "x".repeat(101)),
            format!("{}/c.txt", "a".repeat(160)),
        ];
        for name in long_names {
            let dir = tempfile::tempdir().unwrap();
            let source = dir.path().join("source");
            let path = source.join(&name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, b"x").unwrap();
            let archive = dir.path().join("too-long.tar");
            let error = pack(&source, &archive).unwrap_err();
            assert_eq!(error.code, Code::BadValue, "{name}");
            assert!(!archive.exists(), "{name}");
        }
    }

    #[test]
    fn symbolic_links_are_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        sample(&source, false);
        let outside = dir.path().join("outside.txt");
        std::fs::write(&outside, b"not part of the folder").unwrap();
        if make_link(&outside, &source.join("link.txt")).is_err() {
            // Making links needs a permission some Windows accounts lack: nothing to skip then.
            return;
        }
        let archive = dir.path().join("links.tar");
        assert_eq!(pack(&source, &archive).unwrap(), 4);
        let target = dir.path().join("restored");
        assert_eq!(unpack(&archive, &target).unwrap(), 4);
        let expected = dir.path().join("expected");
        sample(&expected, false);
        assert_eq!(contents(&target), contents(&expected));
    }

    #[test]
    fn an_empty_folder_packs_to_two_zero_blocks() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("empty");
        std::fs::create_dir(&source).unwrap();
        let archive = dir.path().join("none.tar");
        assert_eq!(pack(&source, &archive).unwrap(), 0);
        assert_eq!(std::fs::read(&archive).unwrap(), vec![0u8; 1024]);
        let target = dir.path().join("restored");
        assert_eq!(unpack(&archive, &target).unwrap(), 0);
        assert!(target.is_dir());
    }

    #[test]
    fn an_archive_inside_the_folder_is_not_packed_into_itself() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        sample(&source, false);
        let archive = source.join("backup.tar");
        assert_eq!(pack(&source, &archive).unwrap(), 4);
        let first = std::fs::read(&archive).unwrap();
        assert_eq!(pack(&source, &archive).unwrap(), 4);
        assert_eq!(std::fs::read(&archive).unwrap(), first);
    }

    #[test]
    fn packing_a_missing_folder_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let error = pack(&dir.path().join("missing"), &dir.path().join("x.tar")).unwrap_err();
        assert_eq!(error.code, Code::NotFound);
    }

    #[test]
    fn names_that_would_leave_the_folder_are_refused() {
        let bad = [
            "../escape.txt",
            "a/../../escape.txt",
            "/absolute.txt",
            "C:escape.txt",
            "d:/escape.txt",
            "back\\slash.txt",
        ];
        for name in bad {
            let dir = tempfile::tempdir().unwrap();
            let archive = dir.path().join("bad.tar");
            std::fs::write(&archive, crafted(&[(name, b"x")])).unwrap();
            let target = dir.path().join("restored");
            let error = unpack(&archive, &target).unwrap_err();
            assert_eq!(error.code, Code::BadValue, "{name}");
            assert!(!target.exists(), "{name}");
            assert!(!dir.path().join("escape.txt").exists(), "{name}");
        }
    }

    #[test]
    fn a_damaged_header_is_refused_and_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let mut bytes = crafted(&[("a.txt", b"abc")]);
        bytes[0] = b'b';
        let archive = dir.path().join("damaged.tar");
        std::fs::write(&archive, bytes).unwrap();
        let target = dir.path().join("restored");
        let error = unpack(&archive, &target).unwrap_err();
        assert_eq!(error.code, Code::Parse);
        assert!(!target.exists());
    }

    #[test]
    fn an_archive_without_a_zero_block_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = crafted(&[("a.txt", b"abc")]);
        let archive = dir.path().join("cut.tar");
        std::fs::write(&archive, &bytes[..1024]).unwrap();
        let target = dir.path().join("restored");
        let error = unpack(&archive, &target).unwrap_err();
        assert_eq!(error.code, Code::Parse);
        assert!(!target.exists());
    }

    #[test]
    fn a_missing_archive_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let error = unpack(&dir.path().join("none.tar"), &dir.path().join("restored")).unwrap_err();
        assert_eq!(error.code, Code::NotFound);
    }
}
