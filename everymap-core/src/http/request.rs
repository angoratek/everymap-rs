//! Backend-agnostic HTTP request values used by the transport seam.
//!
//! Provider code builds an [`HttpRequest`] through the chainable methods on the
//! type (`query`, `header`, `json`, `form`, `body`, `timeout`) and hands it to
//! [`crate::http::HttpTransport`]. No backend type appears in this module.

use crate::error::EveryMapResult;
use serde::Serialize;
use std::time::Duration;

/// HTTP method used by [`HttpRequest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Retrieves a resource.
    Get,
    /// Creates a resource or submits a payload.
    Post,
    /// Replaces a resource.
    Put,
    /// Partially updates a resource.
    Patch,
    /// Removes a resource.
    Delete,
    /// Retrieves response headers only.
    Head,
}

impl Method {
    /// Uppercase spelling of the method, as sent on the wire.
    pub const GET: Self = Self::Get;
    /// Uppercase spelling of the method, as sent on the wire.
    pub const POST: Self = Self::Post;
    /// Uppercase spelling of the method, as sent on the wire.
    pub const PUT: Self = Self::Put;
    /// Uppercase spelling of the method, as sent on the wire.
    pub const PATCH: Self = Self::Patch;
    /// Uppercase spelling of the method, as sent on the wire.
    pub const DELETE: Self = Self::Delete;
    /// Uppercase spelling of the method, as sent on the wire.
    pub const HEAD: Self = Self::Head;

    /// The method name in uppercase.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
        }
    }

    /// Whether a failed request with this method may be safely repeated.
    pub fn is_idempotent(self) -> bool {
        matches!(self, Self::Get | Self::Head)
    }
}

/// A request body together with the content type used to send it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpBody {
    /// MIME type sent in the `Content-Type` header.
    pub content_type: String,
    /// Raw body bytes.
    pub bytes: Vec<u8>,
}

/// Content type used by [`HttpRequest::json`].
const JSON_CONTENT_TYPE: &str = "application/json";

/// Content type used by [`HttpRequest::form`].
const FORM_CONTENT_TYPE: &str = "application/x-www-form-urlencoded";

/// Value of the `Content-Type` header.
pub const CONTENT_TYPE_HEADER: &str = "content-type";

/// Percent-encode one query or form component.
///
/// Unreserved characters are kept verbatim, a space becomes `+`, and every
/// other byte becomes `%XX`.
fn encode_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(*byte));
            }
            b' ' => encoded.push('+'),
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

/// One name/value pair accepted by [`HttpRequest::query`] and
/// [`HttpRequest::form`].
///
/// Implemented for owned and borrowed pairs of string-like values, so provider
/// code can pass `&Vec<(String, String)>`, `&Vec<(&str, String)>`, or an array
/// of `(&str, &str)` without converting first.
pub trait QueryPair {
    /// Convert the pair into owned strings.
    fn into_query_pair(self) -> (String, String);
}

impl<A: AsRef<str>, B: AsRef<str>> QueryPair for (A, B) {
    fn into_query_pair(self) -> (String, String) {
        (self.0.as_ref().to_string(), self.1.as_ref().to_string())
    }
}

impl<A: AsRef<str>, B: AsRef<str>> QueryPair for &(A, B) {
    fn into_query_pair(self) -> (String, String) {
        (self.0.as_ref().to_string(), self.1.as_ref().to_string())
    }
}

/// Query-parameter and header names whose values are credentials or session
/// material and must never reach a log line through `Debug`.
const SENSITIVE_PARAMETER_NAMES: &[&str] = &[
    "key",
    "apikey",
    "accesstoken",
    "token",
    "subscription-key",
    "signature",
    "sig",
    "session",
    "sessionid",
    "authorization",
    "proxy-authorization",
    "x-api-key",
    "x-goog-api-key",
    "cookie",
    "set-cookie",
];

/// Whether a parameter or header name carries a credential. Spelling variants
/// (`api_key`, `apiKey`, `access_token`) normalize to the same comparison.
fn is_sensitive_name(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase().replace('_', "");
    SENSITIVE_PARAMETER_NAMES
        .iter()
        .any(|sensitive| *sensitive == normalized)
}

/// Replaces the value of every credential-carrying parameter in a URL's own
/// query string with `REDACTED`, leaving the rest of the URL intact.
fn redacted_url(url: &str) -> String {
    let Some((base, query)) = url.split_once('?') else {
        return url.to_string();
    };
    let redacted_query = query
        .split('&')
        .map(|pair| match pair.split_once('=') {
            Some((name, _value)) if is_sensitive_name(name) => format!("{name}=REDACTED"),
            Some(_) | None => pair.to_string(),
        })
        .collect::<Vec<_>>()
        .join("&");
    format!("{base}?{redacted_query}")
}

/// A single HTTP request, independent of any HTTP backend.
///
/// `Debug` redacts the values of credential-carrying query parameters and
/// headers and the whole body, so a request handed to a logger cannot print a
/// key, token, signature or session and cannot print a form or JSON body that
/// carries one.
#[derive(Clone, PartialEq, Eq)]
pub struct HttpRequest {
    method: Method,
    url: String,
    query: Vec<(String, String)>,
    headers: Vec<(String, String)>,
    body: Option<HttpBody>,
    timeout: Option<Duration>,
}

impl std::fmt::Debug for HttpRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let redacted_query = self
            .query
            .iter()
            .map(|(name, value)| {
                let shown_value = if is_sensitive_name(name) {
                    "REDACTED".to_string()
                } else {
                    value.clone()
                };
                (name.clone(), shown_value)
            })
            .collect::<Vec<_>>();
        let redacted_headers = self
            .headers
            .iter()
            .map(|(name, value)| {
                let shown_value = if is_sensitive_name(name) {
                    "REDACTED".to_string()
                } else {
                    value.clone()
                };
                (name.clone(), shown_value)
            })
            .collect::<Vec<_>>();
        formatter
            .debug_struct("HttpRequest")
            .field("method", &self.method)
            .field("url", &redacted_url(&self.url))
            .field("query", &redacted_query)
            .field("headers", &redacted_headers)
            .field(
                "body",
                &self.body.as_ref().map(|body| body.content_type.as_str()),
            )
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl HttpRequest {
    /// Create a request with the given method and URL.
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            query: Vec::new(),
            headers: Vec::new(),
            body: None,
            timeout: None,
        }
    }

    /// Create a `GET` request.
    pub fn get(url: impl Into<String>) -> Self {
        Self::new(Method::Get, url)
    }

    /// Create a `POST` request.
    pub fn post(url: impl Into<String>) -> Self {
        Self::new(Method::Post, url)
    }

    /// Append query parameters.
    pub fn query<I>(mut self, pairs: I) -> Self
    where
        I: IntoIterator,
        I::Item: QueryPair,
    {
        self.query
            .extend(pairs.into_iter().map(QueryPair::into_query_pair));
        self
    }

    /// Append one query parameter.
    pub fn query_param(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.push((name.into(), value.into()));
        self
    }

    /// Add a header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Set a JSON body.
    ///
    /// Returns an error only when the value cannot be serialized.
    pub fn json<T: Serialize>(mut self, value: &T) -> EveryMapResult<Self> {
        let bytes = serde_json::to_vec(value)?;
        self.body = Some(HttpBody {
            content_type: JSON_CONTENT_TYPE.to_string(),
            bytes,
        });
        Ok(self)
    }

    /// Set an `application/x-www-form-urlencoded` body.
    pub fn form<I>(mut self, pairs: I) -> Self
    where
        I: IntoIterator,
        I::Item: QueryPair,
    {
        let encoded = pairs
            .into_iter()
            .map(QueryPair::into_query_pair)
            .map(|(name, value)| {
                format!("{}={}", encode_component(&name), encode_component(&value))
            })
            .collect::<Vec<_>>()
            .join("&");
        self.body = Some(HttpBody {
            content_type: FORM_CONTENT_TYPE.to_string(),
            bytes: encoded.into_bytes(),
        });
        self
    }

    /// Set a raw body with an explicit content type.
    pub fn raw_body(mut self, content_type: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        self.body = Some(HttpBody {
            content_type: content_type.into(),
            bytes: bytes.into(),
        });
        self
    }

    /// Set the per-request timeout, overriding the client default.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// The request method.
    pub fn method(&self) -> Method {
        self.method
    }

    /// The base URL, without appended query parameters.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// The full URL, with query parameters appended and percent-encoded.
    pub fn url_with_query(&self) -> String {
        if self.query.is_empty() {
            return self.url.clone();
        }
        let separator = if self.url.contains('?') { '&' } else { '?' };
        let encoded = self
            .query
            .iter()
            .map(|(name, value)| format!("{}={}", encode_component(name), encode_component(value)))
            .collect::<Vec<_>>()
            .join("&");
        format!("{}{separator}{encoded}", self.url)
    }

    /// All headers on the request.
    pub fn headers(&self) -> &[(String, String)] {
        &self.headers
    }

    /// The request body, when present.
    pub fn body(&self) -> Option<&HttpBody> {
        self.body.as_ref()
    }

    /// The per-request timeout, when set.
    pub fn timeout_value(&self) -> Option<Duration> {
        self.timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_names_and_idempotence() {
        assert_eq!(Method::GET.as_str(), "GET");
        assert_eq!(Method::POST.as_str(), "POST");
        assert!(Method::GET.is_idempotent());
        assert!(Method::HEAD.is_idempotent());
        assert!(!Method::POST.is_idempotent());
    }

    /// A query string never ends with the separator: Photon and OSRM refuse a
    /// trailing `&` as a malformed query, and MapQuest silently tolerates it.
    #[test]
    fn the_query_string_never_ends_with_a_separator() {
        let request = HttpRequest::get("https://router.project-osrm.org/route/v1/driving/1,2;3,4")
            .query(&[("overview", "full"), ("steps", "true")]);
        let url = request.url_with_query();
        assert!(url.contains("?overview=full&steps=true"));
        assert!(!url.ends_with('&'), "{url}");

        let with_existing_query =
            HttpRequest::get("https://example.test/api?key=test").query(&[("q", "paris")]);
        let existing_url = with_existing_query.url_with_query();
        assert_eq!(existing_url, "https://example.test/api?key=test&q=paris");

        let without_query = HttpRequest::get("https://example.test/api");
        assert_eq!(without_query.url_with_query(), "https://example.test/api");
    }

    /// A request's `Debug` form is what a log line would print, so no
    /// credential, body byte or session value may survive it.
    #[test]
    fn debug_redacts_credentials_in_the_url_query_headers_and_body() {
        let request = HttpRequest::get("https://example.test/api?apikey=url-secret")
            .query(&[("api_key", "query-secret"), ("q", "paris")])
            .header("Authorization", "Bearer header-secret")
            .header("X-Api-Key", "header-api-key-secret")
            .header("Accept", "application/json")
            .json(&serde_json::json!({ "access_token": "body-secret" }))
            .expect("the test body serializes");
        let debug = format!("{request:?}");
        for secret in [
            "url-secret",
            "query-secret",
            "header-secret",
            "header-api-key-secret",
            "body-secret",
        ] {
            assert!(!debug.contains(secret), "{secret} leaked through {debug}");
        }
        assert!(debug.contains("paris"), "ordinary values stay readable");
        assert!(
            debug.contains("application/json"),
            "the body's content type stays visible"
        );
    }

    #[test]
    fn query_encodes_values() {
        let request = HttpRequest::get("https://example.com/search")
            .query(&[("q", "a b,c"), ("apiKey", "secret/key")]);
        assert_eq!(
            request.url_with_query(),
            "https://example.com/search?q=a+b%2Cc&apiKey=secret%2Fkey"
        );
    }

    #[test]
    fn query_appends_to_existing_query() {
        let request =
            HttpRequest::get("https://example.com/search?lang=en").query([("q", "berlin")]);
        assert_eq!(
            request.url_with_query(),
            "https://example.com/search?lang=en&q=berlin"
        );
    }

    #[test]
    fn query_accepts_owned_borrowed_and_array_pairs() {
        let owned: Vec<(String, String)> = vec![("a".to_string(), "1".to_string())];
        let borrowed: Vec<(&str, String)> = vec![("b", "2".to_string())];
        let request = HttpRequest::get("https://example.com")
            .query(&owned)
            .query(&borrowed)
            .query(&[("c", "3")]);
        assert_eq!(request.url_with_query(), "https://example.com?a=1&b=2&c=3");
    }

    #[test]
    fn json_body_sets_content_type() {
        let request = HttpRequest::post("https://example.com")
            .json(&serde_json::json!({"key": "value"}))
            .unwrap();
        let body = request.body().expect("body");
        assert_eq!(body.content_type, JSON_CONTENT_TYPE);
        assert_eq!(body.bytes, br#"{"key":"value"}"#);
    }

    #[test]
    fn form_body_encodes_pairs() {
        let pairs: Vec<(&str, String)> = vec![("scope", "maps read".to_string())];
        let request = HttpRequest::post("https://example.com").form(&pairs);
        let body = request.body().expect("body");
        assert_eq!(body.content_type, FORM_CONTENT_TYPE);
        assert_eq!(body.bytes, b"scope=maps+read");
    }

    #[test]
    fn headers_and_timeout_are_recorded() {
        let request = HttpRequest::get("https://example.com")
            .header("X-Request-ID", "abc")
            .timeout(Duration::from_secs(5));
        assert_eq!(
            request.headers(),
            &[("X-Request-ID".to_string(), "abc".to_string())]
        );
        assert_eq!(request.timeout_value(), Some(Duration::from_secs(5)));
    }
}
