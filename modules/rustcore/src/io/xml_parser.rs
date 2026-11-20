//! `core/io/xml_parser.*`: the irrXML-derived pull parser.
//!
//! Ported operation for operation: node offset and line tracking, text nodes
//! (with the engine's "fewer than three characters and all whitespace is
//! skipped" rule and XML unescaping), opening elements with attributes (single
//! or double quotes, unescaped values), self-closing detection, closing elements,
//! `<?...?>` definitions, comments (`<!-- -->` and bracket-balanced `<!...>`),
//! CDATA and `skip_section`.
//!
//! Byte offsets are preserved because the parser works on the raw bytes: node
//! names and values are decoded as UTF-8, exactly like `String::utf8`. Encoding
//! detection (`open`/`open_buffer` format sniffing) is not ported — callers hand
//! the parser UTF-8 bytes.
//!
//! As in the engine, `get_node_data` fails unless the current node is text and
//! `get_node_name` fails on text nodes; both report an empty string here.

use crate::string::escape::xml_unescape;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum NodeType {
    None,
    Element,
    ElementEnd,
    Text,
    Comment,
    CData,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XmlError {
    /// `ERR_FILE_EOF`.
    Eof,
    /// `ERR_INVALID_DATA`.
    InvalidData,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XmlAttribute {
    pub name: String,
    pub value: String,
}

fn is_white_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r')
}

pub struct XmlParser {
    /// Document bytes followed by a terminating zero, like `data_copy`.
    data: Vec<u8>,
    length: usize,
    pos: usize,
    line: usize,
    node_name: String,
    node_empty: bool,
    node_type: NodeType,
    node_offset: usize,
    attributes: Vec<XmlAttribute>,
}

impl Default for XmlParser {
    fn default() -> Self {
        Self::new()
    }
}

impl XmlParser {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            length: 0,
            pos: 0,
            line: 0,
            node_name: String::new(),
            node_empty: false,
            node_type: NodeType::None,
            node_offset: 0,
            attributes: Vec::new(),
        }
    }

    /// `open_buffer`: rejects empty input with `ERR_INVALID_DATA`.
    pub fn open_buffer(&mut self, bytes: &[u8]) -> Result<(), XmlError> {
        if bytes.is_empty() {
            return Err(XmlError::InvalidData);
        }
        self.data = Vec::with_capacity(bytes.len() + 1);
        self.data.extend_from_slice(bytes);
        self.data.push(0);
        self.length = bytes.len();
        self.pos = 0;
        self.line = 0;
        Ok(())
    }

    pub fn close(&mut self) {
        self.data.clear();
        self.length = 0;
        self.pos = 0;
        self.node_empty = false;
        self.node_type = NodeType::None;
        self.node_offset = 0;
        self.attributes.clear();
        self.node_name.clear();
    }

    /// The byte at the cursor, with the terminating zero past the end.
    fn at(&self) -> u8 {
        self.data.get(self.pos).copied().unwrap_or(0)
    }

    fn at_offset(&self, offset: usize) -> u8 {
        self.data.get(self.pos + offset).copied().unwrap_or(0)
    }

    /// `next_char`: advances and counts newlines.
    fn next_char(&mut self) {
        if self.at() == b'\n' {
            self.line += 1;
        }
        self.pos += 1;
    }

    fn text(&self, start: usize, end: usize) -> String {
        if end <= start {
            return String::new();
        }
        String::from_utf8_lossy(&self.data[start..end]).into_owned()
    }

    /// Reads the next node; `Err(XmlError::Eof)` once the document is consumed.
    pub fn read(&mut self) -> Result<(), XmlError> {
        if !self.data.is_empty() && self.pos < self.length.saturating_sub(1) && self.at() != 0 {
            self.parse_current_node();
            return Ok(());
        }
        Err(XmlError::Eof)
    }

    /// `seek`: re-reads from `position`, failing at or past the end.
    pub fn seek(&mut self, position: usize) -> Result<(), XmlError> {
        if self.data.is_empty() || position >= self.length {
            return Err(XmlError::Eof);
        }
        self.pos = position;
        self.read()
    }

    pub fn get_node_type(&self) -> NodeType {
        self.node_type
    }

    /// Fails (empty) unless the current node is text.
    pub fn get_node_data(&self) -> String {
        if self.node_type != NodeType::Text {
            return String::new();
        }
        self.node_name.clone()
    }

    /// Fails (empty) on text nodes.
    pub fn get_node_name(&self) -> String {
        if self.node_type == NodeType::Text {
            return String::new();
        }
        self.node_name.clone()
    }

    pub fn get_node_offset(&self) -> usize {
        self.node_offset
    }

    pub fn get_current_line(&self) -> usize {
        self.line
    }

    pub fn get_attribute_count(&self) -> usize {
        self.attributes.len()
    }

    pub fn get_attribute_name(&self, index: usize) -> String {
        self.attributes
            .get(index)
            .map(|a| a.name.clone())
            .unwrap_or_default()
    }

    pub fn get_attribute_value(&self, index: usize) -> String {
        self.attributes
            .get(index)
            .map(|a| a.value.clone())
            .unwrap_or_default()
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.iter().any(|a| a.name == name)
    }

    /// Fails (empty) when the attribute is missing.
    pub fn get_named_attribute_value(&self, name: &str) -> String {
        self.attributes
            .iter()
            .find(|a| a.name == name)
            .map(|a| a.value.clone())
            .unwrap_or_default()
    }

    /// The same lookup without the "attribute not found" error path.
    pub fn get_named_attribute_value_safe(&self, name: &str) -> String {
        self.get_named_attribute_value(name)
    }

    /// `is_empty`: the last opening element was self-closing.
    pub fn is_empty(&self) -> bool {
        self.node_empty
    }

    /// `skip_section`: advances to the end of the current element.
    pub fn skip_section(&mut self) {
        if self.is_empty() {
            return;
        }
        let mut tagcount = 1i32;
        while tagcount != 0 && self.read().is_ok() {
            if self.get_node_type() == NodeType::Element && !self.is_empty() {
                tagcount += 1;
            } else if self.get_node_type() == NodeType::ElementEnd {
                tagcount -= 1;
            }
        }
    }

    // -- internals ----------------------------------------------------------

    /// `_set_text`: returns whether a text node was produced.
    fn set_text(&mut self, start: usize, end: usize) -> bool {
        if end.saturating_sub(start) < 3 {
            let all_white = self.data[start..end].iter().all(|&c| is_white_space(c));
            if all_white {
                return false;
            }
        }
        self.node_name = xml_unescape(&self.text(start, end));
        self.node_type = NodeType::Text;
        true
    }

    fn parse_current_node(&mut self) {
        let start = self.pos;
        self.node_offset = self.pos;

        // Advance to the next '<'.
        while self.at() != b'<' && self.at() != 0 {
            self.next_char();
        }

        if self.pos.saturating_sub(start) > 0 && self.set_text(start, self.pos) {
            return;
        }
        if self.at() == 0 {
            return;
        }

        self.next_char();

        match self.at() {
            b'/' => self.parse_closing_element(),
            b'?' => self.ignore_definition(),
            b'!' => {
                if !self.parse_cdata() {
                    self.parse_comment();
                }
            }
            _ => self.parse_opening_element(),
        }
    }

    fn parse_closing_element(&mut self) {
        self.node_type = NodeType::ElementEnd;
        self.node_empty = false;
        self.attributes.clear();

        self.next_char();
        let begin = self.pos;
        while self.at() != 0 && self.at() != b'>' {
            self.next_char();
        }
        self.node_name = self.text(begin, self.pos);
        if self.at() != 0 {
            self.next_char();
        }
    }

    fn ignore_definition(&mut self) {
        self.node_type = NodeType::Unknown;

        let begin = self.pos;
        while self.at() != 0 && self.at() != b'>' {
            self.next_char();
        }
        self.node_name = self.text(begin, self.pos);
        if self.at() != 0 {
            self.next_char();
        }
    }

    /// `_parse_cdata`; `false` means "not CDATA, try a comment".
    fn parse_cdata(&mut self) -> bool {
        if self.at_offset(1) != b'[' {
            return false;
        }

        self.node_type = NodeType::CData;

        // Skip `<![CDATA[`.
        let mut count = 0;
        while self.at() != 0 && count < 8 {
            self.next_char();
            count += 1;
        }

        if self.at() == 0 {
            self.node_name = String::new();
            return true;
        }

        let begin = self.pos;
        let mut end: Option<usize> = None;
        while self.at() != 0 && end.is_none() {
            if self.at() == b'>'
                && self.pos >= 2
                && self.data[self.pos - 1] == b']'
                && self.data[self.pos - 2] == b']'
            {
                end = Some(self.pos - 2);
            }
            self.next_char();
        }
        let end = end.unwrap_or(self.pos);
        self.node_name = self.text(begin, end);
        true
    }

    fn parse_comment(&mut self) {
        self.node_type = NodeType::Comment;
        self.pos += 1;

        let end_of_input = self.length;
        let begin;
        let end;

        if self.pos + 1 < end_of_input && self.at() == b'-' && self.at_offset(1) == b'-' {
            // `<!-- ... -->`.
            begin = self.pos + 2;
            let mut cursor = begin;
            while cursor + 2 < end_of_input {
                if self.data[cursor] == b'-'
                    && self.data[cursor + 1] == b'-'
                    && self.data[cursor + 2] == b'>'
                {
                    break;
                }
                cursor += 1;
            }
            if cursor + 2 < end_of_input {
                self.pos = cursor + 3;
                end = cursor;
            } else {
                self.pos = end_of_input;
                end = end_of_input;
            }
        } else {
            // A definition: match angle brackets.
            begin = self.pos;
            let mut count = 1i32;
            while self.at() != 0 && count != 0 {
                if self.at() == b'>' {
                    count -= 1;
                } else if self.at() == b'<' {
                    count += 1;
                }
                self.next_char();
            }
            end = if count != 0 {
                self.pos
            } else {
                self.pos.saturating_sub(1)
            };
        }

        self.node_name = self.text(begin, end);
    }

    fn parse_opening_element(&mut self) {
        self.node_type = NodeType::Element;
        self.node_empty = false;
        self.attributes.clear();

        let start_name = self.pos;
        while self.at() != 0 && self.at() != b'>' && !is_white_space(self.at()) {
            self.next_char();
        }
        let mut end_name = self.pos;

        while self.at() != 0 && self.at() != b'>' {
            if is_white_space(self.at()) {
                self.next_char();
            } else if self.at() != b'/' {
                // An attribute.
                let name_begin = self.pos;
                while self.at() != 0 && !is_white_space(self.at()) && self.at() != b'=' {
                    self.next_char();
                }
                if self.at() == 0 {
                    break;
                }
                let name_end = self.pos;
                self.next_char();

                // Skip to the opening quote.
                while self.at() != b'"' && self.at() != b'\'' && self.at() != 0 {
                    self.next_char();
                }
                if self.at() == 0 {
                    break;
                }
                let quote = self.at();
                self.next_char();
                let value_begin = self.pos;
                while self.at() != quote && self.at() != 0 {
                    self.next_char();
                }
                let value_end = self.pos;
                if self.at() != 0 {
                    self.next_char();
                }

                let name = self.text(name_begin, name_end);
                let value = xml_unescape(&self.text(value_begin, value_end));
                self.attributes.push(XmlAttribute { name, value });
            } else {
                // The tag closes directly.
                self.next_char();
                self.node_empty = true;
                break;
            }
        }

        // `<name/>` also sets the empty flag and drops the slash from the name.
        if end_name > start_name && self.data[end_name - 1] == b'/' {
            self.node_empty = true;
            end_name -= 1;
        }

        self.node_name = self.text(start_name, end_name);

        if self.at() != 0 {
            self.next_char();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser(doc: &str) -> XmlParser {
        let mut p = XmlParser::new();
        p.open_buffer(doc.as_bytes()).unwrap();
        p
    }

    #[test]
    fn walks_a_small_document() {
        let mut p = parser(
            r#"<?xml version="1.0"?><!-- hi --><root a="1" b='two &amp; three'><child/>text &lt;x&gt;</root>"#,
        );

        // The XML definition is an UNKNOWN node; the hunks keep the '?' markers.
        assert_eq!(p.read(), Ok(()));
        assert_eq!(p.get_node_type(), NodeType::Unknown);
        assert_eq!(p.get_node_name(), "?xml version=\"1.0\"?");

        assert_eq!(p.read(), Ok(()));
        assert_eq!(p.get_node_type(), NodeType::Comment);
        assert_eq!(p.get_node_name(), " hi ");

        assert_eq!(p.read(), Ok(()));
        assert_eq!(p.get_node_type(), NodeType::Element);
        assert_eq!(p.get_node_name(), "root");
        assert!(!p.is_empty());
        assert_eq!(p.get_attribute_count(), 2);
        assert_eq!(p.get_attribute_name(0), "a");
        assert_eq!(p.get_attribute_value(0), "1");
        assert_eq!(p.get_attribute_value(1), "two & three");
        assert!(p.has_attribute("b"));
        assert!(!p.has_attribute("c"));
        assert_eq!(p.get_named_attribute_value("b"), "two & three");
        assert_eq!(p.get_named_attribute_value_safe("c"), "");
        assert_eq!(p.get_attribute_name(9), "");
        assert_eq!(p.get_attribute_value(9), "");
        assert_eq!(p.get_node_offset(), 32);

        assert_eq!(p.read(), Ok(()));
        assert_eq!(p.get_node_type(), NodeType::Element);
        assert_eq!(p.get_node_name(), "child");
        assert!(p.is_empty());

        assert_eq!(p.read(), Ok(()));
        assert_eq!(p.get_node_type(), NodeType::Text);
        assert_eq!(p.get_node_data(), "text <x>");
        assert_eq!(p.get_node_name(), "", "names fail on text nodes");
        assert_eq!(p.get_attribute_count(), 0);

        assert_eq!(p.read(), Ok(()));
        assert_eq!(p.get_node_type(), NodeType::ElementEnd);
        assert_eq!(p.get_node_name(), "root");
        assert_eq!(p.get_node_data(), "", "data fails outside text nodes");

        assert_eq!(p.read(), Err(XmlError::Eof));
    }

    #[test]
    fn whitespace_text_is_skipped_only_when_short() {
        // Three characters: the engine's "shorter than three" shortcut does not
        // apply, so whitespace is reported as text.
        let mut p = parser("<a>   </a>");
        assert_eq!(p.read(), Ok(()));
        assert_eq!(p.get_node_name(), "a");
        assert_eq!(p.read(), Ok(()));
        assert_eq!(p.get_node_type(), NodeType::Text);
        assert_eq!(p.get_node_data(), "   ");
        assert_eq!(p.read(), Ok(()));
        assert_eq!(p.get_node_type(), NodeType::ElementEnd);

        // Two blank characters are dropped, so the next node is the end tag.
        let mut p = parser("<a> \n</a>");
        p.read().unwrap();
        assert_eq!(p.read(), Ok(()));
        assert_eq!(p.get_node_type(), NodeType::ElementEnd);

        // A single non-space character is kept (short but not blank).
        let mut p = parser("<a>x</a>");
        p.read().unwrap();
        p.read().unwrap();
        assert_eq!(p.get_node_type(), NodeType::Text);
        assert_eq!(p.get_node_data(), "x");
    }

    #[test]
    fn cdata_comments_and_definitions() {
        let mut p = parser("<a><![CDATA[raw <x> & stuff]]><!DOCTYPE foo [<!ENTITY x>]></a>");
        p.read().unwrap();
        assert_eq!(p.get_node_name(), "a");

        p.read().unwrap();
        assert_eq!(p.get_node_type(), NodeType::CData);
        assert_eq!(p.get_node_name(), "raw <x> & stuff");

        p.read().unwrap();
        assert_eq!(p.get_node_type(), NodeType::Comment);
        assert_eq!(p.get_node_name(), "DOCTYPE foo [<!ENTITY x>]");

        p.read().unwrap();
        assert_eq!(p.get_node_type(), NodeType::ElementEnd);

        // An unterminated comment consumes the rest.
        let mut p = parser("<!-- open");
        p.read().unwrap();
        assert_eq!(p.get_node_type(), NodeType::Comment);
        assert_eq!(p.get_node_name(), " open");
    }

    #[test]
    fn self_closing_and_attribute_quirks() {
        let mut p = parser(r#"<a x = "1" y="2"/>"#);
        p.read().unwrap();
        assert_eq!(p.get_node_name(), "a");
        assert!(p.is_empty());
        assert_eq!(p.get_attribute_count(), 2);
        assert_eq!(p.get_named_attribute_value("x"), "1");
        assert_eq!(p.get_named_attribute_value("y"), "2");

        // No value at all leaves only the first attribute parsed.
        let mut p = parser("<a x= >");
        p.read().unwrap();
        assert_eq!(p.get_node_name(), "a");

        // A lone '<' is not enough data to read anything.
        let mut p = parser("<");
        assert_eq!(p.read(), Err(XmlError::Eof));
    }

    #[test]
    fn skip_section_and_seek() {
        let mut p = parser("<a><b><c/></b>noise</a><next/>");
        p.read().unwrap();
        assert_eq!(p.get_node_name(), "a");
        p.skip_section();
        assert_eq!(p.get_node_type(), NodeType::ElementEnd);
        assert_eq!(p.get_node_name(), "a");

        p.read().unwrap();
        assert_eq!(p.get_node_name(), "next");
        assert!(p.is_empty());

        // Seeking to the start re-reads the first element.
        assert_eq!(p.seek(0), Ok(()));
        assert_eq!(p.get_node_name(), "a");
        // Seeking to or past the end fails.
        assert_eq!(p.seek(1000), Err(XmlError::Eof));

        // Skipping an empty element does nothing.
        let mut p = parser("<a/><b/>");
        p.read().unwrap();
        p.skip_section();
        assert!(p.is_empty());
        assert_eq!(p.get_node_name(), "a");
    }

    #[test]
    fn close_and_empty_buffer() {
        let mut p = XmlParser::new();
        assert_eq!(p.open_buffer(b""), Err(XmlError::InvalidData));
        assert_eq!(p.read(), Err(XmlError::Eof));

        let mut p = parser("<a/>");
        p.read().unwrap();
        p.close();
        assert_eq!(p.get_node_type(), NodeType::None);
        assert!(!p.is_empty());
        assert_eq!(p.get_node_offset(), 0);
        assert_eq!(p.read(), Err(XmlError::Eof));
    }

    #[test]
    fn line_counting() {
        let mut p = parser("<a>\n<b>\n</b>\n</a>");
        p.read().unwrap();
        assert_eq!(p.get_current_line(), 0);
        p.read().unwrap();
        assert_eq!(p.get_current_line(), 1);
        p.read().unwrap();
        assert_eq!(p.get_current_line(), 2);
    }
}
