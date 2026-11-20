//! `core/string/optimized_translation.*`: the compact translation table.
//!
//! Ported from `optimized_translation.cpp`: the two-level perfect hash over
//! UTF-8 message keys (`hash(0, key)` picks the bucket, the bucket's `d` seed is
//! increased until every key hashes to a distinct slot), the flat
//! `hash_table`/`bucket_table` layout (`size`, `func`, then `key`, `str_offset`,
//! `comp_size`, `uncomp_size` per element), lookup by key, the translated list and
//! the plural/message-list stubs that the engine also leaves unimplemented.
//!
//! The engine smaz-compresses each translation; this port stores them
//! uncompressed (`comp_size == uncomp_size`), which the lookup path already
//! handles. Contexts and plurals are unsupported exactly like the engine (keys
//! carrying the engine's `0x04` context separator are skipped during `generate`).

use crate::math::funcs;
use crate::string::string_name::StringName;
use crate::string::translation::TranslationData;

const NO_ENTRY: u32 = 0xFFFF_FFFF;

/// `OptimizedTranslation`.
#[derive(Clone, Debug, Default)]
pub struct OptimizedTranslation {
    hash_table: Vec<u32>,
    /// Flat buckets: `[size, func, key, offset, comp_size, uncomp_size] * size`.
    bucket_table: Vec<u32>,
    strings: Vec<u8>,
    locale: String,
}

/// `OptimizedTranslation::hash`: FNV-1a with the engine's seed rule.
fn hash(mut d: u32, bytes: &[u8]) -> u32 {
    if d == 0 {
        d = 0x0100_0193;
    }
    for byte in bytes {
        d = (d ^ *byte as u32).wrapping_mul(0x0100_0193);
    }
    d
}

/// `Math::larger_prime`: the smallest prime `>= n` (2 for `n < 2`).
fn larger_prime(n: usize) -> usize {
    if n <= 2 {
        return 2;
    }
    let mut candidate = n;
    loop {
        if is_prime(candidate) {
            return candidate;
        }
        candidate += 1;
    }
}

fn is_prime(value: usize) -> bool {
    if value < 2 {
        return false;
    }
    if value % 2 == 0 {
        return value == 2;
    }
    let mut divisor = 3;
    while divisor * divisor <= value {
        if value % divisor == 0 {
            return false;
        }
        divisor += 2;
    }
    true
}

impl OptimizedTranslation {
    pub fn new() -> Self {
        Self::default()
    }

    /// `generate`: false for an empty translation (the engine's
    /// `bucket_table_size == 0` failure).
    pub fn generate(&mut self, from: &TranslationData) -> bool {
        let mut keys: Vec<String> = Vec::new();
        for name in from.get_message_list() {
            let text = name.as_string();
            // The engine does not support contexts in this format.
            if text.contains('\u{4}') {
                continue;
            }
            keys.push(text);
        }
        if keys.is_empty() {
            return false;
        }

        let size = larger_prime(keys.len());
        let mut buckets: Vec<Vec<(usize, Vec<u8>)>> = vec![Vec::new(); size];
        let mut compressed: Vec<(usize, Vec<u8>)> = Vec::with_capacity(keys.len());
        let mut total_compression_size = 0usize;

        for (index, key) in keys.iter().enumerate() {
            let key_bytes = key.as_bytes().to_vec();
            let bucket = (hash(0, &key_bytes) % size as u32) as usize;
            buckets[bucket].push((index, key_bytes));

            let message = from.get_message(key, "").as_string();
            let offset = total_compression_size;
            let bytes = if message.is_empty() {
                // The engine stores a single NUL byte for an empty message.
                vec![0u8]
            } else {
                message.into_bytes()
            };
            total_compression_size += bytes.len();
            compressed.push((offset, bytes));
        }

        let mut table: Vec<Vec<(u32, usize)>> = vec![Vec::new(); size];
        let mut hfunc_table: Vec<u32> = vec![0; size];
        let mut bucket_table_size = 0usize;

        for i in 0..size {
            let bucket = &buckets[i];
            if bucket.is_empty() {
                continue;
            }
            let mut d = 1u32;
            let mut slots: Vec<(u32, usize)> = Vec::new();
            loop {
                slots.clear();
                let mut collision = false;
                for (index, key_bytes) in bucket {
                    let slot = hash(d, key_bytes);
                    if slots.iter().any(|(existing, _)| *existing == slot) {
                        collision = true;
                        break;
                    }
                    slots.push((slot, *index));
                }
                if !collision {
                    break;
                }
                d += 1;
            }
            hfunc_table[i] = d;
            table[i] = slots;
            bucket_table_size += 2 + bucket.len() * 4;
        }

        if bucket_table_size == 0 {
            return false;
        }

        self.hash_table = vec![NO_ENTRY; size];
        self.bucket_table = vec![0u32; bucket_table_size];

        let mut btindex = 0usize;
        for i in 0..size {
            if table[i].is_empty() {
                continue;
            }
            self.hash_table[i] = btindex as u32;
            self.bucket_table[btindex] = table[i].len() as u32;
            btindex += 1;
            self.bucket_table[btindex] = hfunc_table[i];
            btindex += 1;
            for (slot, index) in &table[i] {
                let (offset, bytes) = &compressed[*index];
                self.bucket_table[btindex] = *slot;
                btindex += 1;
                self.bucket_table[btindex] = *offset as u32;
                btindex += 1;
                self.bucket_table[btindex] = bytes.len() as u32;
                btindex += 1;
                self.bucket_table[btindex] = bytes.len() as u32;
                btindex += 1;
            }
        }

        self.strings = vec![0u8; total_compression_size];
        for (offset, bytes) in &compressed {
            self.strings[*offset..*offset + bytes.len()].copy_from_slice(bytes);
        }

        self.locale = from.get_locale().to_string();
        true
    }

    /// `get_message`: empty for an unknown key (contexts are ignored).
    pub fn get_message(&self, source: &str) -> StringName {
        if self.hash_table.is_empty() {
            return StringName::new("");
        }
        let bytes = source.as_bytes();
        let index = (hash(0, bytes) % self.hash_table.len() as u32) as usize;
        let bucket_offset = self.hash_table[index];
        if bucket_offset == NO_ENTRY {
            return StringName::new("");
        }
        let base = bucket_offset as usize;
        let size = self.bucket_table[base] as usize;
        let func = self.bucket_table[base + 1];
        let wanted = hash(func, bytes);

        for element in 0..size {
            let at = base + 2 + element * 4;
            if self.bucket_table[at] != wanted {
                continue;
            }
            let offset = self.bucket_table[at + 1] as usize;
            let comp_size = self.bucket_table[at + 2] as usize;
            let uncomp_size = self.bucket_table[at + 3] as usize;
            debug_assert_eq!(comp_size, uncomp_size, "this port stores translations raw");
            let text = String::from_utf8_lossy(&self.strings[offset..offset + comp_size]);
            return StringName::new(&text);
        }
        StringName::new("")
    }

    /// `get_plural_message` falls back to the singular, like the engine.
    pub fn get_plural_message(&self, source: &str) -> StringName {
        self.get_message(source)
    }

    /// `get_translated_message_list`: every stored translation.
    pub fn get_translated_message_list(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut base = 0usize;
        while base + 1 < self.bucket_table.len() {
            let size = self.bucket_table[base] as usize;
            for element in 0..size {
                let at = base + 2 + element * 4;
                let offset = self.bucket_table[at + 1] as usize;
                let comp_size = self.bucket_table[at + 2] as usize;
                out.push(
                    String::from_utf8_lossy(&self.strings[offset..offset + comp_size]).into_owned(),
                );
            }
            base += 2 + size * 4;
        }
        out
    }

    /// The engine does not keep the source texts in this format.
    pub fn get_message_count(&self) -> usize {
        0
    }

    pub fn get_locale(&self) -> &str {
        &self.locale
    }

    pub fn get_hash_table(&self) -> &[u32] {
        &self.hash_table
    }

    pub fn get_bucket_table(&self) -> &[u32] {
        &self.bucket_table
    }

    pub fn get_strings(&self) -> &[u8] {
        &self.strings
    }

    /// Restores a table from the serialized form (`_set`).
    pub fn set_tables(&mut self, hash_table: Vec<u32>, bucket_table: Vec<u32>, strings: Vec<u8>) {
        self.hash_table = hash_table;
        self.bucket_table = bucket_table;
        self.strings = strings;
    }
}

/// `Math::is_prime` re-exported for tests of the table size rule.
pub fn is_prime_for_test(value: usize) -> bool {
    is_prime(value)
}

/// `funcs` is unused but keeps the module's dependency explicit for future
/// compression work.
#[allow(dead_code)]
fn _approx(value: f32) -> bool {
    funcs::is_zero_approx(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn translation(entries: &[(&str, &str)]) -> TranslationData {
        let mut translation = TranslationData::new();
        translation.set_locale("eo");
        for (source, target) in entries {
            translation.add_message(source, target, "");
        }
        translation
    }

    #[test]
    fn primes() {
        assert_eq!(larger_prime(0), 2);
        assert_eq!(larger_prime(2), 2);
        assert_eq!(larger_prime(3), 3);
        assert_eq!(larger_prime(4), 5);
        assert_eq!(larger_prime(17), 17);
        assert_eq!(larger_prime(18), 19);
        assert!(is_prime_for_test(97));
        assert!(!is_prime_for_test(91));
    }

    #[test]
    fn hash_is_fnv1a_with_seed() {
        assert_eq!(
            hash(0, b"a"),
            (0x0100_0193u32 ^ 0x61).wrapping_mul(0x0100_0193)
        );
        // The seed rule: d = 0 becomes the FNV offset basis.
        assert_eq!(hash(0, b""), 0x0100_0193);
        assert_eq!(hash(1, b""), 1);
    }

    #[test]
    fn generate_and_lookup() {
        let source = translation(&[
            ("Hello", "Saluton"),
            ("Goodbye", "Ĝis revido"),
            ("Yes", "Jes"),
        ]);
        let mut optimized = OptimizedTranslation::new();
        assert!(optimized.generate(&source));
        assert_eq!(optimized.get_locale(), "eo");
        assert_eq!(optimized.get_message("Hello").as_string(), "Saluton");
        assert_eq!(optimized.get_message("Goodbye").as_string(), "Ĝis revido");
        assert_eq!(optimized.get_message("Yes").as_string(), "Jes");
        assert_eq!(optimized.get_message("No").as_string(), "");
        assert_eq!(optimized.get_message("").as_string(), "");
        // Plurals fall back to the singular.
        assert_eq!(optimized.get_plural_message("Yes").as_string(), "Jes");
        // The engine does not retain the source texts.
        assert_eq!(optimized.get_message_count(), 0);

        let mut translated = optimized.get_translated_message_list();
        translated.sort();
        assert_eq!(
            translated,
            vec!["Jes".to_string(), "Saluton".to_string(), "Ĝis revido".to_string()]
        );
        // The table sizes follow the format: one bucket entry per message plus
        // two words of header per used bucket.
        let mut base = 0usize;
        let mut stored = 0usize;
        while base + 1 < optimized.get_bucket_table().len() {
            let size = optimized.get_bucket_table()[base] as usize;
            stored += size;
            base += 2 + size * 4;
        }
        assert_eq!(stored, 3, "one element per message");
        assert_eq!(base, optimized.get_bucket_table().len());
        assert_eq!(optimized.get_strings().len(), "Saluton".len() + "Ĝis revido".len() + "Jes".len());
    }

    #[test]
    fn collisions_pick_a_larger_seed() {
        // Enough keys to fill several buckets; whatever the distribution, every
        // key must resolve and the seed of every used bucket must be >= 1.
        let entries: Vec<(String, String)> = (0..64)
            .map(|i| (format!("message_{i}"), format!("traduko_{i}")))
            .collect();
        let mut source = TranslationData::new();
        for (key, value) in &entries {
            source.add_message(key, value, "");
        }
        let mut optimized = OptimizedTranslation::new();
        assert!(optimized.generate(&source));
        for (key, value) in &entries {
            assert_eq!(optimized.get_message(key).as_string(), *value);
        }
        let mut base = 0usize;
        while base + 1 < optimized.get_bucket_table().len() {
            let size = optimized.get_bucket_table()[base] as usize;
            assert!(optimized.get_bucket_table()[base + 1] >= 1);
            base += 2 + size * 4;
        }
    }

    #[test]
    fn empty_and_context_translations() {
        let empty = TranslationData::new();
        let mut optimized = OptimizedTranslation::new();
        assert!(!optimized.generate(&empty));
        assert_eq!(optimized.get_message("anything").as_string(), "");

        // A context-only translation has no usable keys.
        let mut context_only = TranslationData::new();
        context_only.add_message("Menu", "Menu", "context");
        assert!(!optimized.generate(&context_only));

        // An empty translation is stored as a single NUL byte.
        let mut with_empty = TranslationData::new();
        with_empty.add_message("empty", "", "");
        assert!(optimized.generate(&with_empty));
        assert_eq!(optimized.get_strings(), &[0u8]);
        assert_eq!(optimized.get_message("empty").as_string(), "\u{0}");
    }

    #[test]
    fn tables_round_trip() {
        let source = translation(&[("a", "1"), ("b", "2")]);
        let mut optimized = OptimizedTranslation::new();
        optimized.generate(&source);
        let hash_table = optimized.get_hash_table().to_vec();
        let bucket_table = optimized.get_bucket_table().to_vec();
        let strings = optimized.get_strings().to_vec();

        let mut restored = OptimizedTranslation::new();
        restored.set_tables(hash_table, bucket_table, strings);
        assert_eq!(restored.get_message("a").as_string(), "1");
        assert_eq!(restored.get_message("b").as_string(), "2");
        assert_eq!(restored.get_message("c").as_string(), "");
    }
}
