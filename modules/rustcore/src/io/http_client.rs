//! `core/io/http_client.*`: the HTTP request helpers (no transport).
//!
//! The transport lives in the platform subclass (`http_client_tcp.cpp`); this
//! port keeps the parts that are pure logic and shared by every backend:
//!
//! - `Method` (GET…PATCH with the engine's names) and `Status` (disconnected →
//!   TLS handshake error) enums;
//! - the `ResponseCode` constants (1xx–5xx);
//! - `query_string_from_dict` (URI-encoded keys/values, arrays repeated, `null`
//!   keys without a value, the leading `&` trimmed);
//! - `verify_headers` (empty headers and headers without a `name: value` pair are
//!   rejected with the failing index);
//! - `headers_as_dictionary` (split at the first `:`, both sides trimmed).
//!
//! Connections, proxies, TLS, chunked/body reading and `poll` are not ported.

use std::collections::BTreeMap;

use crate::string::escape;
use crate::variant::Variant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Options,
    Trace,
    Connect,
    Patch,
    Max,
}

impl Method {
    /// `HTTPClient::_methods`.
    pub fn name(self) -> Option<&'static str> {
        match self {
            Method::Get => Some("GET"),
            Method::Head => Some("HEAD"),
            Method::Post => Some("POST"),
            Method::Put => Some("PUT"),
            Method::Delete => Some("DELETE"),
            Method::Options => Some("OPTIONS"),
            Method::Trace => Some("TRACE"),
            Method::Connect => Some("CONNECT"),
            Method::Patch => Some("PATCH"),
            Method::Max => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    Disconnected,
    Resolving,
    CantResolve,
    Connecting,
    CantConnect,
    Connected,
    Requesting,
    Body,
    ConnectionError,
    TlsHandshakeError,
}

macro_rules! response_codes {
    ($($name:ident = $value:expr),* $(,)?) => {
        $(pub const $name: i32 = $value;)*

        /// Every constant, sorted by code (the engine's `ResponseCode` set).
        pub const RESPONSE_CODES: &[(i32, &str)] = &[
            $(( $value, stringify!($name) )),*
        ];
    };
}

response_codes! {
    RESPONSE_CONTINUE = 100,
    RESPONSE_SWITCHING_PROTOCOLS = 101,
    RESPONSE_PROCESSING = 102,
    RESPONSE_OK = 200,
    RESPONSE_CREATED = 201,
    RESPONSE_ACCEPTED = 202,
    RESPONSE_NON_AUTHORITATIVE_INFORMATION = 203,
    RESPONSE_NO_CONTENT = 204,
    RESPONSE_RESET_CONTENT = 205,
    RESPONSE_PARTIAL_CONTENT = 206,
    RESPONSE_MULTI_STATUS = 207,
    RESPONSE_ALREADY_REPORTED = 208,
    RESPONSE_IM_USED = 226,
    RESPONSE_MULTIPLE_CHOICES = 300,
    RESPONSE_MOVED_PERMANENTLY = 301,
    RESPONSE_FOUND = 302,
    RESPONSE_SEE_OTHER = 303,
    RESPONSE_NOT_MODIFIED = 304,
    RESPONSE_USE_PROXY = 305,
    RESPONSE_SWITCH_PROXY = 306,
    RESPONSE_TEMPORARY_REDIRECT = 307,
    RESPONSE_PERMANENT_REDIRECT = 308,
    RESPONSE_BAD_REQUEST = 400,
    RESPONSE_UNAUTHORIZED = 401,
    RESPONSE_PAYMENT_REQUIRED = 402,
    RESPONSE_FORBIDDEN = 403,
    RESPONSE_NOT_FOUND = 404,
    RESPONSE_METHOD_NOT_ALLOWED = 405,
    RESPONSE_NOT_ACCEPTABLE = 406,
    RESPONSE_PROXY_AUTHENTICATION_REQUIRED = 407,
    RESPONSE_REQUEST_TIMEOUT = 408,
    RESPONSE_CONFLICT = 409,
    RESPONSE_GONE = 410,
    RESPONSE_LENGTH_REQUIRED = 411,
    RESPONSE_PRECONDITION_FAILED = 412,
    RESPONSE_REQUEST_ENTITY_TOO_LARGE = 413,
    RESPONSE_REQUEST_URI_TOO_LONG = 414,
    RESPONSE_UNSUPPORTED_MEDIA_TYPE = 415,
    RESPONSE_REQUESTED_RANGE_NOT_SATISFIABLE = 416,
    RESPONSE_EXPECTATION_FAILED = 417,
    RESPONSE_IM_A_TEAPOT = 418,
    RESPONSE_MISDIRECTED_REQUEST = 421,
    RESPONSE_UNPROCESSABLE_ENTITY = 422,
    RESPONSE_LOCKED = 423,
    RESPONSE_FAILED_DEPENDENCY = 424,
    RESPONSE_UPGRADE_REQUIRED = 426,
    RESPONSE_PRECONDITION_REQUIRED = 428,
    RESPONSE_TOO_MANY_REQUESTS = 429,
    RESPONSE_REQUEST_HEADER_FIELDS_TOO_LARGE = 431,
    RESPONSE_UNAVAILABLE_FOR_LEGAL_REASONS = 451,
    RESPONSE_INTERNAL_SERVER_ERROR = 500,
    RESPONSE_NOT_IMPLEMENTED = 501,
    RESPONSE_BAD_GATEWAY = 502,
    RESPONSE_SERVICE_UNAVAILABLE = 503,
    RESPONSE_GATEWAY_TIMEOUT = 504,
    RESPONSE_HTTP_VERSION_NOT_SUPPORTED = 505,
    RESPONSE_VARIANT_ALSO_NEGOTIATES = 506,
    RESPONSE_INSUFFICIENT_STORAGE = 507,
    RESPONSE_LOOP_DETECTED = 508,
    RESPONSE_NOT_EXTENDED = 510,
    RESPONSE_NETWORK_AUTH_REQUIRED = 511,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderError {
    /// The header is empty after trimming: `(index, "empty")`.
    Empty { index: usize },
    /// The header has no `name: value` pair: `(index, "missing colon")`.
    MissingPair { index: usize },
}

/// `HTTPClient::query_string_from_dict`.
pub fn query_string_from_dict(entries: &[(String, Variant)]) -> String {
    let mut query = String::new();
    for (key, value) in entries {
        let encoded_key = escape::uri_encode(key);
        match value {
            Variant::Array(values) => {
                // Repeat the key once per element.
                for element in values {
                    query.push('&');
                    query.push_str(&encoded_key);
                    query.push('=');
                    query.push_str(&escape::uri_encode(&variant_to_string(element)));
                }
            }
            Variant::Nil => {
                query.push('&');
                query.push_str(&encoded_key);
            }
            _ => {
                query.push('&');
                query.push_str(&encoded_key);
                query.push('=');
                query.push_str(&escape::uri_encode(&variant_to_string(value)));
            }
        }
    }
    // Trim the leading '&'.
    query.strip_prefix('&').unwrap_or(&query).to_string()
}

/// `String(value)` for the query builder: scalars print like the engine's
/// `Variant::operator String`, containers fall back to their type tag.
fn variant_to_string(value: &Variant) -> String {
    match value {
        Variant::Nil => String::new(),
        Variant::Bool(value) => value.to_string(),
        Variant::Int(value) => value.to_string(),
        Variant::Float(value) => format!("{value}"),
        Variant::String(value) => value.as_string(),
        Variant::Vector2(value) => format!("({}, {})", value.x, value.y),
        Variant::Vector2i(value) => format!("({}, {})", value.x, value.y),
        Variant::Vector3(value) => format!("({}, {}, {})", value.x, value.y, value.z),
        other => format!("{other:?}"),
    }
}

/// `HTTPClient::verify_headers`.
pub fn verify_headers(headers: &[String]) -> Result<(), HeaderError> {
    for (index, header) in headers.iter().enumerate() {
        let sanitized = header.trim();
        if sanitized.is_empty() {
            return Err(HeaderError::Empty { index });
        }
        match sanitized.find(':') {
            Some(position) if position >= 1 => {}
            _ => return Err(HeaderError::MissingPair { index }),
        }
    }
    Ok(())
}

/// `_get_response_headers_as_dictionary`: split at the first `:`, trim both
/// sides, skip lines without a colon.
pub fn headers_as_dictionary(headers: &[String]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for header in headers {
        let Some(position) = header.find(':') else {
            continue;
        };
        let key = header[..position].trim();
        let value = header[position + 1..].trim();
        out.insert(key.to_string(), value.to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::gstring::GoString;

    #[test]
    fn method_and_status_values() {
        assert_eq!(Method::Get as i32, 0);
        assert_eq!(Method::Head as i32, 1);
        assert_eq!(Method::Patch as i32, 8);
        assert_eq!(Method::Max as i32, 9);
        assert_eq!(Method::Get.name(), Some("GET"));
        assert_eq!(Method::Delete.name(), Some("DELETE"));
        assert_eq!(Method::Max.name(), None);

        assert_eq!(Status::Disconnected as i32, 0);
        assert_eq!(Status::Body as i32, 7);
        assert_eq!(Status::TlsHandshakeError as i32, 9);

        assert_eq!(RESPONSE_OK, 200);
        assert_eq!(RESPONSE_NOT_FOUND, 404);
        assert_eq!(RESPONSE_NETWORK_AUTH_REQUIRED, 511);
        assert_eq!(RESPONSE_IM_A_TEAPOT, 418);
        // The published table is ordered and unique.
        let mut previous = 0;
        for (code, _) in RESPONSE_CODES {
            assert!(*code > previous);
            previous = *code;
        }
    }

    #[test]
    fn query_strings() {
        // Scalars, arrays and nulls, URI-encoded.
        let entries = vec![
            ("name".to_string(), Variant::String(GoString::from_str("a b"))),
            ("n".to_string(), Variant::Int(3)),
            (
                "tags".to_string(),
                Variant::Array(vec![
                    Variant::String(GoString::from_str("x")),
                    Variant::String(GoString::from_str("y z")),
                ]),
            ),
            ("flag".to_string(), Variant::Nil),
            ("bool".to_string(), Variant::Bool(true)),
        ];
        assert_eq!(
            query_string_from_dict(&entries),
            "name=a%20b&n=3&tags=x&tags=y%20z&flag&bool=true"
        );

        // A single entry has no leading '&' and an empty dict is empty.
        assert_eq!(
            query_string_from_dict(&[("k".to_string(), Variant::Float(1.5))]),
            "k=1.5"
        );
        assert_eq!(query_string_from_dict(&[]), "");

        // Characters that need escaping.
        assert_eq!(
            query_string_from_dict(&[(
                "q".to_string(),
                Variant::String(GoString::from_str("a&b=c"))
            )]),
            "q=a%26b%3Dc"
        );
    }

    #[test]
    fn header_validation() {
        assert!(verify_headers(&[]).is_ok());
        assert!(verify_headers(&[
            "Content-Type: application/json".to_string(),
            "X-Test:1".to_string(),
        ])
        .is_ok());
        // Whitespace around the pair is fine.
        assert!(verify_headers(&["  Accept : text/plain  ".to_string()]).is_ok());

        assert_eq!(
            verify_headers(&["".to_string()]),
            Err(HeaderError::Empty { index: 0 })
        );
        assert_eq!(
            verify_headers(&["Accept: ok".to_string(), "  ".to_string()]),
            Err(HeaderError::Empty { index: 1 })
        );
        assert_eq!(
            verify_headers(&["NoColon".to_string()]),
            Err(HeaderError::MissingPair { index: 0 })
        );
        // A colon at position zero is not a header pair.
        assert_eq!(
            verify_headers(&[": value".to_string()]),
            Err(HeaderError::MissingPair { index: 0 })
        );
    }

    #[test]
    fn response_headers_dictionary() {
        let headers = vec![
            "Content-Type: text/html".to_string(),
            "Content-Length: 1234".to_string(),
            "  X-Trimmed  :  spaced value  ".to_string(),
            "NoColonLine".to_string(),
            "Duplicate: first".to_string(),
            "Duplicate: second".to_string(),
        ];
        let dictionary = headers_as_dictionary(&headers);
        assert_eq!(dictionary.get("Content-Type").map(String::as_str), Some("text/html"));
        assert_eq!(dictionary.get("Content-Length").map(String::as_str), Some("1234"));
        assert_eq!(dictionary.get("X-Trimmed").map(String::as_str), Some("spaced value"));
        assert_eq!(dictionary.get("Duplicate").map(String::as_str), Some("second"));
        assert!(!dictionary.contains_key("NoColonLine"));
        assert_eq!(dictionary.len(), 4);
    }
}
