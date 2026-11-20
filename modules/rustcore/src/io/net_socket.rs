//! `core/io/net_socket.*`, `core/io/stream_peer_tcp.*` and
//! `core/io/tcp_server.*`: TCP helpers over `std::net`.
//!
//! The engine's sockets are asynchronous: `poll()` advances the state machine
//! and `get_status()` reports it. This port keeps the same surface on top of
//! non-blocking `TcpStream`/`TcpListener`:
//!
//! - `StreamPeerTCP` (`connect_to_host`, `poll`, `get_status`,
//!   `disconnect_from_host`, `set_no_delay`, `put_data`, `get_partial_data`,
//!   `get_data`, `is_connected`);
//! - `TcpServer` (`listen` with the `*` wildcard and ephemeral ports, `stop`,
//!   `is_listening`, `get_local_port`, `is_connection_available`,
//!   `take_connection`) with a one-connection pending slot (the engine's queue
//!   is unbounded; documented).
//!
//! `connect_to_host` resolves and connects synchronously (the engine's status
//! passes through `Resolving`/`Connecting`), which cannot be observed from a
//! single-threaded test; the status jumps to `Connected` or `Error`.
//!
//! Loopback tests exercise the pair end to end.

use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    None,
    Connecting,
    Connected,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetError {
    Ok,
    /// `ERR_CANT_CONNECT`.
    CantConnect,
    /// `ERR_CANT_OPEN` (bind/listen failure).
    CantOpen,
    /// `ERR_UNAVAILABLE` (no pending connection).
    Unavailable,
}

pub struct StreamPeerTcp {
    stream: Option<TcpStream>,
    status: Status,
}

impl Default for StreamPeerTcp {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamPeerTcp {
    pub fn new() -> Self {
        Self {
            stream: None,
            status: Status::None,
        }
    }

    /// `connect_to_host` (synchronous in this port).
    pub fn connect_to_host(&mut self, host: &str, port: u16) -> NetError {
        self.disconnect_from_host();
        let address = match (host, port).to_socket_addrs() {
            Ok(mut addresses) => match addresses.next() {
                Some(address) => address,
                None => {
                    self.status = Status::Error;
                    return NetError::CantConnect;
                }
            },
            Err(_) => {
                self.status = Status::Error;
                return NetError::CantConnect;
            }
        };
        match TcpStream::connect(address) {
            Ok(stream) => {
                let _ = stream.set_nonblocking(true);
                self.stream = Some(stream);
                self.status = Status::Connected;
                NetError::Ok
            }
            Err(_) => {
                self.status = Status::Error;
                NetError::CantConnect
            }
        }
    }

    /// A socket accepted by `TcpServer`.
    pub fn from_stream(stream: TcpStream) -> Self {
        let _ = stream.set_nonblocking(true);
        Self {
            stream: Some(stream),
            status: Status::Connected,
        }
    }

    /// `poll`: refreshes the status (a closed peer becomes `None`).
    pub fn poll(&mut self) {
        let Some(stream) = self.stream.as_mut() else {
            self.status = Status::None;
            return;
        };
        if let Err(error) = stream.write(&[]) {
            if error.kind() != ErrorKind::WouldBlock {
                self.disconnect_from_host();
                self.status = Status::Error;
                return;
            }
        }
        self.status = Status::Connected;
    }

    pub fn get_status(&self) -> Status {
        self.status
    }

    pub fn is_connected(&self) -> bool {
        self.status == Status::Connected
    }

    pub fn disconnect_from_host(&mut self) {
        if let Some(stream) = self.stream.take() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
        self.status = Status::None;
    }

    /// `set_no_delay` (`TCP_NODELAY`).
    pub fn set_no_delay(&mut self, enabled: bool) -> bool {
        match self.stream.as_ref() {
            Some(stream) => stream.set_nodelay(enabled).is_ok(),
            None => false,
        }
    }

    /// `put_data`: writes everything (blocking-free sockets may ShortWrite; the
    /// engine returns the sent count through `put_partial_data`).
    pub fn put_data(&mut self, data: &[u8]) -> NetError {
        let Some(stream) = self.stream.as_mut() else {
            return NetError::Unavailable;
        };
        match stream.write_all(data) {
            Ok(()) => NetError::Ok,
            Err(_) => NetError::CantConnect,
        }
    }

    /// `put_partial_data`: the number of bytes written.
    pub fn put_partial_data(&mut self, data: &[u8]) -> (NetError, usize) {
        let Some(stream) = self.stream.as_mut() else {
            return (NetError::Unavailable, 0);
        };
        match stream.write(data) {
            Ok(sent) => (NetError::Ok, sent),
            Err(error) if error.kind() == ErrorKind::WouldBlock => (NetError::Ok, 0),
            Err(_) => (NetError::CantConnect, 0),
        }
    }

    /// `get_partial_data`: everything currently available up to `count`.
    pub fn get_partial_data(&mut self, count: usize) -> Vec<u8> {
        let Some(stream) = self.stream.as_mut() else {
            return Vec::new();
        };
        let mut buffer = vec![0u8; count];
        match stream.read(&mut buffer) {
            Ok(read) => {
                buffer.truncate(read);
                buffer
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => Vec::new(),
            Err(_) => {
                self.disconnect_from_host();
                Vec::new()
            }
        }
    }

    /// `get_data`: fails unless the full count was available.
    pub fn get_data(&mut self, count: usize) -> Result<Vec<u8>, NetError> {
        let data = self.get_partial_data(count);
        if data.len() == count {
            Ok(data)
        } else {
            Err(NetError::Unavailable)
        }
    }

    pub fn get_available_bytes(&mut self) -> usize {
        let Some(stream) = self.stream.as_mut() else {
            return 0;
        };
        let mut probe = [0u8; 1];
        match stream.peek(&mut probe) {
            Ok(read) => read,
            Err(_) => 0,
        }
    }

    pub fn get_remote_address(&mut self) -> Option<String> {
        self.stream
            .as_ref()
            .and_then(|stream| stream.peer_addr().ok())
            .map(|address| address.ip().to_string())
    }

    pub fn get_remote_port(&mut self) -> Option<u16> {
        self.stream
            .as_ref()
            .and_then(|stream| stream.peer_addr().ok())
            .map(|address| address.port())
    }
}

pub struct TcpServer {
    listener: Option<TcpListener>,
    pending: Option<TcpStream>,
    local_port: u16,
}

impl Default for TcpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl TcpServer {
    pub fn new() -> Self {
        Self {
            listener: None,
            pending: None,
            local_port: 0,
        }
    }

    /// `listen`: `bind_address` `*` maps to `0.0.0.0`; port 0 picks one.
    pub fn listen(&mut self, port: u16, bind_address: &str) -> NetError {
        self.stop();
        let host = if bind_address.is_empty() || bind_address == "*" {
            "0.0.0.0"
        } else {
            bind_address
        };
        match TcpListener::bind((host, port)) {
            Ok(listener) => {
                let _ = listener.set_nonblocking(true);
                self.local_port = listener.local_addr().map(|address| address.port()).unwrap_or(0);
                self.listener = Some(listener);
                NetError::Ok
            }
            Err(_) => NetError::CantOpen,
        }
    }

    pub fn stop(&mut self) {
        self.listener = None;
        self.pending = None;
        self.local_port = 0;
    }

    pub fn is_listening(&self) -> bool {
        self.listener.is_some()
    }

    pub fn get_local_port(&self) -> u16 {
        self.local_port
    }

    fn accept_pending(&mut self) -> bool {
        if self.pending.is_some() {
            return true;
        }
        let Some(listener) = self.listener.as_ref() else {
            return false;
        };
        match listener.accept() {
            Ok((stream, _)) => {
                self.pending = Some(stream);
                true
            }
            Err(_) => false,
        }
    }

    /// `is_connection_available`: accepts one pending client.
    pub fn is_connection_available(&mut self) -> bool {
        self.accept_pending()
    }

    /// `take_connection`.
    pub fn take_connection(&mut self) -> Option<StreamPeerTcp> {
        self.accept_pending();
        self.pending.take().map(StreamPeerTcp::from_stream)
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
    fn enums() {
        assert_eq!(Status::None as i32, 0);
        assert_eq!(Status::Connecting as i32, 1);
        assert_eq!(Status::Connected as i32, 2);
        assert_eq!(Status::Error as i32, 3);
        assert_eq!(NetError::Ok as i32 + 1, NetError::CantConnect as i32);
    }

    #[test]
    fn server_and_stream_round_trip() {
        let mut server = TcpServer::new();
        assert!(!server.is_listening());
        assert_eq!(server.listen(0, "127.0.0.1"), NetError::Ok);
        assert!(server.is_listening());
        let port = server.get_local_port();
        assert!(port > 0);
        assert!(!server.is_connection_available());
        assert!(server.take_connection().is_none());

        let mut client = StreamPeerTcp::new();
        assert_eq!(client.connect_to_host("127.0.0.1", port), NetError::Ok);
        assert!(client.is_connected());
        assert_eq!(client.get_status(), Status::Connected);

        assert!(wait_until(|| server.is_connection_available()));
        let mut peer = server.take_connection().expect("accepted");
        assert!(peer.is_connected());
        assert!(peer.get_remote_port().is_some());
        assert_eq!(peer.get_remote_address().as_deref(), Some("127.0.0.1"));

        // Client -> server.
        assert_eq!(client.put_data(b"hello"), NetError::Ok);
        let received = {
            let mut received = Vec::new();
            assert!(wait_until(|| {
                received.extend(peer.get_partial_data(32));
                received.len() >= 5
            }));
            received
        };
        assert_eq!(&received[..5], b"hello");

        // Server -> client (partial writes are allowed).
        let mut sent = 0usize;
        assert!(wait_until(|| {
            let (error, amount) = peer.put_partial_data(b"world!");
            assert_eq!(error, NetError::Ok);
            sent += amount;
            sent >= 6
        }));
        let echoed = {
            let mut echoed = Vec::new();
            assert!(wait_until(|| {
                echoed.extend(client.get_partial_data(32));
                echoed.len() >= 6
            }));
            echoed
        };
        assert_eq!(&echoed[..6], b"world!");

        // get_data fails when nothing is buffered.
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(client.get_data(4), Err(NetError::Unavailable));
        assert_eq!(peer.get_available_bytes(), 0);

        // Disconnect closes the peer's view.
        client.disconnect_from_host();
        assert!(!client.is_connected());
        assert_eq!(client.get_status(), Status::None);
        assert_eq!(client.put_data(b"x"), NetError::Unavailable);
        assert_eq!(client.get_partial_data(4), Vec::new());
        assert!(!client.set_no_delay(true));
        peer.disconnect_from_host();
        server.stop();
        assert!(!server.is_listening());
    }

    #[test]
    fn refused_connections_and_binding() {
        // Connecting to a closed port fails.
        let mut probe = TcpServer::new();
        assert_eq!(probe.listen(0, "127.0.0.1"), NetError::Ok);
        let port = probe.get_local_port();
        probe.stop();

        let mut client = StreamPeerTcp::new();
        assert_eq!(
            client.connect_to_host("127.0.0.1", port),
            NetError::CantConnect
        );
        assert_eq!(client.get_status(), Status::Error);
        assert!(!client.is_connected());

        // Bad hostnames are refused too.
        assert_eq!(
            client.connect_to_host("no-such-host.invalid", 80),
            NetError::CantConnect
        );

        // The wildcard binds on all interfaces; port 0 picks one.
        let mut server = TcpServer::new();
        assert_eq!(server.listen(0, "*"), NetError::Ok);
        assert!(server.is_listening());
        assert!(server.get_local_port() > 0);
        // Listening on an already used port fails.
        let mut clash = TcpServer::new();
        assert_eq!(
            clash.listen(server.get_local_port(), "0.0.0.0"),
            NetError::CantOpen
        );
        server.stop();
        assert_eq!(server.get_local_port(), 0);
    }

    #[test]
    fn no_delay_and_poll() {
        let mut server = TcpServer::new();
        server.listen(0, "127.0.0.1");
        let port = server.get_local_port();

        let mut client = StreamPeerTcp::new();
        assert!(!client.set_no_delay(true), "not connected yet");
        client.connect_to_host("127.0.0.1", port);
        assert!(client.set_no_delay(true));
        client.poll();
        assert_eq!(client.get_status(), Status::Connected);

        assert!(wait_until(|| server.is_connection_available()));
        let mut peer = server.take_connection().unwrap();
        peer.poll();
        assert!(peer.is_connected());
        assert_eq!(peer.get_available_bytes(), 0);
        client.disconnect_from_host();
        peer.poll();
    }
}
