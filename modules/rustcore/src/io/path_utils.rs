//! `core/string/ustring.cpp`: path helpers (`simplify_path`, `get_base_dir`,
//! `is_network_share_path`).
//!
//! `simplify_path` keeps the engine's exact algorithm: the drive/`scheme://`
//! prefix is split off first, backslashes become slashes, repeated slashes
//! collapse, then `.` and `..` are resolved with the engine's two exceptions —
//! `res://` paths keep leading `..` (the glTF importer relies on reaching above
//! the project), and absolute non-`res://` paths drop a leading `..`.
//!
//! `get_base_dir` handles the same four prefix shapes in the same order: URL
//! scheme, Windows drive, UNC share, Unix root.

/// `is_network_share_path`: `//` or `\\\\` prefix.
pub fn is_network_share_path(path: &str) -> bool {
    path.starts_with("//") || path.starts_with("\\\\")
}

/// `is_absolute_path` as used by `simplify_path`.
fn is_absolute_path(path: &str) -> bool {
    let mut chars = path.chars();
    match chars.next() {
        None => false,
        Some(c) if c == '/' || c == '\\' => true,
        Some(_) => path.contains(":/") || path.contains(":\\"),
    }
}

fn find_from(chars: &[char], needle: &[char], from: usize) -> Option<usize> {
    if needle.is_empty() || chars.len() < needle.len() {
        return None;
    }
    (from..=chars.len() - needle.len()).find(|&i| chars[i..i + needle.len()] == *needle)
}

fn find_str(chars: &[char], needle: &str) -> Option<usize> {
    let n: Vec<char> = needle.chars().collect();
    find_from(chars, &n, 0)
}

fn collect(chars: &[char]) -> String {
    chars.iter().collect()
}

/// `String::simplify_path`.
pub fn simplify_path(path: &str) -> String {
    let original: Vec<char> = path.chars().collect();
    let mut s: Vec<char> = original.clone();
    let mut drive = String::new();
    let mut found = false;

    if let Some(p) = find_str(&original, "://") {
        if p > 0 && original[..p].iter().all(|c| c.is_ascii_alphanumeric()) {
            found = true;
            drive = collect(&original[..p + 3]);
            s = original[p + 3..].to_vec();
        }
    }

    if !found {
        if is_network_share_path(path) {
            drive = collect(&original[..2.min(original.len())]);
            s = original[2.min(original.len())..].to_vec();
        } else if matches!(original.first(), Some('/') | Some('\\')) {
            drive = collect(&original[..1]);
            s = original[1..].to_vec();
        } else {
            let p = find_str(&original, ":/").or_else(|| find_str(&original, ":\\"));
            if let Some(p) = p {
                let slash = original.iter().position(|&c| c == '/');
                let applies = matches!(slash, Some(index) if p < index);
                if applies {
                    drive = collect(&original[..p + 2]);
                    s = original[p + 2..].to_vec();
                }
            }
        }
    }

    // Backslashes are separators, and repeated separators collapse.
    s = s
        .into_iter()
        .map(|c| if c == '\\' { '/' } else { c })
        .collect();
    loop {
        let mut collapsed = Vec::with_capacity(s.len());
        for (index, &c) in s.iter().enumerate() {
            if c == '/' && index > 0 && s[index - 1] == '/' {
                continue;
            }
            collapsed.push(c);
        }
        if collapsed == s {
            break;
        }
        s = collapsed;
    }

    let mut dirs: Vec<String> = collect(&s)
        .split('/')
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect();

    let absolute_path = is_absolute_path(path) && !path.starts_with("res://");

    let mut i: isize = 0;
    while i < dirs.len() as isize {
        let d = &dirs[i as usize];
        if d == "." {
            dirs.remove(i as usize);
            i -= 1;
        } else if d == ".." {
            if i != 0 && dirs[(i - 1) as usize] != ".." {
                dirs.remove(i as usize);
                dirs.remove((i - 1) as usize);
                i -= 2;
            } else if absolute_path && i == 0 {
                dirs.remove(0);
                i -= 1;
            }
        }
        i += 1;
    }

    drive + &dirs.join("/")
}

/// `String::get_base_dir`.
pub fn get_base_dir(path: &str) -> String {
    let chars: Vec<char> = path.chars().collect();
    let mut end: usize = 0;

    // URL scheme style base.
    if let Some(basepos) = find_str(&chars, "://") {
        end = basepos + 3;
    }

    // Windows top level directory base.
    if end == 0 {
        let basepos = find_str(&chars, ":/").or_else(|| find_str(&chars, ":\\"));
        if let Some(basepos) = basepos {
            end = basepos + 2;
        }
    }

    // Windows UNC network share path.
    if end == 0 && is_network_share_path(path) {
        let sep = chars
            .iter()
            .enumerate()
            .skip(2)
            .find(|(_, &c)| c == '/' || c == '\\')
            .map(|(i, _)| i);
        if let Some(sep) = sep {
            let servpos = chars
                .iter()
                .enumerate()
                .skip(sep + 1)
                .find(|(_, &c)| c == '/' || c == '\\')
                .map(|(i, _)| i);
            if let Some(servpos) = servpos {
                end = servpos + 1;
            }
        }
    }

    // Unix root directory base.
    if end == 0 && chars.first() == Some(&'/') {
        end = 1;
    }

    let (base, rs): (String, Vec<char>) = if end != 0 {
        (collect(&chars[..end]), chars[end..].to_vec())
    } else {
        (String::new(), chars.clone())
    };

    let sep = rs.iter().rposition(|&c| c == '/' || c == '\\');
    match sep {
        None => base,
        Some(sep) => base + &collect(&rs[..sep]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplify_resolves_dots_and_duplicate_separators() {
        assert_eq!(simplify_path("res://a/./b/../c"), "res://a/c");
        assert_eq!(simplify_path("res://a//b///c"), "res://a/b/c");
        assert_eq!(simplify_path("/a//b/"), "/a/b");
        assert_eq!(simplify_path("a/b/"), "a/b");
        assert_eq!(simplify_path(""), "");
        assert_eq!(simplify_path("."), "");
        assert_eq!(simplify_path("./a"), "a");
    }

    #[test]
    fn simplify_handles_backslashes_and_drives() {
        assert_eq!(simplify_path("C:/a/./b/../c"), "C:/a/c");
        // A drive-only path keeps its separator and resolves to itself.
        assert_eq!(simplify_path("C:/"), "C:/");
        assert_eq!(simplify_path("C:\\a\\b"), "C:/a/b");
        // A single leading backslash is the drive part and keeps its form; only
        // the remainder is normalised.
        assert_eq!(simplify_path("\\a\\b"), "\\a/b");
        assert_eq!(simplify_path("//server/share/../file"), "//server/file");
    }

    #[test]
    fn simplify_dotdot_rules() {
        // Relative paths keep leading `..`.
        assert_eq!(simplify_path("a/../../b"), "../b");
        assert_eq!(simplify_path("../a"), "../a");
        // Absolute non-res paths drop leading `..`.
        assert_eq!(simplify_path("/../a"), "/a");
        // `res://` is special-cased and keeps its `..` (glTF importer case).
        assert_eq!(simplify_path("res://../a"), "res://../a");
        assert_eq!(simplify_path("res://a/../../b"), "res://../b");
        // A `..` directly after another `..` is preserved.
        assert_eq!(simplify_path("../../a"), "../../a");
    }

    #[test]
    fn network_share_detection() {
        assert!(is_network_share_path("//server/share"));
        assert!(is_network_share_path("\\\\server\\share"));
        assert!(!is_network_share_path("/server/share"));
        assert!(!is_network_share_path("res://x"));
        assert!(!is_network_share_path(""));
    }

    #[test]
    fn base_dir_prefixes() {
        assert_eq!(get_base_dir("res://a/b/c.txt"), "res://a/b");
        assert_eq!(get_base_dir("res://file.txt"), "res://");
        assert_eq!(get_base_dir("C:/a/b.txt"), "C:/a");
        assert_eq!(get_base_dir("/a.txt"), "/");
        assert_eq!(get_base_dir("file.txt"), "");
        // UNC bases keep the separator that ends the share name.
        assert_eq!(get_base_dir("//server/share/f.txt"), "//server/share/");
        assert_eq!(get_base_dir(""), "");
        // Backslash separators count too.
        assert_eq!(get_base_dir("a\\b.txt"), "a");
    }
}
