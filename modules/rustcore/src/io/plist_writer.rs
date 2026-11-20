//! `core/io/plist.cpp` (text writer): `PList::save_text`.
//!
//! Serialises a [`PlistNode`] back to the Apple XML property list the engine's
//! `PListNode::store_text` emits: the XML/DOCTYPE header, `<plist version="1.0">`,
//! one element per node with tab indentation, `<key>` entries inside dictionaries
//! and the trailing `</plist>` plus a blank line.
//!
//! Data nodes are ASCII base64 (as the engine expects); dates keep their ISO
//! text; nil nodes emit nothing.

use crate::io::plist::PlistNode;

fn push_indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push('\t');
    }
}

fn store(node: &PlistNode, out: &mut String, level: usize) {
    match node {
        PlistNode::Nil => {}
        PlistNode::Data(text) => {
            push_indent(out, level);
            out.push_str("<data>\n");
            push_indent(out, level);
            out.push_str(text);
            out.push('\n');
            push_indent(out, level);
            out.push_str("</data>\n");
        }
        PlistNode::Date(text) => {
            push_indent(out, level);
            out.push_str("<date>");
            out.push_str(text);
            out.push_str("</date>\n");
        }
        PlistNode::String(text) => {
            push_indent(out, level);
            out.push_str("<string>");
            out.push_str(text);
            out.push_str("</string>\n");
        }
        PlistNode::Bool(true) => {
            push_indent(out, level);
            out.push_str("<true/>\n");
        }
        PlistNode::Bool(false) => {
            push_indent(out, level);
            out.push_str("<false/>\n");
        }
        PlistNode::Int(value) => {
            push_indent(out, level);
            out.push_str("<integer>");
            out.push_str(&value.to_string());
            out.push_str("</integer>\n");
        }
        PlistNode::Real(value) => {
            push_indent(out, level);
            out.push_str("<real>");
            out.push_str(&format_real(*value));
            out.push_str("</real>\n");
        }
        PlistNode::Array(items) => {
            push_indent(out, level);
            out.push_str("<array>\n");
            for item in items {
                store(item, out, level + 1);
            }
            push_indent(out, level);
            out.push_str("</array>\n");
        }
        PlistNode::Dict(entries) => {
            push_indent(out, level);
            out.push_str("<dict>\n");
            for (key, value) in entries {
                push_indent(out, level + 1);
                out.push_str("<key>");
                out.push_str(key);
                out.push_str("</key>\n");
                store(value, out, level + 1);
            }
            push_indent(out, level);
            out.push_str("</dict>\n");
        }
    }
}

/// Real formatting: integral values get a `.0` so the text stays a real (the
/// engine's `rtos` does the same).
fn format_real(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{value:.1}")
    } else {
        format!("{value}")
    }
}

/// `PList::save_text`: the whole document for `root`.
pub fn save_text(root: &PlistNode) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n");
    out.push_str("<plist version=\"1.0\">\n");
    store(root, &mut out, 0);
    out.push_str("</plist>\n\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::plist;

    fn sample() -> PlistNode {
        PlistNode::Dict(vec![
            ("name".to_string(), PlistNode::String("Gödel".to_string())),
            ("count".to_string(), PlistNode::Int(-42)),
            ("pi".to_string(), PlistNode::Real(3.5)),
            ("whole".to_string(), PlistNode::Real(2.0)),
            ("ok".to_string(), PlistNode::Bool(true)),
            ("off".to_string(), PlistNode::Bool(false)),
            (
                "items".to_string(),
                PlistNode::Array(vec![
                    PlistNode::Int(1),
                    PlistNode::String("two".to_string()),
                    PlistNode::Array(vec![PlistNode::Int(3)]),
                ]),
            ),

        ])
    }

    #[test]
    fn header_footer_and_indentation() {
        let text = save_text(&PlistNode::Array(vec![PlistNode::Int(1)]));
        assert!(text.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"));
        assert!(text.contains("<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\""));
        assert!(text.contains("<plist version=\"1.0\">\n"));
        assert!(text.contains("<array>\n\t<integer>1</integer>\n</array>\n"));
        assert!(text.ends_with("</plist>\n\n"));

        // Scalars and containers get one more tab per level.
        let text = save_text(&PlistNode::Dict(vec![(
            "k".to_string(),
            PlistNode::Array(vec![PlistNode::Bool(true)]),
        )]));
        assert!(text.contains("\t<key>k</key>\n\t<array>\n\t\t<true/>\n\t</array>\n"));

        // Nil emits nothing but the surrounding tags.
        assert_eq!(
            save_text(&PlistNode::Nil),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n</plist>\n\n"
        );
    }

    #[test]
    fn data_and_date_layout() {
        let text = save_text(&PlistNode::Dict(vec![
            ("blob".to_string(), PlistNode::Data("AQI=".to_string())),
            (
                "when".to_string(),
                PlistNode::Date("2020-01-02 03:04:05".to_string()),
            ),
        ]));
        assert!(text.contains(
            "\t<key>blob</key>\n\t<data>\n\tAQI=\n\t</data>\n\t<key>when</key>\n\t<date>2020-01-02 03:04:05</date>\n"
        ));
    }

    #[test]
    fn round_trips_through_the_parser() {
        let node = sample();
        let text = save_text(&node);
        let parsed = plist::parse(&text).expect("parsed back");
        assert_eq!(parsed, node);
    }

    #[test]
    fn real_formatting() {
        assert_eq!(format_real(3.5), "3.5");
        assert_eq!(format_real(2.0), "2.0");
        assert_eq!(format_real(-0.25), "-0.25");
        assert_eq!(format_real(f64::INFINITY), "inf");
    }
}
