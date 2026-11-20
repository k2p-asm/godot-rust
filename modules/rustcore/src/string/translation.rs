//! `core/string/translation.*`: the translation message store.
//!
//! Messages are keyed by `(context, source)`; the value is a list whose first
//! entry is the singular translation and whose remaining entries are the plural
//! forms. Ported: `add_message`, `add_plural_message` (rejecting an empty list),
//! `get_message` (first entry, empty when missing), `erase_message`,
//! `get_message_list` (with the `context \x04 source` form when a context is
//! set, as in the MO format), `get_message_count` and
//! `get_translated_message_list`.
//!
//! Two honest limits: `set_locale` stores the string as given (the engine routes
//! it through `TranslationServer::standardize_locale`), and plural evaluation
//! uses the one/other rule (`n == 1` → first form, otherwise the second, clamped)
//! instead of parsing gettext plural-rule expressions. `plural_rules_override`
//! is kept but not evaluated.

use std::collections::HashMap;

use crate::string::plural_rules::PluralRules;
use crate::string::string_name::StringName;

#[derive(Clone, Debug)]
pub struct TranslationData {
    locale: String,
    plural_rules_override: String,
    /// `(context, source)` → plural forms (the first is the singular).
    messages: HashMap<(StringName, StringName), Vec<StringName>>,
}

impl Default for TranslationData {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationData {
    pub fn new() -> Self {
        Self {
            locale: "en".to_string(),
            plural_rules_override: String::new(),
            messages: HashMap::new(),
        }
    }

    /// Stored as given (no locale standardisation in this port).
    pub fn set_locale(&mut self, locale: &str) {
        self.locale = locale.to_string();
    }

    pub fn get_locale(&self) -> &str {
        &self.locale
    }

    pub fn set_plural_rules_override(&mut self, rules: &str) {
        self.plural_rules_override = rules.to_string();
    }

    pub fn get_plural_rules_override(&self) -> &str {
        &self.plural_rules_override
    }

    /// `add_message`: replaces any previous singular translation.
    pub fn add_message(&mut self, source: &str, translated: &str, context: &str) {
        let key = (StringName::new(context), StringName::new(source));
        self.messages.insert(key, vec![StringName::new(translated)]);
    }

    /// `add_plural_message`: an empty list is rejected, leaving the store alone.
    pub fn add_plural_message(
        &mut self,
        source: &str,
        plural_translations: &[String],
        context: &str,
    ) -> bool {
        if plural_translations.is_empty() {
            return false;
        }
        let key = (StringName::new(context), StringName::new(source));
        let forms: Vec<StringName> = plural_translations
            .iter()
            .map(|text| StringName::new(text))
            .collect();
        self.messages.insert(key, forms);
        true
    }

    /// First stored form, or an empty name when the key is missing.
    pub fn get_message(&self, source: &str, context: &str) -> StringName {
        self.messages
            .get(&(StringName::new(context), StringName::new(source)))
            .and_then(|forms| forms.first())
            .copied()
            .unwrap_or_else(|| StringName::new(""))
    }

    /// Plural form for `n`: negative counts and missing keys read as empty.
    /// The engine evaluates the locale's gettext plural expression; this port
    /// implements the common one/other rule.
    pub fn get_plural_message(
        &self,
        source: &str,
        _plural_source: &str,
        n: i32,
        context: &str,
    ) -> StringName {
        if n < 0 {
            return StringName::new("");
        }
        let Some(forms) = self
            .messages
            .get(&(StringName::new(context), StringName::new(source)))
        else {
            return StringName::new("");
        };
        if forms.is_empty() {
            return StringName::new("");
        }
        // With a parseable `plural_rules_override` the engine evaluates the
        // gettext rule; otherwise it falls back to the one/other rule here.
        let index = match PluralRules::parse(&self.plural_rules_override) {
            Ok(rules) => rules.evaluate(n),
            Err(_) => {
                if n == 1 {
                    0
                } else {
                    1
                }
            }
        };
        if index < 0 || index as usize >= forms.len() {
            return StringName::new("");
        }
        forms[index as usize]
    }

    pub fn erase_message(&mut self, source: &str, context: &str) {
        self.messages
            .remove(&(StringName::new(context), StringName::new(source)));
    }

    /// `get_message_list`: plain sources, or `context \x04 source`.
    pub fn get_message_list(&self) -> Vec<StringName> {
        let mut list: Vec<StringName> = self
            .messages
            .keys()
            .map(|(context, source)| {
                if context.as_string().is_empty() {
                    *source
                } else {
                    StringName::new(&format!("{}\u{4}{}", context.as_string(), source.as_string()))
                }
            })
            .collect();
        list.sort_by_key(|name| name.as_string());
        list
    }

    pub fn get_message_count(&self) -> usize {
        self.messages.len()
    }

    /// The singular translations, sorted by their message key.
    pub fn get_translated_message_list(&self) -> Vec<String> {
        let mut pairs: Vec<((String, String), String)> = self
            .messages
            .iter()
            .filter_map(|((context, source), forms)| {
                forms
                    .first()
                    .map(|form| ((context.as_string(), source.as_string()), form.as_string()))
            })
            .collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        pairs.into_iter().map(|(_, translated)| translated).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let t = TranslationData::new();
        assert_eq!(t.get_locale(), "en");
        assert_eq!(t.get_plural_rules_override(), "");
        assert_eq!(t.get_message_count(), 0);
        assert_eq!(t.get_message("missing", "").as_string(), "");
    }

    #[test]
    fn single_messages_and_contexts() {
        let mut t = TranslationData::new();
        t.add_message("Hello", "Hola", "");
        t.add_message("Open", "Abrir", "menu");
        t.add_message("Open", "Abrir archivo", "file");

        assert_eq!(t.get_message("Hello", "").as_string(), "Hola");
        assert_eq!(t.get_message("Open", "menu").as_string(), "Abrir");
        assert_eq!(t.get_message("Open", "file").as_string(), "Abrir archivo");
        // Contexts are part of the key.
        assert_eq!(t.get_message("Open", "").as_string(), "");
        assert_eq!(t.get_message_count(), 3);

        // Adding again replaces the singular form.
        t.add_message("Hello", "Buenas", "");
        assert_eq!(t.get_message("Hello", "").as_string(), "Buenas");
        assert_eq!(t.get_message_count(), 3);

        t.erase_message("Hello", "");
        assert_eq!(t.get_message("Hello", "").as_string(), "");
        assert_eq!(t.get_message_count(), 2);
        // Erasing a missing key is a no-op.
        t.erase_message("Hello", "");
    }

    #[test]
    fn plural_messages_require_a_non_empty_list() {
        let mut t = TranslationData::new();
        assert!(!t.add_plural_message("apple", &[], ""));
        assert_eq!(t.get_message_count(), 0);

        assert!(t.add_plural_message(
            "apple",
            &["manzana".to_string(), "manzanas".to_string()],
            ""
        ));
        assert_eq!(t.get_message_count(), 1);

        // Singular lookup returns the first form.
        assert_eq!(t.get_message("apple", "").as_string(), "manzana");

        assert_eq!(t.get_plural_message("apple", "apples", 0, "").as_string(), "manzanas");
        assert_eq!(t.get_plural_message("apple", "apples", 1, "").as_string(), "manzana");
        assert_eq!(t.get_plural_message("apple", "apples", 2, "").as_string(), "manzanas");
        // Negative counts are rejected.
        assert_eq!(t.get_plural_message("apple", "apples", -1, "").as_string(), "");
        // Missing keys are empty.
        assert_eq!(t.get_plural_message("pear", "pears", 2, "").as_string(), "");
    }

    #[test]
    fn plural_index_must_exist_in_the_form_list() {
        let mut t = TranslationData::new();
        // Only one form: the one/other rule asks for index 1 for n != 1, which
        // does not exist, so the engine's ERR_FAIL_INDEX_V returns an empty name.
        assert!(t.add_plural_message("one", &["solo".to_string()], ""));
        assert_eq!(t.get_plural_message("one", "ones", 1, "").as_string(), "solo");
        assert_eq!(t.get_plural_message("one", "ones", 7, "").as_string(), "");
    }

    #[test]
    fn plural_rules_override_selects_the_form() {
        let mut t = TranslationData::new();
        // Russian-style: 1 -> 0, 2..4 -> 1, everything else -> 2.
        t.set_plural_rules_override(
            "nplurals=3; plural=n%10==1 && n%100!=11 ? 0 : n%10>=2 && n%10<=4 && (n%100<10 || n%100>=20) ? 1 : 2;",
        );
        assert!(t.add_plural_message(
            "apple",
            &["yabloko".to_string(), "yabloka".to_string(), "yablok".to_string()],
            ""
        ));
        assert_eq!(t.get_plural_message("apple", "apples", 1, "").as_string(), "yabloko");
        assert_eq!(t.get_plural_message("apple", "apples", 3, "").as_string(), "yabloka");
        assert_eq!(t.get_plural_message("apple", "apples", 5, "").as_string(), "yablok");
        assert_eq!(t.get_plural_message("apple", "apples", 21, "").as_string(), "yabloko");
        assert_eq!(t.get_plural_message("apple", "apples", 11, "").as_string(), "yablok");

        // An unparseable override falls back to the one/other rule.
        t.set_plural_rules_override("not a rule");
        assert_eq!(t.get_plural_message("apple", "apples", 1, "").as_string(), "yabloko");
        assert_eq!(t.get_plural_message("apple", "apples", 2, "").as_string(), "yabloka");
    }

    #[test]
    fn message_lists_and_translations() {
        let mut t = TranslationData::new();
        t.add_message("B", "bee", "");
        t.add_message("A", "ay", "ctx");
        t.add_message("C", "see", "");

        let list: Vec<String> = t
            .get_message_list()
            .iter()
            .map(|name| name.as_string())
            .collect();
        // Sorted; contextual keys use the EOT separator.
        assert_eq!(list, vec!["B", "C", "ctx\u{4}A"]);

        assert_eq!(t.get_translated_message_list(), vec!["bee", "see", "ay"]);
    }

    #[test]
    fn locale_and_rules_override() {
        let mut t = TranslationData::new();
        t.set_locale("es");
        assert_eq!(t.get_locale(), "es");
        // No standardisation: the string is kept verbatim.
        t.set_locale("en_US.UTF-8");
        assert_eq!(t.get_locale(), "en_US.UTF-8");

        t.set_plural_rules_override("nplurals=2; plural=n != 1;");
        assert_eq!(
            t.get_plural_rules_override(),
            "nplurals=2; plural=n != 1;"
        );
    }
}
