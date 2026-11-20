//! `core/math/expression.*`: tokenizer, parser and evaluator.
//!
//! Values are `Variant`s and all arithmetic/comparison goes through
//! `variant::ops` and `variant::compare`, so the semantics are the port's, not a
//! second set invented here.
//!
//! Implemented: numeric/bool/string literals, array and dictionary literals,
//! variables, indexing, unary `- ! ~`, `+ - * / % **`, comparisons, `and/or/not`,
//! bitwise and shifts, and a subset of the math builtins.
//! Not implemented: `self`/member access, assignment, `in`/`not in`, and the
//! engine's full 100-function builtin table.
//!
//! Non-differential: `expression.cpp` is bound through `ClassDB`.

use crate::variant::compare::{compare, equals, get_index, stringify};
use crate::variant::ops::{apply, Op};
use crate::variant::{Variant, VariantType};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    In,
    NotIn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnOp {
    Neg,
    Not,
    BitNot,
}

#[derive(Clone, Debug)]
enum Node {
    Const(Variant),
    Var(String),
    SelfRef,
    Member(Box<Node>, String),
    MethodCall(Box<Node>, String, Vec<Node>),
    Array(Vec<Node>),
    Dict(Vec<(Node, Node)>),
    Unary(UnOp, Box<Node>),
    Binary(BinOp, Box<Node>, Box<Node>),
    Call(String, Vec<Node>),
    Index(Box<Node>, Box<Node>),
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(Variant),
    Str(String),
    Ident(String),
    Op(&'static str),
    End,
}

pub struct Expression {
    root: Node,
    inputs: Vec<String>,
}

// ---------------------------------------------------------------------------
// Tokenizer

struct Lexer<'a> {
    chars: Vec<char>,
    pos: usize,
    _src: &'a str,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            chars: src.chars().collect(),
            pos: 0,
            _src: src,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next_token(&mut self) -> Result<Token, String> {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }

        let Some(c) = self.peek() else {
            return Ok(Token::End);
        };
        self.pos += 1;

        match c {
            '0'..='9' => {
                self.pos -= 1;
                self.read_number()
            }
            '.' if matches!(self.peek(), Some('0'..='9')) => {
                self.pos -= 1;
                self.read_number()
            }
            '"' | '\'' => self.read_string(c),
            'a'..='z' | 'A'..='Z' | '_' => {
                self.pos -= 1;
                self.read_ident()
            }
            _ => self.read_operator(c),
        }
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let start = self.pos;
        let mut has_dot = false;
        let mut has_exp = false;

        while let Some(c) = self.peek() {
            match c {
                '0'..='9' => self.pos += 1,
                '.' if !has_dot && !has_exp => {
                    has_dot = true;
                    self.pos += 1;
                }
                'e' | 'E' if !has_exp => {
                    has_exp = true;
                    self.pos += 1;
                    if matches!(self.peek(), Some('+') | Some('-')) {
                        self.pos += 1;
                    }
                }
                _ => break,
            }
        }

        let text: String = self.chars[start..self.pos].iter().collect();
        if text.is_empty() || text == "." {
            return Err(format!("Invalid number at position {start}"));
        }

        if has_dot || has_exp {
            text.parse::<f64>()
                .map(|v| Token::Number(Variant::Float(v)))
                .map_err(|_| format!("Invalid float '{text}'"))
        } else {
            text.parse::<i64>()
                .map(|v| Token::Number(Variant::Int(v)))
                .map_err(|_| format!("Invalid integer '{text}'"))
        }
    }

    fn read_string(&mut self, quote: char) -> Result<Token, String> {
        let mut out = String::new();
        loop {
            let Some(c) = self.peek() else {
                return Err("Unterminated string".into());
            };
            self.pos += 1;
            if c == quote {
                return Ok(Token::Str(out));
            }
            if c == '\\' {
                let Some(esc) = self.peek() else {
                    return Err("Unterminated escape".into());
                };
                self.pos += 1;
                out.push(match esc {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    other => other,
                });
            } else {
                out.push(c);
            }
        }
    }

    fn read_ident(&mut self) -> Result<Token, String> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        Ok(Token::Ident(self.chars[start..self.pos].iter().collect()))
    }

    fn read_operator(&mut self, c: char) -> Result<Token, String> {
        let two: String = {
            let mut s = String::new();
            s.push(c);
            if let Some(n) = self.peek() {
                s.push(n);
            }
            s
        };

        let op: &'static str = match two.as_str() {
            "==" => "==",
            "!=" => "!=",
            "<=" => "<=",
            ">=" => ">=",
            "&&" => "&&",
            "||" => "||",
            "**" => "**",
            "<<" => "<<",
            ">>" => ">>",
            _ => match c {
                '+' => "+",
                '-' => "-",
                '*' => "*",
                '/' => "/",
                '%' => "%",
                '<' => "<",
                '>' => ">",
                '!' => "!",
                '&' => "&",
                '|' => "|",
                '^' => "^",
                '~' => "~",
                '(' => "(",
                ')' => ")",
                '[' => "[",
                ']' => "]",
                '{' => "{",
                '}' => "}",
                ',' => ",",
                ':' => ":",
                '.' => ".",
                _ => return Err(format!("Unexpected character '{c}'")),
            },
        };

        if op.len() == 2 {
            self.pos += 1; // consume the second character
        }
        Ok(Token::Op(op))
    }
}

// ---------------------------------------------------------------------------
// Parser

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    inputs: Vec<String>,
}

fn precedence(op: BinOp) -> (u8, bool) {
    // (binding power, right-associative)
    match op {
        BinOp::Or => (1, false),
        BinOp::And => (2, false),
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::In | BinOp::NotIn => (3, false),
        BinOp::BitOr => (4, false),
        BinOp::BitXor => (5, false),
        BinOp::BitAnd => (6, false),
        BinOp::Shl | BinOp::Shr => (7, false),
        BinOp::Add | BinOp::Sub => (8, false),
        BinOp::Mul | BinOp::Div | BinOp::Mod => (9, false),
        BinOp::Pow => (10, true),
    }
}

impl Parser {
    fn new(src: &str) -> Result<Self, String> {
        let mut lexer = Lexer::new(src);
        let mut tokens = Vec::new();
        loop {
            let t = lexer.next_token()?;
            let end = t == Token::End;
            tokens.push(t);
            if end {
                break;
            }
        }
        Ok(Self {
            tokens,
            pos: 0,
            inputs: Vec::new(),
        })
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn eat_op(&mut self, op: &str) -> bool {
        if let Token::Op(text) = self.peek() {
            if *text == op {
                self.pos += 1;
                return true;
            }
        }
        false
    }

    fn expect_op(&mut self, op: &'static str) -> Result<(), String> {
        if self.eat_op(op) {
            Ok(())
        } else {
            Err(format!("Expected '{op}'"))
        }
    }

    fn parse_expression(&mut self, min_bp: u8) -> Result<Node, String> {
        let mut lhs = self.parse_unary()?;

        loop {
            let (op, advance) = match self.peek().clone() {
                Token::Op(text) => match binop_from(text) {
                    Some(op) => (op, 1),
                    None => break,
                },
                // `and` / `or` are keywords, not operator tokens.
                Token::Ident(id) if id == "and" => (BinOp::And, 1),
                Token::Ident(id) if id == "or" => (BinOp::Or, 1),
                Token::Ident(id) if id == "in" => (BinOp::In, 1),
                Token::Ident(id) if id == "not"
                    && matches!(self.tokens.get(self.pos + 1), Some(Token::Ident(n)) if n == "in") =>
                {
                    (BinOp::NotIn, 2)
                }
                _ => break,
            };
            let (bp, right) = precedence(op);
            if bp < min_bp {
                break;
            }
            self.pos += advance;
            let next_min = if right { bp } else { bp + 1 };
            let rhs = self.parse_expression(next_min)?;
            lhs = Node::Binary(op, Box::new(lhs), Box::new(rhs));
        }

        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Node, String> {
        match self.peek().clone() {
            Token::Op("-") => {
                self.pos += 1;
                Ok(Node::Unary(UnOp::Neg, Box::new(self.parse_unary()?)))
            }
            Token::Op("!") => {
                self.pos += 1;
                Ok(Node::Unary(UnOp::Not, Box::new(self.parse_unary()?)))
            }
            Token::Op("~") => {
                self.pos += 1;
                Ok(Node::Unary(UnOp::BitNot, Box::new(self.parse_unary()?)))
            }
            Token::Ident(ref id) if id == "not" => {
                self.pos += 1;
                Ok(Node::Unary(UnOp::Not, Box::new(self.parse_unary()?)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Node, String> {
        let mut node = self.parse_primary()?;

        loop {
            if self.eat_op("[") {
                let index = self.parse_expression(0)?;
                self.expect_op("]")?;
                node = Node::Index(Box::new(node), Box::new(index));
            } else if self.eat_op(".") {
                let name = match self.peek().clone() {
                    Token::Ident(n) => {
                        self.pos += 1;
                        n
                    }
                    _ => return Err("Expected a member name after '.'".into()),
                };
                node = Node::Member(Box::new(node), name);
            } else if self.eat_op("(") {
                // Parse the argument list first: both named functions and
                // object methods share it.
                let mut args = Vec::new();
                if !self.eat_op(")") {
                    loop {
                        args.push(self.parse_expression(0)?);
                        if self.eat_op(",") {
                            continue;
                        }
                        self.expect_op(")")?;
                        break;
                    }
                }
                node = match node {
                    Node::Var(name) => Node::Call(name, args),
                    Node::Member(base, method) => Node::MethodCall(base, method, args),
                    _ => {
                        return Err(
                            "Only named functions and object methods can be called".into(),
                        )
                    }
                };
            } else {
                break;
            }
        }

        Ok(node)
    }

    fn parse_primary(&mut self) -> Result<Node, String> {
        match self.peek().clone() {
            Token::Number(v) => {
                self.pos += 1;
                Ok(Node::Const(v))
            }
            Token::Str(s) => {
                self.pos += 1;
                Ok(Node::Const(Variant::String(crate::string::gstring::GoString::from_str(
                    &s,
                ))))
            }
            Token::Ident(id) => {
                self.pos += 1;
                match id.as_str() {
                    "true" => Ok(Node::Const(Variant::Bool(true))),
                    "false" => Ok(Node::Const(Variant::Bool(false))),
                    "null" | "nil" => Ok(Node::Const(Variant::Nil)),
                    "self" => Ok(Node::SelfRef),
                    "PI" => Ok(Node::Const(Variant::Float(std::f64::consts::PI))),
                    "TAU" => Ok(Node::Const(Variant::Float(std::f64::consts::TAU))),
                    "INF" => Ok(Node::Const(Variant::Float(f64::INFINITY))),
                    "NAN" => Ok(Node::Const(Variant::Float(f64::NAN))),
                    _ => {
                        if !self.inputs.contains(&id) {
                            self.inputs.push(id.clone());
                        }
                        Ok(Node::Var(id))
                    }
                }
            }
            Token::Op("(") => {
                self.pos += 1;
                let e = self.parse_expression(0)?;
                self.expect_op(")")?;
                Ok(e)
            }
            Token::Op("[") => {
                self.pos += 1;
                let mut items = Vec::new();
                if !self.eat_op("]") {
                    loop {
                        items.push(self.parse_expression(0)?);
                        if self.eat_op(",") {
                            continue;
                        }
                        self.expect_op("]")?;
                        break;
                    }
                }
                Ok(Node::Array(items))
            }
            Token::Op("{") => {
                self.pos += 1;
                let mut pairs = Vec::new();
                if !self.eat_op("}") {
                    loop {
                        let key = self.parse_expression(0)?;
                        self.expect_op(":")?;
                        let value = self.parse_expression(0)?;
                        pairs.push((key, value));
                        if self.eat_op(",") {
                            continue;
                        }
                        self.expect_op("}")?;
                        break;
                    }
                }
                Ok(Node::Dict(pairs))
            }
            other => Err(format!("Unexpected token {other:?}")),
        }
    }
}

fn binop_from(text: &str) -> Option<BinOp> {
    Some(match text {
        "+" => BinOp::Add,
        "-" => BinOp::Sub,
        "*" => BinOp::Mul,
        "/" => BinOp::Div,
        "%" => BinOp::Mod,
        "**" => BinOp::Pow,
        "==" => BinOp::Eq,
        "!=" => BinOp::Ne,
        "<" => BinOp::Lt,
        "<=" => BinOp::Le,
        ">" => BinOp::Gt,
        ">=" => BinOp::Ge,
        "&&" | "and" => BinOp::And,
        "||" | "or" => BinOp::Or,
        "&" => BinOp::BitAnd,
        "|" => BinOp::BitOr,
        "^" => BinOp::BitXor,
        "<<" => BinOp::Shl,
        ">>" => BinOp::Shr,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Evaluator

fn eval(node: &Node, inputs: &HashMap<String, Variant>) -> Result<Variant, String> {
    match node {
        Node::Const(v) => Ok(v.clone()),
        Node::Var(name) => inputs
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Unknown input '{name}'")),
        Node::SelfRef => inputs
            .get("self")
            .cloned()
            .ok_or_else(|| "'self' is not set".to_string()),
        Node::Member(base, name) => {
            let v = eval(base, inputs)?;
            match v {
                Variant::Object(id) => crate::object::class_db::property_get(id, name)
                    .ok_or_else(|| format!("Object has no property '{name}'")),
                _ => Err("Member access needs an object".into()),
            }
        }
        Node::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for i in items {
                out.push(eval(i, inputs)?);
            }
            Ok(Variant::Array(out))
        }
        Node::Dict(pairs) => {
            let mut map = HashMap::new();
            for (k, v) in pairs {
                let key = eval(k, inputs)?;
                let name = crate::string::string_name::StringName::new(&stringify(&key).as_string());
                map.insert(name, eval(v, inputs)?);
            }
            Ok(Variant::Dictionary(map))
        }
        Node::Unary(op, inner) => {
            let v = eval(inner, inputs)?;
            match op {
                UnOp::Neg => match v {
                    Variant::Int(i) => Ok(Variant::Int(i.wrapping_neg())),
                    Variant::Float(f) => Ok(Variant::Float(-f)),
                    Variant::Vector2(a) => Ok(Variant::Vector2(-a)),
                    Variant::Vector3(a) => Ok(Variant::Vector3(-a)),
                    _ => Err("Cannot negate this value".into()),
                },
                UnOp::Not => Ok(Variant::Bool(!v.booleanize())),
                UnOp::BitNot => match v {
                    Variant::Int(i) => Ok(Variant::Int(!i)),
                    _ => Err("Cannot bit-invert a non-integer".into()),
                },
            }
        }
        Node::Binary(op, l, r) => eval_binary(*op, l, r, inputs),
        Node::Call(name, args) => {
            let mut values = Vec::with_capacity(args.len());
            for a in args {
                values.push(eval(a, inputs)?);
            }
            call_builtin(name, &values)
        }
        Node::MethodCall(base, method, args) => {
            let object = eval(base, inputs)?;
            let Variant::Object(id) = object else {
                return Err("Method call needs an object".into());
            };
            let mut values = Vec::with_capacity(args.len());
            for a in args {
                values.push(eval(a, inputs)?);
            }
            crate::object::class_db::call(id, method, values)
                .map_err(|e| format!("Method call failed: {e:?}"))
        }
        Node::Index(container, index) => {
            let c = eval(container, inputs)?;
            let i = eval(index, inputs)?;
            get_index(&c, &i).ok_or_else(|| "Value is not indexable".into())
        }
    }
}

fn eval_binary(
    op: BinOp,
    l: &Node,
    r: &Node,
    inputs: &HashMap<String, Variant>,
) -> Result<Variant, String> {
    // Short-circuit before evaluating the right side.
    if op == BinOp::And || op == BinOp::Or {
        let left = eval(l, inputs)?.booleanize();
        if op == BinOp::And && !left {
            return Ok(Variant::Bool(false));
        }
        if op == BinOp::Or && left {
            return Ok(Variant::Bool(true));
        }
        let right = eval(r, inputs)?.booleanize();
        return Ok(Variant::Bool(right));
    }

    let a = eval(l, inputs)?;
    let b = eval(r, inputs)?;

    let arith = match op {
        BinOp::Add => Some(Op::Add),
        BinOp::Sub => Some(Op::Sub),
        BinOp::Mul => Some(Op::Mul),
        BinOp::Div => Some(Op::Div),
        _ => None,
    };
    if let Some(aop) = arith {
        return apply(&a, &b, aop).ok_or_else(|| "Invalid operands".into());
    }

    match op {
        BinOp::In | BinOp::NotIn => {
            let contained = match (&a, &b) {
                (Variant::String(needle), Variant::String(hay)) => hay.find(needle, 0) != -1,
                (v, Variant::Array(items)) => items.iter().any(|x| equals(x, v) == Some(true)),
                (k, Variant::Dictionary(map)) => map.contains_key(
                    &crate::string::string_name::StringName::new(&stringify(k).as_string()),
                ),
                _ => false,
            };
            Ok(Variant::Bool(if op == BinOp::In { contained } else { !contained }))
        }
        BinOp::Mod => match (&a, &b) {
            (Variant::Int(x), Variant::Int(y)) => {
                if *y == 0 {
                    Err("Division by zero".into())
                } else {
                    Ok(Variant::Int(x % y))
                }
            }
            (Variant::Float(x), Variant::Float(y)) => Ok(Variant::Float(x % y)),
            _ => Err("Invalid modulo operands".into()),
        },
        BinOp::Pow => Ok(Variant::Float(a.to_float().powf(b.to_float()))),
        BinOp::Eq => equals(&a, &b)
            .map(Variant::Bool)
            .ok_or_else(|| "Cannot compare these values".into()),
        BinOp::Ne => equals(&a, &b)
            .map(|v| Variant::Bool(!v))
            .ok_or_else(|| "Cannot compare these values".into()),
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            let ord = compare(&a, &b).ok_or_else(|| "Cannot order these values".to_string())?;
            use std::cmp::Ordering::*;
            let res = match op {
                BinOp::Lt => ord == Less,
                BinOp::Le => ord != Greater,
                BinOp::Gt => ord == Greater,
                BinOp::Ge => ord != Less,
                _ => unreachable!(),
            };
            Ok(Variant::Bool(res))
        }
        BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => {
            let (Variant::Int(x), Variant::Int(y)) = (&a, &b) else {
                return Err("Bitwise operators need integers".into());
            };
            Ok(Variant::Int(match op {
                BinOp::BitAnd => x & y,
                BinOp::BitOr => x | y,
                BinOp::BitXor => x ^ y,
                BinOp::Shl => x.wrapping_shl(*y as u32),
                BinOp::Shr => x.wrapping_shr(*y as u32),
                _ => unreachable!(),
            }))
        }
        BinOp::And | BinOp::Or => unreachable!("handled above"),
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => unreachable!("handled above"),
    }
}

fn call_builtin(name: &str, args: &[Variant]) -> Result<Variant, String> {
    let arity = |n: usize| -> Result<(), String> {
        if args.len() == n {
            Ok(())
        } else {
            Err(format!("{name}() expects {n} arguments"))
        }
    };

    match name {
        "abs" => {
            arity(1)?;
            Ok(match &args[0] {
                Variant::Int(i) => Variant::Int(i.abs()),
                other => Variant::Float(other.to_float().abs()),
            })
        }
        "sign" => {
            arity(1)?;
            Ok(Variant::Int(args[0].to_float().signum() as i64))
        }
        "floor" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float().floor()))
        }
        "ceil" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float().ceil()))
        }
        "round" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float().round()))
        }
        "sqrt" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float().sqrt()))
        }
        "exp" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float().exp()))
        }
        "log" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float().ln()))
        }
        "sin" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float().sin()))
        }
        "cos" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float().cos()))
        }
        "tan" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float().tan()))
        }
        "asin" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float().asin()))
        }
        "acos" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float().acos()))
        }
        "atan" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float().atan()))
        }
        "atan2" => {
            arity(2)?;
            Ok(Variant::Float(args[0].to_float().atan2(args[1].to_float())))
        }
        "pow" => {
            arity(2)?;
            Ok(Variant::Float(args[0].to_float().powf(args[1].to_float())))
        }
        "min" => {
            arity(2)?;
            Ok(if args[0].to_float() <= args[1].to_float() {
                args[0].clone()
            } else {
                args[1].clone()
            })
        }
        "max" => {
            arity(2)?;
            Ok(if args[0].to_float() >= args[1].to_float() {
                args[0].clone()
            } else {
                args[1].clone()
            })
        }
        "clamp" => {
            arity(3)?;
            let v = args[0].to_float();
            let lo = args[1].to_float();
            let hi = args[2].to_float();
            Ok(Variant::Float(v.clamp(lo, hi)))
        }
        "lerp" => {
            arity(3)?;
            let a = args[0].to_float();
            let b = args[1].to_float();
            let t = args[2].to_float();
            Ok(Variant::Float(a + (b - a) * t))
        }
        "deg_to_rad" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float().to_radians()))
        }
        "rad_to_deg" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float().to_degrees()))
        }
        "bool" => {
            arity(1)?;
            Ok(Variant::Bool(args[0].booleanize()))
        }
        "int" => {
            arity(1)?;
            Ok(Variant::Int(args[0].to_int()))
        }
        "float" => {
            arity(1)?;
            Ok(Variant::Float(args[0].to_float()))
        }
        "str" => {
            arity(1)?;
            Ok(Variant::String(stringify(&args[0])))
        }
        _ => Err(format!("Unknown function '{name}'")),
    }
}

impl Expression {
    pub fn parse(src: &str) -> Result<Self, String> {
        let mut parser = Parser::new(src)?;
        let root = parser.parse_expression(0)?;
        if parser.peek() != &Token::End {
            return Err(format!("Unexpected trailing token {:?}", parser.peek()));
        }
        Ok(Self {
            root,
            inputs: parser.inputs,
        })
    }

    pub fn input_names(&self) -> &[String] {
        &self.inputs
    }

    pub fn execute(&self, inputs: &HashMap<String, Variant>) -> Result<Variant, String> {
        eval(&self.root, inputs)
    }

    /// Convenience for tests and callers with a single named input.
    pub fn execute_with(&self, name: &str, value: Variant) -> Result<Variant, String> {
        let mut map = HashMap::new();
        map.insert(name.to_string(), value);
        self.execute(&map)
    }
}

/// Numeric type conversion helper used by callers of the evaluator.
pub fn as_number(v: &Variant) -> Option<f64> {
    match v {
        Variant::Int(i) => Some(*i as f64),
        Variant::Float(f) => Some(*f),
        _ => None,
    }
}

/// `VariantType` of an expression result, for callers that dispatch on it.
pub fn result_type(v: &Variant) -> VariantType {
    v.vtype()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::gstring::GoString;

    fn run(src: &str) -> Result<Variant, String> {
        Expression::parse(src)?.execute(&HashMap::new())
    }

    #[test]
    fn arithmetic_precedence() {
        assert_eq!(run("1 + 2 * 3").unwrap(), Variant::Int(7));
        assert_eq!(run("(1 + 2) * 3").unwrap(), Variant::Int(9));
        assert_eq!(run("10 - 4 - 3").unwrap(), Variant::Int(3));
        assert_eq!(run("7 / 2").unwrap(), Variant::Int(3));
        assert_eq!(run("7.0 / 2").unwrap(), Variant::Float(3.5));
    }

    #[test]
    fn power_is_right_associative() {
        // 2 ** (3 ** 2) = 512, not (2 ** 3) ** 2 = 64.
        assert_eq!(run("2 ** 3 ** 2").unwrap(), Variant::Float(512.0));
    }

    #[test]
    fn unary_operators() {
        assert_eq!(run("-5 + 2").unwrap(), Variant::Int(-3));
        assert_eq!(run("!true").unwrap(), Variant::Bool(false));
        assert_eq!(run("not 0").unwrap(), Variant::Bool(true));
        assert_eq!(run("~0").unwrap(), Variant::Int(-1));
    }

    #[test]
    fn comparisons_and_logic() {
        assert_eq!(run("1 < 2").unwrap(), Variant::Bool(true));
        assert_eq!(run("2.0 >= 2").unwrap(), Variant::Bool(true));
        assert_eq!(run("1 == 1.0").unwrap(), Variant::Bool(true));
        assert_eq!(run("\"a\" < \"b\"").unwrap(), Variant::Bool(true));
        assert_eq!(run("true or false").unwrap(), Variant::Bool(true));
        assert_eq!(run("false and true").unwrap(), Variant::Bool(false));
    }

    #[test]
    fn short_circuit_avoids_the_division_by_zero() {
        assert_eq!(run("false and (1 / 0)").unwrap(), Variant::Bool(false));
        assert_eq!(run("true or (1 / 0)").unwrap(), Variant::Bool(true));
        assert!(run("true and (1 / 0)").is_err());
    }

    #[test]
    fn strings_concatenate_and_compare() {
        let v = run("\"foo\" + \"bar\"").unwrap();
        assert_eq!(v, Variant::String(GoString::from_str("foobar")));
        assert_eq!(run("\"abc\" == \"abc\"").unwrap(), Variant::Bool(true));
        assert!(run("\"a\" - \"b\"").is_err());
    }

    #[test]
    fn variables_are_inputs() {
        let e = Expression::parse("x * 2 + y").unwrap();
        assert_eq!(e.input_names(), &["x".to_string(), "y".to_string()]);

        let mut map = HashMap::new();
        map.insert("x".to_string(), Variant::Int(3));
        map.insert("y".to_string(), Variant::Int(4));
        assert_eq!(e.execute(&map).unwrap(), Variant::Int(10));

        // Missing input is an error, not a silent zero.
        assert!(e.execute(&HashMap::new()).is_err());
    }

    #[test]
    fn builtins() {
        assert_eq!(run("abs(-3)").unwrap(), Variant::Int(3));
        assert_eq!(run("min(2, 5)").unwrap(), Variant::Int(2));
        assert_eq!(run("max(2, 5)").unwrap(), Variant::Int(5));
        assert_eq!(run("pow(2, 10)").unwrap(), Variant::Float(1024.0));
        assert_eq!(run("int(3.9)").unwrap(), Variant::Int(3));
        assert_eq!(run("str(12)").unwrap(), Variant::String(GoString::from_str("12")));
        let s = run("sin(0)").unwrap();
        assert_eq!(s, Variant::Float(0.0));
        let c = run("clamp(5, 0, 3)").unwrap();
        assert_eq!(c, Variant::Float(3.0));
        assert!(run("nope(1)").is_err());
        assert!(run("abs(1, 2)").is_err());
    }

    #[test]
    fn array_and_dictionary_literals_with_indexing() {
        assert_eq!(
            run("[10, 20, 30][1]").unwrap(),
            Variant::Int(20)
        );
        let v = run("{ \"a\": 1, \"b\": 2 }[\"b\"]").unwrap();
        assert_eq!(v, Variant::Int(2));
        // Out-of-range indexing yields nil, like the evaluator's read path.
        assert_eq!(run("[1, 2][5]").unwrap(), Variant::Nil);
    }

    #[test]
    fn in_operator_checks_containment() {
        assert_eq!(run("2 in [1, 2, 3]").unwrap(), Variant::Bool(true));
        assert_eq!(run("5 in [1, 2]").unwrap(), Variant::Bool(false));
        assert_eq!(run("\"ll\" in \"hello\"").unwrap(), Variant::Bool(true));
        assert_eq!(run("\"z\" in \"hello\"").unwrap(), Variant::Bool(false));
        assert_eq!(run("\"k\" in { \"k\": 1 }").unwrap(), Variant::Bool(true));
        assert_eq!(run("5 not in [1, 2]").unwrap(), Variant::Bool(true));
        assert_eq!(run("2 not in [1, 2]").unwrap(), Variant::Bool(false));
    }

    #[test]
    fn self_member_access_reads_classdb_properties() {
        use crate::object::{class_db, Object};

        class_db::register_class("ExprObj", None);
        class_db::register_property("ExprObj", "speed");
        let obj = Object::new();
        class_db::bind_instance(obj.id(), "ExprObj");
        assert!(class_db::property_set(obj.id(), "speed", Variant::Float(2.5)));

        let e = Expression::parse("self.speed * 2").unwrap();
        let mut map = HashMap::new();
        map.insert("self".to_string(), Variant::Object(obj.id()));
        assert_eq!(e.execute(&map).unwrap(), Variant::Float(5.0));

        // A missing property is an error, not nil.
        let bad = Expression::parse("self.missing").unwrap();
        assert!(bad.execute(&map).is_err());

        // Member access on a non-object is an error too.
        let bad2 = Expression::parse("x.member").unwrap();
        let mut m2 = HashMap::new();
        m2.insert("x".to_string(), Variant::Int(1));
        assert!(bad2.execute(&m2).is_err());

        // `self` without a bound object errors.
        let e3 = Expression::parse("self.speed").unwrap();
        assert!(e3.execute(&HashMap::new()).is_err());
    }

    #[test]
    fn self_method_calls_dispatch_through_classdb() {
        use crate::object::class_db::{self, CallCtx};
        use crate::object::Object;

        fn twice(ctx: &CallCtx) -> Result<Variant, class_db::CallError> {
            Ok(Variant::Int(ctx.args[0].to_int() * 2))
        }

        class_db::register_class("ExprCallObj", None);
        class_db::bind_method("ExprCallObj", "twice", 1, twice);
        let obj = Object::new();
        class_db::bind_instance(obj.id(), "ExprCallObj");

        let e = Expression::parse("self.twice(21)").unwrap();
        let mut map = HashMap::new();
        map.insert("self".to_string(), Variant::Object(obj.id()));
        assert_eq!(e.execute(&map).unwrap(), Variant::Int(42));

        // Wrong arity is an error.
        let bad = Expression::parse("self.twice()").unwrap();
        assert!(bad.execute(&map).is_err());
        // Calling a method on a non-object is an error.
        let bad2 = Expression::parse("x.twice(1)").unwrap();
        let mut m2 = HashMap::new();
        m2.insert("x".to_string(), Variant::Int(1));
        assert!(bad2.execute(&m2).is_err());
    }

    #[test]
    fn parse_errors_are_reported() {
        assert!(Expression::parse("1 +").is_err());
        assert!(Expression::parse("(1 + 2").is_err());
        assert!(Expression::parse("[1, 2").is_err());
        assert!(Expression::parse("1 2").is_err());
        assert!(Expression::parse("\"unterminated").is_err());
        assert!(Expression::parse("$").is_err());
    }
}
