//! `core/io/packet_peer_udp.*` and `core/io/udp_server.*`: UDP over `std::net`.
//!
//! The engine's UDP peers are non-blocking and asynchronous; this port keeps the
//! same surface on `UdpSocket`:
//!
//! - `PacketPeerUdp` (`bind`, `connect_to_host`, `is_bound`, `get_local_port`,
//!   `put_packet`, `get_packet`, `get_packet_ip`/`port`,
//!   `get_available_packet_count`, `set_broadcast_enabled`, `close`);
//! - `UdpServer` (`listen`, `is_listening`, `get_local_port`, `poll` for the next
//!   datagram with its sender, `send_to`, `stop`).
//!
//! Queueing follows the engine: packets stay available until read. Multicast and
//! `wait()` (blocking) are not ported. Loopback tests cover both directions.

use std::collections::VecDeque;
use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UdpError {
    Ok,
    /// `ERR_CANT_CREATE`/`ERR_CANT_OPEN`.
    CantOpen,
    /// `ERR_UNAVAILABLE`.
    Unavailable,
}

fn resolve(host: &str, port: u16) -> Option<SocketAddr> {
    (host, port).to_socket_addrs().ok()?.next()
}

fn normalize_bind_host(bind_address: &str) -> &str {
    if bind_address.is_empty() || bind_address == "*" {
        "0.0.0.0"
    } else {
        bind_address
    }
}

#[derive(Debug)]
pub struct PacketPeerUdp {
    socket: Option<UdpSocket>,
    connected: Option<SocketAddr>,
    queue: VecDeque<(Vec<u8>, SocketAddr)>,
    broadcast_enabled: bool,
}

impl Default for PacketPeerUdp {
    fn default() -> Self {
        Self::new()
    }
}

impl PacketPeerUdp {
    pub fn new() -> Self {
        Self {
            socket: None,
            connected: None,
            queue: VecDeque::new(),
            broadcast_enabled: false,
        }
    }

    /// `bind`: port 0 picks an ephemeral one.
    pub fn bind(&mut self, port: u16, bind_address: &str) -> UdpError {
        self.close();
        match UdpSocket::bind((normalize_bind_host(bind_address), port)) {
            Ok(socket) => {
                let _ = socket.set_nonblocking(true);
                self.socket = Some(socket);
                UdpError::Ok
            }
            Err(_) => UdpError::CantOpen,
        }
    }

    /// `connect_to_host`: remembers the default destination.
    pub fn connect_to_host(&mut self, host: &str, port: u16) -> UdpError {
        if self.socket.is_none() {
            if self.bind(0, "*") != UdpError::Ok {
                return UdpError::CantOpen;
            }
        }
        match resolve(host, port) {
            Some(address) => {
                self.connected = Some(address);
                UdpError::Ok
            }
            None => UdpError::CantOpen,
        }
    }

    pub fn is_bound(&self) -> bool {
        self.socket.is_some()
    }

    pub fn get_local_port(&self) -> u16 {
        self.socket
            .as_ref()
            .and_then(|socket| socket.local_addr().ok())
            .map(|address| address.port())
            .unwrap_or(0)
    }

    /// `set_broadcast_enabled`.
    pub fn set_broadcast_enabled(&mut self, enabled: bool) -> bool {
        match self.socket.as_ref() {
            Some(socket) => match socket.set_broadcast(enabled) {
                Ok(()) => {
                    self.broadcast_enabled = enabled;
                    true
                }
                Err(_) => false,
            },
            None => false,
        }
    }

    pub fn is_broadcast_enabled(&self) -> bool {
        self.broadcast_enabled
    }

    /// `put_packet`: requires a connected peer (like the engine).
    pub fn put_packet(&mut self, data: &[u8]) -> UdpError {
        let Some(socket) = self.socket.as_ref() else {
            return UdpError::Unavailable;
        };
        let Some(peer) = self.connected else {
            return UdpError::Unavailable;
        };
        match socket.send_to(data, peer) {
            Ok(_) => UdpError::Ok,
            Err(_) => UdpError::CantOpen,
        }
    }

    /// `put_packet` to an explicit address.
    pub fn put_packet_to(&mut self, data: &[u8], address: SocketAddr) -> UdpError {
        let Some(socket) = self.socket.as_ref() else {
            return UdpError::Unavailable;
        };
        match socket.send_to(data, address) {
            Ok(_) => UdpError::Ok,
            Err(_) => UdpError::CantOpen,
        }
    }

    fn fill_queue(&mut self) {
        let Some(socket) = self.socket.as_ref() else {
            return;
        };
        loop {
            let mut buffer = vec![0u8; 65535];
            match socket.recv_from(&mut buffer) {
                Ok((read, from)) => {
                    buffer.truncate(read);
                    self.queue.push_back((buffer, from));
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
    }

    /// `get_available_packet_count`.
    pub fn get_available_packet_count(&mut self) -> usize {
        self.fill_queue();
        self.queue.len()
    }

    /// `get_packet`.
    pub fn get_packet(&mut self) -> Option<Vec<u8>> {
        self.fill_queue();
        self.queue.pop_front().map(|(packet, _)| packet)
    }

    /// `get_packet_ip` for the packet just read.
    pub fn peek_sender(&mut self) -> Option<SocketAddr> {
        self.fill_queue();
        self.queue.front().map(|(_, from)| *from)
    }

    pub fn close(&mut self) {
        self.socket = None;
        self.connected = None;
        self.queue.clear();
    }
}

#[derive(Debug)]
pub struct UdpServer {
    socket: Option<UdpSocket>,
    queue: VecDeque<(Vec<u8>, SocketAddr)>,
}

impl Default for UdpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl UdpServer {
    pub fn new() -> Self {
        Self {
            socket: None,
            queue: VecDeque::new(),
        }
    }

    /// `listen`.
    pub fn listen(&mut self, port: u16, bind_address: &str) -> UdpError {
        self.socket = None;
        self.queue.clear();
        match UdpSocket::bind((normalize_bind_host(bind_address), port)) {
            Ok(socket) => {
                let _ = socket.set_nonblocking(true);
                self.socket = Some(socket);
                UdpError::Ok
            }
            Err(_) => UdpError::CantOpen,
        }
    }

    pub fn is_listening(&self) -> bool {
        self.socket.is_some()
    }

    pub fn get_local_port(&self) -> u16 {
        self.socket
            .as_ref()
            .and_then(|socket| socket.local_addr().ok())
            .map(|address| address.port())
            .unwrap_or(0)
    }

    /// `poll`: reads every pending datagram into the queue.
    pub fn poll(&mut self) -> usize {
        let Some(socket) = self.socket.as_ref() else {
            return 0;
        };
        let mut read = 0usize;
        loop {
            let mut buffer = vec![0u8; 65535];
            match socket.recv_from(&mut buffer) {
                Ok((amount, from)) => {
                    buffer.truncate(amount);
                    self.queue.push_back((buffer, from));
                    read += 1;
                }
                Err(_) => break,
            }
        }
        read
    }

    pub fn get_available_packet_count(&mut self) -> usize {
        self.poll();
        self.queue.len()
    }

    /// The next datagram and its sender.
    pub fn next_packet(&mut self) -> Option<(Vec<u8>, SocketAddr)> {
        self.poll();
        self.queue.pop_front()
    }

    /// Reply to a sender.
    pub fn send_to(&self, data: &[u8], address: SocketAddr) -> UdpError {
        match self.socket.as_ref() {
            Some(socket) => match socket.send_to(data, address) {
                Ok(_) => UdpError::Ok,
                Err(_) => UdpError::CantOpen,
            },
            None => UdpError::Unavailable,
        }
    }

    pub fn stop(&mut self) {
        self.socket = None;
        self.queue.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn wait_until<F: FnMut() -> bool>(mut predicate: F) -> bool {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if predicate() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        false
    }

    #[test]
    fn client_server_round_trip() {
        let mut server = UdpServer::new();
        assert_eq!(server.listen(0, "127.0.0.1"), UdpError::Ok);
        assert!(server.is_listening());
        let port = server.get_local_port();
        assert!(port > 0);
        assert_eq!(server.next_packet(), None);

        let mut client = PacketPeerUdp::new();
        assert_eq!(client.bind(0, "127.0.0.1"), UdpError::Ok);
        assert!(client.is_bound());
        assert!(client.get_local_port() > 0);
        assert_eq!(client.connect_to_host("127.0.0.1", port), UdpError::Ok);

        // Client -> server.
        assert_eq!(client.put_packet(b"ping"), UdpError::Ok);
        let mut received = None;
        assert!(wait_until(|| {
            if let Some(packet) = server.next_packet() {
                received = Some(packet);
                true
            } else {
                false
            }
        }));
        let (data, sender) = received.expect("datagram");
        assert_eq!(data, b"ping");
        assert_eq!(sender.port(), client.get_local_port());

        // Server -> client echo.
        assert_eq!(server.send_to(b"pong", sender), UdpError::Ok);
        let mut echoed = None;
        assert!(wait_until(|| {
            if let Some(packet) = client.get_packet() {
                echoed = Some(packet);
                true
            } else {
                false
            }
        }));
        assert_eq!(echoed.expect("echo"), b"pong");
        assert_eq!(client.get_available_packet_count(), 0);
        assert_eq!(client.peek_sender(), None);

        client.close();
        assert!(!client.is_bound());
        assert_eq!(client.put_packet(b"x"), UdpError::Unavailable);
        server.stop();
        assert!(!server.is_listening());
        assert_eq!(server.get_local_port(), 0);
    }

    #[test]
    fn unconnected_and_errors() {
        // Sending without a bound socket fails.
        let mut peer = PacketPeerUdp::new();
        assert_eq!(peer.put_packet(b"x"), UdpError::Unavailable);
        assert_eq!(peer.get_packet(), None);
        assert!(!peer.set_broadcast_enabled(true));

        // Binding then enabling broadcast works; an unconnected peer still
        // cannot send.
        assert_eq!(peer.bind(0, "127.0.0.1"), UdpError::Ok);
        assert!(peer.set_broadcast_enabled(true));
        assert!(peer.is_broadcast_enabled());
        assert_eq!(peer.put_packet(b"x"), UdpError::Unavailable);

        // Binding the same port twice fails.
        let mut clash = UdpServer::new();
        assert_eq!(clash.listen(0, "127.0.0.1"), UdpError::Ok);
        let port = clash.get_local_port();
        let mut other = UdpServer::new();
        assert_eq!(other.listen(port, "127.0.0.1"), UdpError::CantOpen);
        clash.stop();
        // An unresolvable host is refused.
        let mut peer = PacketPeerUdp::new();
        assert_eq!(
            peer.connect_to_host("no-such-host.invalid", 1234),
            UdpError::CantOpen
        );
    }

    #[test]
    fn direct_send_to() {
        let mut receiver = UdpServer::new();
        receiver.listen(0, "127.0.0.1");
        let port = receiver.get_local_port();
        let address: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

        let mut sender = PacketPeerUdp::new();
        sender.bind(0, "127.0.0.1");
        assert_eq!(sender.put_packet_to(b"direct", address), UdpError::Ok);
        assert!(wait_until(|| receiver.next_packet().is_some()));
    }
}
