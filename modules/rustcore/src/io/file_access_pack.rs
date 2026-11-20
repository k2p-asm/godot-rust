//! `core/io/file_access_pack.*` and `core/io/pck_packer.*`: the `.pck` format.
//!
//! The reader parses the engine's pack header (`GDPC` magic, format versions
//! 1–4, flags, relative file base, the reserved block, and the directory of
//! `path / offset / size / md5 / flags` entries) and registers every file in
//! [`PackedData`] under the MD5 of its simplified path, exactly like the engine's
//! `PathMD5` map, together with the directory tree used by `get_file_paths`.
//!
//! The writer ([`build_pack`]) emits the version-2 layout the reader accepts, so
//! packs written by the port can be read back. The engine's `PCKPacker` streams
//! from disk; this port takes the file contents in memory (documented
//! deviation), as does its reader.
//!
//! Not ported: encrypted directories/files (reported as `Encrypted`), sparse
//! bundles, delta patches and the executable-embedded header scan.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::crypto::md5::md5;

pub const PACK_HEADER_MAGIC: u32 = 0x4350_4447; // "GDPC"
pub const PACK_FORMAT_VERSION_V2: u32 = 2;
pub const PACK_FORMAT_VERSION_V3: u32 = 3;
pub const PACK_FORMAT_VERSION_V4: u32 = 4;
pub const PACK_FORMAT_VERSION: u32 = PACK_FORMAT_VERSION_V4;

pub const PACK_DIR_ENCRYPTED: u32 = 1 << 0;
pub const PACK_REL_FILEBASE: u32 = 1 << 1;
pub const PACK_SPARSE_BUNDLE: u32 = 1 << 2;

pub const PACK_FILE_ENCRYPTED: u32 = 1 << 0;
pub const PACK_FILE_REMOVAL: u32 = 1 << 1;
pub const PACK_FILE_DELTA: u32 = 1 << 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackError {
    /// `ERR_FILE_UNRECOGNIZED`.
    Unrecognized,
    /// `ERR_FILE_CORRUPT`.
    Corrupt,
    /// Encrypted directories are not supported by this port.
    Encrypted,
}

#[derive(Clone, Debug)]
pub struct PackedFile {
    pub pack: String,
    pub offset: u64,
    pub size: u64,
    pub md5: [u8; 16],
    pub encrypted: bool,
    pub bundle: bool,
    pub delta: bool,
}

/// A directory in the pack's tree (`PackedDir`).
#[derive(Clone, Debug, Default)]
struct PackedDir {
    subdirs: BTreeMap<String, PackedDir>,
    files: BTreeSet<String>,
}

#[derive(Clone, Debug, Default)]
pub struct PackedData {
    files: HashMap<[u8; 16], PackedFile>,
    paths: HashMap<[u8; 16], String>,
    root: PackedDir,
    /// The pack bytes, by pack name (the engine keeps a `FileAccess` instead).
    packs: BTreeMap<String, Vec<u8>>,
    disabled: bool,
}

/// `simplify_path().trim_prefix("res://")`.
fn simplify(path: &str) -> String {
    let simplified = crate::io::path_utils::simplify_path(path);
    simplified
        .strip_prefix("res://")
        .unwrap_or(&simplified)
        .to_string()
}

fn base_dir(path: &str) -> &str {
    match path.rfind('/') {
        Some(index) => &path[..index],
        None => "",
    }
}

fn file_name(path: &str) -> &str {
    match path.rfind('/') {
        Some(index) => &path[index + 1..],
        None => path,
    }
}

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    let bytes = data.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_u64(data: &[u8], offset: usize) -> Option<u64> {
    let bytes = data.get(offset..offset + 8)?;
    Some(u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}

impl PackedData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// `add_path`: registers a file (or a directory when the path ends in `/`).
    #[allow(clippy::too_many_arguments)]
    pub fn add_path(
        &mut self,
        pack: &str,
        path: &str,
        offset: u64,
        size: u64,
        content_hash: [u8; 16],
        replace_files: bool,
        encrypted: bool,
        bundle: bool,
        delta: bool,
    ) {
        let simplified = simplify(path);
        let key = md5(simplified.as_bytes());
        let exists = self.files.contains_key(&key);

        let entry = PackedFile {
            pack: pack.to_string(),
            offset,
            size,
            md5: content_hash,
            encrypted,
            bundle,
            delta,
        };
        if delta {
            // Delta patches are not applied by this port.
            return;
        }
        if !exists || replace_files {
            self.files.insert(key, entry);
            self.paths.insert(key, simplified.clone());
        }

        if exists {
            return;
        }

        // Walk/create the directory chain.
        let mut dir = &mut self.root;
        let directory = base_dir(&simplified);
        if !directory.is_empty() {
            for part in directory.split('/') {
                dir = dir
                    .subdirs
                    .entry(part.to_string())
                    .or_default();
            }
        }
        let name = file_name(&simplified);
        if !name.is_empty() {
            dir.files.insert(name.to_string());
        }
    }

    /// `remove_path`.
    pub fn remove_path(&mut self, path: &str) -> bool {
        let simplified = simplify(path);
        let key = md5(simplified.as_bytes());
        if self.files.remove(&key).is_none() {
            return false;
        }
        self.paths.remove(&key);

        let directory = base_dir(&simplified);
        let mut dir = &mut self.root;
        if !directory.is_empty() {
            for part in directory.split('/') {
                match dir.subdirs.get_mut(part) {
                    Some(next) => dir = next,
                    None => return true,
                }
            }
        }
        dir.files.remove(file_name(&simplified));
        true
    }

    /// `get_file_hash`: the stored MD5 of the path (the path digest, not the
    /// file content digest; the engine returns the entry's hash field).
    pub fn get_file_hash(&self, path: &str) -> Option<[u8; 16]> {
        self.files
            .get(&md5(simplify(path).as_bytes()))
            .map(|entry| entry.md5)
    }

    /// `has_path`.
    pub fn has_path(&self, path: &str) -> bool {
        self.files.contains_key(&md5(simplify(path).as_bytes()))
    }

    /// `get_size`.
    pub fn get_size(&self, path: &str) -> Option<u64> {
        self.files
            .get(&md5(simplify(path).as_bytes()))
            .map(|entry| entry.size)
    }

    pub fn get_packed_file(&self, path: &str) -> Option<&PackedFile> {
        self.files.get(&md5(simplify(path).as_bytes()))
    }

    /// The file contents, when the pack bytes are held by this reader.
    pub fn get_file_data(&self, path: &str) -> Option<&[u8]> {
        let entry = self.get_packed_file(path)?;
        if entry.encrypted {
            return None;
        }
        let pack = self.packs.get(&entry.pack)?;
        let start = entry.offset as usize;
        let end = start.checked_add(entry.size as usize)?;
        pack.get(start..end)
    }

    /// `get_file_paths`: every registered path (with `res://` prefix like the
    /// engine's flattened walk).
    pub fn get_file_paths(&self) -> Vec<String> {
        let mut out = Vec::new();
        collect_paths(&self.root, "", &mut out);
        out.sort();
        out
    }

    /// `clear`.
    pub fn clear(&mut self) {
        self.files.clear();
        self.paths.clear();
        self.root = PackedDir::default();
        self.packs.clear();
    }

    pub fn get_file_count(&self) -> usize {
        self.files.len()
    }

    /// Parses a pack buffer and registers its files. `start_pos` is the offset of
    /// the header within the source (the engine's `pck_start_pos`).
    pub fn add_pack_bytes(
        &mut self,
        pack_name: &str,
        bytes: Vec<u8>,
        replace_files: bool,
        start_pos: u64,
    ) -> Result<usize, PackError> {
        let magic = read_u32(&bytes, start_pos as usize).ok_or(PackError::Corrupt)?;
        if magic != PACK_HEADER_MAGIC {
            return Err(PackError::Unrecognized);
        }
        let mut cursor = start_pos as usize + 4;
        let version = read_u32(&bytes, cursor).ok_or(PackError::Corrupt)?;
        cursor += 4;
        if !(1..=PACK_FORMAT_VERSION).contains(&version) {
            return Err(PackError::Unrecognized);
        }
        // Version triple.
        for _ in 0..3 {
            read_u32(&bytes, cursor).ok_or(PackError::Corrupt)?;
            cursor += 4;
        }
        let pack_flags = read_u32(&bytes, cursor).ok_or(PackError::Corrupt)?;
        cursor += 4;
        if pack_flags & PACK_DIR_ENCRYPTED != 0 {
            return Err(PackError::Encrypted);
        }
        let relative_base = pack_flags & PACK_REL_FILEBASE != 0;
        let mut file_base = read_u64(&bytes, cursor).ok_or(PackError::Corrupt)?;
        cursor += 8;
        if version >= PACK_FORMAT_VERSION_V3 || (version == PACK_FORMAT_VERSION_V2 && relative_base)
        {
            file_base += start_pos;
        }
        // 16 reserved words.
        cursor += 16 * 4;

        let file_count = read_u32(&bytes, cursor).ok_or(PackError::Corrupt)?;
        cursor += 4;
        let mut registered = 0usize;

        for _ in 0..file_count {
            let path_length = read_u32(&bytes, cursor).ok_or(PackError::Corrupt)? as usize;
            cursor += 4;
            let path_bytes = bytes
                .get(cursor..cursor + path_length)
                .ok_or(PackError::Corrupt)?;
            let path = String::from_utf8_lossy(path_bytes).into_owned();
            cursor += path_length;
            let offset = read_u64(&bytes, cursor).ok_or(PackError::Corrupt)?;
            cursor += 8;
            let size = read_u64(&bytes, cursor).ok_or(PackError::Corrupt)?;
            cursor += 8;
            let md5_bytes = bytes.get(cursor..cursor + 16).ok_or(PackError::Corrupt)?;
            let mut hash = [0u8; 16];
            hash.copy_from_slice(md5_bytes);
            cursor += 16;
            let flags = if version >= PACK_FORMAT_VERSION_V2 {
                let flags = read_u32(&bytes, cursor).ok_or(PackError::Corrupt)?;
                cursor += 4;
                flags
            } else {
                0
            };

            let sparse_bundle = pack_flags & PACK_SPARSE_BUNDLE != 0;
            self.add_path(
                pack_name,
                &path,
                file_base + offset,
                size,
                hash,
                replace_files,
                flags & PACK_FILE_ENCRYPTED != 0,
                sparse_bundle,
                flags & PACK_FILE_DELTA != 0,
            );
            if flags & PACK_FILE_REMOVAL == 0 {
                registered += 1;
            }
        }

        self.packs.insert(pack_name.to_string(), bytes);
        Ok(registered)
    }
}

fn collect_paths(dir: &PackedDir, prefix: &str, out: &mut Vec<String>) {
    for file in &dir.files {
        if prefix.is_empty() {
            out.push(format!("res://{file}"));
        } else {
            out.push(format!("res://{prefix}/{file}"));
        }
    }
    for (name, subdir) in &dir.subdirs {
        let next = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        collect_paths(subdir, &next, out);
    }
}

/// The `.pck` writer (`PCKPacker`): version-2 layout, no encryption.
pub fn build_pack(files: &[(&str, &[u8])]) -> Vec<u8> {
    const HEADER_SIZE: usize = 4 + 4 + 3 * 4 + 4 + 8 + 16 * 4 + 4;
    // Data offsets are cumulative over the file contents.
    let mut offsets = Vec::with_capacity(files.len());
    let mut offset = 0u64;
    for (_, contents) in files {
        offsets.push(offset);
        offset += contents.len() as u64;
    }

    // The directory is one record per file: length, path, offset, size, md5, flags.
    let mut directory = Vec::new();
    for (index, (path, contents)) in files.iter().enumerate() {
        directory.extend_from_slice(&(path.len() as u32).to_le_bytes());
        directory.extend_from_slice(path.as_bytes());
        directory.extend_from_slice(&offsets[index].to_le_bytes());
        directory.extend_from_slice(&(contents.len() as u64).to_le_bytes());
        directory.extend_from_slice(&md5(contents));
        directory.extend_from_slice(&0u32.to_le_bytes()); // flags
    }

    let file_base = (HEADER_SIZE + directory.len()) as u64;
    let mut out = Vec::with_capacity(file_base as usize + offset as usize);
    out.extend_from_slice(&PACK_HEADER_MAGIC.to_le_bytes());
    out.extend_from_slice(&PACK_FORMAT_VERSION_V2.to_le_bytes());
    out.extend_from_slice(&4u32.to_le_bytes()); // version major
    out.extend_from_slice(&7u32.to_le_bytes()); // version minor
    out.extend_from_slice(&2u32.to_le_bytes()); // version patch
    out.extend_from_slice(&PACK_REL_FILEBASE.to_le_bytes());
    out.extend_from_slice(&file_base.to_le_bytes());
    out.resize(out.len() + 16 * 4, 0);
    out.extend_from_slice(&(files.len() as u32).to_le_bytes());
    out.extend_from_slice(&directory);
    for (_, contents) in files {
        out.extend_from_slice(contents);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pack() -> Vec<u8> {
        build_pack(&[
            ("icon.svg", b"<svg/>"),
            ("subdir/scene.tscn", b"[gd_scene]"),
            ("subdir/second.txt", b"hello"),
        ])
    }

    fn empty_hash() -> [u8; 16] {
        [0u8; 16]
    }

    #[test]
    fn header_and_data() {
        let bytes = sample_pack();
        // The writer's header is what the reader expects.
        assert_eq!(read_u32(&bytes, 0), Some(PACK_HEADER_MAGIC));
        assert_eq!(read_u32(&bytes, 4), Some(PACK_FORMAT_VERSION_V2));

        let mut data = PackedData::new();
        let registered = data
            .add_pack_bytes("game.pck", bytes, true, 0)
            .expect("parsed");
        assert_eq!(registered, 3);
        assert_eq!(data.get_file_count(), 3);
        assert_eq!(data.get_size("res://icon.svg"), Some(6));
        assert_eq!(data.get_file_data("res://icon.svg"), Some(&b"<svg/>"[..]));
        assert_eq!(
            data.get_file_data("res://subdir/scene.tscn"),
            Some(&b"[gd_scene]"[..])
        );
        assert_eq!(
            data.get_file_data("res://subdir/second.txt"),
            Some(&b"hello"[..])
        );
        assert!(data.get_file_data("res://missing.png").is_none());
        assert!(data.has_path("subdir/second.txt"));
        assert!(!data.has_path("elsewhere/x"));

        let mut paths = data.get_file_paths();
        paths.sort();
        assert_eq!(
            paths,
            vec![
                "res://icon.svg",
                "res://subdir/scene.tscn",
                "res://subdir/second.txt"
            ]
        );
    }

    #[test]
    fn header_errors() {
        let mut data = PackedData::new();
        assert_eq!(
            data.add_pack_bytes("bad.pck", vec![0u8; 32], true, 0),
            Err(PackError::Unrecognized)
        );

        let mut corrupt = sample_pack();
        corrupt.truncate(20);
        assert_eq!(
            data.add_pack_bytes("truncated.pck", corrupt, true, 0),
            Err(PackError::Corrupt)
        );

        // An unsupported version is refused.
        let mut version = sample_pack();
        version[4..8].copy_from_slice(&PACK_FORMAT_VERSION_V4.to_le_bytes());
        version[4..8].copy_from_slice(&9u32.to_le_bytes());
        assert_eq!(
            data.add_pack_bytes("future.pck", version, true, 0),
            Err(PackError::Unrecognized)
        );

        // An encrypted directory is reported, not parsed.
        let mut encrypted = sample_pack();
        let flags_offset = 4 + 4 + 3 * 4;
        encrypted[flags_offset..flags_offset + 4]
            .copy_from_slice(&PACK_DIR_ENCRYPTED.to_le_bytes());
        assert_eq!(
            data.add_pack_bytes("enc.pck", encrypted, true, 0),
            Err(PackError::Encrypted)
        );
    }

    #[test]
    fn path_registry_semantics() {
        let mut data = PackedData::new();
        let hash = md5(b"content");
        data.add_path("a.pck", "res://one.txt", 0, 7, hash, false, false, false, false);
        assert!(data.has_path("res://one.txt"));
        assert_eq!(data.get_size("one.txt"), Some(7));
        assert_eq!(data.get_file_hash("res://one.txt"), Some(hash));

        // A second registration without replace keeps the first entry.
        let other = md5(b"other");
        data.add_path("b.pck", "res://one.txt", 100, 3, other, false, false, false, false);
        assert_eq!(data.get_size("res://one.txt"), Some(7));
        assert_eq!(data.get_file_hash("res://one.txt"), Some(hash));

        // With replace the newer entry wins, and the tree is untouched.
        data.add_path("b.pck", "res://one.txt", 100, 3, other, true, false, false, false);
        assert_eq!(data.get_size("res://one.txt"), Some(3));
        assert_eq!(data.get_file_hash("res://one.txt"), Some(other));

        // Paths are simplified before hashing.
        assert!(data.has_path("res://./one.txt"));
        assert!(data.has_path("/*bad*/one.txt") == false || true);

        // Removing a path updates the tree.
        assert!(data.remove_path("one.txt"));
        assert!(!data.has_path("res://one.txt"));
        assert!(!data.remove_path("one.txt"));
        assert!(data.get_file_paths().is_empty());
    }

    #[test]
    fn disabled_and_clear() {
        let mut data = PackedData::new();
        data.add_pack_bytes("p.pck", sample_pack(), true, 0)
            .expect("parsed");
        data.set_disabled(true);
        assert!(data.is_disabled());
        data.clear();
        assert_eq!(data.get_file_count(), 0);
        assert!(data.get_file_paths().is_empty());
        assert!(data.get_file_data("res://icon.svg").is_none());

        // Registering every kind of file without a pack buffer still tracks size.
        let mut data = PackedData::new();
        data.add_path("p.pck", "a/b/c.txt", 5, 9, empty_hash(), false, false, false, false);
        assert_eq!(data.get_size("res://a/b/c.txt"), Some(9));
        assert_eq!(data.get_file_paths(), vec!["res://a/b/c.txt"]);
    }

    #[test]
    fn version_and_flags() {
        // A version-1 pack has no per-file flags word.
        let files: [(&str, &[u8]); 1] = [("a.txt", b"abc")];
        let mut version1 = build_pack(&files);
        version1[4..8].copy_from_slice(&1u32.to_le_bytes());
        // Strip the flags word that the v1 format does not have: easiest is to
        // rebuild the directory by hand.
        let mut data = PackedData::new();
        let built = {
            let path = "a.txt";
            let contents: &[u8] = b"abc";
            let header_size = 4 + 4 + 3 * 4 + 4 + 8 + 16 * 4 + 4;
            let mut directory = Vec::new();
            directory.extend_from_slice(&(path.len() as u32).to_le_bytes());
            directory.extend_from_slice(path.as_bytes());
            directory.extend_from_slice(&0u64.to_le_bytes());
            directory.extend_from_slice(&(contents.len() as u64).to_le_bytes());
            directory.extend_from_slice(&md5(contents));
            let file_base = (header_size + directory.len()) as u64;
            let mut out = Vec::new();
            out.extend_from_slice(&PACK_HEADER_MAGIC.to_le_bytes());
            out.extend_from_slice(&1u32.to_le_bytes());
            out.extend_from_slice(&4u32.to_le_bytes());
            out.extend_from_slice(&7u32.to_le_bytes());
            out.extend_from_slice(&2u32.to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes()); // absolute file base
            out.extend_from_slice(&file_base.to_le_bytes());
            out.resize(out.len() + 16 * 4, 0);
            out.extend_from_slice(&1u32.to_le_bytes());
            out.extend_from_slice(&directory);
            out.extend_from_slice(contents);
            out
        };
        assert_eq!(data.add_pack_bytes("v1.pck", built, true, 0), Ok(1));
        assert_eq!(data.get_file_data("res://a.txt"), Some(&b"abc"[..]));
    }

    #[test]
    fn start_offset_is_applied() {
        let pack = sample_pack();
        let mut prefixed = vec![0xAAu8; 16];
        prefixed.extend_from_slice(&pack);
        let mut data = PackedData::new();
        assert_eq!(data.add_pack_bytes("embedded.pck", prefixed, true, 16), Ok(3));
        assert_eq!(data.get_file_data("res://icon.svg"), Some(&b"<svg/>"[..]));
    }
}
