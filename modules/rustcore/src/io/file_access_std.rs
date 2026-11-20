//! `core/io/file_access.*` over the host file system.
//!
//! The engine's `FileAccess` is an abstract handle with platform subclasses.
//! This port provides a `std::fs`-backed implementation with the engine's
//! surface subset:
//!
//! - `open` with the four modes (`READ`, `WRITE`, `READ_WRITE`, `WRITE_READ`;
//!   `WRITE` truncates or creates, `READ_WRITE` requires the file to exist);
//! - endian-aware `get_8/16/32/64`, `get_float/double/real` and `store_*`,
//!   `set_big_endian`;
//! - `get_buffer`/`store_buffer`, `seek`/`seek_end`, position/length, the sticky
//!   `eof_reached` flag, `flush` and `close`;
//! - `get_line` (drops `\n` and a preceding `\r`), `get_as_text(skip_cr)`,
//!   `store_string`/`store_line`, `get_csv_line`/`store_csv_line`.
//!
//! Writes go straight to the OS handle (the engine buffers; the visible result
//! after `flush` is the same). Error strings, compression and `get_var` are not
//! ported.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileMode {
    Read,
    Write,
    ReadWrite,
    WriteRead,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileError {
    Ok,
    /// `ERR_FILE_NOT_FOUND`.
    FileNotFound,
    /// `ERR_FILE_CANT_OPEN`.
    CantOpen,
    /// `ERR_FILE_CANT_WRITE`.
    CantWrite,
    /// `ERR_FILE_EOF`.
    Eof,
    /// The mode does not allow the operation.
    Unsupported,
}

pub struct FileAccessStd {
    file: File,
    path: String,
    readable: bool,
    writable: bool,
    big_endian: bool,
    eof: bool,
}

impl FileAccessStd {
    /// `FileAccess::open`.
    pub fn open(path: &str, mode: FileMode) -> Result<Self, FileError> {
        let mut options = OpenOptions::new();
        let (readable, writable) = match mode {
            FileMode::Read => {
                options.read(true);
                (true, false)
            }
            FileMode::Write => {
                options.write(true).create(true).truncate(true);
                (false, true)
            }
            FileMode::ReadWrite => {
                options.read(true).write(true);
                (true, true)
            }
            FileMode::WriteRead => {
                options.read(true).write(true).create(true);
                (true, true)
            }
        };
        match options.open(path) {
            Ok(file) => Ok(Self {
                file,
                path: path.to_string(),
                readable,
                writable,
                big_endian: false,
                eof: false,
            }),
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => Err(FileError::FileNotFound),
                _ => Err(FileError::CantOpen),
            },
        }
    }

    /// Creates a file with the given contents (helper for tests and callers
    /// that need a fresh file).
    pub fn create_with(path: &str, contents: &[u8]) -> Result<Self, FileError> {
        let mut access = Self::open(path, FileMode::Write)?;
        access.store_buffer(contents);
        access.flush();
        Self::open(path, FileMode::Read)
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn set_big_endian(&mut self, big_endian: bool) {
        self.big_endian = big_endian;
    }

    pub fn is_big_endian(&self) -> bool {
        self.big_endian
    }

    pub fn eof_reached(&self) -> bool {
        self.eof
    }

    pub fn get_error(&self) -> FileError {
        if self.eof {
            FileError::Eof
        } else {
            FileError::Ok
        }
    }

    pub fn get_position(&mut self) -> Result<u64, FileError> {
        self.file.stream_position().map_err(|_| FileError::CantOpen)
    }

    pub fn get_length(&mut self) -> Result<u64, FileError> {
        let position = self.get_position()?;
        let length = self
            .file
            .seek(SeekFrom::End(0))
            .map_err(|_| FileError::CantOpen)?;
        self.file
            .seek(SeekFrom::Start(position))
            .map_err(|_| FileError::CantOpen)?;
        Ok(length)
    }

    pub fn seek(&mut self, position: u64) -> Result<(), FileError> {
        self.file
            .seek(SeekFrom::Start(position))
            .map_err(|_| FileError::CantOpen)?;
        self.eof = false;
        Ok(())
    }

    /// `seek_end`: a negative offset is allowed down to the start.
    pub fn seek_end(&mut self, offset: i64) -> Result<(), FileError> {
        self.file
            .seek(SeekFrom::End(offset))
            .map_err(|_| FileError::CantOpen)?;
        self.eof = false;
        Ok(())
    }

    fn read_bytes(&mut self, count: usize) -> Vec<u8> {
        if !self.readable {
            self.eof = true;
            return Vec::new();
        }
        let mut buffer = vec![0u8; count];
        let mut read = 0usize;
        while read < count {
            match self.file.read(&mut buffer[read..]) {
                Ok(0) => break,
                Ok(amount) => read += amount,
                Err(_) => break,
            }
        }
        if read < count {
            self.eof = true;
        }
        buffer.truncate(read);
        buffer
    }

    pub fn get_8(&mut self) -> u8 {
        self.read_bytes(1).first().copied().unwrap_or(0)
    }

    pub fn get_16(&mut self) -> u16 {
        let mut bytes = [0u8; 2];
        let read = self.read_bytes(2);
        bytes[..read.len()].copy_from_slice(&read);
        if self.big_endian {
            u16::from_be_bytes(bytes)
        } else {
            u16::from_le_bytes(bytes)
        }
    }

    pub fn get_32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        let read = self.read_bytes(4);
        bytes[..read.len()].copy_from_slice(&read);
        if self.big_endian {
            u32::from_be_bytes(bytes)
        } else {
            u32::from_le_bytes(bytes)
        }
    }

    pub fn get_64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        let read = self.read_bytes(8);
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

    pub fn get_real(&mut self) -> f32 {
        self.get_float()
    }

    pub fn get_buffer(&mut self, count: usize) -> Vec<u8> {
        self.read_bytes(count)
    }

    /// `get_line`: up to `\n`, dropping it and a preceding `\r`.
    pub fn get_line(&mut self) -> String {
        let mut line = Vec::new();
        loop {
            let byte = self.get_8();
            if self.eof {
                break;
            }
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

    /// `get_as_text(skip_cr)`: consumes the rest of the file.
    pub fn get_as_text(&mut self, skip_cr: bool) -> String {
        let mut all = Vec::new();
        loop {
            let chunk = self.get_buffer(4096);
            if chunk.is_empty() {
                break;
            }
            all.extend_from_slice(&chunk);
            if self.eof {
                break;
            }
        }
        let text = if skip_cr {
            all.into_iter().filter(|byte| *byte != b'\r').collect()
        } else {
            all
        };
        String::from_utf8_lossy(&text).into_owned()
    }

    /// `get_csv_line`: quoted fields with `""` escapes; CR before LF is dropped.
    pub fn get_csv_line(&mut self, delimiter: char) -> Vec<String> {
        let delimiter = delimiter as u8;
        let mut fields = Vec::new();
        let mut field = Vec::new();
        let mut in_quotes = false;
        loop {
            if self.eof {
                break;
            }
            let byte = self.get_8();
            if self.eof {
                break;
            }
            if in_quotes {
                if byte == b'"' {
                    let next = self.get_8();
                    if !self.eof && next == b'"' {
                        field.push(b'"');
                    } else {
                        in_quotes = false;
                        if !self.eof {
                            if next == delimiter {
                                fields.push(String::from_utf8_lossy(&field).into_owned());
                                field.clear();
                            } else if next == b'\n' {
                                break;
                            } else {
                                field.push(next);
                            }
                        }
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
            } else if byte != b'\r' {
                field.push(byte);
            }
        }
        fields.push(String::from_utf8_lossy(&field).into_owned());
        fields
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> bool {
        if !self.writable {
            return false;
        }
        match self.file.write_all(bytes) {
            Ok(()) => true,
            Err(_) => false,
        }
    }

    pub fn store_8(&mut self, value: u8) -> bool {
        self.write_bytes(&[value])
    }

    pub fn store_16(&mut self, value: u16) -> bool {
        let bytes = if self.big_endian {
            value.to_be_bytes()
        } else {
            value.to_le_bytes()
        };
        self.write_bytes(&bytes)
    }

    pub fn store_32(&mut self, value: u32) -> bool {
        let bytes = if self.big_endian {
            value.to_be_bytes()
        } else {
            value.to_le_bytes()
        };
        self.write_bytes(&bytes)
    }

    pub fn store_64(&mut self, value: u64) -> bool {
        let bytes = if self.big_endian {
            value.to_be_bytes()
        } else {
            value.to_le_bytes()
        };
        self.write_bytes(&bytes)
    }

    pub fn store_float(&mut self, value: f32) -> bool {
        self.store_32(value.to_bits())
    }

    pub fn store_double(&mut self, value: f64) -> bool {
        self.store_64(value.to_bits())
    }

    pub fn store_real(&mut self, value: f32) -> bool {
        self.store_float(value)
    }

    pub fn store_buffer(&mut self, bytes: &[u8]) -> bool {
        self.write_bytes(bytes)
    }

    pub fn store_string(&mut self, text: &str) -> bool {
        self.write_bytes(text.as_bytes())
    }

    pub fn store_line(&mut self, text: &str) -> bool {
        if !self.store_string(text) {
            return false;
        }
        self.store_8(b'\n')
    }

    /// `store_csv_line`: quotes fields containing the delimiter, a quote, CR or
    /// LF, doubling embedded quotes.
    pub fn store_csv_line(&mut self, values: &[String], delimiter: char) -> bool {
        let delimiter_byte = delimiter as u8;
        let mut line = String::new();
        for (index, value) in values.iter().enumerate() {
            if index > 0 {
                line.push(delimiter);
            }
            let needs_quotes = value.bytes().any(|byte| {
                byte == delimiter_byte || byte == b'"' || byte == b'\n' || byte == b'\r'
            });
            if needs_quotes {
                line.push('"');
                line.push_str(&value.replace('"', "\"\""));
                line.push('"');
            } else {
                line.push_str(value);
            }
        }
        if !self.store_string(&line) {
            return false;
        }
        self.store_8(b'\n')
    }

    pub fn flush(&mut self) {
        let _ = self.file.flush();
    }

    /// `close`: the handle drops; flushing keeps the same guarantee.
    pub fn close(mut self) {
        self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_path(name: &str) -> String {
        let mut path = PathBuf::from(std::env::temp_dir());
        path.push(format!(
            "godot_rust_file_access_{}_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
            name
        ));
        path.to_string_lossy().into_owned()
    }

    fn cleanup(path: &str) {
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn write_and_read_scalars() {
        let path = temp_path("scalars.bin");
        {
            let mut file = FileAccessStd::open(&path, FileMode::Write).expect("created");
            assert!(file.store_8(0x12));
            assert!(file.store_16(0x3456));
            assert!(file.store_32(0x789a_bcde));
            assert!(file.store_64(0x0123_4567_89ab_cdef));
            assert!(file.store_float(1.5));
            assert!(file.store_double(-2.25));
            file.flush();
        }
        let mut file = FileAccessStd::open(&path, FileMode::Read).expect("opened");
        assert_eq!(file.get_8(), 0x12);
        assert_eq!(file.get_16(), 0x3456);
        assert_eq!(file.get_32(), 0x789a_bcde);
        assert_eq!(file.get_64(), 0x0123_4567_89ab_cdef);
        assert_eq!(file.get_float(), 1.5);
        assert_eq!(file.get_double(), -2.25);
        assert!(!file.eof_reached());
        assert_eq!(file.get_length().unwrap(), 27, "1 + 2 + 4 + 8 + 4 + 8 bytes");
        cleanup(&path);

        // Big-endian round trip.
        let path = temp_path("be.bin");
        {
            let mut file = FileAccessStd::open(&path, FileMode::Write).unwrap();
            file.set_big_endian(true);
            file.store_16(0x1234);
            file.store_32(0x0102_0304);
            file.flush();
        }
        let mut file = FileAccessStd::open(&path, FileMode::Read).unwrap();
        file.set_big_endian(true);
        assert_eq!(file.get_16(), 0x1234);
        assert_eq!(file.get_32(), 0x0102_0304);
        cleanup(&path);
    }

    #[test]
    fn seeking_and_eof() {
        let path = temp_path("seek.txt");
        FileAccessStd::create_with(&path, b"0123456789").expect("created");
        let mut file = FileAccessStd::open(&path, FileMode::Read).unwrap();
        assert_eq!(file.get_buffer(4), b"0123".to_vec());
        assert_eq!(file.get_position().unwrap(), 4);
        file.seek(8).unwrap();
        assert_eq!(file.get_buffer(10), b"89".to_vec());
        assert!(file.eof_reached());
        assert_eq!(file.get_error(), FileError::Eof);
        file.seek(0).unwrap();
        assert!(!file.eof_reached());
        file.seek_end(-2).unwrap();
        assert_eq!(file.get_buffer(2), b"89".to_vec());
        assert_eq!(file.get_length().unwrap(), 10);
        cleanup(&path);

        // Reading a missing file fails; WRITE creates.
        assert_eq!(
            FileAccessStd::open("definitely_missing_file_xyz.bin", FileMode::Read).err(),
            Some(FileError::FileNotFound)
        );
    }

    #[test]
    fn lines_and_text() {
        let path = temp_path("lines.txt");
        {
            let mut file = FileAccessStd::open(&path, FileMode::Write).unwrap();
            file.store_string("one\r\ntwo\nthree");
            file.flush();
        }
        let mut file = FileAccessStd::open(&path, FileMode::Read).unwrap();
        assert_eq!(file.get_line(), "one");
        assert_eq!(file.get_line(), "two");
        assert_eq!(file.get_line(), "three");
        assert!(file.eof_reached());
        assert_eq!(file.get_line(), "");
        cleanup(&path);

        let path = temp_path("text.txt");
        FileAccessStd::create_with(&path, b"a\r\nb\r\n").unwrap();
        let mut file = FileAccessStd::open(&path, FileMode::Read).unwrap();
        assert_eq!(file.get_as_text(true), "a\nb\n");
        cleanup(&path);
        FileAccessStd::create_with(&path, b"a\r\nb").unwrap();
        let mut file = FileAccessStd::open(&path, FileMode::Read).unwrap();
        assert_eq!(file.get_as_text(false), "a\r\nb");
        cleanup(&path);
    }

    #[test]
    fn csv_round_trip() {
        let path = temp_path("csv.txt");
        {
            let mut file = FileAccessStd::open(&path, FileMode::Write).unwrap();
            assert!(file.store_csv_line(
                &[
                    "plain".to_string(),
                    "with,comma".to_string(),
                    "with\"quote".to_string(),
                ],
                ','
            ));
            file.flush();
        }
        let mut file = FileAccessStd::open(&path, FileMode::Read).unwrap();
        assert_eq!(
            file.get_csv_line(','),
            vec!["plain", "with,comma", "with\"quote"]
        );
        cleanup(&path);

        // Explicit content exercises the parser edges.
        let path = temp_path("csv2.txt");
        FileAccessStd::create_with(&path, b"a,b,\n\"\",x\r\n").unwrap();
        let mut file = FileAccessStd::open(&path, FileMode::Read).unwrap();
        assert_eq!(file.get_csv_line(','), vec!["a", "b", ""]);
        assert_eq!(file.get_csv_line(','), vec!["", "x"]);
        cleanup(&path);
    }

    #[test]
    fn read_only_rejects_writes() {
        let path = temp_path("ro.txt");
        FileAccessStd::create_with(&path, b"data").unwrap();
        let mut file = FileAccessStd::open(&path, FileMode::Read).unwrap();
        assert!(!file.store_8(1));
        assert!(file.get_buffer(4) == b"data".to_vec());
        cleanup(&path);

        // READ_WRITE keeps existing contents; WRITE truncates.
        let path = temp_path("rw.txt");
        FileAccessStd::create_with(&path, b"0123456789").unwrap();
        {
            let mut file = FileAccessStd::open(&path, FileMode::ReadWrite).unwrap();
            file.seek(5).unwrap();
            assert!(file.store_8(b'X'));
            file.flush();
        }
        let mut file = FileAccessStd::open(&path, FileMode::Read).unwrap();
        assert_eq!(file.get_buffer(10), b"01234X6789".to_vec());
        cleanup(&path);

        let path = temp_path("trunc.txt");
        FileAccessStd::create_with(&path, b"old").unwrap();
        FileAccessStd::open(&path, FileMode::Write).unwrap().close();
        let mut file = FileAccessStd::open(&path, FileMode::Read).unwrap();
        assert!(file.get_buffer(10).is_empty());
        cleanup(&path);
    }
}
