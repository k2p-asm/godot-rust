//! `core/io/dir_access.*`: directory operations over a virtual file system.
//!
//! The engine's `DirAccess` is an abstract handle over the platform's
//! directories. This port models the same surface over an in-memory tree (the
//! companion of `io/file_access_memory.rs`) so the path rules and return codes
//! are testable without touching the host:
//!
//! - `change_dir` (absolute or relative, `.`/`..`, must exist), `get_current_dir`;
//! - `make_dir` (parent must exist, fails when present), `make_dir_recursive`;
//! - `dir_exists`/`file_exists`/`exists`, `remove` (files and empty directories),
//!   `rename` (destination parent must exist and the destination must not),
//!   `copy` (files and whole directory trees);
//! - `list_dir_begin`/`list_dir_next`/`list_dir_is_dir` with sorted names, plus
//!   `get_files_at`/`get_directories_at` helpers.
//!
//! Error codes use the engine's names. Host paths, links, permissions and space
//! queries are not modelled.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirAccessError {
    Ok,
    /// `ERR_FILE_NOT_FOUND`.
    FileNotFound,
    /// `ERR_CANT_OPEN`.
    CantOpen,
    /// `ERR_CANT_CREATE`.
    CantCreate,
    /// `ERR_ALREADY_EXISTS`.
    AlreadyExists,
    /// `ERR_INVALID_PARAMETER` (e.g. removing a non-empty directory).
    InvalidParameter,
}

#[derive(Clone, Debug)]
pub struct VirtualDirAccess {
    /// Every directory except the implicit root.
    dirs: BTreeSet<String>,
    /// File path -> contents.
    files: std::collections::BTreeMap<String, Vec<u8>>,
    cwd: String,
    listing: Vec<(String, bool)>,
    listing_index: usize,
}

impl Default for VirtualDirAccess {
    fn default() -> Self {
        Self::new()
    }
}

fn normalize(path: &str) -> String {
    let simplified = crate::io::path_utils::simplify_path(path);
    if simplified.is_empty() {
        "/".to_string()
    } else {
        simplified
    }
}

fn parent_of(path: &str) -> String {
    match path.rfind('/') {
        Some(0) => "/".to_string(),
        Some(index) => path[..index].to_string(),
        None => "/".to_string(),
    }
}

fn join(base: &str, path: &str) -> String {
    if path.starts_with('/') {
        return normalize(path);
    }
    if base == "/" {
        normalize(&format!("/{path}"))
    } else {
        normalize(&format!("{base}/{path}"))
    }
}

impl VirtualDirAccess {
    pub fn new() -> Self {
        Self {
            dirs: BTreeSet::new(),
            files: std::collections::BTreeMap::new(),
            cwd: "/".to_string(),
            listing: Vec::new(),
            listing_index: 0,
        }
    }

    pub fn get_current_dir(&self) -> &str {
        &self.cwd
    }

    /// `change_dir`: the target must exist and be a directory.
    pub fn change_dir(&mut self, path: &str) -> DirAccessError {
        let target = join(&self.cwd, path);
        if !self.dir_exists(&target) {
            return DirAccessError::FileNotFound;
        }
        self.cwd = target;
        DirAccessError::Ok
    }

    pub fn dir_exists(&self, path: &str) -> bool {
        let path = join(&self.cwd, path);
        path == "/" || self.dirs.contains(&path)
    }

    pub fn file_exists(&self, path: &str) -> bool {
        self.files.contains_key(&join(&self.cwd, path))
    }

    pub fn exists(&self, path: &str) -> bool {
        self.dir_exists(path) || self.file_exists(path)
    }

    /// `make_dir`: the parent must exist and the directory must be new.
    pub fn make_dir(&mut self, path: &str) -> DirAccessError {
        let target = join(&self.cwd, path);
        if target == "/" || self.dirs.contains(&target) {
            return DirAccessError::AlreadyExists;
        }
        let parent = parent_of(&target);
        if !self.dir_exists(&parent) {
            return DirAccessError::CantCreate;
        }
        self.dirs.insert(target);
        DirAccessError::Ok
    }

    /// `make_dir_recursive`: creates every missing ancestor.
    pub fn make_dir_recursive(&mut self, path: &str) -> DirAccessError {
        let target = join(&self.cwd, path);
        if target == "/" || self.dirs.contains(&target) {
            return DirAccessError::AlreadyExists;
        }
        let mut current = String::new();
        for part in target.split('/') {
            if part.is_empty() {
                continue;
            }
            current.push('/');
            current.push_str(part);
            self.dirs.insert(current.clone());
        }
        DirAccessError::Ok
    }

    /// `remove`: files and empty directories only.
    pub fn remove(&mut self, path: &str) -> DirAccessError {
        let target = join(&self.cwd, path);
        if self.files.remove(&target).is_some() {
            return DirAccessError::Ok;
        }
        if !self.dirs.contains(&target) {
            return DirAccessError::FileNotFound;
        }
        // A directory with any child cannot be removed.
        let prefix = format!("{target}/");
        if self.dirs.iter().any(|dir| dir.starts_with(&prefix))
            || self.files.keys().any(|file| file.starts_with(&prefix))
        {
            return DirAccessError::InvalidParameter;
        }
        self.dirs.remove(&target);
        DirAccessError::Ok
    }

    /// `rename`: source must exist, the destination must not, and the
    /// destination's parent must exist.
    pub fn rename(&mut self, from: &str, to: &str) -> DirAccessError {
        let source = join(&self.cwd, from);
        let target = join(&self.cwd, to);
        if source == target {
            return DirAccessError::Ok;
        }
        if !self.exists(&source) {
            return DirAccessError::FileNotFound;
        }
        if self.exists(&target) {
            return DirAccessError::AlreadyExists;
        }
        if !self.dir_exists(&parent_of(&target)) {
            return DirAccessError::CantCreate;
        }

        if let Some(data) = self.files.remove(&source) {
            self.files.insert(target, data);
            return DirAccessError::Ok;
        }

        // Move the directory and everything below it.
        let prefix = format!("{source}/");
        let mut moved_dirs: Vec<(String, String)> = Vec::new();
        for dir in self.dirs.iter().filter(|dir| **dir == source || dir.starts_with(&prefix)) {
            let suffix = &dir[source.len()..];
            moved_dirs.push((dir.clone(), format!("{target}{suffix}")));
        }
        let mut moved_files: Vec<(String, Vec<u8>)> = Vec::new();
        let file_paths: Vec<String> = self
            .files
            .keys()
            .filter(|file| file.starts_with(&prefix))
            .cloned()
            .collect();
        for file in file_paths {
            if let Some(data) = self.files.remove(&file) {
                moved_files.push((format!("{target}{}", &file[source.len()..]), data));
            }
        }
        for (old, new) in moved_dirs {
            self.dirs.remove(&old);
            self.dirs.insert(new);
        }
        for (path, data) in moved_files {
            self.files.insert(path, data);
        }
        DirAccessError::Ok
    }

    /// `copy`: files overwrite nothing (the destination must be new), whole
    /// directory trees are copied recursively.
    pub fn copy(&mut self, from: &str, to: &str) -> DirAccessError {
        let source = join(&self.cwd, from);
        let target = join(&self.cwd, to);
        if source == target || self.exists(&target) {
            return DirAccessError::AlreadyExists;
        }
        if !self.dir_exists(&parent_of(&target)) {
            return DirAccessError::CantCreate;
        }

        if let Some(data) = self.files.get(&source).cloned() {
            self.files.insert(target, data);
            return DirAccessError::Ok;
        }
        if !self.dir_exists(&source) {
            return DirAccessError::FileNotFound;
        }

        let prefix = format!("{source}/");
        let dirs: Vec<String> = self
            .dirs
            .iter()
            .filter(|dir| **dir == source || dir.starts_with(&prefix))
            .cloned()
            .collect();
        let files: Vec<(String, Vec<u8>)> = self
            .files
            .iter()
            .filter(|(file, _)| file.starts_with(&prefix))
            .map(|(file, data)| (file.clone(), data.clone()))
            .collect();
        self.dirs.insert(target.clone());
        for dir in dirs {
            if dir == source {
                continue;
            }
            self.dirs.insert(format!("{target}{}", &dir[source.len()..]));
        }
        for (file, data) in files {
            self.files
                .insert(format!("{target}{}", &file[source.len()..]), data);
        }
        DirAccessError::Ok
    }

    /// `list_dir_begin`: captures a sorted snapshot of the current directory.
    pub fn list_dir_begin(&mut self) -> DirAccessError {
        self.listing = self.list_entries(&self.cwd.clone());
        self.listing_index = 0;
        DirAccessError::Ok
    }

    /// `list_dir_next`: `None` ends the listing.
    pub fn list_dir_next(&mut self) -> Option<String> {
        if self.listing_index >= self.listing.len() {
            return None;
        }
        let (name, _) = self.listing[self.listing_index].clone();
        self.listing_index += 1;
        Some(name)
    }

    /// `current_is_dir`.
    pub fn list_dir_is_dir(&self) -> bool {
        if self.listing_index == 0 {
            return false;
        }
        self.listing
            .get(self.listing_index - 1)
            .map(|(_, is_dir)| *is_dir)
            .unwrap_or(false)
    }

    /// Sorted `(name, is_dir)` entries of `path` (defaults to the cwd).
    pub fn list_entries(&self, path: &str) -> Vec<(String, bool)> {
        let path = normalize(path);
        if !self.dir_exists(&path) {
            return Vec::new();
        }
        let prefix = if path == "/" {
            "/".to_string()
        } else {
            format!("{path}/")
        };
        let mut entries = Vec::new();
        for dir in &self.dirs {
            if let Some(rest) = dir.strip_prefix(&prefix) {
                if !rest.is_empty() && !rest.contains('/') {
                    entries.push((rest.to_string(), true));
                }
            }
        }
        for file in self.files.keys() {
            if let Some(rest) = file.strip_prefix(&prefix) {
                if !rest.is_empty() && !rest.contains('/') {
                    entries.push((rest.to_string(), false));
                }
            }
        }
        entries.sort();
        entries
    }

    pub fn get_files_at(&self, path: &str) -> Vec<String> {
        self.list_entries(path)
            .into_iter()
            .filter(|(_, is_dir)| !is_dir)
            .map(|(name, _)| name)
            .collect()
    }

    pub fn get_directories_at(&self, path: &str) -> Vec<String> {
        self.list_entries(path)
            .into_iter()
            .filter(|(_, is_dir)| *is_dir)
            .map(|(name, _)| name)
            .collect()
    }

    pub fn get_file_size(&self, path: &str) -> Option<usize> {
        self.files.get(&join(&self.cwd, path)).map(|data| data.len())
    }

    pub fn get_file_data(&self, path: &str) -> Option<&[u8]> {
        self.files
            .get(&join(&self.cwd, path))
            .map(|data| data.as_slice())
    }

    pub fn set_file_data(&mut self, path: &str, data: Vec<u8>) -> DirAccessError {
        let path = join(&self.cwd, path);
        if !self.dir_exists(&parent_of(&path)) {
            return DirAccessError::CantCreate;
        }
        self.files.insert(path, data);
        DirAccessError::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directories_and_navigation() {
        let mut dir = VirtualDirAccess::new();
        assert_eq!(dir.get_current_dir(), "/");
        assert!(dir.dir_exists("/"));
        assert!(!dir.dir_exists("/a"));

        assert_eq!(dir.make_dir("/a"), DirAccessError::Ok);
        assert_eq!(dir.make_dir("/a"), DirAccessError::AlreadyExists);
        assert_eq!(dir.make_dir("/missing/b"), DirAccessError::CantCreate);
        assert_eq!(dir.change_dir("/a"), DirAccessError::Ok);
        assert_eq!(dir.get_current_dir(), "/a");
        assert_eq!(dir.change_dir(".."), DirAccessError::Ok);
        assert_eq!(dir.get_current_dir(), "/");
        assert_eq!(dir.change_dir("nope"), DirAccessError::FileNotFound);

        assert_eq!(dir.make_dir_recursive("/x/y/z"), DirAccessError::Ok);
        assert!(dir.dir_exists("/x") && dir.dir_exists("/x/y") && dir.dir_exists("/x/y/z"));
        assert_eq!(
            dir.make_dir_recursive("/x/y/z"),
            DirAccessError::AlreadyExists
        );
        // Relative creation resolves against the cwd.
        dir.change_dir("/x");
        assert_eq!(dir.make_dir("rel"), DirAccessError::Ok);
        assert!(dir.dir_exists("/x/rel"));
    }

    #[test]
    fn files_listing_and_removal() {
        let mut dir = VirtualDirAccess::new();
        dir.make_dir_recursive("/a/b");
        dir.make_dir_recursive("/a/c");
        dir.change_dir("/a");
        assert_eq!(dir.set_file_data("/a/file2.txt", vec![2]), DirAccessError::Ok);
        assert_eq!(dir.set_file_data("/a/file1.txt", vec![1, 1]), DirAccessError::Ok);
        dir.set_file_data("/a/b/deep.txt", vec![9]);
        assert_eq!(dir.set_file_data("/nope/x", vec![0]), DirAccessError::CantCreate);

        assert!(dir.file_exists("/a/file1.txt"));
        assert_eq!(dir.get_file_size("/a/file1.txt"), Some(2));
        assert!(dir.exists("/a/b") && dir.exists("/a/file2.txt"));
        assert!(!dir.exists("/a/none"));

        // Sorted mixing of files and directories.
        assert_eq!(
            dir.list_entries("/a"),
            vec![
                ("b".to_string(), true),
                ("c".to_string(), true),
                ("file1.txt".to_string(), false),
                ("file2.txt".to_string(), false),
            ]
        );
        assert_eq!(dir.get_files_at("/a"), vec!["file1.txt", "file2.txt"]);
        assert_eq!(dir.get_directories_at("/a"), vec!["b", "c"]);

        assert_eq!(dir.list_dir_begin(), DirAccessError::Ok);
        assert_eq!(dir.list_dir_next().as_deref(), Some("b"));
        assert!(dir.list_dir_is_dir());
        assert_eq!(dir.list_dir_next().as_deref(), Some("c"));
        assert_eq!(dir.list_dir_next().as_deref(), Some("file1.txt"));
        assert!(!dir.list_dir_is_dir());
        assert_eq!(dir.list_dir_next().as_deref(), Some("file2.txt"));
        assert_eq!(dir.list_dir_next(), None);
        assert!(!dir.list_dir_is_dir());

        // Removal: files and empty dirs only.
        assert_eq!(dir.remove("/a/b"), DirAccessError::InvalidParameter);
        assert_eq!(dir.remove("/a/b/deep.txt"), DirAccessError::Ok);
        assert_eq!(dir.remove("/a/b"), DirAccessError::Ok);
        assert_eq!(dir.remove("/a/b"), DirAccessError::FileNotFound);
        assert_eq!(dir.remove("/a/file1.txt"), DirAccessError::Ok);
        assert!(!dir.file_exists("/a/file1.txt"));
    }

    #[test]
    fn renaming_and_copying() {
        let mut dir = VirtualDirAccess::new();
        dir.make_dir_recursive("/src/inner");
        dir.set_file_data("/src/one.txt", vec![1]);
        dir.set_file_data("/src/inner/two.txt", vec![2]);

        // Rename a single file.
        assert_eq!(dir.rename("/src/one.txt", "/src/renamed.txt"), DirAccessError::Ok);
        assert!(!dir.file_exists("/src/one.txt"));
        assert_eq!(dir.get_file_data("/src/renamed.txt"), Some(&[1u8][..]));
        // Destination existing, missing source, missing parent.
        assert_eq!(
            dir.rename("/src/renamed.txt", "/src/renamed.txt"),
            DirAccessError::Ok
        );
        assert_eq!(
            dir.rename("/src/none.txt", "/src/x.txt"),
            DirAccessError::FileNotFound
        );
        assert_eq!(
            dir.rename("/src/renamed.txt", "/missing/x.txt"),
            DirAccessError::CantCreate
        );

        // Rename a whole directory.
        assert_eq!(dir.rename("/src", "/dst"), DirAccessError::Ok);
        assert!(!dir.dir_exists("/src"));
        assert!(dir.dir_exists("/dst/inner"));
        assert_eq!(dir.get_file_data("/dst/renamed.txt"), Some(&[1u8][..]));
        assert_eq!(dir.get_file_data("/dst/inner/two.txt"), Some(&[2u8][..]));

        // Copy a tree and a file.
        assert_eq!(dir.copy("/dst", "/copy"), DirAccessError::Ok);
        assert!(dir.dir_exists("/copy/inner"));
        assert_eq!(dir.get_file_data("/copy/inner/two.txt"), Some(&[2u8][..]));
        assert_eq!(dir.copy("/dst", "/copy"), DirAccessError::AlreadyExists);
        assert_eq!(
            dir.copy("/dst/renamed.txt", "/dst/third.txt"),
            DirAccessError::Ok
        );
        assert_eq!(dir.get_file_data("/dst/third.txt"), Some(&[1u8][..]));
        assert_eq!(
            dir.copy("/missing", "/dst/x"),
            DirAccessError::FileNotFound
        );
    }

    #[test]
    fn normalization_rules() {
        let mut dir = VirtualDirAccess::new();
        dir.make_dir_recursive("/a/b");
        // Redundant separators and dots simplify.
        assert!(dir.dir_exists("/a//b"));
        assert!(dir.dir_exists("/a/./b"));
        assert!(dir.dir_exists("/a/b/../b"));
        // Relative paths use the cwd.
        dir.change_dir("/a/b");
        dir.set_file_data("f.txt", vec![7]);
        assert!(dir.file_exists("/a/b/f.txt"));
        assert_eq!(dir.get_file_data("../b/f.txt"), Some(&[7u8][..]));
    }
}
