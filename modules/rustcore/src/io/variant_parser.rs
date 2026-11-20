//! `core/variant/variant_parser.*`: the text form (`parse_value`).
//!
//! Ported: the tokenizer (`;` comments, `{ } [ ] ( ) : , . =`, `#RRGGBB`
//! colours, `&"name"`/`@"name"` string names, quoted strings with the engine's
//! escape set including `\uXXXX`/`\UXXXXXX` and surrogate pairs, the number state
//! machine that decides int vs float, and identifiers), plus `parse_value` for
//! nil/bool/inf/nan literals, arrays, dictionaries and the value constructors
//! (`Vector2/2i`, `Rect2/2i`, `Vector3/3i`, `Vector4/4i`, `Plane`, `Quaternion`,
//! `AABB`, `Basis`, `Transform2D`, `Transform3D`, `Projection`, `Color`,
//! `NodePath`, `StringName`).
//!
//! Two deliberate limits: dictionary keys must be strings (the port's dictionary
//! is keyed by `StringName`), and `NodePath` decodes to `Variant::String` because
//! the port's `Variant` has no node-path payload. Objects, resources, RIDs,
//! Callables, typed containers and tags are not ported.

use crate::math::aabb::Aabb;
use crate::math::basis::Basis;
use crate::math::color::Color;
use crate::math::plane::Plane;
use crate::math::projection::Projection;
use crate::math::quaternion::Quaternion;
use crate::math::rect2::Rect2;
use crate::math::rect2i::Rect2i;
use crate::math::transform_2d::Transform2D;
use crate::math::transform_3d::Transform3D;
use crate::math::vector2::Vector2;
use crate::math::vector2i::Vector2i;
use crate::math::vector3::Vector3;
use crate::math::vector3i::Vector3i;
use crate::math::vector4::Vector4;
use crate::math::vector4i::Vector4i;
use crate::string::gstring::GoString;
use crate::string::string_name::StringName;
use crate::variant::Variant;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (line {})", self.message, self.line)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    CurlyOpen,
    CurlyClose,
    BracketOpen,
    BracketClose,
    ParenOpen,
    ParenClose,
    Colon,
    Comma,
    Period,
    Equal,
    /// An int token (the engine distinguishes int from float).
    Int(i64),
    Float(f64),
    String(String),
    StringName(String),
    Color(Color),
    Identifier(String),
    Eof,
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    line: usize,
}

impl Parser {
    fn error<T>(&self, message: impl Into<String>) -> Result<T, ParseError> {
        Err(ParseError {
            message: message.into(),
            line: self.line,
        })
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
        }
        Some(c)
    }

    fn hex_value(c: char) -> Option<u32> {
        c.to_digit(16)
    }

    fn get_token(&mut self) -> Result<Token, ParseError> {
        loop {
            let Some(c) = self.peek() else {
                return Ok(Token::Eof);
            };
            match c {
                '\n' => {
                    self.bump();
                }
                c if c <= ' ' => {
                    self.bump();
                }
                ';' => {
                    while let Some(ch) = self.bump() {
                        if ch == '\n' {
                            break;
                        }
                    }
                    if self.peek().is_none() {
                        return Ok(Token::Eof);
                    }
                }
                '{' => {
                    self.bump();
                    return Ok(Token::CurlyOpen);
                }
                '}' => {
                    self.bump();
                    return Ok(Token::CurlyClose);
                }
                '[' => {
                    self.bump();
                    return Ok(Token::BracketOpen);
                }
                ']' => {
                    self.bump();
                    return Ok(Token::BracketClose);
                }
                '(' => {
                    self.bump();
                    return Ok(Token::ParenOpen);
                }
                ')' => {
                    self.bump();
                    return Ok(Token::ParenClose);
                }
                ':' => {
                    self.bump();
                    return Ok(Token::Colon);
                }
                ',' => {
                    self.bump();
                    return Ok(Token::Comma);
                }
                '.' => {
                    self.bump();
                    return Ok(Token::Period);
                }
                '=' => {
                    self.bump();
                    return Ok(Token::Equal);
                }
                '#' => {
                    self.bump();
                    let mut text = String::from("#");
                    while let Some(ch) = self.peek() {
                        if ch.is_ascii_hexdigit() {
                            text.push(ch);
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    // Invalid colour codes fall back to opaque black, like
                    // `Color::html`'s `ERR_FAIL_V_MSG(Color(), ...)`.
                    let color = Color::html(&text).unwrap_or(Color::new(0.0, 0.0, 0.0, 1.0));
                    return Ok(Token::Color(color));
                }
                '&' | '@' => {
                    self.bump();
                    if self.peek() != Some('"') {
                        return self.error("Expected '\"' after '&'");
                    }
                    let text = self.parse_quoted()?;
                    return Ok(Token::StringName(text));
                }
                '"' => {
                    let text = self.parse_quoted()?;
                    return Ok(Token::String(text));
                }
                '-' => {
                    // `-inf`/`-nan` are identifiers; a digit starts a number.
                    let next_is_digit = matches!(self.chars.get(self.pos + 1), Some(c) if c.is_ascii_digit());
                    if next_is_digit {
                        return self.parse_number();
                    }
                    self.bump();
                    let mut text = String::from("-");
                    let mut first = true;
                    while let Some(ch) = self.peek() {
                        if ch.is_ascii_alphabetic() || ch == '_' || (!first && ch.is_ascii_digit()) {
                            text.push(ch);
                            self.bump();
                            first = false;
                        } else {
                            break;
                        }
                    }
                    return Ok(Token::Identifier(text));
                }
                '0'..='9' => {
                    return self.parse_number();
                }
                c if c.is_ascii_alphabetic() || c == '_' => {
                    let mut text = String::new();
                    let mut first = true;
                    while let Some(ch) = self.peek() {
                        if ch.is_ascii_alphabetic() || ch == '_' || (!first && ch.is_ascii_digit()) {
                            text.push(ch);
                            self.bump();
                            first = false;
                        } else {
                            break;
                        }
                    }
                    return Ok(Token::Identifier(text));
                }
                _ => {
                    return self.error("Unexpected character");
                }
            }
        }
    }

    fn parse_quoted(&mut self) -> Result<String, ParseError> {
        debug_assert_eq!(self.peek(), Some('"'));
        self.bump();
        let mut out = String::new();
        let mut prev_surrogate: Option<u32> = None;
        loop {
            let Some(ch) = self.bump() else {
                return self.error("Unterminated string");
            };
            match ch {
                '"' => break,
                '\\' => {
                    let Some(next) = self.bump() else {
                        return self.error("Unterminated string");
                    };
                    let mut res: u32 = match next {
                        'b' => 8,
                        't' => 9,
                        'n' => 10,
                        'f' => 12,
                        'r' => 13,
                        'U' | 'u' => {
                            let hex_len = if next == 'U' { 6 } else { 4 };
                            let mut value = 0u32;
                            for _ in 0..hex_len {
                                let Some(c) = self.bump() else {
                                    return self.error("Unterminated string");
                                };
                                match Self::hex_value(c) {
                                    Some(v) => value = (value << 4) | v,
                                    None => return self.error("Malformed hex constant in string"),
                                }
                            }
                            value
                        }
                        other => other as u32,
                    };

                    if (res & 0xffff_fc00) == 0xd800 {
                        if prev_surrogate.is_none() {
                            prev_surrogate = Some(res);
                            continue;
                        }
                        return self
                            .error("Invalid UTF-16 sequence in string, unpaired lead surrogate");
                    } else if (res & 0xffff_fc00) == 0xdc00 {
                        let Some(lead) = prev_surrogate.take() else {
                            return self
                                .error("Invalid UTF-16 sequence in string, unpaired trail surrogate");
                        };
                        res = (lead << 10) + res - ((0xd800 << 10) + 0xdc00 - 0x10000);
                    }
                    if prev_surrogate.is_some() {
                        return self
                            .error("Invalid UTF-16 sequence in string, unpaired lead surrogate");
                    }
                    out.push(char::from_u32(res).unwrap_or('\u{fffd}'));
                }
                other => {
                    if prev_surrogate.is_some() {
                        return self
                            .error("Invalid UTF-16 sequence in string, unpaired lead surrogate");
                    }
                    out.push(other);
                }
            }
        }
        if prev_surrogate.is_some() {
            return self.error("Invalid UTF-16 sequence in string, unpaired lead surrogate");
        }
        Ok(out)
    }

    /// The engine's `READING_INT/DEC/EXP` state machine.
    fn parse_number(&mut self) -> Result<Token, ParseError> {
        #[derive(PartialEq)]
        enum Reading {
            Int,
            Dec,
            Exp,
            Done,
        }

        let mut text = String::new();
        if self.peek() == Some('-') {
            text.push('-');
            self.bump();
        }
        if !matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            return self.error("Unexpected character");
        }

        let mut reading = Reading::Int;
        let mut is_float = false;
        let mut exp_sign = false;
        let mut exp_beg = false;

        while let Some(c) = self.peek() {
            match reading {
                Reading::Int => {
                    if c.is_ascii_digit() {
                    } else if c == '.' {
                        reading = Reading::Dec;
                        is_float = true;
                    } else if c == 'e' || c == 'E' {
                        reading = Reading::Exp;
                        is_float = true;
                    } else {
                        reading = Reading::Done;
                    }
                }
                Reading::Dec => {
                    if c.is_ascii_digit() {
                    } else if c == 'e' || c == 'E' {
                        reading = Reading::Exp;
                    } else {
                        reading = Reading::Done;
                    }
                }
                Reading::Exp => {
                    if c.is_ascii_digit() {
                        exp_beg = true;
                    } else if (c == '-' || c == '+') && !exp_sign && !exp_beg {
                        exp_sign = true;
                    } else {
                        reading = Reading::Done;
                    }
                }
                Reading::Done => break,
            }
            if reading == Reading::Done {
                break;
            }
            text.push(c);
            self.bump();
        }

        if is_float {
            match text.parse::<f64>() {
                Ok(value) => Ok(Token::Float(value)),
                Err(_) => self.error("Invalid number"),
            }
        } else {
            match text.parse::<i64>() {
                Ok(value) => Ok(Token::Int(value)),
                Err(_) => self.error("Invalid number"),
            }
        }
    }

    /// `_parse_construct`: `(` numbers separated by commas `)`.
    fn parse_construct(&mut self) -> Result<Vec<f64>, ParseError> {
        if self.get_token()? != Token::ParenOpen {
            return self.error("Expected '('");
        }
        let mut args = Vec::new();
        loop {
            let token = self.get_token()?;
            if token == Token::ParenClose {
                break;
            }
            match token {
                Token::Int(v) => args.push(v as f64),
                Token::Float(v) => args.push(v),
                _ => return self.error("Expected number"),
            }
            match self.get_token()? {
                Token::ParenClose => break,
                Token::Comma => {}
                _ => return self.error("Expected ','"),
            }
        }
        Ok(args)
    }

    fn parse_value(&mut self, token: Token, depth: usize) -> Result<Variant, ParseError> {
        if depth > 512 {
            return self.error("Variant is too deep");
        }
        match token {
            Token::CurlyOpen => self.parse_dictionary(depth + 1),
            Token::BracketOpen => self.parse_array(depth + 1),
            Token::Color(color) => Ok(Variant::Color(color)),
            Token::String(text) => Ok(Variant::String(GoString::from_str(&text))),
            // The port's Variant has no StringName payload; keep the text.
            Token::StringName(text) => Ok(Variant::String(GoString::from_str(&text))),
            Token::Int(value) => Ok(Variant::Int(value)),
            Token::Float(value) => Ok(Variant::Float(value)),
            Token::Identifier(id) => self.parse_identifier(&id),
            other => self.error(format!("Expected value, got {}", token_name(&other))),
        }
    }

    fn parse_identifier(&mut self, id: &str) -> Result<Variant, ParseError> {
        match id {
            "true" => return Ok(Variant::Bool(true)),
            "false" => return Ok(Variant::Bool(false)),
            "null" | "nil" => return Ok(Variant::Nil),
            "inf" => return Ok(Variant::Float(f64::INFINITY)),
            "-inf" | "inf_neg" => return Ok(Variant::Float(f64::NEG_INFINITY)),
            "nan" => return Ok(Variant::Float(f64::NAN)),
            _ => {}
        }

        // `NodePath("...")` and `StringName("...")` take a string argument.
        if id == "NodePath" || id == "StringName" {
            if self.get_token()? != Token::ParenOpen {
                return self.error("Expected '('");
            }
            let token = self.get_token()?;
            let Token::String(text) = token else {
                return self.error(format!("Expected string as argument for {id}()"));
            };
            if self.get_token()? != Token::ParenClose {
                return self.error("Expected ')'");
            }
            return Ok(Variant::String(GoString::from_str(&text)));
        }

        let args = self.parse_construct()?;
        let arity_error = |parser: &Self, expected: usize| -> Result<Variant, ParseError> {
            parser.error(format!("Expected {expected} arguments for constructor"))
        };

        match id {
            "Vector2" => {
                if args.len() != 2 {
                    return arity_error(self, 2);
                }
                Ok(Variant::Vector2(Vector2::new(args[0] as f32, args[1] as f32)))
            }
            "Vector2i" => {
                if args.len() != 2 {
                    return arity_error(self, 2);
                }
                Ok(Variant::Vector2i(Vector2i::new(args[0] as i32, args[1] as i32)))
            }
            "Rect2" => {
                if args.len() != 4 {
                    return arity_error(self, 4);
                }
                Ok(Variant::Rect2(Rect2::new(
                    Vector2::new(args[0] as f32, args[1] as f32),
                    Vector2::new(args[2] as f32, args[3] as f32),
                )))
            }
            "Rect2i" => {
                if args.len() != 4 {
                    return arity_error(self, 4);
                }
                Ok(Variant::Rect2i(Rect2i::new_xywh(
                    args[0] as i32,
                    args[1] as i32,
                    args[2] as i32,
                    args[3] as i32,
                )))
            }
            "Vector3" => {
                if args.len() != 3 {
                    return arity_error(self, 3);
                }
                Ok(Variant::Vector3(Vector3::new(
                    args[0] as f32,
                    args[1] as f32,
                    args[2] as f32,
                )))
            }
            "Vector3i" => {
                if args.len() != 3 {
                    return arity_error(self, 3);
                }
                Ok(Variant::Vector3i(Vector3i::new(
                    args[0] as i32,
                    args[1] as i32,
                    args[2] as i32,
                )))
            }
            "Vector4" => {
                if args.len() != 4 {
                    return arity_error(self, 4);
                }
                Ok(Variant::Vector4(Vector4::new(
                    args[0] as f32,
                    args[1] as f32,
                    args[2] as f32,
                    args[3] as f32,
                )))
            }
            "Vector4i" => {
                if args.len() != 4 {
                    return arity_error(self, 4);
                }
                Ok(Variant::Vector4i(Vector4i::new(
                    args[0] as i32,
                    args[1] as i32,
                    args[2] as i32,
                    args[3] as i32,
                )))
            }
            "Plane" => {
                if args.len() != 4 {
                    return arity_error(self, 4);
                }
                Ok(Variant::Plane(Plane::new(
                    args[0] as f32,
                    args[1] as f32,
                    args[2] as f32,
                    args[3] as f32,
                )))
            }
            "Quaternion" => {
                if args.len() != 4 {
                    return arity_error(self, 4);
                }
                Ok(Variant::Quaternion(Quaternion::new(
                    args[0] as f32,
                    args[1] as f32,
                    args[2] as f32,
                    args[3] as f32,
                )))
            }
            "AABB" => {
                if args.len() != 6 {
                    return arity_error(self, 6);
                }
                Ok(Variant::Aabb(Aabb::new(
                    Vector3::new(args[0] as f32, args[1] as f32, args[2] as f32),
                    Vector3::new(args[3] as f32, args[4] as f32, args[5] as f32),
                )))
            }
            "Basis" => {
                if args.len() != 9 {
                    return arity_error(self, 9);
                }
                let mut basis = Basis::IDENTITY;
                for column in 0..3 {
                    basis.set_column(
                        column,
                        Vector3::new(
                            args[column * 3] as f32,
                            args[column * 3 + 1] as f32,
                            args[column * 3 + 2] as f32,
                        ),
                    );
                }
                Ok(Variant::Basis(basis))
            }
            "Transform2D" => {
                if args.len() != 6 {
                    return arity_error(self, 6);
                }
                Ok(Variant::Transform2D(Transform2D {
                    columns: [
                        Vector2::new(args[0] as f32, args[1] as f32),
                        Vector2::new(args[2] as f32, args[3] as f32),
                        Vector2::new(args[4] as f32, args[5] as f32),
                    ],
                }))
            }
            "Transform3D" => {
                if args.len() != 12 {
                    return arity_error(self, 12);
                }
                let mut basis = Basis::IDENTITY;
                for column in 0..3 {
                    basis.set_column(
                        column,
                        Vector3::new(
                            args[column * 3] as f32,
                            args[column * 3 + 1] as f32,
                            args[column * 3 + 2] as f32,
                        ),
                    );
                }
                Ok(Variant::Transform3D(Transform3D {
                    basis,
                    origin: Vector3::new(args[9] as f32, args[10] as f32, args[11] as f32),
                }))
            }
            "Projection" => {
                if args.len() != 16 {
                    return arity_error(self, 16);
                }
                let mut projection = Projection::default();
                for column in 0..4 {
                    projection.columns[column] = Vector4::new(
                        args[column * 4] as f32,
                        args[column * 4 + 1] as f32,
                        args[column * 4 + 2] as f32,
                        args[column * 4 + 3] as f32,
                    );
                }
                Ok(Variant::Projection(projection))
            }
            "Color" => {
                if args.len() != 4 {
                    return arity_error(self, 4);
                }
                Ok(Variant::Color(Color::new(
                    args[0] as f32,
                    args[1] as f32,
                    args[2] as f32,
                    args[3] as f32,
                )))
            }
            other => self.error(format!("Unknown constructor '{other}'")),
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<Variant, ParseError> {
        let mut values = Vec::new();
        let mut need_comma = false;
        loop {
            let token = self.get_token()?;
            if token == Token::Eof {
                return self.error("Unexpected EOF while parsing array");
            }
            if token == Token::BracketClose {
                return Ok(Variant::Array(values));
            }
            if need_comma {
                if token != Token::Comma {
                    return self.error("Expected ','");
                }
                need_comma = false;
                continue;
            }
            values.push(self.parse_value(token, depth)?);
            need_comma = true;
        }
    }

    fn parse_dictionary(&mut self, depth: usize) -> Result<Variant, ParseError> {
        let mut map: HashMap<StringName, Variant> = HashMap::new();
        let mut need_comma = false;
        loop {
            let token = self.get_token()?;
            match token {
                Token::Eof => return self.error("Unexpected EOF while parsing dictionary"),
                Token::CurlyClose => return Ok(Variant::Dictionary(map)),
                _ => {}
            }
            if need_comma {
                if token != Token::Comma {
                    return self.error("Expected '}' or ','");
                }
                need_comma = false;
                continue;
            }

            let key = self.parse_value(token, depth)?;
            let key_text = match &key {
                Variant::String(text) => text.as_string(),
                _ => {
                    return self.error("Dictionary keys must be strings in this port");
                }
            };
            if self.get_token()? != Token::Colon {
                return self.error("Expected ':'");
            }
            let value_token = self.get_token()?;
            let value = self.parse_value(value_token, depth)?;
            map.insert(StringName::new(&key_text), value);
            need_comma = true;
        }
    }
}

fn token_name(token: &Token) -> &'static str {
    match token {
        Token::CurlyOpen => "'{'",
        Token::CurlyClose => "'}'",
        Token::BracketOpen => "'['",
        Token::BracketClose => "']'",
        Token::ParenOpen => "'('",
        Token::ParenClose => "')'",
        Token::Colon => "':'",
        Token::Comma => "','",
        Token::Period => "'.'",
        Token::Equal => "'='",
        Token::Int(_) | Token::Float(_) => "number",
        Token::String(_) => "string",
        Token::StringName(_) => "string name",
        Token::Color(_) => "color",
        Token::Identifier(_) => "identifier",
        Token::Eof => "EOF",
    }
}

/// `VariantParser::parse`: one value, then EOF (trailing whitespace/comments are
/// allowed).
pub fn parse(text: &str) -> Result<Variant, ParseError> {
    let mut parser = Parser {
        chars: text.chars().collect(),
        pos: 0,
        line: 0,
    };
    let token = parser.get_token()?;
    let value = parser.parse_value(token, 0)?;
    if parser.get_token()? != Token::Eof {
        return parser.error("Expected end of text");
    }
    Ok(value)
}

/// Parses one value and returns how many characters of `text` it consumed, so
/// callers such as `ConfigFile` can continue after it. Trailing whitespace and
/// comments are left for the caller.
pub fn parse_prefix(text: &str) -> Result<(Variant, usize), ParseError> {
    let mut parser = Parser {
        chars: text.chars().collect(),
        pos: 0,
        line: 0,
    };
    let token = parser.get_token()?;
    let value = parser.parse_value(token, 0)?;
    Ok((value, parser.pos))
}

/// Parses a constructor's argument list from `text` (used by tests and by
/// callers that already consumed the identifier).
pub fn parse_construct_args(text: &str) -> Result<Vec<f64>, ParseError> {
    let mut parser = Parser {
        chars: text.chars().collect(),
        pos: 0,
        line: 0,
    };
    let args = parser.parse_construct()?;
    if parser.get_token()? != Token::Eof {
        return parser.error("Expected end of text");
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalars_and_literals() {
        assert_eq!(parse("null").unwrap(), Variant::Nil);
        assert_eq!(parse("nil").unwrap(), Variant::Nil);
        assert_eq!(parse("true").unwrap(), Variant::Bool(true));
        assert_eq!(parse("false").unwrap(), Variant::Bool(false));
        assert_eq!(parse("42").unwrap(), Variant::Int(42));
        assert_eq!(parse("-7").unwrap(), Variant::Int(-7));
        assert_eq!(parse("1.5").unwrap(), Variant::Float(1.5));
        assert_eq!(parse("-2e3").unwrap(), Variant::Float(-2000.0));
        assert_eq!(parse("1E+2").unwrap(), Variant::Float(100.0));
        assert_eq!(
            parse("\"hi\"").unwrap(),
            Variant::String(GoString::from_str("hi"))
        );
        match parse("inf").unwrap() {
            Variant::Float(v) => assert!(v.is_infinite() && v > 0.0),
            other => panic!("{other:?}"),
        }
        match parse("-inf").unwrap() {
            Variant::Float(v) => assert!(v.is_infinite() && v < 0.0),
            other => panic!("{other:?}"),
        }
        match parse("nan").unwrap() {
            Variant::Float(v) => assert!(v.is_nan()),
            other => panic!("{other:?}"),
        }
        // A trailing `;` comment is allowed.
        assert_eq!(parse("1 ; comment").unwrap(), Variant::Int(1));
    }

    #[test]
    fn strings_and_names() {
        assert_eq!(
            parse(r#""a\nb\t\"c\\d""#).unwrap(),
            Variant::String(GoString::from_str("a\nb\t\"c\\d"))
        );
        assert_eq!(
            parse(r#""\u00e9""#).unwrap(),
            Variant::String(GoString::from_str("é"))
        );
        assert_eq!(
            parse(r#""\U01F600""#).unwrap(),
            Variant::String(GoString::from_str("😀"))
        );
        assert_eq!(
            parse(r#"&"my_name""#).unwrap(),
            Variant::String(GoString::from_str("my_name"))
        );
        // 3.x compatibility prefix.
        assert_eq!(
            parse(r#"@"old_name""#).unwrap(),
            Variant::String(GoString::from_str("old_name"))
        );
        assert_eq!(
            parse(r#"NodePath("A/B")"#).unwrap(),
            Variant::String(GoString::from_str("A/B"))
        );
        assert_eq!(
            parse(r#"StringName("hero")"#).unwrap(),
            Variant::String(GoString::from_str("hero"))
        );
    }

    #[test]
    fn colors() {
        assert_eq!(parse("#ff0000").unwrap(), Variant::Color(Color::rgb(1.0, 0.0, 0.0)));
        // Eight digits are RRGGBBAA (alpha last).
        assert_eq!(
            parse("#ff000080").unwrap(),
            Variant::Color(Color::new(1.0, 0.0, 0.0, 128.0 / 255.0))
        );
        assert_eq!(
            parse("Color(0.5, 0.25, 1, 1)").unwrap(),
            Variant::Color(Color::new(0.5, 0.25, 1.0, 1.0))
        );
    }

    #[test]
    fn arrays_and_dictionaries() {
        let value = parse("[1, 2.5, \"x\", null]").unwrap();
        let Variant::Array(items) = value else {
            panic!("not an array")
        };
        assert_eq!(items.len(), 4);
        assert_eq!(items[0], Variant::Int(1));
        assert_eq!(items[1], Variant::Float(2.5));
        assert_eq!(items[3], Variant::Nil);

        let empty = parse("[]").unwrap();
        assert_eq!(empty, Variant::Array(vec![]));

        let dict = parse("{\"a\": 1, \"b\": [true]}").unwrap();
        assert_eq!(dict.dict_get("a"), Some(&Variant::Int(1)));
        let Variant::Array(inner) = dict.dict_get("b").unwrap() else {
            panic!("not an array")
        };
        assert_eq!(inner[0], Variant::Bool(true));

        // Trailing commas are accepted by the comma-then-close loop.
        assert_eq!(parse("[1,]").unwrap(), Variant::Array(vec![Variant::Int(1)]));
        assert_eq!(
            parse("{\"a\": 1,}").unwrap().dict_get("a"),
            Some(&Variant::Int(1))
        );
    }

    #[test]
    fn math_constructors() {
        assert_eq!(
            parse("Vector2(1, 2)").unwrap(),
            Variant::Vector2(Vector2::new(1.0, 2.0))
        );
        assert_eq!(
            parse("Vector2i(-1, 2)").unwrap(),
            Variant::Vector2i(Vector2i::new(-1, 2))
        );
        assert_eq!(
            parse("Vector3(1, 2, 3)").unwrap(),
            Variant::Vector3(Vector3::new(1.0, 2.0, 3.0))
        );
        assert_eq!(
            parse("Rect2(1, 2, 3, 4)").unwrap(),
            Variant::Rect2(Rect2::new(Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0)))
        );
        assert_eq!(
            parse("Quaternion(0, 0, 0, 1)").unwrap(),
            Variant::Quaternion(Quaternion::new(0.0, 0.0, 0.0, 1.0))
        );
        assert_eq!(
            parse("AABB(1, 2, 3, 4, 5, 6)").unwrap(),
            Variant::Aabb(Aabb::new(Vector3::new(1.0, 2.0, 3.0), Vector3::new(4.0, 5.0, 6.0)))
        );
        assert_eq!(
            parse("Transform3D(1, 0, 0, 0, 1, 0, 0, 0, 1, 5, 6, 7)").unwrap(),
            Variant::Transform3D(Transform3D {
                basis: Basis::IDENTITY,
                origin: Vector3::new(5.0, 6.0, 7.0),
            })
        );
        assert_eq!(
            parse("Plane(0, 1, 0, -2)").unwrap(),
            Variant::Plane(Plane::new(0.0, 1.0, 0.0, -2.0))
        );
    }

    #[test]
    fn parameter_and_nesting_errors() {
        let e = parse("Vector2(1)").unwrap_err();
        assert_eq!(e.message, "Expected 2 arguments for constructor");
        let e = parse("Vector3(1, 2)").unwrap_err();
        assert_eq!(e.message, "Expected 3 arguments for constructor");
        let e = parse("Nope(1)").unwrap_err();
        assert_eq!(e.message, "Unknown constructor 'Nope'");
        let e = parse("[1 2]").unwrap_err();
        assert_eq!(e.message, "Expected ','");
        let e = parse("{\"a\" 1}").unwrap_err();
        assert_eq!(e.message, "Expected ':'");
        let e = parse("{\"a\":1 \"b\":2}").unwrap_err();
        assert_eq!(e.message, "Expected '}' or ','");
        let e = parse("[1").unwrap_err();
        assert_eq!(e.message, "Unexpected EOF while parsing array");
        assert_eq!(parse("").unwrap_err().message, "Expected value, got EOF");
        assert_eq!(parse("1 2").unwrap_err().message, "Expected end of text");
        assert_eq!(parse("$").unwrap_err().message, "Unexpected character");
        assert_eq!(parse("\"x").unwrap_err().message, "Unterminated string");
        assert_eq!(
            parse(r#""\ud83d""#).unwrap_err().message,
            "Invalid UTF-16 sequence in string, unpaired lead surrogate"
        );
        // Non-string dictionary keys are refused by this port.
        assert_eq!(
            parse("{1: 2}").unwrap_err().message,
            "Dictionary keys must be strings in this port"
        );
    }

    #[test]
    fn construct_args_helper() {
        assert_eq!(parse_construct_args("(1, 2.5, -3)").unwrap(), vec![1.0, 2.5, -3.0]);
        assert_eq!(parse_construct_args("()").unwrap(), Vec::<f64>::new());
        assert!(parse_construct_args("(1 2)").is_err());
        assert!(parse_construct_args("1").is_err());
    }
}
