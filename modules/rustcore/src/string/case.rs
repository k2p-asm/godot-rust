//! `core/string/ustring.cpp`: case conversion (`to_snake_case`,
//! `to_camel_case`, `to_pascal_case`, `to_kebab_case`, `capitalize`).
//!
//! All of them go through `_separate_compound_words`, ported here operation by
//! operation: the string is split at lower→upper (`aA`), digit/upper→upper→lower
//! (`AAa`, `2Aa`), digit→lower→lower (`2aa`) and letter→digit (`A2`, `a2`)
//! boundaries, then every whitespace, underscore and hyphen becomes a space and
//! the result is lowercased.
//!
//! `capitalize` upper-cases the first character of each word and joins them with
//! single spaces; `to_camel_case` lower-cases the first word's initial (already
//! lowercase) and upper-cases the rest; `to_pascal_case` upper-cases all of them;
//! snake/kebab only replace the separating space.
//!
//! Unicode case classification uses Rust's `char::is_uppercase`/`is_lowercase`,
//! which follows the Unicode tables like the engine; the engine's ASCII fast
//! paths give the same answers for ASCII input.

/// `_separate_compound_words`.
pub fn separate_compound_words(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    if chars.is_empty() {
        return String::new();
    }

    let mut start_index = 0usize;
    let mut out = String::new();

    let mut prev_upper = chars[0].is_uppercase();
    let mut prev_lower = chars[0].is_lowercase();
    let mut prev_digit = chars[0].is_ascii_digit();

    for i in 1..chars.len() {
        let curr = chars[i];
        let curr_upper = curr.is_uppercase();
        let curr_lower = curr.is_lowercase();
        let curr_digit = curr.is_ascii_digit();
        let next_lower = chars.get(i + 1).map(|c| c.is_lowercase()).unwrap_or(false);

        let cond_a = prev_lower && curr_upper;
        let cond_b = (prev_upper || prev_digit) && curr_upper && next_lower;
        let cond_c = prev_digit && curr_lower && next_lower;
        let cond_d = (prev_upper || prev_lower) && curr_digit;

        if cond_a || cond_b || cond_c || cond_d {
            out.extend(&chars[start_index..i]);
            out.push(' ');
            start_index = i;
        }

        prev_upper = curr_upper;
        prev_lower = curr_lower;
        prev_digit = curr_digit;
    }

    out.extend(&chars[start_index..]);

    out.chars()
        .map(|c| {
            if c.is_whitespace() || c == '_' || c == '-' {
                ' '
            } else {
                c
            }
        })
        .collect::<String>()
        .to_lowercase()
}

/// `String::capitalize`.
pub fn capitalize(input: &str) -> String {
    let words = separate_compound_words(input);
    let words = words.trim_matches(|c: char| c.is_whitespace());
    let mut ret = String::new();
    let mut first = true;
    for slice in words.split(' ') {
        if slice.is_empty() {
            continue;
        }
        if !first {
            ret.push(' ');
        }
        first = false;
        let mut chars = slice.chars();
        if let Some(c) = chars.next() {
            for upper in c.to_uppercase() {
                ret.push(upper);
            }
            ret.extend(chars);
        }
    }
    ret
}

/// `String::to_camel_case`.
pub fn to_camel_case(input: &str) -> String {
    let words = separate_compound_words(input);
    let words = words.trim_matches(|c: char| c.is_whitespace());
    let mut ret = String::new();
    let mut first = true;
    for slice in words.split(' ') {
        if slice.is_empty() {
            continue;
        }
        let mut chars = slice.chars();
        if let Some(c) = chars.next() {
            if first {
                // The words are already lowercase, but keep the engine's step.
                for lower in c.to_lowercase() {
                    ret.push(lower);
                }
                first = false;
            } else {
                for upper in c.to_uppercase() {
                    ret.push(upper);
                }
            }
            ret.extend(chars);
        }
    }
    ret
}

/// `String::to_pascal_case`.
pub fn to_pascal_case(input: &str) -> String {
    let words = separate_compound_words(input);
    let words = words.trim_matches(|c: char| c.is_whitespace());
    let mut ret = String::new();
    for slice in words.split(' ') {
        if slice.is_empty() {
            continue;
        }
        let mut chars = slice.chars();
        if let Some(c) = chars.next() {
            for upper in c.to_uppercase() {
                ret.push(upper);
            }
            ret.extend(chars);
        }
    }
    ret
}

/// `String::to_snake_case`.
pub fn to_snake_case(input: &str) -> String {
    separate_compound_words(input).replace(' ', "_")
}

/// `String::to_kebab_case`.
pub fn to_kebab_case(input: &str) -> String {
    separate_compound_words(input).replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separation_boundaries() {
        assert_eq!(separate_compound_words("HelloWorld"), "hello world");
        assert_eq!(separate_compound_words("myURL"), "my url");
        assert_eq!(separate_compound_words("2DAxis"), "2d axis");
        // A letter followed by a digit splits before the digit.
        assert_eq!(separate_compound_words("A2B"), "a 2b");
        assert_eq!(separate_compound_words("2aa"), "2 aa");
        assert_eq!(separate_compound_words(""), "");
        assert_eq!(separate_compound_words("already"), "already");
    }

    #[test]
    fn separators_become_spaces() {
        assert_eq!(
            separate_compound_words("hello_world-foo\tbar"),
            "hello world foo bar"
        );
        assert_eq!(separate_compound_words("A_B-C"), "a b c");
        // Existing spaces survive as spaces.
        assert_eq!(separate_compound_words("Hello World"), "hello world");
    }

    #[test]
    fn snake_and_kebab() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("myURL"), "my_url");
        assert_eq!(to_snake_case("2DAxis"), "2d_axis");
        // The letter→digit rule splits before the digit.
        assert_eq!(to_snake_case("Node2D"), "node_2d");
        assert_eq!(to_snake_case(""), "");
        assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
    }

    #[test]
    fn capitalize_and_pascal() {
        assert_eq!(capitalize("hello world"), "Hello World");
        assert_eq!(capitalize("helloWorld"), "Hello World");
        assert_eq!(capitalize("  hello_world  "), "Hello World");
        assert_eq!(capitalize(""), "");
        assert_eq!(to_pascal_case("hello world"), "HelloWorld");
        assert_eq!(to_pascal_case("helloWorld"), "HelloWorld");
        assert_eq!(to_pascal_case(""), "");
    }

    #[test]
    fn camel_case_lowercases_the_first_word() {
        assert_eq!(to_camel_case("hello world"), "helloWorld");
        assert_eq!(to_camel_case("HelloWorld"), "helloWorld");
        assert_eq!(to_camel_case("myURL"), "myUrl");
        assert_eq!(to_camel_case(""), "");
        // A single word keeps its lowercase form.
        assert_eq!(to_camel_case("Already"), "already");
    }

    #[test]
    fn all_caps_and_acronyms() {
        assert_eq!(to_snake_case("HTTPRequest"), "http_request");
        assert_eq!(to_pascal_case("HTTP_REQUEST"), "HttpRequest");
        assert_eq!(to_camel_case("HTTP_REQUEST"), "httpRequest");
        assert_eq!(capitalize("HTTP_REQUEST"), "Http Request");
    }
}
