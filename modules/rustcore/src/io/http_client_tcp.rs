//! `core/io/http_client_tcp.*`: a minimal HTTP/1.1 client over TCP.
//!
//! The engine's `HTTPClientTCP` is an asynchronous state machine; this port
//! implements the same wire behaviour synchronously on `std::net::TcpStream`,
//! which is enough for tools and tests:
//!
//! - URL parsing (only `http://`, host, optional port, path/query, default `/`);
//! - request writing (`METHOD path HTTP/1.1`, `Host`, `Connection: close` by
//!   default, caller headers, body);
//! - status-line parsing (`HTTP/x.y CODE reason`), header lines, and the body via
//!   `Content-Length`, `Transfer-Encoding: chunked` or read-to-EOF;
//! - the response struct and error codes.
//!
//! HTTPS/TLS, proxies, redirects and keep-alive reuse are not ported.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::io::http_client::Method;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpClientError {
    /// `ERR_CANT_CONNECT`.
    CantConnect,
    /// `ERR_CANT_RESOLVE`.
    CantResolve,
    /// `ERR_INVALID_PARAMETER` (bad URL/scheme).
    InvalidUrl,
    /// `ERR_PARSE_ERROR` on the status line or headers.
    InvalidResponse,
    /// TLS is not available in this port.
    TlsUnsupported,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HttpResponse {
    pub http_version: String,
    pub status_code: i32,
    pub status_text: String,
    /// Raw `Name: value` lines (trimmed), in order.
    pub headers: Vec<String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Case-insensitive header lookup.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find_map(|line| {
            let (key, value) = line.split_once(':')?;
            if key.trim().eq_ignore_ascii_case(name) {
                Some(value.trim())
            } else {
                None
            }
        })
    }
}

#[derive(Debug)]
struct Url {
    host: String,
    port: u16,
    path: String,
}

fn parse_url(url: &str) -> Result<Url, HttpClientError> {
    let Some(rest) = url.strip_prefix("http://") else {
        if url.starts_with("https://") {
            return Err(HttpClientError::TlsUnsupported);
        }
        return Err(HttpClientError::InvalidUrl);
    };
    let (authority, path) = match rest.find('/') {
        Some(index) => (&rest[..index], &rest[index..]),
        None => (rest, "/"),
    };
    if authority.is_empty() {
        return Err(HttpClientError::InvalidUrl);
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (
            host.to_string(),
            port.parse::<u16>()
                .map_err(|_| HttpClientError::InvalidUrl)?,
        ),
        None => (authority.to_string(), 80),
    };
    if host.is_empty() {
        return Err(HttpClientError::InvalidUrl);
    }
    let path = if path.is_empty() { "/" } else { path };
    Ok(Url {
        host,
        port,
        path: path.to_string(),
    })
}

/// One HTTP request; returns the parsed response.
pub fn request(
    url: &str,
    method: Method,
    headers: &[String],
    body: &[u8],
) -> Result<HttpResponse, HttpClientError> {
    let url = parse_url(url)?;
    let addresses: Vec<_> = (url.host.as_str(), url.port)
        .to_socket_addrs()
        .map_err(|_| HttpClientError::CantResolve)?
        .collect();
    let Some(address) = addresses.first().copied() else {
        return Err(HttpClientError::CantResolve);
    };
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(10))
        .map_err(|_| HttpClientError::CantConnect)?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));

    let request_line = format!(
        "{} {} HTTP/1.1\r\n",
        method.name().unwrap_or("GET"),
        url.path
    );
    let mut head = String::new();
    head.push_str(&request_line);
    head.push_str(&format!("Host: {}\r\n", url.host));
    let mut has_connection = false;
    let mut has_content_length = false;
    for header in headers {
        let lower = header.to_ascii_lowercase();
        if lower.starts_with("connection:") {
            has_connection = true;
        }
        if lower.starts_with("content-length:") {
            has_content_length = true;
        }
        head.push_str(header);
        head.push_str("\r\n");
    }
    if !has_connection {
        head.push_str("Connection: close\r\n");
    }
    if !body.is_empty() && !has_content_length {
        head.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    head.push_str("\r\n");

    stream
        .write_all(head.as_bytes())
        .map_err(|_| HttpClientError::CantConnect)?;
    if !body.is_empty() {
        stream
            .write_all(body)
            .map_err(|_| HttpClientError::CantConnect)?;
    }
    let _ = stream.flush();

    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|_| HttpClientError::InvalidResponse)?;
    parse_response(&raw)
}

/// Parses a complete HTTP response buffer.
pub fn parse_response(raw: &[u8]) -> Result<HttpResponse, HttpClientError> {
    let header_end = find_headers_end(raw).ok_or(HttpClientError::InvalidResponse)?;
    let head = std::str::from_utf8(&raw[..header_end]).map_err(|_| HttpClientError::InvalidResponse)?;
    let mut lines = head.split("\r\n");
    let status_line = lines.next().ok_or(HttpClientError::InvalidResponse)?;
    let mut status_parts = status_line.splitn(3, ' ');
    let http_version = status_parts
        .next()
        .filter(|version| version.starts_with("HTTP/"))
        .ok_or(HttpClientError::InvalidResponse)?
        .to_string();
    let status_code = status_parts
        .next()
        .and_then(|code| code.parse::<i32>().ok())
        .ok_or(HttpClientError::InvalidResponse)?;
    let status_text = status_parts.next().unwrap_or("").to_string();

    let mut headers = Vec::new();
    for line in lines {
        if !line.is_empty() {
            headers.push(line.to_string());
        }
    }

    let body_start = header_end + 4;
    let body_end = raw.len();
    let chunked = headers.iter().any(|line| {
        line.to_ascii_lowercase()
            .starts_with("transfer-encoding:")
            && line.to_ascii_lowercase().contains("chunked")
    });
    let content_length = headers.iter().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if key.trim().eq_ignore_ascii_case("content-length") {
            value.trim().parse::<usize>().ok()
        } else {
            None
        }
    });

    let body = if chunked {
        decode_chunked(&raw[body_start..body_end])?
    } else if let Some(length) = content_length {
        let available = &raw[body_start..body_end];
        available[..length.min(available.len())].to_vec()
    } else {
        raw[body_start..body_end].to_vec()
    };

    Ok(HttpResponse {
        http_version,
        status_code,
        status_text,
        headers,
        body,
    })
}

fn find_headers_end(raw: &[u8]) -> Option<usize> {
    raw.windows(4).position(|window| window == b"\r\n\r\n")
}

fn decode_chunked(mut data: &[u8]) -> Result<Vec<u8>, HttpClientError> {
    let mut out = Vec::new();
    loop {
        let Some(line_end) = data.windows(2).position(|window| window == b"\r\n") else {
            return Err(HttpClientError::InvalidResponse);
        };
        let size_text = std::str::from_utf8(&data[..line_end])
            .map_err(|_| HttpClientError::InvalidResponse)?;
        let size_text = size_text.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_text, 16)
            .map_err(|_| HttpClientError::InvalidResponse)?;
        data = &data[line_end + 2..];
        if size == 0 {
            return Ok(out);
        }
        if data.len() < size + 2 {
            return Err(HttpClientError::InvalidResponse);
        }
        out.extend_from_slice(&data[..size]);
        data = &data[size + 2..];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read as _;
    use std::net::TcpListener;
    use std::thread;

    fn one_shot_server(response: &'static [u8]) -> (u16, thread::JoinHandle<Vec<u8>>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind");
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut request = Vec::new();
            let mut buffer = [0u8; 4096];
            // Read until the headers are complete and, when advertised, the body
            // has arrived too; closing with unread request data would reset the
            // connection before the response is read.
            loop {
                let read = match stream.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(read) => read,
                    Err(_) => break,
                };
                request.extend_from_slice(&buffer[..read]);
                if let Some(header_end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                    let content_length = String::from_utf8_lossy(&request[..header_end])
                        .lines()
                        .find_map(|line| {
                            let (key, value) = line.split_once(':')?;
                            if key.trim().eq_ignore_ascii_case("content-length") {
                                value.trim().parse::<usize>().ok()
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0);
                    if request.len() >= header_end + 4 + content_length {
                        break;
                    }
                }
            }
            let _ = stream.write_all(response);
            let _ = stream.flush();
            request
        });
        (port, handle)
    }

    #[test]
    fn get_with_content_length() {
        let (port, handle) = one_shot_server(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello",
        );
        let response = request(
            &format!("http://127.0.0.1:{port}/path?x=1"),
            Method::Get,
            &["Accept: text/plain".to_string()],
            &[],
        )
        .expect("response");
        let sent = handle.join().unwrap();
        let sent = String::from_utf8_lossy(&sent);
        assert!(sent.starts_with("GET /path?x=1 HTTP/1.1\r\n"), "request line");
        assert!(sent.contains(&format!("Host: 127.0.0.1")));
        assert!(sent.contains("Accept: text/plain"));
        assert!(sent.contains("Connection: close"));

        assert_eq!(response.http_version, "HTTP/1.1");
        assert_eq!(response.status_code, 200);
        assert_eq!(response.status_text, "OK");
        assert_eq!(response.header("content-type"), Some("text/plain"));
        assert_eq!(response.header("CONTENT-LENGTH"), Some("5"));
        assert_eq!(response.header("missing"), None);
        assert_eq!(response.body, b"hello");
    }

    #[test]
    fn chunked_response() {
        let (port, handle) = one_shot_server(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n",
        );
        let response = request(&format!("http://127.0.0.1:{port}/"), Method::Post, &[], b"body")
            .expect("response");
        let sent = handle.join().unwrap();
        let sent = String::from_utf8_lossy(&sent);
        assert!(sent.contains("POST / HTTP/1.1"));
        assert!(sent.contains("Content-Length: 4"));

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, b"hello world");
        assert!(response.header("transfer-encoding").is_some());
    }

    #[test]
    fn error_status_and_read_to_eof() {
        let (port, handle) = one_shot_server(b"HTTP/1.1 404 Not Found\r\n\r\nmissing");
        let response = request(&format!("http://127.0.0.1:{port}/none"), Method::Get, &[], &[])
            .expect("response");
        let _ = handle.join();
        assert_eq!(response.status_code, 404);
        assert_eq!(response.status_text, "Not Found");
        assert_eq!(response.body, b"missing");
    }

    #[test]
    fn url_and_connection_errors() {
        assert_eq!(parse_url("https://x/y").unwrap_err(), HttpClientError::TlsUnsupported);
        assert_eq!(parse_url("ftp://x/y").unwrap_err(), HttpClientError::InvalidUrl);
        assert_eq!(parse_url("http://").unwrap_err(), HttpClientError::InvalidUrl);
        let url = parse_url("http://example.com").expect("url");
        assert_eq!(url.path, "/");
        assert_eq!(url.port, 80);
        let url = parse_url("http://example.com:8080/a/b").expect("url");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, 8080);
        assert_eq!(url.path, "/a/b");

        // A port nobody listens on is refused.
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let error = request(&format!("http://127.0.0.1:{port}/"), Method::Get, &[], &[])
            .unwrap_err();
        assert_eq!(error, HttpClientError::CantConnect);
    }

    #[test]
    fn response_parser_edges() {
        assert_eq!(parse_response(b""), Err(HttpClientError::InvalidResponse));
        assert_eq!(
            parse_response(b"NOT HTTP\r\n\r\n"),
            Err(HttpClientError::InvalidResponse)
        );
        assert_eq!(
            parse_response(b"HTTP/1.0 204 No Content\r\n\r\n"),
            Ok(HttpResponse {
                http_version: "HTTP/1.0".to_string(),
                status_code: 204,
                status_text: "No Content".to_string(),
                headers: Vec::new(),
                body: Vec::new(),
            })
        );
        // A truncated chunk stream is an error.
        assert_eq!(
            parse_response(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhel"),
            Err(HttpClientError::InvalidResponse)
        );
    }
}
