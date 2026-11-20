//! `core/io/file_access_memory.*`: the named in-memory file registry.
//!
//! The engine exposes a `FileAccessMemory` handle with a static
//! `name -> bytes` map. This port models the same registry directly on
//! [`MemoryFileSystem`], keeping a read/write cursor per open path:
//!
//! - `register_file` / `file_exists` / `cleanup` for the registry;
//! - `open` (a missing file fails, like `ERR_FILE_NOT_FOUND`) and `close`;
//! - `seek` (rejects positions past the end), `seek_end`, `get_position`,
//!   `get_length`, `eof_reached`;
//! - `get_buffer` reads forward (clamped to the end) and `store_buffer` writes
//!   at the cursor, growing the file as needed.
//!
//! `ProjectSettings::globalize_path` normalisation is not ported: names are
//! used as given.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryFileError {
    /// `ERR_FILE_NOT_FOUND`.
    NotFound,
    /// `ERR_FILE_CANT_OPEN` / invalid position.
    InvalidPosition,
}

#[derive(Clone, Debug, Default)]
pub struct MemoryFileSystem {
    files: BTreeMap<String, Vec<u8>>,
    positions: BTreeMap<String, usize>,
}

impl MemoryFileSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// `register_file`: replaces any previous contents.
    pub fn register_file(&mut self, name: &str, data: Vec<u8>) {
        self.files.insert(name.to_string(), data);
        self.positions.insert(name.to_string(), 0);
    }

    pub fn file_exists(&self, name: &str) -> bool {
        self.files.contains_key(name)
    }

    /// `cleanup`: drops the registry.
    pub fn cleanup(&mut self) {
        self.files.clear();
        self.positions.clear();
    }

    /// `open_internal`: fails when the name is not registered.
    pub fn open(&mut self, name: &str) -> Result<(), MemoryFileError> {
        if !self.files.contains_key(name) {
            return Err(MemoryFileError::NotFound);
        }
        self.positions.insert(name.to_string(), 0);
        Ok(())
    }

    pub fn close(&mut self, name: &str) {
        self.positions.remove(name);
    }

    pub fn is_open(&self, name: &str) -> bool {
        self.positions.contains_key(name)
    }

    /// `seek`: positions past the end are rejected.
    pub fn seek(&mut self, name: &str, position: usize) -> Result<(), MemoryFileError> {
        let length = self.files.get(name).map(|data| data.len()).unwrap_or(0);
        if !self.files.contains_key(name) || position > length {
            return Err(MemoryFileError::InvalidPosition);
        }
        self.positions.insert(name.to_string(), position);
        Ok(())
    }

    /// `seek_end`.
    pub fn seek_end(&mut self, name: &str, offset: isize) -> Result<(), MemoryFileError> {
        let length = match self.files.get(name) {
            Some(data) => data.len() as isize,
            None => return Err(MemoryFileError::InvalidPosition),
        };
        let target = length + offset;
        if target < 0 {
            return Err(MemoryFileError::InvalidPosition);
        }
        self.seek(name, target as usize)
    }

    pub fn get_position(&self, name: &str) -> u64 {
        self.positions.get(name).copied().unwrap_or(0) as u64
    }

    pub fn get_length(&self, name: &str) -> u64 {
        self.files.get(name).map(|data| data.len() as u64).unwrap_or(0)
    }

    pub fn eof_reached(&self, name: &str) -> bool {
        self.get_position(name) as usize >= self.get_length(name) as usize
    }

    /// `get_buffer`: reads up to `count` bytes and advances the cursor.
    pub fn get_buffer(&mut self, name: &str, count: usize) -> Option<Vec<u8>> {
        let data = self.files.get(name)?;
        let position = *self.positions.get(name)?;
        let end = (position + count).min(data.len());
        let chunk = data[position..end].to_vec();
        self.positions.insert(name.to_string(), end);
        Some(chunk)
    }

    /// `store_buffer`: writes at the cursor, growing the file, and advances.
    pub fn store_buffer(&mut self, name: &str, bytes: &[u8]) -> Option<usize> {
        let position = *self.positions.get(name)?;
        let data = self.files.get_mut(name)?;
        if data.len() < position + bytes.len() {
            data.resize(position + bytes.len(), 0);
        }
        data[position..position + bytes.len()].copy_from_slice(bytes);
        self.positions.insert(name.to_string(), position + bytes.len());
        Some(bytes.len())
    }

    /// The raw file contents.
    pub fn get_data(&self, name: &str) -> Option<&[u8]> {
        self.files.get(name).map(|data| data.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_and_open() {
        let mut fs = MemoryFileSystem::new();
        assert!(!fs.file_exists("a"));
        assert_eq!(fs.open("a"), Err(MemoryFileError::NotFound));
        fs.register_file("a", vec![1, 2, 3, 4]);
        assert!(fs.file_exists("a"));
        assert_eq!(fs.open("a"), Ok(()));
        assert!(fs.is_open("a"));
        assert_eq!(fs.get_length("a"), 4);
        assert_eq!(fs.get_position("a"), 0);
        assert!(!fs.eof_reached("a"));

        fs.close("a");
        assert!(!fs.is_open("a"));
        fs.cleanup();
        assert!(!fs.file_exists("a"));
    }

    #[test]
    fn reading_and_seeking() {
        let mut fs = MemoryFileSystem::new();
        fs.register_file("f", vec![10, 20, 30, 40]);
        fs.open("f").unwrap();
        assert_eq!(fs.get_buffer("f", 2), Some(vec![10, 20]));
        assert_eq!(fs.get_position("f"), 2);
        assert_eq!(fs.get_buffer("f", 100), Some(vec![30, 40]));
        assert_eq!(fs.get_position("f"), 4);
        assert!(fs.eof_reached("f"));
        assert_eq!(fs.get_buffer("f", 1), Some(Vec::new()));

        assert!(fs.seek("f", 1).is_ok());
        assert_eq!(fs.get_buffer("f", 1), Some(vec![20]));
        assert_eq!(fs.seek("f", 5), Err(MemoryFileError::InvalidPosition));
        // Seeking to the very end is allowed (that is where `seek_end(0)` lands).
        assert!(fs.seek_end("f", 0).is_ok());
        assert_eq!(fs.get_position("f"), 4);
        assert!(fs.seek_end("f", -4).is_ok());
        assert_eq!(fs.get_position("f"), 0);
        assert_eq!(fs.seek_end("f", -5), Err(MemoryFileError::InvalidPosition));
    }

    #[test]
    fn writing_grows_and_overwrites() {
        let mut fs = MemoryFileSystem::new();
        fs.register_file("w", vec![1, 2, 3]);
        fs.open("w").unwrap();
        fs.seek_end("w", 0).unwrap(); // position = 3 is allowed (== length)
        assert_eq!(fs.store_buffer("w", &[4, 5]), Some(2));
        assert_eq!(fs.get_length("w"), 5);
        assert_eq!(fs.get_data("w"), Some(&[1, 2, 3, 4, 5][..]));

        fs.seek("w", 1).unwrap();
        assert_eq!(fs.store_buffer("w", &[9, 9]), Some(2));
        assert_eq!(fs.get_data("w"), Some(&[1, 9, 9, 4, 5][..]));

        // Appending at the end is allowed; seeking past the end is not.
        fs.seek_end("w", 0).unwrap();
        assert_eq!(fs.store_buffer("w", &[7]), Some(1));
        assert_eq!(fs.get_data("w"), Some(&[1, 9, 9, 4, 5, 7][..]));
        assert_eq!(fs.seek_end("w", 1), Err(MemoryFileError::InvalidPosition));
    }
}
