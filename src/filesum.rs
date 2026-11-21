//! Checksums and stamps of files outside the dossier (format.md §18), and counts of a folder.

use crate::error::{Code, Error, Result};
use chrono::{DateTime, Local};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Files above this size (1 GiB) get the short checksum (format.md §18).
pub const SHORT_ABOVE: u64 = 1 << 30;

/// The bytes hashed at each end of a file by the short checksum (16 MiB).
const EDGE: u64 = 16 << 20;

/// The most bytes read at a time (1 MiB), so that a big file is never held in memory.
const CHUNK: usize = 1 << 20;

/// "sha256:<hex>", or "short-sha256:<hex>" above SHORT_ABOVE (format.md §18). Reads in chunks.
pub fn checksum(path: &Path) -> Result<String> {
    checksum_with(path, SHORT_ABOVE, EDGE)
}

/// The checksum of the file at `path`: the whole file, or when it is larger than `limit`, the
/// first and the last `edge` bytes and the size as eight bytes little-endian (format.md §18).
fn checksum_with(path: &Path, limit: u64, edge: u64) -> Result<String> {
    let mut file = File::open(path).map_err(|error| io_error(path, error))?;
    let size = file
        .metadata()
        .map_err(|error| io_error(path, error))?
        .len();
    let mut buffer = vec![0u8; CHUNK];
    let mut hasher = Sha256::new();

    if size <= limit {
        feed(&mut hasher, &mut file, u64::MAX, &mut buffer)
            .map_err(|error| io_error(path, error))?;
        return Ok(format!("sha256:{}", hex(&hasher.finalize())));
    }

    let edge = edge.min(size);
    feed(&mut hasher, &mut file, edge, &mut buffer).map_err(|error| io_error(path, error))?;
    file.seek(SeekFrom::Start(size - edge))
        .map_err(|error| io_error(path, error))?;
    feed(&mut hasher, &mut file, edge, &mut buffer).map_err(|error| io_error(path, error))?;
    hasher.update(size.to_le_bytes());
    Ok(format!("short-sha256:{}", hex(&hasher.finalize())))
}

/// Hashes the next `count` bytes of `file`, or fewer when the file ends first, one chunk at a
/// time through `buffer`.
fn feed(
    hasher: &mut Sha256,
    file: &mut File,
    mut count: u64,
    buffer: &mut [u8],
) -> std::io::Result<()> {
    while count > 0 {
        let wanted = count.min(buffer.len() as u64) as usize;
        let read = match file.read(&mut buffer[..wanted]) {
            Ok(0) => break,
            Ok(read) => read,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        hasher.update(&buffer[..read]);
        count -= read as u64;
    }
    Ok(())
}

/// The size and modification time of a file.
pub struct Stamp {
    pub size: u64,
    /// format.md §7, local time.
    pub modified: String,
}

/// The size and the modification time of the file at `path`.
pub fn stamp(path: &Path) -> Result<Stamp> {
    let metadata = std::fs::metadata(path).map_err(|error| io_error(path, error))?;
    let modified = metadata.modified().map_err(|error| io_error(path, error))?;
    let local: DateTime<Local> = modified.into();
    Ok(Stamp {
        size: metadata.len(),
        modified: local.format("%Y-%m-%dT%H:%M:%S%:z").to_string(),
    })
}

/// The number of regular files in a folder, recursive. Folders and symbolic links are not
/// counted, and links are not followed.
pub fn count_files(folder: &Path) -> Result<u64> {
    let mut count: u64 = 0;
    let mut pending = vec![folder.to_path_buf()];
    while let Some(current) = pending.pop() {
        let entries = std::fs::read_dir(&current).map_err(|error| io_error(&current, error))?;
        for entry in entries {
            let path = entry.map_err(|error| io_error(&current, error))?.path();
            let kind = std::fs::symlink_metadata(&path)
                .map_err(|error| io_error(&path, error))?
                .file_type();
            if kind.is_dir() {
                pending.push(path);
            } else if kind.is_file() {
                count += 1;
            }
        }
    }
    Ok(count)
}

/// A failure on `path` as Code::Io, with the path in the message.
fn io_error(path: &Path, error: std::io::Error) -> Error {
    Error::new(Code::Io, format!("{}: {error}", path.display()))
}

/// Lower-case hex of `bytes`.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::sha256_hex;
    use chrono::TimeZone;

    const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    fn write_file(folder: &Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = folder.join(name);
        std::fs::write(&path, bytes).expect("write test file");
        path
    }

    #[test]
    fn checksum_of_abc_is_the_known_digest() {
        let folder = tempfile::tempdir().expect("temporary folder");
        let path = write_file(folder.path(), "abc.txt", b"abc");
        assert_eq!(
            checksum(&path).expect("checksum"),
            format!("sha256:{ABC_SHA256}")
        );
    }

    #[test]
    fn checksum_of_an_empty_file_is_the_digest_of_nothing() {
        let folder = tempfile::tempdir().expect("temporary folder");
        let path = write_file(folder.path(), "empty.txt", b"");
        assert_eq!(
            checksum(&path).expect("checksum"),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn checksum_of_a_file_over_several_chunks_is_the_digest_of_all_of_it() {
        // Two full chunks and a partial one: the chunk boundaries must not change the digest.
        let folder = tempfile::tempdir().expect("temporary folder");
        let bytes: Vec<u8> = (0..(2 * CHUNK + 17)).map(|i| (i % 251) as u8).collect();
        let path = write_file(folder.path(), "big.bin", &bytes);
        assert_eq!(
            checksum(&path).expect("checksum"),
            format!("sha256:{}", sha256_hex(&bytes))
        );
    }

    #[test]
    fn file_names_with_spaces_and_non_ascii_work() {
        let folder = tempfile::tempdir().expect("temporary folder");
        let path = write_file(folder.path(), "rapport final ü.txt", b"abc");
        assert_eq!(
            checksum(&path).expect("checksum"),
            format!("sha256:{ABC_SHA256}")
        );
        assert_eq!(stamp(&path).expect("stamp").size, 3);
    }

    #[test]
    fn short_checksum_hashes_the_two_edges_and_the_size() {
        // 40 bytes, a limit of 10 and edges of 4: first 4, last 4 and the size.
        let folder = tempfile::tempdir().expect("temporary folder");
        let bytes: Vec<u8> = (0u8..40).collect();
        let path = write_file(folder.path(), "edges.bin", &bytes);
        let mut expected = Vec::new();
        expected.extend_from_slice(&bytes[..4]);
        expected.extend_from_slice(&bytes[36..]);
        expected.extend_from_slice(&40u64.to_le_bytes());
        assert_eq!(
            checksum_with(&path, 10, 4).expect("checksum"),
            format!("short-sha256:{}", sha256_hex(&expected))
        );
    }

    #[test]
    fn short_checksum_does_not_read_the_middle() {
        let folder = tempfile::tempdir().expect("temporary folder");
        let bytes: Vec<u8> = (0u8..40).collect();
        let path = write_file(folder.path(), "middle.bin", &bytes);
        let before = checksum_with(&path, 10, 4).expect("checksum");

        let mut changed = bytes.clone();
        changed[20] ^= 0xff;
        std::fs::write(&path, &changed).expect("rewrite test file");
        assert_eq!(checksum_with(&path, 10, 4).expect("checksum"), before);

        changed[0] ^= 0xff;
        std::fs::write(&path, &changed).expect("rewrite test file");
        assert_ne!(checksum_with(&path, 10, 4).expect("checksum"), before);
    }

    #[test]
    fn a_file_exactly_at_the_limit_gets_the_full_checksum() {
        let folder = tempfile::tempdir().expect("temporary folder");
        let at_limit = write_file(folder.path(), "ten.bin", &[7u8; 10]);
        let over_limit = write_file(folder.path(), "eleven.bin", &[7u8; 11]);
        assert_eq!(
            checksum_with(&at_limit, 10, 4).expect("checksum"),
            format!("sha256:{}", sha256_hex(&[7u8; 10]))
        );
        assert!(
            checksum_with(&over_limit, 10, 4)
                .expect("checksum")
                .starts_with("short-sha256:")
        );
    }

    #[test]
    fn stamp_gives_the_size_and_the_local_time_of_the_last_change() {
        let folder = tempfile::tempdir().expect("temporary folder");
        let path = write_file(folder.path(), "hello.txt", b"hello");
        let local = Local
            .with_ymd_and_hms(2026, 11, 20, 14, 3, 27)
            .single()
            .expect("a local time that exists");
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .expect("open test file");
        file.set_modified(local.into()).expect("set modified time");
        drop(file);

        let stamped = stamp(&path).expect("stamp");
        assert_eq!(stamped.size, 5);
        assert_eq!(
            stamped.modified,
            local.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
        );
        assert!(
            crate::time::valid(&stamped.modified),
            "{}",
            stamped.modified
        );
    }

    #[test]
    fn checksum_of_a_missing_file_is_an_io_error_naming_it() {
        let folder = tempfile::tempdir().expect("temporary folder");
        let path = folder.path().join("missing.txt");
        let error = checksum(&path).expect_err("a missing file has no checksum");
        assert_eq!(error.code, Code::Io);
        assert!(
            error.message.starts_with(&path.display().to_string()),
            "{}",
            error.message
        );
    }

    #[test]
    fn stamp_of_a_missing_file_is_an_io_error_naming_it() {
        let folder = tempfile::tempdir().expect("temporary folder");
        let path = folder.path().join("missing.txt");
        let error = stamp(&path).err().expect("a missing file has no stamp");
        assert_eq!(error.code, Code::Io);
        assert!(
            error.message.starts_with(&path.display().to_string()),
            "{}",
            error.message
        );
    }

    #[test]
    fn count_files_counts_the_files_of_nested_folders() {
        let folder = tempfile::tempdir().expect("temporary folder");
        let root = folder.path();
        std::fs::create_dir_all(root.join("sub").join("deeper")).expect("folders");
        std::fs::create_dir_all(root.join("empty folder")).expect("folder");
        write_file(root, "a.txt", b"a");
        write_file(&root.join("sub"), "b.txt", b"b");
        write_file(&root.join("sub").join("deeper"), "c.txt", b"c");
        write_file(&root.join("sub"), "ünï cödé.txt", b"d");
        assert_eq!(count_files(root).expect("count"), 4);
        assert_eq!(count_files(&root.join("empty folder")).expect("count"), 0);
    }

    #[test]
    fn count_files_of_a_missing_folder_is_an_io_error_naming_it() {
        let folder = tempfile::tempdir().expect("temporary folder");
        let missing = folder.path().join("missing");
        let error = count_files(&missing).expect_err("a missing folder has no count");
        assert_eq!(error.code, Code::Io);
        assert!(
            error.message.starts_with(&missing.display().to_string()),
            "{}",
            error.message
        );
    }

    // Making a link on Windows needs Developer Mode or an elevated shell, so the test is ignored
    // there by default. Run it with `-- --ignored` where links are allowed.
    #[cfg_attr(
        windows,
        ignore = "making links on Windows needs Developer Mode or an elevated shell"
    )]
    #[test]
    fn count_files_skips_symbolic_links() {
        let folder = tempfile::tempdir().expect("temporary folder");
        let root = folder.path();
        std::fs::create_dir_all(root.join("real")).expect("folder");
        write_file(&root.join("real"), "one.txt", b"1");
        let target = write_file(&root.join("real"), "two.txt", b"2");
        let linked_folder = root.join("linked folder");
        let linked_file = root.join("linked.txt");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.join("real"), &linked_folder).expect("link folder");
            std::os::unix::fs::symlink(&target, &linked_file).expect("link file");
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(root.join("real"), &linked_folder)
                .expect("link folder");
            std::os::windows::fs::symlink_file(&target, &linked_file).expect("link file");
        }
        assert_eq!(count_files(root).expect("count"), 2);
    }
}
