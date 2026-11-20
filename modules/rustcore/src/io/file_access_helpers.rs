//! `core/io/file_access.*`: the binary/text helpers on a byte buffer.
//!
//! The engine's `FileAccess` is an abstract handle over platform files; this
//! port models the same read/write helpers over an in-memory buffer (it pairs
//! with `io/file_access_memory.rs`):
//!
//! - endian-aware `get_8/16/32/64`, `get_float/double/real` and the matching
//!   `store_*` forms, with `set_big_endian`;
//! - `get_buffer`, `store_buffer`, `seek`, `seek_end`, position/length and the
//!   sticky `eof_reached` flag (a short read sets it, like the engine);
//! - `get_line` (drops `\n` and a preceding `\r`), `get_as_text(skip_cr)`,
//!   `store_string`/`store_line`;
//! - `get_csv_line`/`store_csv_line` with quoted fields and doubled quotes.
//!
//! Platform files, compression, `get_var`/`store_var` and the error strings are
//! not ported; errors are reported through [`FileAccessError`].

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileAccessError {
    Ok,
    /// `ERR_FILE_EOF`.
    Eof,
}

#[derive(Clone, Debug)]
pub struct FileAccessBuffer {
    data: Vec<u8>,
    pos: usize,
    big_endian: bool,
    eof: bool,
}

impl FileAccessBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            pos: 0,
            big_endian: false,
            eof: false,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::new(bytes.to_vec())
    }

    pub fn set_big_endian(&mut self, big_endian: bool) {
        self.big_endian = big_endian;
    }

    pub fn is_big_endian(&self) -> bool {
        self.big_endian
    }

    pub fn get_length(&self) -> usize {
        self.data.len()
    }

    pub fn get_position(&self) -> usize {
        self.pos
    }

    pub fn eof_reached(&self) -> bool {
        self.eof
    }

    pub fn get_error(&self) -> FileAccessError {
        if self.eof {
            FileAccessError::Eof
        } else {
            FileAccessError::Ok
        }
    }

    pub fn seek(&mut self, position: usize) {
        // The engine also accepts the end position.
        if position <= self.data.len() {
            self.pos = position;
            if self.pos < self.data.len() {
                self.eof = false;
            }
        }
    }

    pub fn seek_end(&mut self, offset: isize) {
        let target = self.data.len() as isize + offset;
        if target >= 0 {
            self.seek(target as usize);
        }
    }

    fn read_bytes(&mut self, count: usize) -> &[u8] {
        let start = self.pos.min(self.data.len());
        let end = (start + count).min(self.data.len());
        if end - start < count {
            self.eof = true;
        }
        self.pos = end;
        &self.data[start..end]
    }

    pub fn get_8(&mut self) -> u8 {
        let bytes = self.read_bytes(1);
        bytes.first().copied().unwrap_or(0)
    }

    pub fn get_16(&mut self) -> u16 {
        let mut bytes = [0u8; 2];
        let read = self.read_bytes(2).to_vec();
        bytes[..read.len()].copy_from_slice(&read);
        if self.big_endian {
            u16::from_be_bytes(bytes)
        } else {
            u16::from_le_bytes(bytes)
        }
    }

    pub fn get_32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        let read = self.read_bytes(4).to_vec();
        bytes[..read.len()].copy_from_slice(&read);
        if self.big_endian {
            u32::from_be_bytes(bytes)
        } else {
            u32::from_le_bytes(bytes)
        }
    }

    pub fn get_64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        let read = self.read_bytes(8).to_vec();
        bytes[..read.len()].copy_from_slice(&read);
        if self.big_endian {
            u64::from_be_bytes(bytes)
        } else {
            u64::from_le_bytes(bytes)
        }
    }

    pub fn get_float(&mut self) -> f32 {
        f32::from_bits(self.get_32())
    }

    pub fn get_double(&mut self) -> f64 {
        f64::from_bits(self.get_64())
    }

    /// `get_real` (the build uses `real_t = float`).
    pub fn get_real(&mut self) -> f32 {
        self.get_float()
    }

    pub fn get_buffer(&mut self, len: usize) -> Vec<u8> {
        self.read_bytes(len).to_vec()
    }

    /// `get_line`: reads up to `\n`, dropping it and a preceding `\r`.
    pub fn get_line(&mut self) -> String {
        if self.pos >= self.data.len() {
            self.eof = true;
            return String::new();
        }
        let mut line = Vec::new();
        loop {
            if self.pos >= self.data.len() {
                self.eof = true;
                break;
            }
            let byte = self.data[self.pos];
            self.pos += 1;
            if byte == b'\n' {
                break;
            }
            line.push(byte);
        }
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        String::from_utf8_lossy(&line).into_owned()
    }

    /// `get_as_text(skip_cr)`: consumes the rest of the buffer.
    pub fn get_as_text(&mut self, skip_cr: bool) -> String {
        let rest = &self.data[self.pos.min(self.data.len())..];
        let text = if skip_cr {
            rest.iter()
                .filter(|byte| **byte != b'\r')
                .copied()
                .collect::<Vec<u8>>()
        } else {
            rest.to_vec()
        };
        self.pos = self.data.len();
        String::from_utf8_lossy(&text).into_owned()
    }

    /// `get_csv_line`: one record with quoted fields (`""` escapes a quote).
    pub fn get_csv_line(&mut self, delim: char) -> Vec<String> {
        let delimiter = delim as u8;
        let mut fields = Vec::new();
        let mut field = Vec::new();
        let mut in_quotes = false;

        loop {
            if self.pos >= self.data.len() {
                self.eof = true;
                break;
            }
            let byte = self.data[self.pos];
            self.pos += 1;

            if in_quotes {
                if byte == b'"' {
                    if self.pos < self.data.len() && self.data[self.pos] == b'"' {
                        field.push(b'"');
                        self.pos += 1;
                    } else {
                        in_quotes = false;
                    }
                } else {
                    field.push(byte);
                }
            } else if byte == b'"' {
                in_quotes = true;
            } else if byte == delimiter {
                fields.push(String::from_utf8_lossy(&field).into_owned());
                field.clear();
            } else if byte == b'\n' {
                break;
            } else if byte == b'\r' {
                // A CR before LF is ignored.
                if self.pos < self.data.len() && self.data[self.pos] == b'\n' {
                    continue;
                }
                field.push(byte);
            } else {
                field.push(byte);
            }
        }

        fields.push(String::from_utf8_lossy(&field).into_owned());
        // A record ending exactly at the buffer's end leaves the stream at EOF.
        if self.pos >= self.data.len() {
            self.eof = true;
        }
        fields
    }

    // -- writing ------------------------------------------------------------

    fn write_bytes(&mut self, bytes: &[u8]) {
        let end = self.pos + bytes.len();
        if self.data.len() < end {
            self.data.resize(end, 0);
        }
        self.data[self.pos..end].copy_from_slice(bytes);
        self.pos = end;
        if self.pos < self.data.len() {
            self.eof = false;
        }
    }

    pub fn store_8(&mut self, value: u8) {
        self.write_bytes(&[value]);
    }

    pub fn store_16(&mut self, value: u16) {
        let bytes = if self.big_endian {
            value.to_be_bytes()
        } else {
            value.to_le_bytes()
        };
        self.write_bytes(&bytes);
    }

    pub fn store_32(&mut self, value: u32) {
        let bytes = if self.big_endian {
            value.to_be_bytes()
        } else {
            value.to_le_bytes()
        };
        self.write_bytes(&bytes);
    }

    pub fn store_64(&mut self, value: u64) {
        let bytes = if self.big_endian {
            value.to_be_bytes()
        } else {
            value.to_le_bytes()
        };
        self.write_bytes(&bytes);
    }

    pub fn store_float(&mut self, value: f32) {
        self.store_32(value.to_bits());
    }

    pub fn store_double(&mut self, value: f64) {
        self.store_64(value.to_bits());
    }

    pub fn store_real(&mut self, value: f32) {
        self.store_float(value);
    }

    pub fn store_buffer(&mut self, bytes: &[u8]) {
        self.write_bytes(bytes);
    }

    pub fn store_string(&mut self, text: &str) {
        self.write_bytes(text.as_bytes());
    }

    pub fn store_line(&mut self, text: &str) {
        self.store_string(text);
        self.store_8(b'\n');
    }

    /// `store_csv_line`: quotes fields containing the delimiter, a quote, CR or
    /// LF, doubling embedded quotes.
    pub fn store_csv_line(&mut self, values: &[String], delim: char) {
        let delimiter = delim as u8;
        let mut line = String::new();
        for (index, value) in values.iter().enumerate() {
            if index > 0 {
                line.push(delim);
            }
            let needs_quotes = value.bytes().any(|byte| {
                byte == delimiter || byte == b'"' || byte == b'\n' || byte == b'\r'
            });
            if needs_quotes {
                line.push('"');
                line.push_str(&value.replace('"', "\"\""));
                line.push('"');
            } else {
                line.push_str(value);
            }
        }
        self.store_line(&line);
    }

    pub fn flush(&mut self) {}

    /// The backing bytes (after writes/seek).
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endian_reads_and_writes() {
        let mut file = FileAccessBuffer::from_bytes(&[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(file.get_16(), 0x0201, "little endian by default");
        assert_eq!(file.get_position(), 2);
        file.seek(0);
        file.set_big_endian(true);
        assert!(file.is_big_endian());
        assert_eq!(file.get_16(), 0x0102);
        file.seek(0);
        assert_eq!(file.get_32(), 0x01020304);
        assert_eq!(file.get_8(), 0, "past the end reads zero");
        assert!(file.eof_reached());

        let mut out = FileAccessBuffer::new(Vec::new());
        out.store_16(0x1234);
        out.store_32(0x01020304);
        out.store_float(1.5);
        out.set_big_endian(true);
        out.store_16(0x1234);
        assert_eq!(
            out.get_data(),
            &[0x34, 0x12, 0x04, 0x03, 0x02, 0x01, 0x00, 0x00, 0xC0, 0x3F, 0x12, 0x34]
        );
    }

    #[test]
    fn lines_and_text() {
        let mut file = FileAccessBuffer::from_bytes(b"one\r\ntwo\nthree");
        assert_eq!(file.get_line(), "one");
        assert_eq!(file.get_line(), "two");
        assert_eq!(file.get_line(), "three");
        assert!(file.eof_reached());
        assert_eq!(file.get_line(), "");

        let mut file = FileAccessBuffer::from_bytes(b"a\r\nb\r\n");
        assert_eq!(file.get_as_text(true), "a\nb\n");
        let mut file = FileAccessBuffer::from_bytes(b"a\r\nb");
        assert_eq!(file.get_as_text(false), "a\r\nb");
        assert_eq!(file.get_position(), file.get_length());
    }

    #[test]
    fn seeking_and_buffers() {
        let mut file = FileAccessBuffer::from_bytes(&[1, 2, 3, 4, 5]);
        assert_eq!(file.get_buffer(2), vec![1, 2]);
        file.seek_end(-2);
        assert_eq!(file.get_position(), 3);
        assert_eq!(file.get_buffer(10), vec![4, 5]);
        assert!(file.eof_reached());
        assert_eq!(file.get_error(), FileAccessError::Eof);
        file.seek(0);
        assert!(!file.eof_reached());
        assert_eq!(file.get_error(), FileAccessError::Ok);
        file.seek(99);
        assert_eq!(file.get_position(), 0, "seeking past the end is ignored");

        let mut file = FileAccessBuffer::from_bytes(&[1, 2, 3]);
        file.seek_end(0);
        file.store_8(4);
        assert_eq!(file.get_data(), &[1, 2, 3, 4]);
        // Overwriting in the middle keeps the tail.
        file.seek(1);
        file.store_8(9);
        assert_eq!(file.get_data(), &[1, 9, 3, 4]);
    }

    #[test]
    fn csv_round_trip() {
        let mut file = FileAccessBuffer::new(Vec::new());
        file.store_csv_line(
            &[
                "plain".to_string(),
                "with,comma".to_string(),
                "with\"quote".to_string(),
                "with\nnewline".to_string(),
            ],
            ',',
        );
        file.seek(0);
        assert_eq!(
            file.get_csv_line(','),
            vec!["plain", "with,comma", "with\"quote", "with\nnewline"]
        );
        assert!(file.eof_reached());

        // Trailing delimiter yields an empty last field.
        let mut file = FileAccessBuffer::from_bytes(b"a,b,\n");
        assert_eq!(file.get_csv_line(','), vec!["a", "b", ""]);
        // Quoted empty fields survive.
        let mut file = FileAccessBuffer::from_bytes(b"\"\",x\n");
        assert_eq!(file.get_csv_line(','), vec!["", "x"]);
        // Custom delimiters.
        let mut file = FileAccessBuffer::from_bytes(b"a;b;c");
        assert_eq!(file.get_csv_line(';'), vec!["a", "b", "c"]);
    }
}
