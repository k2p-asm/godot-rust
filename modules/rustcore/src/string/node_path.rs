//! `core/string/node_path.*`: node paths (`NodePath`).
//!
//! Ported from `node_path.cpp`: parsing (`res://`-style absolute paths, `/`
//! separators, `:` subnames, rejected empty subname components), the name and
//! subname accessors, the concatenated forms and `as_string`, `slice` with the
//! engine's clamping, `rel_path_to` (common-parent walk with `..` padding),
//! `get_as_property_path`, `prepend_period`, `simplify`/`simplified` (`.` removal
//! and `..` cancellation, keeping a lone `.`), and equality (absolute flag, names
//! and subnames). The reference-counted shared data is replaced by owned vectors;
//! the hash cache is not needed because the port derives `Hash`.
//!
//! Invalid paths return `None` instead of logging `ERR_FAIL_MSG`.

use crate::string::string_name::StringName;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct NodePath {
    path: Vec<StringName>,
    subpath: Vec<StringName>,
    absolute: bool,
    /// The engine leaves `data` null for an empty path; the flag models that.
    empty: bool,
}

impl NodePath {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            subpath: Vec::new(),
            absolute: false,
            empty: true,
        }
    }

    /// `NodePath(Vector<StringName>, Vector<StringName>, bool)`.
    pub fn from_names(names: Vec<StringName>, subnames: Vec<StringName>, absolute: bool) -> Self {
        let empty = names.is_empty() && subnames.is_empty() && !absolute;
        Self {
            path: names,
            subpath: subnames,
            absolute,
            empty,
        }
    }

    /// `NodePath(const String &)`; `None` mirrors the engine's parse failure.
    pub fn parse(text: &str) -> Option<Self> {
        if text.is_empty() {
            return Some(Self::new());
        }

        let mut path_text = text;
        let mut subpath: Vec<StringName> = Vec::new();

        if let Some(colon) = path_text.find(':') {
            let tail = &path_text[colon + 1..];
            // A single trailing ':' (end-of-path) is allowed; any other empty
            // component is a parse failure.
            let parts: Vec<&str> = tail.split(':').collect();
            let mut used = parts.len();
            if used > 0 && parts[used - 1].is_empty() {
                used -= 1;
            }
            for part in &parts[..used] {
                if part.is_empty() {
                    return None;
                }
                subpath.push(StringName::new(part));
            }
            path_text = &path_text[..colon];
        }

        let absolute = path_text.starts_with('/');
        let body = if absolute { &path_text[1..] } else { path_text };
        let mut names: Vec<StringName> = Vec::new();
        for part in body.split('/') {
            if part.is_empty() {
                continue;
            }
            names.push(StringName::new(part));
        }

        if names.is_empty() && !absolute && subpath.is_empty() {
            return Some(Self::new());
        }

        Some(Self {
            path: names,
            subpath,
            absolute,
            empty: false,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn is_absolute(&self) -> bool {
        self.absolute
    }

    pub fn get_name_count(&self) -> usize {
        self.path.len()
    }

    /// `ERR_FAIL_INDEX` returns an empty name.
    pub fn get_name(&self, index: usize) -> StringName {
        self.path
            .get(index)
            .copied()
            .unwrap_or_else(|| StringName::new(""))
    }

    pub fn get_subname_count(&self) -> usize {
        self.subpath.len()
    }

    pub fn get_subname(&self, index: usize) -> StringName {
        self.subpath
            .get(index)
            .copied()
            .unwrap_or_else(|| StringName::new(""))
    }

    pub fn get_total_name_count(&self) -> usize {
        self.path.len() + self.subpath.len()
    }

    pub fn get_names(&self) -> &[StringName] {
        &self.path
    }

    pub fn get_subnames(&self) -> &[StringName] {
        &self.subpath
    }

    pub fn get_concatenated_names(&self) -> String {
        self.path
            .iter()
            .map(|name| name.as_string())
            .collect::<Vec<_>>()
            .join("/")
    }

    pub fn get_concatenated_subnames(&self) -> String {
        self.subpath
            .iter()
            .map(|name| name.as_string())
            .collect::<Vec<_>>()
            .join(":")
    }

    /// `operator String`.
    pub fn as_string(&self) -> String {
        if self.empty {
            return String::new();
        }
        let mut out = String::new();
        if self.absolute {
            out.push('/');
        }
        out.push_str(&self.get_concatenated_names());
        let subnames = self.get_concatenated_subnames();
        if !subnames.is_empty() {
            out.push(':');
            out.push_str(&subnames);
        }
        out
    }

    /// `slice(begin, end)`: negative indices count from the end and both ends
    /// are clamped to the total name count.
    pub fn slice(&self, begin: i32, end: i32) -> NodePath {
        let name_count = self.get_name_count() as i32;
        let total_count = self.get_total_name_count() as i32;
        let clamp = |value: i32| value.clamp(-total_count, total_count);
        let mut begin = clamp(begin);
        if begin < 0 {
            begin += total_count;
        }
        let mut end = clamp(end);
        if end < 0 {
            end += total_count;
        }
        let sub_begin = (begin - name_count).max(0);
        let sub_end = (end - name_count).max(0);

        let names = slice_range(&self.path, begin, end);
        let subnames = slice_range(&self.subpath, sub_begin, sub_end);
        let absolute = self.absolute && begin == 0;
        NodePath::from_names(names, subnames, absolute)
    }

    /// `rel_path_to`: both paths must be absolute.
    pub fn rel_path_to(&self, other: &NodePath) -> Option<NodePath> {
        if !self.absolute || !other.absolute {
            return None;
        }
        let src = &self.path;
        let dst = &other.path;
        let mut common = 0usize;
        while common < src.len() && common < dst.len() && src[common] == dst[common] {
            common += 1;
        }
        let common = common as i32 - 1;

        let mut relpath: Vec<StringName> = Vec::new();
        for _ in (common + 1) as usize..src.len() {
            relpath.push(StringName::new(".."));
        }
        for index in (common + 1) as usize..dst.len() {
            relpath.push(dst[index]);
        }
        if relpath.is_empty() {
            relpath.push(StringName::new("."));
        }
        Some(NodePath::from_names(
            relpath,
            other.subpath.clone(),
            false,
        ))
    }

    /// `get_as_property_path`: the whole node path becomes the first subname.
    pub fn get_as_property_path(&self) -> NodePath {
        if self.path.is_empty() {
            return self.clone();
        }
        let mut subnames = Vec::with_capacity(self.subpath.len() + 1);
        let mut initial = self.path[0].as_string();
        for name in &self.path[1..] {
            initial.push('/');
            initial.push_str(&name.as_string());
        }
        subnames.push(StringName::new(&initial));
        subnames.extend_from_slice(&self.subpath);
        NodePath::from_names(Vec::new(), subnames, false)
    }

    /// `prepend_period`.
    pub fn prepend_period(&mut self) {
        if !self.path.is_empty() && self.path[0].as_string() != "." {
            self.path.insert(0, StringName::new("."));
            self.empty = false;
        }
    }

    /// `simplify`.
    pub fn simplify(&mut self) {
        if self.empty {
            return;
        }
        let mut i = 0usize;
        while i < self.path.len() {
            if self.path.len() == 1 {
                break;
            }
            let name = self.path[i].as_string();
            if name == "." {
                self.path.remove(i);
                continue;
            }
            if i > 0 && name == ".." {
                let previous = self.path[i - 1].as_string();
                if previous != "." && previous != ".." {
                    self.path.remove(i);
                    self.path.remove(i - 1);
                    i = i.saturating_sub(2);
                    if self.path.is_empty() {
                        self.path.push(StringName::new("."));
                        break;
                    }
                    continue;
                }
            }
            i += 1;
        }
    }

    pub fn simplified(&self) -> NodePath {
        let mut copy = self.clone();
        copy.simplify();
        copy
    }
}

fn slice_range(items: &[StringName], begin: i32, end: i32) -> Vec<StringName> {
    if begin >= end {
        return Vec::new();
    }
    let start = begin.max(0) as usize;
    let stop = (end as usize).min(items.len());
    if start >= stop || start >= items.len() {
        return Vec::new();
    }
    items[start..stop].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let path = NodePath::parse("Player/Sprite2D:position").expect("parsed");
        assert!(!path.is_empty());
        assert!(!path.is_absolute());
        assert_eq!(path.get_name_count(), 2);
        assert_eq!(path.get_name(0).as_string(), "Player");
        assert_eq!(path.get_name(1).as_string(), "Sprite2D");
        assert_eq!(path.get_name(9).as_string(), "");
        assert_eq!(path.get_subname_count(), 1);
        assert_eq!(path.get_subname(0).as_string(), "position");
        assert_eq!(path.get_subname(4).as_string(), "");
        assert_eq!(path.get_total_name_count(), 3);
        assert_eq!(path.as_string(), "Player/Sprite2D:position");

        let absolute = NodePath::parse("/root/Player").expect("parsed");
        assert!(absolute.is_absolute());
        assert_eq!(absolute.as_string(), "/root/Player");

        let with_subnames = NodePath::parse("A:B:C").expect("parsed");
        assert_eq!(with_subnames.get_names().len(), 1);
        assert_eq!(with_subnames.get_subnames().len(), 2);
        assert_eq!(with_subnames.as_string(), "A:B:C");

        // Empty and trailing-slash forms.
        assert!(NodePath::parse("").expect("empty").is_empty());
        assert_eq!(
            NodePath::parse("A//B").expect("parsed").as_string(),
            "A/B"
        );
        assert_eq!(NodePath::parse("A/").expect("parsed").as_string(), "A");
        assert_eq!(NodePath::parse("A:").expect("parsed").as_string(), "A");
        // An interior empty subname is invalid.
        assert!(NodePath::parse("A::B").is_none());
        // A bare subname keeps only the subpath.
        let sub_only = NodePath::parse(":position").expect("parsed");
        assert_eq!(sub_only.get_name_count(), 0);
        assert_eq!(sub_only.as_string(), ":position");
    }

    #[test]
    fn concatenation_and_equality() {
        let path = NodePath::parse("/root/Level/Player:position:x").expect("parsed");
        assert_eq!(path.get_concatenated_names(), "root/Level/Player");
        assert_eq!(path.get_concatenated_subnames(), "position:x");
        assert_eq!(path.as_string(), "/root/Level/Player:position:x");

        let same = NodePath::parse("/root/Level/Player:position:x").expect("parsed");
        assert_eq!(path, same);
        let relative = NodePath::parse("root/Level/Player:position:x").expect("parsed");
        assert_ne!(path, relative, "the absolute flag is part of equality");
        let different = NodePath::parse("/root/Level/Player:position:y").expect("parsed");
        assert_ne!(path, different);
    }

    #[test]
    fn slicing() {
        let path = NodePath::parse("/A/B/C:prop:x").expect("parsed");
        assert_eq!(path.slice(0, 2).as_string(), "/A/B");
        assert_eq!(path.slice(1, 3).as_string(), "B/C");
        // Negative indices count over names *and* subnames, so this keeps the
        // two subnames and drops every node name (the engine's behaviour).
        assert_eq!(path.slice(-2, 100).as_string(), ":prop:x");
        // Slicing across the name/subname boundary keeps the subnames.
        assert_eq!(path.slice(2, 4).as_string(), "C:prop");
        assert!(path.slice(3, 3).is_empty());
        assert_eq!(path.slice(2, 2).get_name_count(), 0);
    }

    #[test]
    fn relative_paths_and_property_paths() {
        let base = NodePath::parse("/root/Level/Player").expect("parsed");
        let target = NodePath::parse("/root/Level/Enemy").expect("parsed");
        assert_eq!(
            base.rel_path_to(&target).expect("absolute").as_string(),
            "../Enemy"
        );
        let child = NodePath::parse("/root/Level/Player/Sprite").expect("parsed");
        assert_eq!(
            base.rel_path_to(&child).expect("absolute").as_string(),
            "Sprite"
        );
        let sibling = NodePath::parse("/root/Other").expect("parsed");
        assert_eq!(
            base.rel_path_to(&sibling).expect("absolute").as_string(),
            "../../Other"
        );
        let same = NodePath::parse("/root/Level/Player").expect("parsed");
        assert_eq!(
            base.rel_path_to(&same).expect("absolute").as_string(),
            "."
        );
        // Subnames travel with the target.
        let with_prop = NodePath::parse("/root/Level/Enemy:position").expect("parsed");
        assert_eq!(
            base.rel_path_to(&with_prop)
                .expect("absolute")
                .as_string(),
            "../Enemy:position"
        );
        // Relative paths cannot be compared.
        let relative = NodePath::parse("A").expect("parsed");
        assert!(base.rel_path_to(&relative).is_none());
        assert!(relative.rel_path_to(&base).is_none());

        let property = NodePath::parse("/root/Player:position").expect("parsed");
        assert_eq!(property.get_as_property_path().as_string(), ":root/Player:position");
    }

    #[test]
    fn simplification_and_period() {
        let mut path = NodePath::parse("A/./B").expect("parsed");
        path.simplify();
        assert_eq!(path.as_string(), "A/B");

        let mut path = NodePath::parse("A/B/../C").expect("parsed");
        path.simplify();
        assert_eq!(path.as_string(), "A/C");

        // A leading ".." is preserved.
        let mut path = NodePath::parse("../A").expect("parsed");
        path.simplify();
        assert_eq!(path.as_string(), "../A");

        // Cancelling everything leaves ".".
        let mut path = NodePath::parse("A/..").expect("parsed");
        path.simplify();
        assert_eq!(path.as_string(), ".");

        // ".." does not cancel a "." step.
        let mut path = NodePath::parse("A/./..").expect("parsed");
        path.simplify();
        assert_eq!(path.as_string(), ".");

        // prepend_period adds a leading dot once.
        let mut path = NodePath::parse("../A").expect("parsed");
        path.prepend_period();
        assert_eq!(path.as_string(), "./../A");
        path.prepend_period();
        assert_eq!(path.as_string(), "./../A");
        assert_eq!(
            NodePath::parse("A/B").expect("parsed").simplified().as_string(),
            "A/B"
        );
    }
}
