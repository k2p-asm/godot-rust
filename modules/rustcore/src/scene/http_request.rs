//! `scene/main/http_request.*`: the HTTP request node's state.
//!
//! Defaults from the engine: method `GET`, port 80, no TLS, gzip accepted, no
//! body-size limit (`-1`), eight redirects, no timeout, 64 KiB download chunks,
//! nothing requested yet. Request execution (sockets, TLS, redirects, files) is
//! not ported; the state and the result enum are.
//!
//! Not ported: `request`/`request_raw`/`cancel_request`, the timer, the
//! decompressor and the thread logic.

/// `HTTPRequest::Result`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum RequestResult {
    Success = 0,
    ChunkedBodySizeMismatch = 1,
    CantConnect = 2,
    CantResolve = 3,
    ConnectionError = 4,
    TlsHandshakeError = 5,
    NoResponse = 6,
    BodySizeLimitExceeded = 7,
    BodyDecompressFailed = 8,
    RequestFailed = 9,
    DownloadFileCantOpen = 10,
    DownloadFileWriteError = 11,
    RedirectLimitReached = 12,
    Timeout = 13,
}

pub const DEFAULT_DOWNLOAD_CHUNK_SIZE: i32 = 65536;

#[derive(Clone, Debug)]
pub struct HttpRequestData {
    url: String,
    port: i32,
    use_tls: bool,
    accept_gzip: bool,
    use_threads: bool,
    body_size_limit: i32,
    max_redirects: i32,
    timeout: f64,
    download_chunk_size: i32,
    download_to_file: String,
    requesting: bool,
    download_complete: bool,
    got_response: bool,
    response_code: i32,
    response_headers: Vec<String>,
    body: Vec<u8>,
    body_len: i32,
    downloaded: i32,
    redirections: i32,
}

impl Default for HttpRequestData {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpRequestData {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            port: 80,
            use_tls: false,
            accept_gzip: true,
            use_threads: false,
            body_size_limit: -1,
            max_redirects: 8,
            timeout: 0.0,
            download_chunk_size: DEFAULT_DOWNLOAD_CHUNK_SIZE,
            download_to_file: String::new(),
            requesting: false,
            download_complete: false,
            got_response: false,
            response_code: 0,
            response_headers: Vec::new(),
            body: Vec::new(),
            body_len: -1,
            downloaded: 0,
            redirections: 0,
        }
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn get_port(&self) -> i32 {
        self.port
    }

    pub fn is_using_tls(&self) -> bool {
        self.use_tls
    }

    pub fn set_use_threads(&mut self, enabled: bool) {
        self.use_threads = enabled;
    }

    pub fn is_using_threads(&self) -> bool {
        self.use_threads
    }

    pub fn set_accept_gzip(&mut self, enabled: bool) {
        self.accept_gzip = enabled;
    }

    pub fn is_accepting_gzip(&self) -> bool {
        self.accept_gzip
    }

    pub fn set_download_file(&mut self, file: &str) {
        self.download_to_file = file.to_string();
    }

    pub fn get_download_file(&self) -> &str {
        &self.download_to_file
    }

    pub fn set_download_chunk_size(&mut self, size: i32) {
        self.download_chunk_size = size;
    }

    pub fn get_download_chunk_size(&self) -> i32 {
        self.download_chunk_size
    }

    pub fn set_body_size_limit(&mut self, bytes: i32) {
        self.body_size_limit = bytes;
    }

    pub fn get_body_size_limit(&self) -> i32 {
        self.body_size_limit
    }

    pub fn set_max_redirects(&mut self, max: i32) {
        self.max_redirects = max;
    }

    pub fn get_max_redirects(&self) -> i32 {
        self.max_redirects
    }

    pub fn set_timeout(&mut self, seconds: f64) {
        self.timeout = seconds;
    }

    pub fn get_timeout(&self) -> f64 {
        self.timeout
    }

    pub fn is_requesting(&self) -> bool {
        self.requesting
    }

    pub fn is_download_complete(&self) -> bool {
        self.download_complete
    }

    pub fn has_response(&self) -> bool {
        self.got_response
    }

    pub fn get_response_code(&self) -> i32 {
        self.response_code
    }

    pub fn get_response_headers(&self) -> &[String] {
        &self.response_headers
    }

    pub fn get_body(&self) -> &[u8] {
        &self.body
    }

    pub fn get_body_size(&self) -> i32 {
        self.body_len
    }

    pub fn get_downloaded_bytes(&self) -> i32 {
        self.downloaded
    }

    /// Port-side helper for tests/callers: records a completed response.
    pub fn set_response(&mut self, code: i32, headers: Vec<String>, body: Vec<u8>) {
        self.response_code = code;
        self.response_headers = headers;
        self.body_len = body.len() as i32;
        self.downloaded = body.len() as i32;
        self.body = body;
        self.got_response = true;
        self.download_complete = true;
        self.requesting = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_setters() {
        let mut request = HttpRequestData::new();
        assert_eq!(request.get_url(), "");
        assert_eq!(request.get_port(), 80);
        assert!(!request.is_using_tls());
        assert!(!request.is_using_threads());
        assert!(request.is_accepting_gzip());
        assert_eq!(request.get_download_file(), "");
        assert_eq!(request.get_download_chunk_size(), 65536);
        assert_eq!(request.get_body_size_limit(), -1);
        assert_eq!(request.get_max_redirects(), 8);
        assert_eq!(request.get_timeout(), 0.0);
        assert!(!request.is_requesting());
        assert!(!request.is_download_complete());
        assert!(!request.has_response());
        assert_eq!(request.get_response_code(), 0);
        assert!(request.get_response_headers().is_empty());
        assert!(request.get_body().is_empty());
        assert_eq!(request.get_body_size(), -1);
        assert_eq!(request.get_downloaded_bytes(), 0);

        request.set_use_threads(true);
        request.set_accept_gzip(false);
        request.set_download_file("user://out.bin");
        request.set_download_chunk_size(1024);
        request.set_body_size_limit(4096);
        request.set_max_redirects(3);
        request.set_timeout(2.5);

        assert!(request.is_using_threads());
        assert!(!request.is_accepting_gzip());
        assert_eq!(request.get_download_file(), "user://out.bin");
        assert_eq!(request.get_download_chunk_size(), 1024);
        assert_eq!(request.get_body_size_limit(), 4096);
        assert_eq!(request.get_max_redirects(), 3);
        assert_eq!(request.get_timeout(), 2.5);
    }

    #[test]
    fn result_enum_and_response_state() {
        assert_eq!(RequestResult::Success as i32, 0);
        assert_eq!(RequestResult::CantResolve as i32, 3);
        assert_eq!(RequestResult::BodySizeLimitExceeded as i32, 7);
        assert_eq!(RequestResult::RedirectLimitReached as i32, 12);
        assert_eq!(RequestResult::Timeout as i32, 13);

        let mut request = HttpRequestData::new();
        request.set_response(
            200,
            vec!["Content-Type: text/plain".to_string()],
            b"hello".to_vec(),
        );
        assert!(request.has_response());
        assert!(request.is_download_complete());
        assert!(!request.is_requesting());
        assert_eq!(request.get_response_code(), 200);
        assert_eq!(request.get_response_headers(), &["Content-Type: text/plain"]);
        assert_eq!(request.get_body(), b"hello");
        assert_eq!(request.get_body_size(), 5);
        assert_eq!(request.get_downloaded_bytes(), 5);
        assert_eq!(DEFAULT_DOWNLOAD_CHUNK_SIZE, 65536);
    }
}
