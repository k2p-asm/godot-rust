//! `core/string/plural_rules.*`: gettext plural-rule evaluation.
//!
//! `PluralRules::parse` accepts `"nplurals=<N>; plural=<expression>;"` and splits
//! the expression into the engine's ternary tree: the rule is cut at unbalanced
//! `?` and `:` (parenthesised regions are skipped), every condition becomes a
//! node and the two branches become its children. `evaluate(n)` walks that tree,
//! evaluating a condition with the single input `n`, and returns the leaf value.
//!
//! The engine's `EQNode` calls the condition its `regex`, but it is an ordinary
//! expression string — there is no regular expression involved, and this port
//! evaluates exactly the same tree with the ported `Expression`.
//!
//! Results are cached in an [`LruCache`] keyed by `n`, capacity 64, like the
//! engine; the cache is only a speed-up, never part of the semantics.

use std::cell::RefCell;

use crate::math::expression::Expression;
use crate::templates::lru::LruCache;
use crate::variant::Variant;

/// One node of the rule tree: a leaf value or a condition with two branches.
enum Node {
    Leaf(i32),
    Branch {
        /// The condition as written (trimmed), kept for parity/debugging.
        condition: String,
        expr: Expression,
        left: Box<Node>,
        right: Box<Node>,
    },
}

pub struct PluralRules {
    nplurals: i32,
    plural: String,
    root: Node,
    cache: RefCell<LruCache<i32, i32>>,
}

impl std::fmt::Debug for PluralRules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PluralRules({} plurals: {})", self.nplurals, self.plural)
    }
}

impl PluralRules {
    pub fn get_nplurals(&self) -> i32 {
        self.nplurals
    }

    pub fn get_plural(&self) -> &str {
        &self.plural
    }

    /// `evaluate`: the plural index for `n`, memoised per value.
    pub fn evaluate(&self, n: i32) -> i32 {
        if let Some(value) = self.cache.borrow_mut().get(&n) {
            return *value;
        }
        let value = self.eval_node(&self.root, n);
        self.cache.borrow_mut().insert(n, value);
        value
    }

    fn eval_node(&self, node: &Node, n: i32) -> i32 {
        match node {
            Node::Leaf(value) => *value,
            Node::Branch {
                expr, left, right, ..
            } => {
                let taken = expr
                    .execute_with("n", Variant::Int(n as i64))
                    .map(|result| result.booleanize())
                    .unwrap_or(false);
                if taken {
                    self.eval_node(left, n)
                } else {
                    self.eval_node(right, n)
                }
            }
        }
    }

    /// `PluralRules::parse`.
    pub fn parse(rules: &str) -> Result<Self, String> {
        let chars: Vec<char> = rules.chars().collect();

        let nplurals_eq = find_char(&chars, '=', 0).ok_or_else(|| {
            "Invalid plural rules format. Missing equal sign for `nplurals`.".to_string()
        })?;
        let nplurals_semi = find_char(&chars, ';', nplurals_eq).ok_or_else(|| {
            "Invalid plural rules format. Missing semicolon for `nplurals`.".to_string()
        })?;

        let nplurals_str = collect(&chars, nplurals_eq + 1, nplurals_semi);
        if !is_valid_int(&nplurals_str) {
            return Err(
                "Invalid plural rules format. `nplurals` should be an integer.".to_string(),
            );
        }
        let nplurals: i32 = nplurals_str
            .parse()
            .map_err(|_| "Invalid plural rules format. `nplurals` should be an integer.".to_string())?;
        if nplurals < 1 {
            return Err(
                "Invalid plural rules format. `nplurals` should be at least 1.".to_string(),
            );
        }

        let expression_eq = find_char(&chars, '=', nplurals_semi + 1).ok_or_else(|| {
            "Invalid plural rules format. Missing equal sign for `plural`.".to_string()
        })?;

        // A missing final semicolon only warns in the engine; use the end here.
        let expression_end = rfind_char(&chars, ';').unwrap_or(chars.len());
        let expression_start = expression_eq + 1;
        if expression_end <= expression_start {
            return Err("Invalid plural rules format. `plural` expression is empty.".to_string());
        }

        let plural = collect(&chars, expression_start, expression_end)
            .trim()
            .to_string();

        let root = build_node(&plural)?;
        Ok(Self {
            nplurals,
            plural,
            root,
            cache: RefCell::new(LruCache::new(64)),
        })
    }
}

/// `_find_unquoted`: the first `needle` outside parentheses.
fn find_unquoted(chars: &[char], needle: char) -> Option<usize> {
    let mut in_quote = false;
    for (i, &c) in chars.iter().enumerate() {
        if in_quote {
            if c == ')' {
                in_quote = false;
            }
        } else if c == '(' {
            in_quote = true;
        } else if c == needle {
            return Some(i);
        }
    }
    None
}

/// `_cache_plural_tests`.
fn build_node(rule: &str) -> Result<Node, String> {
    let mut rule: Vec<char> = rule.chars().collect();

    // Drop one pair of wrapping parentheses when they match.
    if rule.first() == Some(&'(') && rule.last() == Some(&')') {
        let mut depth: i32 = 0;
        let mut balanced = true;
        for &c in &rule[1..rule.len() - 1] {
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
                if depth < 0 {
                    balanced = false;
                    break;
                }
            }
        }
        if balanced && depth == 0 {
            rule = rule[1..rule.len() - 1].to_vec();
        }
    }

    let Some(question) = find_unquoted(&rule, '?') else {
        // No ternary left: this is either a value or a bare condition.
        let text: String = rule.iter().collect();
        let text = text.trim();
        if let Ok(value) = text.parse::<i32>() {
            return Ok(Node::Leaf(value));
        }
        // A bare condition (`n != 1`) yields 1 when true and 0 when false, which
        // is how the engine's last `_eq_test` call resolves the leaf.
        let expr = Expression::parse(text)
            .map_err(|error| format!("Cannot parse expression \"{text}\". Error: {error}"))?;
        return Ok(Node::Branch {
            condition: text.to_string(),
            expr,
            left: Box::new(Node::Leaf(1)),
            right: Box::new(Node::Leaf(0)),
        });
    };
    let colon = find_unquoted(&rule, ':')
        .ok_or_else(|| format!("Missing ':' in plural rule \"{}\"", collect_all(&rule)))?;

    let condition: String = rule[..question].iter().collect();
    let condition = condition.trim().to_string();
    let expr = Expression::parse(&condition)
        .map_err(|error| format!("Cannot parse expression \"{condition}\". Error: {error}"))?;

    let left_text: String = rule[question + 1..colon].iter().collect();
    let right_text: String = rule[colon + 1..].iter().collect();

    Ok(Node::Branch {
        condition,
        expr,
        left: Box::new(build_node(left_text.trim())?),
        right: Box::new(build_node(right_text.trim())?),
    })
}

fn collect_all(chars: &[char]) -> String {
    chars.iter().collect()
}

fn collect(chars: &[char], start: usize, end: usize) -> String {
    chars[start.min(chars.len())..end.min(chars.len())]
        .iter()
        .collect()
}

fn find_char(chars: &[char], needle: char, from: usize) -> Option<usize> {
    chars
        .iter()
        .enumerate()
        .skip(from)
        .find(|(_, &c)| c == needle)
        .map(|(i, _)| i)
}

fn rfind_char(chars: &[char], needle: char) -> Option<usize> {
    chars.iter().rposition(|&c| c == needle)
}

/// `String::is_valid_int`: an optional sign followed by digits only.
fn is_valid_int(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
}

/// `PluralRules::parse` over the engine's String; `None` mirrors the null return.
pub fn parse_rules(rules: &str) -> Option<PluralRules> {
    PluralRules::parse(rules).ok()
}

/// Formats a rule the way `PluralRules::get_plural` would, for logs and tests.
pub fn describe(rules: &PluralRules) -> String {
    format!("nplurals={}; plural={};", rules.get_nplurals(), rules.get_plural())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_rule() {
        let rules = PluralRules::parse("nplurals=2; plural=n != 1;").unwrap();
        assert_eq!(rules.get_nplurals(), 2);
        assert_eq!(rules.get_plural(), "n != 1");
        assert_eq!(rules.evaluate(0), 1);
        assert_eq!(rules.evaluate(1), 0);
        assert_eq!(rules.evaluate(2), 1);
        assert_eq!(rules.evaluate(100), 1);
    }

    #[test]
    fn french_rule() {
        let rules = PluralRules::parse("nplurals=2; plural=n > 1;").unwrap();
        assert_eq!(rules.evaluate(0), 0);
        assert_eq!(rules.evaluate(1), 0);
        assert_eq!(rules.evaluate(2), 1);
    }

    #[test]
    fn parenthesised_and_arithmetic_rules() {
        let rules = PluralRules::parse("nplurals=2; plural=(n % 10 == 1 && n % 100 != 11);").unwrap();
        assert_eq!(rules.evaluate(1), 1);
        assert_eq!(rules.evaluate(21), 1);
        assert_eq!(rules.evaluate(11), 0);
        assert_eq!(rules.evaluate(2), 0);

        let rules = PluralRules::parse("nplurals=2; plural=n >= 2;").unwrap();
        assert_eq!(rules.evaluate(1), 0);
        assert_eq!(rules.evaluate(2), 1);
    }

    #[test]
    fn ternary_chain_arabic() {
        let rules = PluralRules::parse(
            "nplurals=6; plural=n==0 ? 0 : n==1 ? 1 : n==2 ? 2 : n%100>=3 && n%100<=10 ? 3 : n%100>=11 && n%100<=99 ? 4 : 5;",
        )
        .unwrap();
        assert_eq!(rules.get_nplurals(), 6);
        assert_eq!(rules.evaluate(0), 0);
        assert_eq!(rules.evaluate(1), 1);
        assert_eq!(rules.evaluate(2), 2);
        assert_eq!(rules.evaluate(3), 3);
        assert_eq!(rules.evaluate(10), 3);
        assert_eq!(rules.evaluate(11), 4);
        assert_eq!(rules.evaluate(99), 4);
        assert_eq!(rules.evaluate(100), 5);
        assert_eq!(rules.evaluate(103), 3);
    }

    #[test]
    fn caching_returns_the_same_value() {
        let rules = PluralRules::parse("nplurals=2; plural=n != 1;").unwrap();
        assert_eq!(rules.evaluate(7), 1);
        assert_eq!(rules.cache.borrow().get_size(), 1);
        assert_eq!(rules.evaluate(7), 1);
        assert_eq!(rules.cache.borrow().get_size(), 1);
        assert_eq!(rules.evaluate(8), 1);
        assert_eq!(rules.cache.borrow().get_size(), 2);
    }

    #[test]
    fn parse_errors() {
        assert_eq!(
            PluralRules::parse("nplurals 2;").unwrap_err(),
            "Invalid plural rules format. Missing equal sign for `nplurals`."
        );
        assert_eq!(
            PluralRules::parse("nplurals=x").unwrap_err(),
            "Invalid plural rules format. Missing semicolon for `nplurals`."
        );
        assert_eq!(
            PluralRules::parse("nplurals=x; plural=n != 1;").unwrap_err(),
            "Invalid plural rules format. `nplurals` should be an integer."
        );
        assert_eq!(
            PluralRules::parse("nplurals=0; plural=n != 1;").unwrap_err(),
            "Invalid plural rules format. `nplurals` should be at least 1."
        );
        assert_eq!(
            PluralRules::parse("nplurals=2; plural=;").unwrap_err(),
            "Invalid plural rules format. `plural` expression is empty."
        );
        assert_eq!(
            PluralRules::parse("nplurals=2; plural=n !=;").unwrap_err(),
            "Cannot parse expression \"n !=\". Error: Unexpected token End"
        );
    }

    #[test]
    fn a_missing_final_semicolon_is_rejected() {
        // The engine looks for the *last* `;` in the string; without a trailing
        // one that is the `nplurals` terminator, so the expression range is empty
        // and the parse fails (its WARN path only triggers when there is no `;`
        // at all, which fails earlier for the missing `nplurals` semicolon).
        assert_eq!(
            PluralRules::parse("nplurals=2; plural=n != 1").unwrap_err(),
            "Invalid plural rules format. `plural` expression is empty."
        );
    }

    #[test]
    fn describe_rebuilds_the_rule_text() {
        let rules = PluralRules::parse("nplurals=3; plural=n % 10 == 1 ? 0 : 1;").unwrap();
        assert_eq!(
            describe(&rules),
            "nplurals=3; plural=n % 10 == 1 ? 0 : 1;"
        );
    }
}
