//! `core/string/ustring.cpp`: slice splitting helpers.
//!
//! Ported with the engine's exact edge cases:
//!
//! - `get_slice_count` is 0 for an empty text *or* an empty splitter, otherwise
//!   one plus the number of (overlapping-free) occurrences;
//! - `get_slice` returns the whole text when the splitter does not occur, and an
//!   empty string for a negative index or a missing splitter;
//! - `get_slicec` splits on a single character, returning empty past the last
//!   slice;
//! - `split` honours `allow_empty` and `maxsplit` (a positive limit puts the rest
//!   of the string into the final element);
//! - `split_floats` parses each slice with the engine's `to_float`.
//!
//! Indices are character indices, like `String`'s.

use crate::string::gstring::GoString;

fn char_len(text: &str) -> usize {
    text.chars().count()
}

/// Character-index find, returning a char index.
fn find_from(text: &[char], needle: &[char], from: usize) -> Option<usize> {
    if needle.is_empty() || text.len() < needle.len() || from > text.len() - needle.len() {
        return None;
    }
    (from..=text.len() - needle.len()).find(|&i| text[i..i + needle.len()] == *needle)
}

fn slice_chars(text: &[char], from: usize, len: isize) -> String {
    if len < 0 {
        return text.get(from..).map(|s| s.iter().collect()).unwrap_or_default();
    }
    text.iter()
        .skip(from)
        .take(len as usize)
        .collect::<String>()
}

/// `String::get_slice_count`.
pub fn get_slice_count(text: &str, splitter: &str) -> usize {
    if text.is_empty() || splitter.is_empty() {
        return 0;
    }
    let text_chars: Vec<char> = text.chars().collect();
    let splitter_chars: Vec<char> = splitter.chars().collect();
    let mut pos = 0usize;
    let mut slices = 1usize;
    while let Some(found) = find_from(&text_chars, &splitter_chars, pos) {
        slices += 1;
        pos = found + splitter_chars.len();
    }
    slices
}

/// `String::get_slice`.
pub fn get_slice(text: &str, splitter: &str, slice: i32) -> String {
    if text.is_empty() || splitter.is_empty() || slice < 0 {
        return String::new();
    }
    let text_chars: Vec<char> = text.chars().collect();
    let splitter_chars: Vec<char> = splitter.chars().collect();
    if find_from(&text_chars, &splitter_chars, 0).is_none() {
        return text.to_string();
    }

    let length = text_chars.len();
    let mut pos = 0usize;
    let mut prev_pos = 0usize;
    let mut index = 0i32;
    loop {
        let found = find_from(&text_chars, &splitter_chars, pos).unwrap_or(length);
        let from = prev_pos;
        if slice == index {
            return slice_chars(&text_chars, from, (found - from) as isize);
        }
        if found == length {
            break;
        }
        pos = found + splitter_chars.len();
        prev_pos = pos;
        index += 1;
    }
    String::new()
}

/// `String::get_slicec`.
pub fn get_slicec(text: &str, splitter: char, slice: i32) -> String {
    if text.is_empty() || slice < 0 {
        return String::new();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;
    let mut prev = 0usize;
    let mut count = 0i32;
    loop {
        let current = chars.get(i).copied().unwrap_or('\0');
        if current == '\0' || current == splitter {
            if slice == count {
                return slice_chars(&chars, prev, (i - prev) as isize);
            } else if current == '\0' {
                return String::new();
            }
            count += 1;
            prev = i + 1;
        }
        i += 1;
    }
}

/// `String::split`.
pub fn split(text: &str, splitter: &str, allow_empty: bool, maxsplit: i32) -> Vec<String> {
    let mut result = Vec::new();
    if text.is_empty() {
        if allow_empty {
            result.push(String::new());
        }
        return result;
    }

    let chars: Vec<char> = text.chars().collect();
    let length = chars.len();
    let mut from = 0usize;
    let splitter_chars: Vec<char> = splitter.chars().collect();

    loop {
        let end = if splitter_chars.is_empty() {
            (from + 1).min(length)
        } else {
            find_from(&chars, &splitter_chars, from).unwrap_or(length)
        };

        if allow_empty || end > from {
            if maxsplit <= 0 {
                result.push(slice_chars(&chars, from, (end - from) as isize));
            } else if maxsplit as usize == result.len() {
                result.push(slice_chars(&chars, from, (length - from) as isize));
                break;
            } else {
                result.push(slice_chars(&chars, from, (end - from) as isize));
            }
        }

        if end == length {
            break;
        }
        from = end + splitter_chars.len();
    }

    result
}

/// `String::split_floats`.
pub fn split_floats(text: &str, splitter: &str, allow_empty: bool) -> Vec<f64> {
    let mut result = Vec::new();
    if text.is_empty() {
        return result;
    }
    for part in split(text, splitter, allow_empty, 0) {
        result.push(GoString::from_str(&part).to_float());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_counts() {
        assert_eq!(get_slice_count("", ","), 0);
        assert_eq!(get_slice_count("a,b", ""), 0, "empty splitter yields 0");
        assert_eq!(get_slice_count("a", ","), 1);
        assert_eq!(get_slice_count("a,b,c", ","), 3);
        assert_eq!(get_slice_count("a,,b", ","), 3);
        assert_eq!(get_slice_count("a::b::c", "::"), 3);
        // Trailing separators still add a slice.
        assert_eq!(get_slice_count("a,", ","), 2);
    }

    #[test]
    fn slices() {
        assert_eq!(get_slice("a,b,c", ",", 0), "a");
        assert_eq!(get_slice("a,b,c", ",", 1), "b");
        assert_eq!(get_slice("a,b,c", ",", 2), "c");
        assert_eq!(get_slice("a,b,c", ",", 3), "");
        assert_eq!(get_slice("a,b,c", ",", -1), "");
        assert_eq!(get_slice("", ",", 0), "");
        assert_eq!(get_slice("abc", ",", 0), "abc", "no splitter = whole text");
        assert_eq!(get_slice("a::b", "::", 1), "b");
        // Empty slices are preserved.
        assert_eq!(get_slice("a,,c", ",", 1), "");
    }

    #[test]
    fn char_slices() {
        assert_eq!(get_slicec("a/b/c", '/', 0), "a");
        assert_eq!(get_slicec("a/b/c", '/', 1), "b");
        assert_eq!(get_slicec("a/b/c", '/', 2), "c");
        assert_eq!(get_slicec("a/b/c", '/', 3), "");
        assert_eq!(get_slicec("", '/', 0), "");
        assert_eq!(get_slicec("abc", '/', 0), "abc");
    }

    #[test]
    fn splitting() {
        assert_eq!(split("a,b,c", ",", true, 0), vec!["a", "b", "c"]);
        assert_eq!(split("a,,c", ",", false, 0), vec!["a", "c"]);
        assert_eq!(split("a,,c", ",", true, 0), vec!["a", "", "c"]);
        assert_eq!(split("a", ",", true, 0), vec!["a"]);
        assert!(split("", ",", false, 0).is_empty());
        assert_eq!(split("", ",", true, 0), vec![""]);

        // A positive limit keeps the rest in the last element.
        assert_eq!(split("a,b,c,d", ",", true, 2), vec!["a", "b", "c,d"]);
        assert_eq!(split("a,b,c", ",", true, 3), vec!["a", "b", "c"]);

        // An empty splitter walks characters.
        assert_eq!(split("abc", "", true, 0), vec!["a", "b", "c"]);
    }

    #[test]
    fn float_splitting() {
        assert_eq!(split_floats("1,2.5,-3", ",", true), vec![1.0, 2.5, -3.0]);
        assert_eq!(split_floats("1,,3", ",", false), vec![1.0, 3.0]);
        assert!(split_floats("", ",", true).is_empty());
        // Parsing follows String::to_float (a leading numeric prefix wins).
        assert_eq!(split_floats("2x,4", ",", true), vec![2.0, 4.0]);
    }

    #[test]
    fn unicode_indices() {
        assert_eq!(get_slice("ñ,é,ü", ",", 1), "é");
        assert_eq!(get_slice_count("ñ,é,ü", ","), 3);
        assert_eq!(get_slicec("ñ|é", '|', 1), "é");
    }
}
