//! `core/io/plist.*`: XML property lists.
//!
//! Ported: `load_string`'s tag scanner (skips `<?xml?>`/`<!DOCTYPE>`, requires a
//! `<plist>` wrapper, keeps a stack of open containers, refuses mismatched
//! `</dict>`/`</array>`/value tags, keys values inside containers and terminates
//! only on `</plist>`), and `save_text`'s exact formatting (header, doctype,
//! `<plist version="1.0">`, one tab per depth, `<true/>`/`<false/>`, `<data>` on
//! its own indented line, and two trailing newlines).
//!
//! Values: `string`, `integer`, `real`, `true`/`false`, `data`, `date`, plus
//! `dict`/`array` containers. Integers/floats parse with the port's `to_int`/
//! `to_float`.
//!
//! Binary (`bplist00`) reading and the ASN.1 writer are not ported.

use crate::string::gstring::GoString;
use crate::string::num::num_float;

#[derive(Clone, Debug, PartialEq)]
pub enum PlistNode {
    /// `PL_NODE_TYPE_NIL`; stored as nothing.
    Nil,
    Dict(Vec<(String, PlistNode)>),
    Array(Vec<PlistNode>),
    Bool(bool),
    Int(i64),
    Real(f64),
    String(String),
    Data(String),
    Date(String),
}

impl PlistNode {
    pub fn as_dict(&self) -> Option<&[(String, PlistNode)]> {
        match self {
            PlistNode::Dict(entries) => Some(entries),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[PlistNode]> {
        match self {
            PlistNode::Array(items) => Some(items),
            _ => None,
        }
    }

    /// Dictionary lookup by key.
    pub fn get(&self, key: &str) -> Option<&PlistNode> {
        self.as_dict()?
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    pub fn get_index(&self, index: usize) -> Option<&PlistNode> {
        self.as_array()?.get(index)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            PlistNode::String(text) | PlistNode::Data(text) | PlistNode::Date(text) => Some(text),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            PlistNode::Int(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            PlistNode::Real(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PlistNode::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// `push_subnode`, with the key consumed by dictionaries and cleared.
    fn push_subnode(&mut self, key: &mut String, node: PlistNode) -> bool {
        match self {
            PlistNode::Dict(entries) => {
                entries.push((std::mem::take(key), node));
                true
            }
            PlistNode::Array(items) => {
                key.clear();
                items.push(node);
                true
            }
            _ => false,
        }
    }

    fn store_text(&self, out: &mut String, indent: u8) {
        let tabs = "\t".repeat(indent as usize);
        match self {
            PlistNode::Nil => {}
            PlistNode::Data(text) => {
                out.push_str(&tabs);
                out.push_str("<data>\n");
                out.push_str(&tabs);
                out.push_str(text);
                out.push('\n');
                out.push_str(&tabs);
                out.push_str("</data>\n");
            }
            PlistNode::Date(text) => {
                out.push_str(&tabs);
                out.push_str("<date>");
                out.push_str(text);
                out.push_str("</date>\n");
            }
            PlistNode::String(text) => {
                out.push_str(&tabs);
                out.push_str("<string>");
                out.push_str(text);
                out.push_str("</string>\n");
            }
            PlistNode::Bool(value) => {
                out.push_str(&tabs);
                out.push_str(if *value { "<true/>\n" } else { "<false/>\n" });
            }
            PlistNode::Int(value) => {
                out.push_str(&tabs);
                out.push_str("<integer>");
                out.push_str(&value.to_string());
                out.push_str("</integer>\n");
            }
            PlistNode::Real(value) => {
                out.push_str(&tabs);
                out.push_str("<real>");
                out.push_str(&num_float(*value, -1).as_string());
                out.push_str("</real>\n");
            }
            PlistNode::Array(items) => {
                out.push_str(&tabs);
                out.push_str("<array>\n");
                for item in items {
                    item.store_text(out, indent + 1);
                }
                out.push_str(&tabs);
                out.push_str("</array>\n");
            }
            PlistNode::Dict(entries) => {
                out.push_str(&tabs);
                out.push_str("<dict>\n");
                for (key, value) in entries {
                    out.push_str(&tabs);
                    out.push('\t');
                    out.push_str("<key>");
                    out.push_str(key);
                    out.push_str("</key>\n");
                    value.store_text(out, indent + 1);
                }
                out.push_str(&tabs);
                out.push_str("</dict>\n");
            }
        }
    }
}

/// `PList::load_string`: returns the root node or the engine's error text.
pub fn parse(text: &str) -> Result<PlistNode, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut root: Option<PlistNode> = None;
    let mut stack: Vec<PlistNode> = Vec::new();
    let mut key = String::new();
    let mut in_plist = false;
    let mut done_plist = false;
    let mut pos: isize = 0;

    while pos >= 0 {
        let Some(open_token_s) = find_char(&chars, '<', pos as usize) else {
            return Err("Unexpected end of data. No tags found.".to_string());
        };
        let open_token_e = find_char(&chars, '>', open_token_s).ok_or_else(|| {
            "Unexpected end of data. No tags found.".to_string()
        })?;
        pos = open_token_e as isize;

        let token: String = chars[open_token_s + 1..open_token_e].iter().collect();
        if token.is_empty() {
            return Err("Invalid token name.".to_string());
        }
        if token.starts_with('?') || token.starts_with('!') {
            continue;
        }
        if token.starts_with("plist") {
            in_plist = true;
            continue;
        }
        if token == "/plist" {
            done_plist = true;
            break;
        }
        if !in_plist {
            return Err("Node outside of <plist> tag.".to_string());
        }

        if token == "dict" {
            let node = PlistNode::Dict(Vec::new());
            if let Some(parent) = stack.last_mut() {
                if !parent.push_subnode(&mut key, node.clone()) {
                    return Err("Can't push subnode, invalid parent type.".to_string());
                }
                stack.push(node);
            } else {
                if root.is_some() {
                    return Err("Root node already set.".to_string());
                }
                root = Some(node.clone());
                stack.push(node);
            }
            continue;
        }
        if token == "/dict" {
            if stack.last().map(|n| matches!(n, PlistNode::Dict(_))) != Some(true) {
                return Err("Mismatched </dict> tag.".to_string());
            }
            // Store the finished dict back into its parent (or the root).
            let finished = stack.pop().unwrap();
            if let Some(parent) = stack.last_mut() {
                replace_last_child(parent, finished);
            } else {
                root = Some(finished);
            }
            continue;
        }
        if token == "array" {
            let node = PlistNode::Array(Vec::new());
            if let Some(parent) = stack.last_mut() {
                if !parent.push_subnode(&mut key, node.clone()) {
                    return Err("Can't push subnode, invalid parent type.".to_string());
                }
                stack.push(node);
            } else {
                if root.is_some() {
                    return Err("Root node already set.".to_string());
                }
                root = Some(node.clone());
                stack.push(node);
            }
            continue;
        }
        if token == "/array" {
            if stack.last().map(|n| matches!(n, PlistNode::Array(_))) != Some(true) {
                return Err("Mismatched </array> tag.".to_string());
            }
            let finished = stack.pop().unwrap();
            if let Some(parent) = stack.last_mut() {
                replace_last_child(parent, finished);
            } else {
                root = Some(finished);
            }
            continue;
        }

        // A leaf tag, or a self-closing one.
        let mut token = token;
        let mut value = String::new();
        if token.ends_with('/') {
            token.pop();
        } else {
            let Some(end_token_s) = find_str(&chars, "</", (pos + 1) as usize) else {
                return Err(format!("Mismatched <{token}> tag."));
            };
            let end_token_e = find_char(&chars, '>', end_token_s).ok_or_else(|| {
                format!("Mismatched <{token}> tag.")
            })?;
            pos = end_token_e as isize;
            let end_token: String = chars[end_token_s + 2..end_token_e].iter().collect();
            if end_token != token {
                return Err(format!(
                    "Mismatched <{token}> and <{end_token}> token pair."
                ));
            }
            value = chars[open_token_e + 1..end_token_s].iter().collect();
        }

        if token == "key" {
            key = value;
            continue;
        }

        let node = match token.as_str() {
            "true" => PlistNode::Bool(true),
            "false" => PlistNode::Bool(false),
            "integer" => PlistNode::Int(GoString::from_str(&value).to_int()),
            "real" => PlistNode::Real(GoString::from_str(&value).to_float()),
            "string" => PlistNode::String(value),
            "data" => PlistNode::Data(value),
            "date" => PlistNode::Date(value),
            other => return Err(format!("Invalid value type: {other}.")),
        };

        let Some(parent) = stack.last_mut() else {
            return Err("Can't push subnode, invalid parent type.".to_string());
        };
        if !parent.push_subnode(&mut key, node) {
            return Err("Can't push subnode, invalid parent type.".to_string());
        }
    }

    if !stack.is_empty() || !done_plist {
        return Err("Unexpected end of data. Root node is not closed.".to_string());
    }
    root.ok_or_else(|| "Unexpected end of data. No tags found.".to_string())
}

/// Replaces a container that was pushed by placeholder with its finished version.
fn replace_last_child(parent: &mut PlistNode, finished: PlistNode) {
    match parent {
        PlistNode::Dict(entries) => {
            if let Some((_, value)) = entries.last_mut() {
                *value = finished;
            }
        }
        PlistNode::Array(items) => {
            if let Some(value) = items.last_mut() {
                *value = finished;
            }
        }
        _ => {}
    }
}

fn find_char(chars: &[char], needle: char, from: usize) -> Option<usize> {
    chars
        .iter()
        .enumerate()
        .skip(from)
        .find(|(_, &c)| c == needle)
        .map(|(i, _)| i)
}

fn find_str(chars: &[char], needle: &str, from: usize) -> Option<usize> {
    let needle: Vec<char> = needle.chars().collect();
    if needle.is_empty() || chars.len() < needle.len() {
        return None;
    }
    (from..=chars.len() - needle.len()).find(|&i| chars[i..i + needle.len()] == needle[..])
}

/// `PList::save_text`.
pub fn save_text(root: &PlistNode) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n");
    out.push_str("<plist version=\"1.0\">\n");
    root.store_text(&mut out, 0);
    out.push_str("</plist>\n\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_dict_document() {
        let text = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>name</key>
	<string>Hero</string>
	<key>level</key>
	<integer>7</integer>
	<key>speed</key>
	<real>1.5</real>
	<key>alive</key>
	<true/>
	<key>dead</key>
	<false/>
	<key>items</key>
	<array>
		<string>sword</string>
		<integer>3</integer>
	</array>
	<key>blob</key>
	<data>AQID</data>
	<key>when</key>
	<date>2026-01-01T00:00:00Z</date>
</dict>
</plist>
"#;
        let root = parse(text).unwrap();
        assert_eq!(root.get("name").and_then(|n| n.as_str()), Some("Hero"));
        assert_eq!(root.get("level").and_then(|n| n.as_i64()), Some(7));
        assert_eq!(root.get("speed").and_then(|n| n.as_f64()), Some(1.5));
        assert_eq!(root.get("alive").and_then(|n| n.as_bool()), Some(true));
        assert_eq!(root.get("dead").and_then(|n| n.as_bool()), Some(false));
        let items = root.get("items").unwrap().as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].as_str(), Some("sword"));
        assert_eq!(items[1].as_i64(), Some(3));
        assert_eq!(root.get("blob").and_then(|n| n.as_str()), Some("AQID"));
        assert_eq!(
            root.get("when").and_then(|n| n.as_str()),
            Some("2026-01-01T00:00:00Z")
        );
        assert_eq!(root.as_dict().unwrap().len(), 8);
    }

    #[test]
    fn parses_arrays_and_scalars_as_root() {
        let root = parse("<plist><array><string>a</string><array><real>2.5</real></array></array></plist>")
            .unwrap();
        let items = root.as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].as_str(), Some("a"));
        assert_eq!(items[1].get_index(0).and_then(|n| n.as_f64()), Some(2.5));
        assert!(items.get(5).is_none());

        // A scalar root is refused: values need a parent container.
        assert_eq!(
            parse("<plist><string>only</string></plist>").unwrap_err(),
            "Can't push subnode, invalid parent type."
        );
        let root = parse("<plist><array></array></plist>").unwrap();
        assert_eq!(root.as_array().map(|a| a.len()), Some(0));
    }

    #[test]
    fn error_messages() {
        assert_eq!(
            parse("<plist><dict></plist>").unwrap_err(),
            "Unexpected end of data. Root node is not closed."
        );
        assert_eq!(
            parse("<dict></dict>").unwrap_err(),
            "Node outside of <plist> tag."
        );
        assert_eq!(parse("nothing here").unwrap_err(), "Unexpected end of data. No tags found.");
        assert_eq!(
            parse("<plist><dict></array></dict></plist>").unwrap_err(),
            "Mismatched </array> tag."
        );
        assert_eq!(
            parse("<plist><dict><key>a</key><bogus>x</bogus></dict></plist>").unwrap_err(),
            "Invalid value type: bogus."
        );
        assert_eq!(
            parse("<plist><dict><string>x").unwrap_err(),
            "Mismatched <string> tag."
        );
        assert_eq!(
            parse("<plist><dict><key>a</key><string>x</integer></dict></plist>").unwrap_err(),
            "Mismatched <string> and <integer> token pair."
        );
        assert_eq!(
            parse("<plist><dict></dict><array></array></plist>").unwrap_err(),
            "Root node already set."
        );
        assert_eq!(parse("<plist><>x</></plist>").unwrap_err(), "Invalid token name.");
        // A pending key without a value is simply dropped, like the engine.
        assert_eq!(parse("<plist><dict><key>a</key></dict></plist>").unwrap(), PlistNode::Dict(Vec::new()));
    }

    #[test]
    fn save_text_round_trips() {
        let root = PlistNode::Dict(vec![
            ("name".to_string(), PlistNode::String("Hero".to_string())),
            ("level".to_string(), PlistNode::Int(7)),
            ("speed".to_string(), PlistNode::Real(1.5)),
            ("alive".to_string(), PlistNode::Bool(true)),
            (
                "items".to_string(),
                PlistNode::Array(vec![
                    PlistNode::String("sword".to_string()),
                    PlistNode::Int(3),
                ]),
            ),
            ("blob".to_string(), PlistNode::Data("AQID".to_string())),
            (
                "when".to_string(),
                PlistNode::Date("2026-01-01T00:00:00Z".to_string()),
            ),
        ]);

        let text = save_text(&root);
        assert!(text.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"));
        assert!(text.contains("<plist version=\"1.0\">\n"));
        assert!(text.contains("\t<key>name</key>\n\t<string>Hero</string>\n"));
        assert!(text.contains("\t<true/>\n"));
        assert!(text.contains("\t\t<string>sword</string>\n\t\t<integer>3</integer>\n"));
        assert!(text.contains("\t<data>\n\tAQID\n\t</data>\n"));
        assert!(text.ends_with("</plist>\n\n"));

        // Round trip: `<data>` is written on its own indented lines, so the
        // parser reads that whitespace back into the value; the rest matches.
        let parsed = parse(&text).unwrap();
        let mut expected = root.clone();
        if let PlistNode::Dict(entries) = &mut expected {
            for (key, value) in entries.iter_mut() {
                if key == "blob" {
                    *value = PlistNode::Data("\n\tAQID\n\t".to_string());
                }
            }
        }
        assert_eq!(parsed, expected);
    }

    #[test]
    fn nil_nodes_write_nothing() {
        let root = PlistNode::Array(vec![PlistNode::Nil, PlistNode::Int(1)]);
        let text = save_text(&root);
        assert!(text.contains("<array>\n\t<integer>1</integer>\n</array>\n"));
    }
}
