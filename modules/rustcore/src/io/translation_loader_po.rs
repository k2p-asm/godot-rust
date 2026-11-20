//! `core/io/translation_loader_po.*`: the `.po`/`.mo` translation loader.
//!
//! Ported from `translation_loader_po.cpp`:
//!
//! - the binary MO path (magic `0x950412de`): version check against major 1,
//!   the id/translation tables with their NUL-separated context (`0x04`) and
//!   plural forms, the empty-id header entry carrying the configuration and the
//!   `Plural-Forms` override;
//! - the text PO path: the engine's status machine (`msgctxt` / `msgid` /
//!   `msgid_plural` / `msgstr` / `msgstr[n]`), `#` comments (including `fuzzy`
//!   skipping the *next* message, as the engine does), quoted multi-line strings
//!   with C unescaping, the `Plural-Forms` header and the trailing
//!   `X-Language`/`Language` locale.
//!
//! Errors are returned as the engine's messages (`String`) instead of logging;
//! `FileAccess` itself is not needed because the bytes arrive directly.

use crate::string::plural_rules;
use crate::string::translation::TranslationData;

const MO_MAGIC: u32 = 0x9504_12de;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    None,
    ReadingId,
    ReadingString,
    ReadingContext,
    ReadingPlural,
}

fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
    let bytes = data.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    let bytes = data.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn utf8_lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// `String::strip_edges` over ASCII whitespace.
fn strip_edges(text: &str) -> &str {
    text.trim_matches(|c: char| c.is_whitespace())
}

/// `String::c_unescape` for the escapes a PO file can carry.
fn c_unescape(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('a') => out.push('\u{7}'),
            Some('b') => out.push('\u{8}'),
            Some('f') => out.push('\u{c}'),
            Some('v') => out.push('\u{b}'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Applies the parsed PO configuration block: the plural override was already
/// recorded while parsing; here the locale is read from `X-Language`/`Language`.
fn apply_config(config: &str, translation: &mut TranslationData) {
    for line in config.split('\n') {
        let line = strip_edges(line);
        let Some(colon) = line.find(':') else {
            continue;
        };
        let property = strip_edges(&line[..colon]);
        let value = strip_edges(&line[colon + 1..]);
        if property == "X-Language" || property == "Language" {
            translation.set_locale(value);
        }
    }
}

/// Records the `Plural-Forms` rule found in a configuration block; returns the
/// number of plural forms (0 when the header is absent or unparsable).
fn record_plural_rule(config: &str, translation: &mut TranslationData) -> i32 {
    let Some(start) = config.find("Plural-Forms") else {
        return 0;
    };
    let end = config[start..]
        .find('\n')
        .map(|offset| start + offset)
        .unwrap_or(config.len());
    let rule = &config[start..end];
    translation.set_plural_rules_override(rule);
    plural_rules::parse_rules(rule).map(|rules| rules.get_nplurals()).unwrap_or(0)
}

/// `TranslationLoaderPO::load_translation`.
pub fn load_translation(data: &[u8]) -> Result<TranslationData, String> {
    let mut translation = TranslationData::new();
    let mut config = String::new();

    if read_u32(data, 0) == Some(MO_MAGIC) {
        let version_major = read_u16(data, 4).ok_or("truncated MO header")?;
        let version_minor = read_u16(data, 6).ok_or("truncated MO header")?;
        if version_major > 1 {
            return Err(format!(
                "Unsupported MO file version {version_major}.{version_minor}"
            ));
        }
        let num_strings = read_u32(data, 8).ok_or("truncated MO header")?;
        let id_table_offset = read_u32(data, 12).ok_or("truncated MO header")? as usize;
        let trans_table_offset = read_u32(data, 16).ok_or("truncated MO header")? as usize;

        for i in 0..num_strings as usize {
            // Original strings: context (0x04) then the singular id, with the
            // plural id after the first NUL.
            let id_length = read_u32(data, id_table_offset + i * 8).ok_or("truncated MO table")? as usize;
            let id_offset = read_u32(data, id_table_offset + i * 8 + 4).ok_or("truncated MO table")? as usize;
            let id_bytes = data
                .get(id_offset..id_offset + id_length)
                .ok_or("truncated MO string")?;

            let mut msg_context = String::new();
            let mut msg_id = String::new();
            let mut msg_id_plural = String::new();
            let mut start = 0usize;
            let mut is_plural = false;
            for j in 0..=id_bytes.len() {
                let byte = id_bytes.get(j).copied().unwrap_or(0);
                if byte == 0x04 {
                    msg_context = utf8_lossy(&id_bytes[..j]);
                    start = j + 1;
                }
                if byte == 0 {
                    if is_plural {
                        msg_id_plural = utf8_lossy(&id_bytes[start..j]);
                    } else {
                        msg_id = utf8_lossy(&id_bytes[start..j]);
                        is_plural = true;
                    }
                    start = j + 1;
                }
            }

            let trans_length =
                read_u32(data, trans_table_offset + i * 8).ok_or("truncated MO table")? as usize;
            let trans_offset =
                read_u32(data, trans_table_offset + i * 8 + 4).ok_or("truncated MO table")? as usize;
            let trans_bytes = data
                .get(trans_offset..trans_offset + trans_length)
                .ok_or("truncated MO string")?;

            if msg_id.is_empty() {
                config = utf8_lossy(trans_bytes);
                record_plural_rule(&config, &mut translation);
                continue;
            }

            if msg_id_plural.is_empty() {
                translation.add_message(&msg_id, &utf8_lossy(trans_bytes), &msg_context);
            } else {
                let forms: Vec<String> = trans_bytes
                    .split(|byte| *byte == 0)
                    .map(utf8_lossy)
                    .collect();
                translation.add_plural_message(&msg_id, &forms, &msg_context);
            }
        }

        if config.is_empty() {
            return Err("No config found in file".to_string());
        }
        apply_config(&config, &mut translation);
        return Ok(translation);
    }

    // Text PO.
    let mut status = Status::None;
    let mut msg_id = String::new();
    let mut msg_str = String::new();
    let mut msg_context = String::new();
    let mut msgs_plural: Vec<String> = Vec::new();
    let mut plural_forms = 0i32;
    let mut plural_index: i32 = -1;
    let mut entered_context = false;
    let mut skip_this = false;
    let mut skip_next = false;
    let mut lines = data.split(|byte| *byte == b'\n');
    let mut line_number = 0usize;

    loop {
        let raw = match lines.next() {
            Some(line) => line,
            None => break,
        };
        line_number += 1;
        let at_eof = lines.clone().next().is_none();
        let mut l = strip_edges(&utf8_lossy(raw)).to_string();

        if at_eof && l.is_empty() {
            if status == Status::ReadingId
                || status == Status::ReadingContext
                || (status == Status::ReadingPlural && plural_index != plural_forms - 1)
            {
                return Err(format!("Unexpected EOF while reading PO file at line {line_number}"));
            }
            break;
        }

        if let Some(rest) = l.strip_prefix("msgctxt") {
            if status != Status::ReadingString && status != Status::ReadingPlural {
                return Err(format!(
                    "Unexpected 'msgctxt' while parsing at line {line_number}"
                ));
            }
            if !skip_this && !msg_id.is_empty() {
                if status == Status::ReadingString {
                    translation.add_message(&msg_id, &msg_str, &msg_context);
                } else if status == Status::ReadingPlural {
                    if plural_index != plural_forms - 1 {
                        return Err(format!(
                            "Number of 'msgstr[]' doesn't match the plural forms at line {line_number}"
                        ));
                    }
                    translation.add_plural_message(&msg_id, &msgs_plural, &msg_context);
                }
            }
            msg_context.clear();
            l = strip_edges(rest).to_string();
            status = Status::ReadingContext;
            entered_context = true;
        }

        if let Some(rest) = l.strip_prefix("msgid_plural") {
            if plural_forms == 0 {
                return Err(format!(
                    "PO file uses 'msgid_plural' but 'Plural-Forms' is missing or invalid at line {line_number}"
                ));
            }
            if status != Status::ReadingId {
                return Err(format!(
                    "Unexpected 'msgid_plural' while parsing at line {line_number}"
                ));
            }
            l = strip_edges(rest).to_string();
            plural_index = -1;
            msgs_plural = vec![String::new(); plural_forms as usize];
            status = Status::ReadingPlural;
        } else if let Some(rest) = l.strip_prefix("msgid") {
            if status == Status::ReadingId {
                return Err(format!(
                    "Unexpected 'msgid' while parsing at line {line_number}"
                ));
            }
            if !msg_id.is_empty() {
                if !skip_this && !entered_context {
                    if status == Status::ReadingString {
                        translation.add_message(&msg_id, &msg_str, &msg_context);
                    } else if status == Status::ReadingPlural {
                        if plural_index != plural_forms - 1 {
                            return Err(format!(
                                "Number of 'msgstr[]' doesn't match the plural forms at line {line_number}"
                            ));
                        }
                        translation.add_plural_message(&msg_id, &msgs_plural, &msg_context);
                    }
                }
            } else if config.is_empty() {
                config = msg_str.clone();
                plural_forms = record_plural_rule(&config, &mut translation);
            }

            l = strip_edges(rest).to_string();
            status = Status::ReadingId;
            if !entered_context {
                msg_context.clear();
            }
            msg_id.clear();
            msg_str.clear();
            skip_this = skip_next;
            skip_next = false;
            entered_context = false;
        }

        if let Some(rest) = l.strip_prefix("msgstr[") {
            if status != Status::ReadingPlural {
                return Err(format!(
                    "Unexpected 'msgstr[]' while parsing at line {line_number}"
                ));
            }
            plural_index += 1;
            // `msgstr[N]` — skip the index and its closing bracket.
            l = match rest.find(']') {
                Some(end) => strip_edges(&rest[end + 1..]).to_string(),
                None => String::new(),
            };
        } else if let Some(rest) = l.strip_prefix("msgstr") {
            if status != Status::ReadingId {
                return Err(format!(
                    "Unexpected 'msgstr' while parsing at line {line_number}"
                ));
            }
            l = strip_edges(rest).to_string();
            status = Status::ReadingString;
        }

        if l.is_empty() || l.starts_with('#') {
            if l.contains("fuzzy") {
                skip_next = true;
            }
            continue;
        }

        if !l.starts_with('"') || status == Status::None {
            return Err(format!(
                "Invalid line '{l}' while parsing at line {line_number}"
            ));
        }

        let body = &l[1..];
        let mut end_pos = None;
        let mut escape_next = false;
        for (index, c) in body.char_indices() {
            if c == '\\' && !escape_next {
                escape_next = true;
                continue;
            }
            if c == '"' && !escape_next {
                end_pos = Some(index);
                break;
            }
            escape_next = false;
        }
        let Some(end_pos) = end_pos else {
            return Err(format!("Expected '\"' at end of message at line {line_number}"));
        };
        let text = c_unescape(&body[..end_pos]);

        match status {
            Status::ReadingId => msg_id.push_str(&text),
            Status::ReadingString => msg_str.push_str(&text),
            Status::ReadingContext => msg_context.push_str(&text),
            Status::ReadingPlural if plural_index >= 0 => {
                if plural_index >= plural_forms {
                    return Err(format!(
                        "Unexpected plural form while parsing at line {line_number}"
                    ));
                }
                if let Some(slot) = msgs_plural.get_mut(plural_index as usize) {
                    slot.push_str(&text);
                }
            }
            _ => {}
        }
    }

    // Add the last message.
    if status == Status::ReadingString {
        if !msg_id.is_empty() {
            if !skip_this {
                translation.add_message(&msg_id, &msg_str, &msg_context);
            }
        } else if config.is_empty() {
            config = msg_str;
        }
    } else if status == Status::ReadingPlural {
        if !skip_this && !msg_id.is_empty() {
            if plural_index != plural_forms - 1 {
                return Err(format!(
                    "Number of 'msgstr[]' doesn't match the plural forms at line {line_number}"
                ));
            }
            translation.add_plural_message(&msg_id, &msgs_plural, &msg_context);
        }
    }

    if config.is_empty() {
        return Err("No config found in file".to_string());
    }
    apply_config(&config, &mut translation);
    Ok(translation)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "msgid \"\"\n\
msgstr \"\"\n\
\"Project-Id-Version: test\\n\"\n\
\"Language: es\\n\"\n\
\"Plural-Forms: nplurals=2; plural=n != 1;\\n\"\n";

    #[test]
    fn simple_text_po() {
        let source = format!(
            "{HEADER}\n\
msgid \"Hello\"\n\
msgstr \"Hola\"\n\
\n\
msgid \"Line\"\n\
msgstr \"\"\n\
\"Línea\\n\"\n\
\"partida\"\n"
        );
        let translation = load_translation(source.as_bytes()).expect("parsed");
        assert_eq!(translation.get_locale(), "es");
        assert_eq!(translation.get_message("Hello", "").as_string(), "Hola");
        // Multi-line strings concatenate after unescaping.
        assert_eq!(
            translation.get_message("Line", "").as_string(),
            "Línea\npartida"
        );
        assert_eq!(translation.get_message("Missing", "").as_string(), "");
    }

    #[test]
    fn contexts_and_plurals() {
        let source = format!(
            "{HEADER}\n\
msgctxt \"menu\"\n\
msgid \"Open\"\n\
msgstr \"Abrir\"\n\
\n\
msgid \"%d file\"\n\
msgid_plural \"%d files\"\n\
msgstr[0] \"%d archivo\"\n\
msgstr[1] \"%d archivos\"\n"
        );
        let translation = load_translation(source.as_bytes()).expect("parsed");
        assert_eq!(
            translation.get_message("Open", "menu").as_string(),
            "Abrir"
        );
        assert_eq!(translation.get_message("Open", "").as_string(), "");
        assert_eq!(
            translation
                .get_plural_message("%d file", "%d files", 1, "")
                .as_string(),
            "%d archivo"
        );
        assert_eq!(
            translation
                .get_plural_message("%d file", "%d files", 3, "")
                .as_string(),
            "%d archivos"
        );
    }

    #[test]
    fn fuzzy_messages_are_skipped() {
        let source = format!(
            "{HEADER}\n\
# fuzzy\n\
msgid \"Sketchy\"\n\
msgstr \"Dudoso\"\n\
\n\
msgid \"Good\"\n\
msgstr \"Bueno\"\n"
        );
        let translation = load_translation(source.as_bytes()).expect("parsed");
        assert_eq!(translation.get_message("Sketchy", "").as_string(), "");
        assert_eq!(translation.get_message("Good", "").as_string(), "Bueno");
    }

    #[test]
    fn errors() {
        // No header configuration.
        assert!(load_translation(b"msgid \"a\"\nmsgstr \"b\"\n").is_err());
        // An unquoted content line is invalid (a quoted one continues the last string).
        assert!(load_translation(format!("{HEADER}\norphan\n").as_bytes()).is_err());
        // msgstr without msgid.
        assert!(load_translation(format!("{HEADER}\nmsgstr \"x\"\n").as_bytes()).is_err());
        // Unterminated string.
        assert!(load_translation(format!("{HEADER}\nmsgid \"a\n").as_bytes()).is_err());
        // msgid_plural without Plural-Forms.
        assert!(load_translation(
            b"msgid \"\"\nmsgstr \"Language: es\\n\"\n\nmsgid \"a\"\nmsgid_plural \"b\"\nmsgstr[0] \"c\"\n"
        )
        .is_err());
    }

    #[test]
    fn binary_mo() {
        // Header id (empty) with translations carrying the config.
        let config = "Language: fr\nPlural-Forms: nplurals=2; plural=n > 1;\n";
        let entries: [(&[u8], &[u8]); 3] = [
            (b"", config.as_bytes()),
            (b"Hello", "Bonjour".as_bytes()),
            (
                b"file\0files",
                "fichier\0fichiers".as_bytes(),
            ),
        ];
        let mut data = Vec::new();
        data.extend_from_slice(&MO_MAGIC.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes()); // major
        data.extend_from_slice(&0u16.to_le_bytes()); // minor
        data.extend_from_slice(&(entries.len() as u32).to_le_bytes());
        data.extend_from_slice(&20u32.to_le_bytes()); // id table
        data.extend_from_slice(&(20 + entries.len() as u32 * 8).to_le_bytes()); // trans table
        let strings_offset = 20 + entries.len() * 16;
        let mut strings = Vec::new();
        let mut id_entries = Vec::new();
        let mut trans_entries = Vec::new();
        let mut offset = strings_offset as u32;
        for (id, trans) in entries {
            id_entries.push((id.len() as u32, offset));
            strings.extend_from_slice(id);
            offset += id.len() as u32;
        }
        for (_, trans) in entries {
            trans_entries.push((trans.len() as u32, offset));
            strings.extend_from_slice(trans);
            offset += trans.len() as u32;
        }
        for (length, offset) in id_entries {
            data.extend_from_slice(&length.to_le_bytes());
            data.extend_from_slice(&offset.to_le_bytes());
        }
        for (length, offset) in trans_entries {
            data.extend_from_slice(&length.to_le_bytes());
            data.extend_from_slice(&offset.to_le_bytes());
        }
        data.extend_from_slice(&strings);

        let translation = load_translation(&data).expect("parsed MO");
        assert_eq!(translation.get_locale(), "fr");
        assert_eq!(translation.get_message("Hello", "").as_string(), "Bonjour");
        assert_eq!(
            translation.get_plural_message("file", "files", 2, "").as_string(),
            "fichiers"
        );
        // A future MO version is refused.
        let mut bad = data.clone();
        bad[4] = 2;
        assert!(load_translation(&bad).is_err());
    }
}
