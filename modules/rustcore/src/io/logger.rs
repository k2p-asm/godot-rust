//! `core/io/logger.*`: log entry formatting and an in-memory sink.
//!
//! Ported: the `ErrorType` descriptions and indent prefixes, and the shape of
//! `Logger::log_error` (`ERROR: msg` / `   at: func (file:line)` plus the code
//! and rationale lines) and `log_message`. The abstract `Logger` is represented
//! by [`MemoryLogger`], a sink that keeps every message so tests and embedders
//! can assert on them.
//!
//! File rotation, stdout flushing, `--quiet`, flush-on-print and the script
//! backtrace printing are not ported.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorType {
    Error,
    Warning,
    Script,
    Shader,
}

impl ErrorType {
    /// `Logger::error_type_string`.
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorType::Error => "ERROR",
            ErrorType::Warning => "WARNING",
            ErrorType::Script => "SCRIPT ERROR",
            ErrorType::Shader => "SHADER ERROR",
        }
    }

    /// `Logger::error_type_indent` (the spaces that align the message).
    pub fn indent(self) -> &'static str {
        match self {
            ErrorType::Error => "   ",
            ErrorType::Warning => "     ",
            ErrorType::Script => "          ",
            ErrorType::Shader => "          ",
        }
    }
}

/// One error line, split into the pieces the engine prints separately.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogEntry {
    pub error_type: ErrorType,
    pub function: String,
    pub file: String,
    pub line: i32,
    pub code: String,
    pub rationale: String,
    pub message: String,
}

impl LogEntry {
    /// The first line: `ERROR: message` (or `WARNING: ...`).
    pub fn headline(&self) -> String {
        format!("{}: {}", self.error_type.as_str(), self.message)
    }

    /// The location line the engine prints when a function/file is known:
    /// `<indent>at: <function> (<file>:<line>)`.
    pub fn location(&self) -> String {
        format!(
            "{}at: {} ({}:{})",
            self.error_type.indent(),
            self.function,
            self.file,
            self.line
        )
    }
}

/// A `Logger` that keeps everything in memory.
#[derive(Debug, Default)]
pub struct MemoryLogger {
    errors: Vec<LogEntry>,
    messages: Vec<String>,
    warnings: usize,
}

impl MemoryLogger {
    pub fn new() -> Self {
        Self::default()
    }

    /// `log_error`, minus the actual I/O.
    #[allow(clippy::too_many_arguments)]
    pub fn log_error(
        &mut self,
        error_type: ErrorType,
        function: &str,
        file: &str,
        line: i32,
        code: &str,
        rationale: &str,
        message: &str,
    ) {
        if error_type == ErrorType::Warning {
            self.warnings += 1;
        }
        self.errors.push(LogEntry {
            error_type,
            function: function.to_string(),
            file: file.to_string(),
            line,
            code: code.to_string(),
            rationale: rationale.to_string(),
            message: message.to_string(),
        });
    }

    pub fn log_message(&mut self, message: &str) {
        self.messages.push(message.to_string());
    }

    pub fn get_error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn get_warning_count(&self) -> usize {
        self.warnings
    }

    pub fn get_message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn get_errors(&self) -> &[LogEntry] {
        &self.errors
    }

    pub fn get_messages(&self) -> &[String] {
        &self.messages
    }

    pub fn clear(&mut self) {
        self.errors.clear();
        self.messages.clear();
        self.warnings = 0;
    }

    /// Everything the engine would write, one line per piece.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for entry in &self.errors {
            out.push_str(&entry.headline());
            out.push('\n');
            if !entry.function.is_empty() || !entry.file.is_empty() {
                out.push_str(&entry.location());
                out.push('\n');
            }
            if !entry.code.is_empty() {
                out.push_str(&format!("{}{}\n", entry.error_type.indent(), entry.code));
            }
            if !entry.rationale.is_empty() {
                out.push_str(&format!("{}{}\n", entry.error_type.indent(), entry.rationale));
            }
        }
        for message in &self.messages {
            out.push_str(message);
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_strings_and_indents() {
        assert_eq!(ErrorType::Error.as_str(), "ERROR");
        assert_eq!(ErrorType::Warning.as_str(), "WARNING");
        assert_eq!(ErrorType::Script.as_str(), "SCRIPT ERROR");
        assert_eq!(ErrorType::Shader.as_str(), "SHADER ERROR");
        assert_eq!(ErrorType::Error.indent(), "   ");
        assert_eq!(ErrorType::Warning.indent(), "     ");
        assert_eq!(ErrorType::Script.indent(), "          ");
        assert_eq!(ErrorType::Shader.indent(), "          ");
        assert_eq!(ErrorType::Error as i32, 0);
        assert_eq!(ErrorType::Warning as i32, 1);
        assert_eq!(ErrorType::Script as i32, 2);
        assert_eq!(ErrorType::Shader as i32, 3);
    }

    #[test]
    fn entries_format_like_the_engine() {
        let mut logger = MemoryLogger::new();
        logger.log_error(
            ErrorType::Error,
            "main",
            "main.cpp",
            42,
            "ERR_CANT_OPEN",
            "Is the path valid?",
            "Cannot open file.",
        );
        logger.log_error(ErrorType::Warning, "", "", 0, "", "", "Heads up.");
        logger.log_message("Engine started.");

        assert_eq!(logger.get_error_count(), 2);
        assert_eq!(logger.get_warning_count(), 1);
        assert_eq!(logger.get_message_count(), 1);

        let entry = &logger.get_errors()[0];
        assert_eq!(entry.headline(), "ERROR: Cannot open file.");
        assert_eq!(entry.location(), "   at: main (main.cpp:42)");

        let text = logger.to_text();
        assert!(text.contains("ERROR: Cannot open file.\n   at: main (main.cpp:42)\n   ERR_CANT_OPEN\n   Is the path valid?\n"));
        // No location line when both function and file are empty.
        assert!(text.contains("WARNING: Heads up.\n"));
        assert!(text.ends_with("Engine started.\n"));

        logger.clear();
        assert_eq!(logger.get_error_count(), 0);
        assert_eq!(logger.get_message_count(), 0);
        assert_eq!(logger.get_warning_count(), 0);
    }
}
