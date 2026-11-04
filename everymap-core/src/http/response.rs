//! Backend-agnostic HTTP response values returned by the transport seam.
//!
//! The response owns its bytes. JSON decoding is available through
//! [`HttpResponse::json`], but the client reads the body as text for its error
//! messages and verbose output.

use serde::de::DeserializeOwned;

/// HTTP response headers.
///
/// Lookups are case-insensitive; iteration preserves the names and values as
/// received from the backend.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HttpHeaders(Vec<(String, String)>);

impl HttpHeaders {
    /// Create a header collection.
    pub fn new(headers: Vec<(String, String)>) -> Self {
        Self(headers)
    }

    /// Look up the first value of a header, ignoring case.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// Iterate over the header pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    /// Whether no headers are present.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Number of headers.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// An HTTP response, independent of any HTTP backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    status: u16,
    headers: HttpHeaders,
    body: Vec<u8>,
}

impl HttpResponse {
    /// Create a response from its parts.
    pub fn new(status: u16, headers: HttpHeaders, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            headers,
            body: body.into(),
        }
    }

    /// The HTTP status code.
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Whether the status is in the `2xx` range.
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// The response headers.
    pub fn headers(&self) -> &HttpHeaders {
        &self.headers
    }

    /// The raw response body.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Consume the response and return the raw body.
    pub fn into_body(self) -> Vec<u8> {
        self.body
    }

    /// Decode the body as UTF-8, replacing invalid sequences.
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    /// Deserialize the body as JSON.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_lookup_is_case_insensitive() {
        let headers = HttpHeaders::new(vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Retry-After".to_string(), "10".to_string()),
        ]);
        assert_eq!(headers.get("content-type"), Some("application/json"));
        assert_eq!(headers.get("CONTENT-TYPE"), Some("application/json"));
        assert_eq!(headers.get("retry-after"), Some("10"));
        assert_eq!(headers.get("missing"), None);
        assert_eq!(headers.len(), 2);
        assert!(!headers.is_empty());
    }

    #[test]
    fn success_range() {
        assert!(HttpResponse::new(200, HttpHeaders::default(), vec![]).is_success());
        assert!(HttpResponse::new(299, HttpHeaders::default(), vec![]).is_success());
        assert!(!HttpResponse::new(300, HttpHeaders::default(), vec![]).is_success());
        assert!(!HttpResponse::new(429, HttpHeaders::default(), vec![]).is_success());
    }

    #[test]
    fn text_is_lossy_and_json_decodes() {
        let response = HttpResponse::new(
            200,
            HttpHeaders::default(),
            br#"{"name":"Berlin"}"#.to_vec(),
        );
        assert_eq!(response.text(), r#"{"name":"Berlin"}"#);
        let value: serde_json::Value = response.json().unwrap();
        assert_eq!(value["name"], "Berlin");
    }

    #[test]
    fn into_body_returns_bytes() {
        let response = HttpResponse::new(200, HttpHeaders::default(), b"abc".to_vec());
        assert_eq!(response.body(), b"abc");
        assert_eq!(response.into_body(), b"abc");
    }
}
