//! `scene/resources/audio_stream_wav.*`: the WAV stream and its RIFF loader.
//!
//! Ported from `AudioStreamWAV::load_from_buffer`: the `RIFF`/`WAVE` checks, the
//! `fmt ` chunk (PCM code 1 or IEEE float code 3, mono/stereo, rate, 8/16/24/32
//! bits, 32/64-bit floats), the `data` chunk conversion (unsigned 8-bit, signed
//! 16-bit, sign-extended 24/32-bit), the `smpl` loop points (forward, ping-pong,
//! backward), the `LIST/INFO` tags with the engine's id remapping, and the final
//! storage as 8- or 16-bit PCM.
//!
//! The importer options (rate limiting, normalization, trimming, forced mono,
//! 8-bit forcing) and the compressed formats (IMA-ADPCM, QOA) are not ported; they
//! are reported as `Unsupported`.
//!
//! Playback/resampling of the stored data is not ported.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum WavFormat {
    Format8Bits = 0,
    Format16Bits = 1,
    FormatImaAdpcm = 2,
    FormatQoa = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum LoopMode {
    Disabled = 0,
    Forward = 1,
    PingPong = 2,
    Backward = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WavError {
    /// Header is not `RIFF`.
    NotRiff,
    /// Header is not `WAVE`.
    NotWave,
    /// Compression code other than PCM (1) or IEEE float (3).
    UnsupportedFormat,
    /// Channel count other than 1 or 2.
    UnsupportedChannels,
    /// Bit depth not a multiple of 8 or zero.
    InvalidBits,
    /// Float depth not 32 or 64 bits.
    InvalidFloatBits,
    /// `data` before `fmt `, an empty channel count, or truncated chunks.
    Corrupt,
}

#[derive(Clone, Debug)]
pub struct AudioStreamWav {
    format: WavFormat,
    loop_mode: LoopMode,
    loop_begin: i32,
    loop_end: i32,
    mix_rate: i32,
    stereo: bool,
    data: Vec<u8>,
    tags: Vec<(String, String)>,
}

impl Default for AudioStreamWav {
    fn default() -> Self {
        Self::new()
    }
}

fn tag_remap(tag: &str) -> Option<&'static str> {
    Some(match tag {
        "IARL" => "location",
        "IART" => "artist",
        "ICMS" => "organization",
        "ICMT" => "comment",
        "ICNT" => "releasecountry",
        "ICOP" => "copyright",
        "ICRD" => "date",
        "IENC" => "encodedby",
        "IENG" => "engineer",
        "IFRM" => "tracktotal",
        "IGNR" => "genre",
        "IKEY" => "keywords",
        "ILNG" => "language",
        "IMED" => "media",
        "IMUS" => "composer",
        "INAM" => "title",
        "IPRD" => "album",
        "IPRO" => "producer",
        "IPRT" => "tracknumber",
        "ISBJ" => "description",
        "ISFT" => "encoder",
        "ISRF" => "media",
        "ITCH" => "encodedby",
        "ITRK" => "tracknumber",
        "IWRI" => "author",
        "TLEN" => "length",
        _ => return None,
    })
}

fn read_u16(data: &[u8], pos: usize) -> Option<u16> {
    let bytes = data.get(pos..pos + 2)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u32(data: &[u8], pos: usize) -> Option<u32> {
    let bytes = data.get(pos..pos + 4)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

impl AudioStreamWav {
    pub fn new() -> Self {
        Self {
            format: WavFormat::Format16Bits,
            loop_mode: LoopMode::Disabled,
            loop_begin: 0,
            loop_end: 0,
            mix_rate: 44100,
            stereo: false,
            data: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// `load_from_buffer` (without the importer options).
    pub fn load_from_buffer(bytes: &[u8]) -> Result<Self, WavError> {
        if bytes.len() < 12 || &bytes[0..4] != b"RIFF" {
            return Err(WavError::NotRiff);
        }
        if &bytes[8..12] != b"WAVE" {
            return Err(WavError::NotWave);
        }

        let mut format_found = false;
        let mut data_found = false;
        let mut compression: u16 = 1;
        let mut channels: u16 = 0;
        let mut rate: u32 = 0;
        let mut bits: u16 = 0;
        let mut loop_mode = LoopMode::Disabled;
        let mut loop_begin: i64 = 0;
        let mut loop_end: i64 = 0;
        let mut samples: Vec<f32> = Vec::new();
        let mut tags: Vec<(String, String)> = Vec::new();

        let mut pos = 12usize;
        while pos + 8 <= bytes.len() {
            let id = &bytes[pos..pos + 4];
            let size = read_u32(bytes, pos + 4).ok_or(WavError::Corrupt)? as usize;
            let chunk_start = pos + 8;
            let chunk_end = chunk_start.checked_add(size).ok_or(WavError::Corrupt)?;
            if chunk_end > bytes.len() {
                // The engine warns and continues with the available bytes for
                // `data`; other truncated chunks end the walk.
                if id != b"data" {
                    break;
                }
            }

            match id {
                b"fmt " if !format_found => {
                    compression = read_u16(bytes, chunk_start).ok_or(WavError::Corrupt)?;
                    if compression != 1 && compression != 3 {
                        return Err(WavError::UnsupportedFormat);
                    }
                    channels = read_u16(bytes, chunk_start + 2).ok_or(WavError::Corrupt)?;
                    if channels != 1 && channels != 2 {
                        return Err(WavError::UnsupportedChannels);
                    }
                    rate = read_u32(bytes, chunk_start + 4).ok_or(WavError::Corrupt)?;
                    bits = read_u16(bytes, chunk_start + 14).ok_or(WavError::Corrupt)?;
                    if bits == 0 || bits % 8 != 0 {
                        return Err(WavError::InvalidBits);
                    }
                    if compression == 3 && bits % 32 != 0 {
                        return Err(WavError::InvalidFloatBits);
                    }
                    format_found = true;
                }
                b"data" if !data_found => {
                    data_found = true;
                    if !format_found {
                        return Err(WavError::Corrupt);
                    }
                    if channels == 0 {
                        return Err(WavError::Corrupt);
                    }
                    let available = bytes.len().saturating_sub(chunk_start);
                    let declared = size.min(available);
                    let bytes_per_sample = (bits >> 3) as usize;
                    let frames = declared / channels as usize / bytes_per_sample;
                    let count = frames * channels as usize;
                    samples.reserve(count);
                    for index in 0..count {
                        let at = chunk_start + index * bytes_per_sample;
                        let sample = match (compression, bits) {
                            (1, 8) => (bytes[at] as i16 - 128) as f32 / 128.0,
                            (1, 16) => {
                                let raw = read_u16(bytes, at).ok_or(WavError::Corrupt)? as i16;
                                raw as f32 / 32768.0
                            }
                            (1, _) => {
                                // 16+ bits samples are signed; the engine keeps
                                // the top 16 bits after shifting.
                                let mut value: u32 = 0;
                                for byte in 0..bytes_per_sample {
                                    value |= (bytes[at + byte] as u32) << (byte * 8);
                                }
                                value <<= 32 - bits as u32;
                                ((value as i32) >> 16) as f32 / 32768.0
                            }
                            (3, 32) => {
                                let raw = read_u32(bytes, at).ok_or(WavError::Corrupt)?;
                                f32::from_bits(raw)
                            }
                            (3, 64) => {
                                let low = read_u32(bytes, at).ok_or(WavError::Corrupt)?;
                                let high = read_u32(bytes, at + 4).ok_or(WavError::Corrupt)?;
                                f64::from_bits(((high as u64) << 32) | low as u64) as f32
                            }
                            _ => return Err(WavError::UnsupportedFormat),
                        };
                        samples.push(sample);
                    }
                }
                b"smpl" => {
                    // 10 unused 32-bit words, then loop type and the two points.
                    let loop_type = read_u32(bytes, chunk_start + 40).ok_or(WavError::Corrupt)?;
                    if let 0..=2 = loop_type {
                        loop_mode = match loop_type {
                            0 => LoopMode::Forward,
                            1 => LoopMode::PingPong,
                            _ => LoopMode::Backward,
                        };
                        loop_begin = read_u32(bytes, chunk_start + 44).ok_or(WavError::Corrupt)?
                            as i64;
                        loop_end = read_u32(bytes, chunk_start + 48).ok_or(WavError::Corrupt)?
                            as i64;
                    }
                }
                b"LIST" => {
                    if bytes.get(chunk_start..chunk_start + 4) == Some(&b"INFO"[..]) {
                        let end = chunk_end.min(bytes.len());
                        let mut at = chunk_start + 4;
                        while at + 8 <= end {
                            let info_id = &bytes[at..at + 4];
                            let text_size = read_u32(bytes, at + 4).ok_or(WavError::Corrupt)? as usize;
                            at += 8;
                            if text_size == 0 {
                                continue;
                            }
                            if at + text_size > end {
                                break;
                            }
                            let text = String::from_utf8_lossy(&bytes[at..at + text_size])
                                .into_owned();
                            at += text_size;
                            if text_size & 1 == 1 {
                                at += 1;
                            }
                            let tag = String::from_utf8_lossy(info_id).into_owned();
                            let key = tag_remap(&tag).unwrap_or(&tag).to_string();
                            tags.push((key, text));
                        }
                    }
                }
                _ => {}
            }

            pos = chunk_end + (size & 1);
        }

        if !format_found || samples.is_empty() {
            return Err(WavError::Corrupt);
        }

        // Final storage: 8-bit unsigned for 8-bit input, otherwise 16-bit signed.
        let is16 = bits != 8;
        let mut data = Vec::with_capacity(if is16 {
            samples.len() * 2
        } else {
            samples.len()
        });
        for sample in &samples {
            if is16 {
                let value = (sample * 32768.0).clamp(-32768.0, 32767.0) as i16;
                data.extend_from_slice(&value.to_le_bytes());
            } else {
                let value = (sample * 128.0 + 128.0).clamp(0.0, 255.0) as u8;
                data.push(value);
            }
        }

        Ok(Self {
            format: if is16 {
                WavFormat::Format16Bits
            } else {
                WavFormat::Format8Bits
            },
            loop_mode,
            loop_begin: loop_begin as i32,
            loop_end: loop_end as i32,
            mix_rate: rate as i32,
            stereo: channels == 2,
            data,
            tags,
        })
    }

    pub fn get_format(&self) -> WavFormat {
        self.format
    }

    pub fn set_format(&mut self, format: WavFormat) {
        self.format = format;
    }

    pub fn get_loop_mode(&self) -> LoopMode {
        self.loop_mode
    }

    pub fn set_loop_mode(&mut self, mode: LoopMode) {
        self.loop_mode = mode;
    }

    pub fn get_loop_begin(&self) -> i32 {
        self.loop_begin
    }

    pub fn set_loop_begin(&mut self, begin: i32) {
        self.loop_begin = begin;
    }

    pub fn get_loop_end(&self) -> i32 {
        self.loop_end
    }

    pub fn set_loop_end(&mut self, end: i32) {
        self.loop_end = end;
    }

    pub fn get_mix_rate(&self) -> i32 {
        self.mix_rate
    }

    pub fn set_mix_rate(&mut self, rate: i32) {
        self.mix_rate = rate;
    }

    pub fn is_stereo(&self) -> bool {
        self.stereo
    }

    pub fn set_stereo(&mut self, stereo: bool) {
        self.stereo = stereo;
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    pub fn get_tags(&self) -> &[(String, String)] {
        &self.tags
    }

    pub fn set_tags(&mut self, tags: Vec<(String, String)>) {
        self.tags = tags;
    }

    /// Port helper: the stored samples as f32 (the inverse of the final
    /// conversion).
    pub fn get_samples(&self) -> Vec<f32> {
        match self.format {
            WavFormat::Format8Bits => self
                .data
                .iter()
                .map(|byte| (*byte as f32 - 128.0) / 128.0)
                .collect(),
            _ => self
                .data
                .chunks_exact(2)
                .map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32768.0)
                .collect(),
        }
    }

    /// `save_to_wav`: the RIFF file bytes (`Error::OK` equivalent is the byte
    /// vector; the engine's file write is not needed here).
    ///
    /// The engine's header layout is reproduced exactly: `RIFF` + 36 + data
    /// bytes, `fmt ` of 16 bytes, the format code (1, or 3 for IMA-ADPCM), the
    /// channel count, rate, byte rate and block align.
    ///
    /// Two engine quirks are preserved: IMA-ADPCM reports its format code and
    /// byte-aligned sizes but writes no sample data, and QOA is written as
    /// 16-bit PCM. The engine adds 128 to 8-bit samples because it stores them
    /// signed; this port's loader stores them unsigned, so they are written
    /// unchanged to keep the round trip exact.
    pub fn to_wav_bytes(&self) -> Vec<u8> {
        let data_bytes = self.data.len();
        let format_code: u16 = if self.format == WavFormat::FormatImaAdpcm {
            3
        } else {
            1
        };
        let channels: u16 = if self.stereo { 2 } else { 1 };
        let byte_pr_sample: u32 = match self.format {
            WavFormat::Format8Bits => 1,
            WavFormat::Format16Bits | WavFormat::FormatQoa => 2,
            WavFormat::FormatImaAdpcm => 4,
        };
        let rate = self.mix_rate as u32;

        let mut out = Vec::with_capacity(data_bytes + 44);
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(data_bytes as u32).wrapping_add(36).to_le_bytes());
        out.extend_from_slice(b"WAVE");
        out.extend_from_slice(b"fmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&format_code.to_le_bytes());
        out.extend_from_slice(&channels.to_le_bytes());
        out.extend_from_slice(&rate.to_le_bytes());
        out.extend_from_slice(&(rate * channels as u32 * byte_pr_sample).to_le_bytes());
        let block_align = (channels as u32 * byte_pr_sample) as u16;
        out.extend_from_slice(&block_align.to_le_bytes());
        out.extend_from_slice(&(byte_pr_sample as u16 * 8).to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&(data_bytes as u32).to_le_bytes());

        match self.format {
            WavFormat::Format8Bits => out.extend_from_slice(&self.data),
            WavFormat::Format16Bits | WavFormat::FormatQoa => {
                for pair in self.data.chunks_exact(2) {
                    out.extend_from_slice(&u16::from_le_bytes([pair[0], pair[1]]).to_le_bytes());
                }
            }
            // The engine leaves IMA-ADPCM sample data unimplemented.
            WavFormat::FormatImaAdpcm => {}
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(id: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(id);
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(body);
        if body.len() % 2 == 1 {
            out.push(0);
        }
        out
    }

    fn wav(channels: u16, rate: u32, bits: u16, samples: &[i16]) -> Vec<u8> {
        let mut fmt = Vec::new();
        fmt.extend_from_slice(&1u16.to_le_bytes()); // PCM
        fmt.extend_from_slice(&channels.to_le_bytes());
        fmt.extend_from_slice(&rate.to_le_bytes());
        fmt.extend_from_slice(&(rate * channels as u32 * bits as u32 / 8).to_le_bytes());
        fmt.extend_from_slice(&(channels * bits / 8).to_le_bytes());
        fmt.extend_from_slice(&bits.to_le_bytes());
        let mut data = Vec::new();
        for sample in samples {
            data.extend_from_slice(&sample.to_le_bytes());
        }
        let mut body = Vec::new();
        body.extend_from_slice(&chunk(b"fmt ", &fmt));
        body.extend_from_slice(&chunk(b"data", &data));
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&((body.len() + 4) as u32).to_le_bytes());
        out.extend_from_slice(b"WAVE");
        out.extend_from_slice(&body);
        out
    }

    #[test]
    fn mono_16_bit() {
        let bytes = wav(1, 22050, 16, &[0, 16384, -16384, 32767]);
        let wav = AudioStreamWav::load_from_buffer(&bytes).expect("parsed");
        assert_eq!(wav.get_format(), WavFormat::Format16Bits);
        assert_eq!(wav.get_mix_rate(), 22050);
        assert!(!wav.is_stereo());
        assert_eq!(wav.get_loop_mode(), LoopMode::Disabled);
        assert_eq!(wav.get_data().len(), 8);
        let samples = wav.get_samples();
        assert_eq!(samples.len(), 4);
        assert_eq!(samples[0], 0.0);
        assert!((samples[1] - 0.5).abs() < 1e-4);
        assert!((samples[2] + 0.5).abs() < 1e-4);
        assert!((samples[3] - 0.99997).abs() < 1e-3);
    }

    #[test]
    fn stereo_and_8_bit() {
        let stereo = wav(2, 48000, 16, &[0, 0, 100, -100]);
        let wav = AudioStreamWav::load_from_buffer(&stereo).expect("parsed");
        assert!(wav.is_stereo());
        assert_eq!(wav.get_mix_rate(), 48000);
        assert_eq!(wav.get_data().len(), 8);

        // 8-bit unsigned samples become FORMAT_8_BITS.
        let mut fmt = Vec::new();
        fmt.extend_from_slice(&1u16.to_le_bytes());
        fmt.extend_from_slice(&1u16.to_le_bytes());
        fmt.extend_from_slice(&8000u32.to_le_bytes());
        fmt.extend_from_slice(&8000u32.to_le_bytes());
        fmt.extend_from_slice(&1u16.to_le_bytes());
        fmt.extend_from_slice(&8u16.to_le_bytes());
        let data = [0u8, 128, 255];
        let mut body = Vec::new();
        body.extend_from_slice(&chunk(b"fmt ", &fmt));
        body.extend_from_slice(&chunk(b"data", &data));
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&((body.len() + 4) as u32).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(&body);
        let wav = AudioStreamWav::load_from_buffer(&bytes).expect("parsed");
        assert_eq!(wav.get_format(), WavFormat::Format8Bits);
        assert_eq!(wav.get_data(), &[0u8, 128, 255]);
        assert_eq!(wav.get_samples(), vec![-1.0, 0.0, 0.9921875]);
    }

    #[test]
    fn loop_points_and_tags() {
        let mut bytes = wav(1, 44100, 16, &[0, 0, 0, 0]);
        // Append `smpl`: 10 words, loop type 1 (ping-pong), begin 1, end 3.
        let mut smpl = Vec::new();
        for _ in 0..10 {
            smpl.extend_from_slice(&0u32.to_le_bytes());
        }
        smpl.extend_from_slice(&1u32.to_le_bytes());
        smpl.extend_from_slice(&1u32.to_le_bytes());
        smpl.extend_from_slice(&3u32.to_le_bytes());
        smpl.extend_from_slice(&[0u8; 8]);
        let mut info = Vec::new();
        info.extend_from_slice(b"INFO");
        info.extend_from_slice(b"INAM");
        info.extend_from_slice(&5u32.to_le_bytes());
        info.extend_from_slice(b"Title");
        info.push(0); // RIFF pads odd-sized entries with a byte.
        info.extend_from_slice(b"IART");
        info.extend_from_slice(&4u32.to_le_bytes());
        info.extend_from_slice(b"Bob\0");
        let mut extra = chunk(b"smpl", &smpl);
        extra.extend_from_slice(&chunk(b"LIST", &info));
        bytes.extend_from_slice(&extra);
        // Fix the RIFF size.
        let size = (bytes.len() - 8) as u32;
        bytes[4..8].copy_from_slice(&size.to_le_bytes());

        let wav = AudioStreamWav::load_from_buffer(&bytes).expect("parsed");
        assert_eq!(wav.get_loop_mode(), LoopMode::PingPong);
        assert_eq!(wav.get_loop_begin(), 1);
        assert_eq!(wav.get_loop_end(), 3);
        let tags = wav.get_tags();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0], ("title".to_string(), "Title".to_string()));
        assert_eq!(tags[1], ("artist".to_string(), "Bob\0".to_string()));
    }

    #[test]
    fn float_32_bit() {
        let mut fmt = Vec::new();
        fmt.extend_from_slice(&3u16.to_le_bytes()); // IEEE float
        fmt.extend_from_slice(&1u16.to_le_bytes());
        fmt.extend_from_slice(&44100u32.to_le_bytes());
        fmt.extend_from_slice(&(44100u32 * 4).to_le_bytes());
        fmt.extend_from_slice(&4u16.to_le_bytes());
        fmt.extend_from_slice(&32u16.to_le_bytes());
        let mut data = Vec::new();
        for value in [0.0f32, 0.25, -0.5] {
            data.extend_from_slice(&value.to_le_bytes());
        }
        let mut body = Vec::new();
        body.extend_from_slice(&chunk(b"fmt ", &fmt));
        body.extend_from_slice(&chunk(b"data", &data));
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&((body.len() + 4) as u32).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(&body);

        let wav = AudioStreamWav::load_from_buffer(&bytes).expect("parsed");
        assert_eq!(wav.get_format(), WavFormat::Format16Bits);
        let samples = wav.get_samples();
        assert!((samples[1] - 0.25).abs() < 1e-3);
        assert!((samples[2] + 0.5).abs() < 1e-3);
    }

    #[test]
    fn errors() {
        assert_eq!(
            AudioStreamWav::load_from_buffer(b"nope").unwrap_err(),
            WavError::NotRiff
        );
        let mut bytes = wav(1, 44100, 16, &[0]);
        bytes[8] = b'X';
        assert_eq!(
            AudioStreamWav::load_from_buffer(&bytes).unwrap_err(),
            WavError::NotWave
        );

        // Three channels are refused.
        let mut fmt = Vec::new();
        fmt.extend_from_slice(&1u16.to_le_bytes());
        fmt.extend_from_slice(&3u16.to_le_bytes());
        fmt.extend_from_slice(&44100u32.to_le_bytes());
        fmt.extend_from_slice(&0u32.to_le_bytes());
        fmt.extend_from_slice(&0u16.to_le_bytes());
        fmt.extend_from_slice(&16u16.to_le_bytes());
        let mut body = Vec::new();
        body.extend_from_slice(&chunk(b"fmt ", &fmt));
        body.extend_from_slice(&chunk(b"data", &[0, 0]));
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&((body.len() + 4) as u32).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(&body);
        assert_eq!(
            AudioStreamWav::load_from_buffer(&bytes).unwrap_err(),
            WavError::UnsupportedChannels
        );

        // Truncated/empty data.
        let mut bytes = wav(1, 44100, 16, &[]);
        assert_eq!(
            AudioStreamWav::load_from_buffer(&bytes).unwrap_err(),
            WavError::Corrupt
        );
        bytes.extend_from_slice(b"");
        // Setting fields round-trips.
        let mut wav = AudioStreamWav::new();
        assert_eq!(wav.get_format(), WavFormat::Format16Bits);
        assert_eq!(wav.get_mix_rate(), 44100);
        wav.set_mix_rate(8000);
        wav.set_stereo(true);
        wav.set_format(WavFormat::FormatImaAdpcm);
        wav.set_loop_mode(LoopMode::Forward);
        wav.set_loop_begin(2);
        wav.set_loop_end(9);
        wav.set_tags(vec![("title".to_string(), "T".to_string())]);
        assert_eq!(wav.get_mix_rate(), 8000);
        assert!(wav.is_stereo());
        assert_eq!(wav.get_format(), WavFormat::FormatImaAdpcm);
        assert_eq!(wav.get_loop_mode(), LoopMode::Forward);
        assert_eq!(wav.get_loop_begin(), 2);
        assert_eq!(wav.get_loop_end(), 9);
        assert_eq!(wav.get_tags().len(), 1);
        assert_eq!(WavFormat::Format8Bits as i32, 0);
        assert_eq!(LoopMode::Backward as i32, 3);
    }

    #[test]
    fn save_round_trips_pcm() {
        // 16-bit stereo: the header is the engine's layout and the payload is
        // the stored little-endian samples.
        let mut wav = AudioStreamWav::new();
        wav.set_mix_rate(22050);
        wav.set_stereo(true);
        // Two complete stereo frames (the loader drops partial frames).
        wav.set_data(vec![0, 0, 0x00, 0x40, 0x00, 0xC0, 0x00, 0x20]);
        let bytes = wav.to_wav_bytes();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(&bytes[20..22], &1u16.to_le_bytes(), "PCM format code");
        assert_eq!(&bytes[22..24], &2u16.to_le_bytes(), "stereo");
        assert_eq!(&bytes[24..28], &22050u32.to_le_bytes());
        assert_eq!(&bytes[28..32], &(22050u32 * 2 * 2).to_le_bytes());
        assert_eq!(&bytes[32..34], &4u16.to_le_bytes(), "block align");
        assert_eq!(&bytes[34..36], &16u16.to_le_bytes(), "bits per sample");
        assert_eq!(&bytes[36..40], b"data");
        assert_eq!(&bytes[40..44], &8u32.to_le_bytes());
        assert_eq!(&bytes[44..], &[0, 0, 0x00, 0x40, 0x00, 0xC0, 0x00, 0x20]);
        assert_eq!(&bytes[4..8], &44u32.to_le_bytes(), "36 + data bytes");

        let parsed = AudioStreamWav::load_from_buffer(&bytes).expect("re-parsed");
        assert_eq!(parsed.get_mix_rate(), 22050);
        assert!(parsed.is_stereo());
        assert_eq!(parsed.get_data(), wav.get_data());

        // 8-bit data is written unchanged (the port's unsigned storage).
        let mut eight = AudioStreamWav::new();
        eight.set_format(WavFormat::Format8Bits);
        eight.set_mix_rate(8000);
        eight.set_data(vec![0, 128, 255]);
        let bytes = eight.to_wav_bytes();
        assert_eq!(&bytes[34..36], &8u16.to_le_bytes());
        assert_eq!(&bytes[44..], &[0, 128, 255]);
        let parsed = AudioStreamWav::load_from_buffer(&bytes).expect("re-parsed");
        assert_eq!(parsed.get_format(), WavFormat::Format8Bits);
        assert_eq!(parsed.get_data(), eight.get_data());
    }

    #[test]
    fn save_quirks_for_compressed_formats() {
        // IMA-ADPCM: format code 3 and 4 bytes per sample in the header, but the
        // engine writes no payload.
        let mut wav = AudioStreamWav::new();
        wav.set_format(WavFormat::FormatImaAdpcm);
        wav.set_data(vec![1, 2, 3, 4]);
        let bytes = wav.to_wav_bytes();
        assert_eq!(&bytes[20..22], &3u16.to_le_bytes());
        assert_eq!(&bytes[28..32], &(44100u32 * 4).to_le_bytes());
        assert_eq!(&bytes[32..34], &4u16.to_le_bytes());
        assert_eq!(&bytes[34..36], &32u16.to_le_bytes());
        assert_eq!(&bytes[40..44], &4u32.to_le_bytes());
        assert_eq!(bytes.len(), 44, "no sample data is written");

        // QOA is written as 16-bit PCM.
        let mut qoa = AudioStreamWav::new();
        qoa.set_format(WavFormat::FormatQoa);
        qoa.set_data(vec![0x34, 0x12, 0x78, 0x56]);
        let bytes = qoa.to_wav_bytes();
        assert_eq!(&bytes[34..36], &16u16.to_le_bytes());
        assert_eq!(&bytes[44..], &[0x34, 0x12, 0x78, 0x56]);
    }
}
