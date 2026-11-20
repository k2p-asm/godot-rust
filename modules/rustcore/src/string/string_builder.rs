//! `core/string/string_builder.*`: incremental string assembly.
//!
//! The engine appends `String`s and C strings to a small vector and caches the
//! total length, then copies everything into one string on `as_string()`. This
//! port keeps the same contract: empty appends are ignored (they still do not
//! count as appended strings), `num_strings_appended` counts real segments and
//! `get_string_length` is the cached code-point/byte length used here (the port
//! counts UTF-8 bytes, which matches the engine for ASCII and is documented).

#[derive(Clone, Debug, Default)]
pub struct StringBuilder {
    parts: Vec<String>,
    string_length: usize,
}

impl StringBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// `append(const String &)`: empty strings are ignored.
    pub fn append(&mut self, text: &str) -> &mut Self {
        if !text.is_empty() {
            self.parts.push(text.to_string());
            self.string_length += text.len();
        }
        self
    }

    /// `operator+`.
    pub fn plus(&self, text: &str) -> StringBuilder {
        let mut builder = self.clone();
        builder.append(text);
        builder
    }

    /// `operator+=`.
    pub fn add_assign(&mut self, text: &str) {
        self.append(text);
    }

    pub fn num_strings_appended(&self) -> usize {
        self.parts.len()
    }

    pub fn get_string_length(&self) -> usize {
        self.string_length
    }

    pub fn as_string(&self) -> String {
        let mut out = String::with_capacity(self.string_length);
        for part in &self.parts {
            out.push_str(part);
        }
        out
    }

    pub fn clear(&mut self) {
        self.parts.clear();
        self.string_length = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_and_lengths() {
        let mut builder = StringBuilder::new();
        assert_eq!(builder.as_string(), "");
        assert_eq!(builder.get_string_length(), 0);
        builder.append("");
        assert_eq!(builder.num_strings_appended(), 0, "empty appends are ignored");
        builder.append("Hello").append(", ").append("world");
        assert_eq!(builder.num_strings_appended(), 3);
        assert_eq!(builder.get_string_length(), 12);
        assert_eq!(builder.as_string(), "Hello, world");

        builder.add_assign("!");
        assert_eq!(builder.as_string(), "Hello, world!");
        assert_eq!(builder.get_string_length(), 13);

        let copy = builder.plus("?");
        assert_eq!(copy.as_string(), "Hello, world!?");
        assert_eq!(copy.num_strings_appended(), 5);
        assert_eq!(builder.as_string(), "Hello, world!", "operator+ copies");

        builder.clear();
        assert_eq!(builder.as_string(), "");
        assert_eq!(builder.num_strings_appended(), 0);
    }

    #[test]
    fn utf8_length_is_byte_length() {
        let mut builder = StringBuilder::new();
        builder.append("ñ");
        assert_eq!(builder.get_string_length(), 2);
        assert_eq!(builder.as_string(), "ñ");
        assert_eq!(builder.as_string().len(), 2);
    }
}
