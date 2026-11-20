//! `core/string/translation_domain.*`: the translation lookup domain.
//!
//! `TranslationServer` delegates every lookup to a `TranslationDomain`: it owns
//! the locale, the fallback locale, a locale override, the enabled flag and the
//! set of `Translation`s, and resolves messages against the best-matching
//! translation.
//!
//! Ported: `add_translation`, `remove_translation`, `clear`, the translation
//! list/count, `find_translations(locale, exact)`, `translate`,
//! `translate_plural` (with the engine's fallback order and the
//! `p_n == 1 ? message : message_plural` untranslated result), the locale and
//! override accessors and the enabled flag.
//!
//! Deviations: the engine's locale standardization (`standardize_locale`,
//! script/variant scoring) is reduced to exact → same-language → `en` matching,
//! and pseudolocalization is state-only (no transformation). The singleton
//! hooks are not ported; callers own a domain.

use crate::string::string_name::StringName;
use crate::string::translation::TranslationData;

#[derive(Clone, Debug)]
pub struct TranslationDomainData {
    locale: String,
    fallback_locale: String,
    locale_override: String,
    enabled: bool,
    translations: Vec<TranslationData>,
    pseudolocalization_enabled: bool,
}

impl Default for TranslationDomainData {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationDomainData {
    pub fn new() -> Self {
        Self {
            locale: "en".to_string(),
            fallback_locale: "en".to_string(),
            locale_override: String::new(),
            enabled: true,
            translations: Vec::new(),
            pseudolocalization_enabled: false,
        }
    }

    pub fn set_locale(&mut self, locale: &str) {
        self.locale = locale.to_string();
    }

    pub fn get_locale(&self) -> &str {
        &self.locale
    }

    pub fn set_fallback_locale(&mut self, locale: &str) {
        self.fallback_locale = locale.to_string();
    }

    pub fn get_fallback_locale(&self) -> &str {
        &self.fallback_locale
    }

    pub fn set_locale_override(&mut self, locale: &str) {
        self.locale_override = locale.to_string();
    }

    pub fn get_locale_override(&self) -> &str {
        &self.locale_override
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_pseudolocalization_enabled(&mut self, enabled: bool) {
        self.pseudolocalization_enabled = enabled;
    }

    pub fn is_pseudolocalization_enabled(&self) -> bool {
        self.pseudolocalization_enabled
    }

    /// `add_translation`.
    pub fn add_translation(&mut self, translation: TranslationData) {
        // A translation with the same locale and contents is not added twice.
        if self.translations.iter().any(|known| {
            known.get_locale() == translation.get_locale()
                && known.get_message_count() == translation.get_message_count()
                && known.get_message_list() == translation.get_message_list()
        }) {
            return;
        }
        self.translations.push(translation);
    }

    pub fn remove_translation(&mut self, index: usize) -> bool {
        if index >= self.translations.len() {
            return false;
        }
        self.translations.remove(index);
        true
    }

    pub fn clear(&mut self) {
        self.translations.clear();
    }

    pub fn get_translations(&self) -> &[TranslationData] {
        &self.translations
    }

    pub fn get_translation_count(&self) -> usize {
        self.translations.len()
    }

    /// `find_translations`: exact locale match, or every translation whose
    /// locale matches the request (best score first, then insertion order).
    pub fn find_translations(&self, locale: &str, exact: bool) -> Vec<&TranslationData> {
        let mut matches: Vec<(i32, usize, &TranslationData)> = Vec::new();
        for (index, translation) in self.translations.iter().enumerate() {
            let available = translation.get_locale();
            if exact {
                if available.eq_ignore_ascii_case(locale) {
                    matches.push((3, index, translation));
                }
                continue;
            }
            if let Some(score) = locale_score(locale, available) {
                matches.push((score, index, translation));
            }
        }
        matches.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        matches.into_iter().map(|(_, _, t)| t).collect()
    }

    fn best_translation(&self, locale: &str) -> Option<&TranslationData> {
        self.find_translations(locale, false).into_iter().next()
    }

    /// `translate`.
    pub fn translate(&self, message: &str, context: &str) -> StringName {
        if !self.enabled {
            return StringName::new(message);
        }
        let locale = if self.locale_override.is_empty() {
            &self.locale
        } else {
            &self.locale_override
        };
        if let Some(translation) = self.best_translation(locale) {
            let translated = translation.get_message(message, context);
            if translated.as_string() != "" || message.is_empty() {
                return translated;
            }
        }
        if self.fallback_locale.len() >= 2 {
            if let Some(translation) = self.best_translation(&self.fallback_locale) {
                let translated = translation.get_message(message, context);
                if translated.as_string() != "" || message.is_empty() {
                    return translated;
                }
            }
        }
        StringName::new(message)
    }

    /// `translate_plural`.
    pub fn translate_plural(
        &self,
        message: &str,
        message_plural: &str,
        n: i32,
        context: &str,
    ) -> StringName {
        if !self.enabled {
            return StringName::new(if n == 1 { message } else { message_plural });
        }
        let primary = if self.locale_override.is_empty() {
            self.locale.clone()
        } else {
            self.locale_override.clone()
        };
        let mut candidates = vec![primary];
        if self.fallback_locale.len() >= 2 && self.fallback_locale != self.locale {
            candidates.push(self.fallback_locale.clone());
        }
        for candidate in candidates {
            if let Some(translation) = self.best_translation(&candidate) {
                let translated = translation.get_plural_message(message, message_plural, n, context);
                if translated.as_string() != "" {
                    return translated;
                }
            }
        }
        StringName::new(if n == 1 { message } else { message_plural })
    }
}

/// The locale score used by `find_translations`: exact 3, same language 2, the
/// `en` fallback 1, otherwise no match.
fn locale_score(requested: &str, available: &str) -> Option<i32> {
    if requested.eq_ignore_ascii_case(available) {
        return Some(3);
    }
    let requested_language = requested.split('_').next().unwrap_or(requested);
    let available_language = available.split('_').next().unwrap_or(available);
    if requested_language.eq_ignore_ascii_case(available_language) {
        return Some(2);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn translation(locale: &str, entries: &[(&str, &str)]) -> TranslationData {
        let mut translation = TranslationData::new();
        translation.set_locale(locale);
        for (source, target) in entries {
            translation.add_message(source, target, "");
        }
        translation
    }

    #[test]
    fn locale_lookup_and_fallback() {
        let mut domain = TranslationDomainData::new();
        assert_eq!(domain.get_locale(), "en");
        assert_eq!(domain.get_fallback_locale(), "en");
        assert!(domain.is_enabled());
        assert_eq!(domain.translate("Hello", "").as_string(), "Hello");

        domain.add_translation(translation("es", &[("Hello", "Hola")]));
        domain.add_translation(translation("es_MX", &[("Hello", "Qué onda")]));
        domain.add_translation(translation("fr", &[("Hello", "Bonjour")]));
        assert_eq!(domain.get_translation_count(), 3);

        // Exact locale wins over the language variant.
        domain.set_locale("es_MX");
        assert_eq!(domain.translate("Hello", "").as_string(), "Qué onda");
        domain.set_locale("es_AR");
        assert_eq!(domain.translate("Hello", "").as_string(), "Hola", "language match");
        domain.set_locale("fr_FR");
        assert_eq!(domain.translate("Hello", "").as_string(), "Bonjour");

        // Unknown locale falls back to English (which has no translation).
        domain.set_locale("de");
        assert_eq!(domain.translate("Hello", "").as_string(), "Hello");

        // The engine's fallback locale is consulted when the locale misses.
        domain.add_translation(translation("en", &[("Hello", "Hi")]));
        domain.set_locale("de");
        assert_eq!(domain.translate("Hello", "").as_string(), "Hi");
        domain.set_fallback_locale("es");
        assert_eq!(domain.translate("Hello", "").as_string(), "Hola");
        assert_eq!(domain.get_fallback_locale(), "es");

        // The locale override beats the locale.
        domain.set_locale_override("fr");
        assert_eq!(domain.translate("Hello", "").as_string(), "Bonjour");
        assert_eq!(domain.get_locale_override(), "fr");
        domain.set_locale_override("");
        assert_eq!(domain.translate("Hello", "").as_string(), "Hola");

        // Disabling the domain returns the source messages.
        domain.set_enabled(false);
        assert_eq!(domain.translate("Hello", "").as_string(), "Hello");
        assert!(!domain.is_enabled());
        domain.set_enabled(true);
        assert_eq!(domain.translate("Hello", "").as_string(), "Hola");
    }

    #[test]
    fn contexts_and_plural() {
        let mut domain = TranslationDomainData::new();
        let mut es = TranslationData::new();
        es.set_locale("es");
        es.add_message("Open", "Abrir", "");
        es.add_message("Open", "Abrir menú", "menu");
        es.add_plural_message("%d file", &["%d archivo".to_string(), "%d archivos".to_string()], "");
        domain.add_translation(es);
        domain.set_locale("es");

        assert_eq!(domain.translate("Open", "").as_string(), "Abrir");
        assert_eq!(domain.translate("Open", "menu").as_string(), "Abrir menú");
        assert_eq!(domain.translate("Open", "missing").as_string(), "Open");

        assert_eq!(
            domain.translate_plural("%d file", "%d files", 1, "").as_string(),
            "%d archivo"
        );
        assert_eq!(
            domain.translate_plural("%d file", "%d files", 2, "").as_string(),
            "%d archivos"
        );
        // An untranslated plural returns the singular/plural source.
        assert_eq!(
            domain
                .translate_plural("%d cat", "%d cats", 1, "")
                .as_string(),
            "%d cat"
        );
        assert_eq!(
            domain
                .translate_plural("%d cat", "%d cats", 2, "")
                .as_string(),
            "%d cats"
        );
        // Disabled domains return the source tuple too.
        domain.set_enabled(false);
        assert_eq!(
            domain.translate_plural("%d file", "%d files", 1, "").as_string(),
            "%d file"
        );
        assert_eq!(
            domain.translate_plural("%d file", "%d files", 2, "").as_string(),
            "%d files"
        );
    }

    #[test]
    fn translation_set_management() {
        let mut domain = TranslationDomainData::new();
        domain.add_translation(translation("es", &[("a", "1")]));
        domain.add_translation(translation("fr", &[("a", "2")]));
        domain.add_translation(translation("es", &[("a", "1")]));
        assert_eq!(domain.get_translation_count(), 2, "duplicates are ignored");

        assert!(domain.remove_translation(0));
        assert_eq!(domain.get_translation_count(), 1);
        assert!(!domain.remove_translation(5));

        // find_translations ordering.
        domain.clear();
        domain.add_translation(translation("es", &[("a", "lang")]));
        domain.add_translation(translation("es_MX", &[("a", "exact")]));
        domain.add_translation(translation("en", &[("a", "fallback")]));
        let found = domain.find_translations("es_MX", false);
        // The engine's locale scoring returns the exact locale and the same
        // language; English is not a variant match (it is only the fallback
        // locale consulted later by `translate`).
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].get_locale(), "es_MX");
        assert_eq!(found[1].get_locale(), "es");
        let exact = domain.find_translations("es_MX", true);
        assert_eq!(exact.len(), 1);
        assert_eq!(exact[0].get_locale(), "es_MX");
        assert!(domain.find_translations("de", true).is_empty());

        // Pseudolocalization is a stored flag in this port.
        domain.set_pseudolocalization_enabled(true);
        assert!(domain.is_pseudolocalization_enabled());
        domain.clear();
        assert_eq!(domain.get_translation_count(), 0);
    }
}
