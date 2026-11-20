//! `core/io/packet_peer.*`: the packet helpers over an in-memory queue.
//!
//! `PacketPeer` is abstract in the engine (sockets, ENet, WebRTC all derive from
//! it); this port models the concrete behaviour with a packet queue so the
//! shared helpers are testable:
//!
//! - `get_available_packet_count`, `put_packet`/`get_packet` (raw bytes),
//!   `get_max_packet_size`;
//! - `put_packet_buffer`/`get_packet_buffer` (empty buffers are a no-op);
//! - `put_var`/`get_var` through the port's marshalling format, with the
//!   `encode_buffer_max_size` guard (`ERR_OUT_OF_MEMORY` when the encoding is
//!   larger) and `last_get_error`;
//! - `set_encode_buffer_max_size`/`get_encode_buffer_max_size` with the 8 MiB
//!   default.
//!
//! The transport itself is not ported; packets travel through the queue.

use std::collections::VecDeque;

use crate::io::marshalls::{decode_variant, encode_variant};
use crate::variant::Variant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketError {
    Ok,
    /// `ERR_UNAVAILABLE` — nothing to read.
    Unavailable,
    /// `ERR_OUT_OF_MEMORY` — the encoded variant is too large.
    OutOfMemory,
    /// `ERR_INVALID_DATA` — the packet did not decode.
    InvalidData,
}

#[derive(Clone, Debug)]
pub struct PacketPeerData {
    packets: VecDeque<Vec<u8>>,
    last_get_error: PacketError,
    encode_buffer_max_size: usize,
    max_packet_size: usize,
}

impl Default for PacketPeerData {
    fn default() -> Self {
        Self::new()
    }
}

impl PacketPeerData {
    /// The engine's 8 MiB encode cap and a 65535-byte packet size limit.
    pub fn new() -> Self {
        Self {
            packets: VecDeque::new(),
            last_get_error: PacketError::Ok,
            encode_buffer_max_size: 8 * 1024 * 1024,
            max_packet_size: 65535,
        }
    }

    pub fn get_available_packet_count(&self) -> usize {
        self.packets.len()
    }

    pub fn get_max_packet_size(&self) -> usize {
        self.max_packet_size
    }

    /// `put_packet`: oversized packets are refused.
    pub fn put_packet(&mut self, data: &[u8]) -> PacketError {
        if data.len() > self.max_packet_size {
            return PacketError::Unavailable;
        }
        self.packets.push_back(data.to_vec());
        PacketError::Ok
    }

    /// `get_packet`.
    pub fn get_packet(&mut self) -> Option<Vec<u8>> {
        match self.packets.pop_front() {
            Some(packet) => {
                self.last_get_error = PacketError::Ok;
                Some(packet)
            }
            None => {
                self.last_get_error = PacketError::Unavailable;
                None
            }
        }
    }

    pub fn get_last_get_error(&self) -> PacketError {
        self.last_get_error
    }

    /// `put_packet_buffer`: an empty buffer sends nothing.
    pub fn put_packet_buffer(&mut self, data: &[u8]) -> PacketError {
        if data.is_empty() {
            return PacketError::Ok;
        }
        self.put_packet(data)
    }

    /// `get_packet_buffer`: an empty packet reads back empty.
    pub fn get_packet_buffer(&mut self) -> Result<Vec<u8>, PacketError> {
        match self.get_packet() {
            Some(packet) => Ok(packet),
            None => Err(PacketError::Unavailable),
        }
    }

    /// `put_var`: refuses encodings above the configured buffer size.
    pub fn put_var(&mut self, value: &Variant) -> PacketError {
        let encoded = encode_variant(value);
        if encoded.len() > self.encode_buffer_max_size {
            return PacketError::OutOfMemory;
        }
        self.put_packet(&encoded)
    }

    /// `get_var`: `Nil` and `InvalidData` when the packet cannot be decoded.
    pub fn get_var(&mut self) -> Result<Variant, PacketError> {
        let Some(packet) = self.get_packet() else {
            return Err(PacketError::Unavailable);
        };
        match decode_variant(&packet) {
            Ok((value, _)) => Ok(value),
            Err(_) => {
                self.last_get_error = PacketError::InvalidData;
                Err(PacketError::InvalidData)
            }
        }
    }

    pub fn set_encode_buffer_max_size(&mut self, max_size: usize) -> bool {
        if max_size < 1024 {
            return false;
        }
        self.encode_buffer_max_size = max_size;
        true
    }

    pub fn get_encode_buffer_max_size(&self) -> usize {
        self.encode_buffer_max_size
    }

    pub fn clear(&mut self) {
        self.packets.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::gstring::GoString;

    #[test]
    fn packet_queue() {
        let mut peer = PacketPeerData::new();
        assert_eq!(peer.get_available_packet_count(), 0);
        assert_eq!(peer.get_last_get_error(), PacketError::Ok);
        assert!(peer.get_packet().is_none());
        assert_eq!(peer.get_last_get_error(), PacketError::Unavailable);
        assert_eq!(peer.get_max_packet_size(), 65535);

        assert_eq!(peer.put_packet(&[1, 2, 3]), PacketError::Ok);
        assert_eq!(peer.put_packet(&[4]), PacketError::Ok);
        assert_eq!(peer.get_available_packet_count(), 2);
        assert_eq!(peer.get_packet(), Some(vec![1, 2, 3]));
        assert_eq!(peer.get_last_get_error(), PacketError::Ok);
        assert_eq!(peer.get_packet(), Some(vec![4]));

        // Empty packet buffers are accepted and read back empty.
        assert_eq!(peer.put_packet_buffer(&[]), PacketError::Ok);
        assert_eq!(peer.get_available_packet_count(), 0, "nothing was queued");
        peer.put_packet(&[]);
        assert_eq!(peer.get_packet_buffer(), Ok(Vec::new()));

        // Oversized packets are refused.
        let big = vec![0u8; 65536];
        assert_eq!(peer.put_packet(&big), PacketError::Unavailable);
        peer.clear();
        assert_eq!(peer.get_available_packet_count(), 0);
    }

    #[test]
    fn variants_and_limits() {
        let mut peer = PacketPeerData::new();
        let value = Variant::Dictionary({
            let mut map = std::collections::HashMap::new();
            map.insert(
                crate::string::string_name::StringName::new("k"),
                Variant::String(GoString::from_str("v")),
            );
            map
        });
        assert_eq!(peer.put_var(&value), PacketError::Ok);
        assert_eq!(peer.get_available_packet_count(), 1);
        assert_eq!(peer.get_var().unwrap(), value);

        // Nothing queued.
        assert_eq!(peer.get_var(), Err(PacketError::Unavailable));

        // A small encode cap refuses the packet without queueing it.
        assert!(!peer.set_encode_buffer_max_size(1000), "below 1 KiB");
        assert!(peer.set_encode_buffer_max_size(1024));
        assert_eq!(peer.get_encode_buffer_max_size(), 1024);
        let large = Variant::String(GoString::from_str(&"x".repeat(2048)));
        assert_eq!(peer.put_var(&large), PacketError::OutOfMemory);
        assert_eq!(peer.get_available_packet_count(), 0);

        // Raw garbage reports a decode error.
        peer.set_encode_buffer_max_size(8 * 1024 * 1024);
        peer.put_packet(&[255, 255, 255, 255]);
        assert_eq!(peer.get_var(), Err(PacketError::InvalidData));
        assert_eq!(peer.get_last_get_error(), PacketError::InvalidData);
    }
}
