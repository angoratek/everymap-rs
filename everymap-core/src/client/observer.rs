//! Redaction and the request/response observer seam.
//!
//! Observers are called by [`crate::client::ProviderClient`] for every attempt.
//! They receive the request URL after credential-bearing query values have been
//! replaced with `***`. Request bodies are never passed to an observer. A
//! response body reaches it only as a bounded excerpt, and only while response
//! capture is enabled: the excerpt has its credential-bearing fields replaced
//! and is truncated on a character boundary.

use crate::http::Method;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Query parameter names whose values are replaced by [`REDACTED_VALUE`].
pub const REDACTED_QUERY_PARAMETERS: &[&str] = &[
    "apikey",
    "api_key",
    "key",
    "access_token",
    "token",
    "client_secret",
    "subscription-key",
    "auth",
    "session",
    "sessionid",
    "signature",
    "sig",
];

/// Header names whose values must never be logged.
pub const REDACTED_HEADER_NAMES: &[&str] = &[
    "authorization",
    "proxy-authorization",
    "x-api-key",
    "x-goog-api-key",
    "cookie",
    "set-cookie",
];

/// Replacement written in place of a credential value.
pub const REDACTED_VALUE: &str = "***";

/// Maximum length of a captured response excerpt, in bytes, counted on
/// character boundaries so a multi-byte glyph is never split.
pub const RESPONSE_EXCERPT_MAXIMUM_BYTES: usize = 512;

/// JSON member and assignment names whose values are replaced in a captured
/// response excerpt. The list extends the query vocabulary with the session
/// and signature fields a provider response may echo.
pub const REDACTED_RESPONSE_FIELD_NAMES: &[&str] = &[
    "apikey",
    "api_key",
    "key",
    "access_token",
    "token",
    "client_secret",
    "subscription-key",
    "auth",
    "session",
    "sessionid",
    "signature",
    "sig",
];

/// Whether transports capture a bounded response excerpt for observers.
static RESPONSE_EXCERPT_CAPTURE_ENABLED: AtomicBool = AtomicBool::new(false);

/// Enables or disables response-excerpt capture for every client in the
/// process.
///
/// Capture is off until a host asks for it, so an application that never
/// shows a response body never has one held in memory or handed to an
/// observer. The switch is process-wide because a host enables it once for
/// its diagnostics surface.
pub fn set_response_excerpt_capture(enabled: bool) {
    RESPONSE_EXCERPT_CAPTURE_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Whether response-excerpt capture is enabled.
pub fn response_excerpt_capture_enabled() -> bool {
    RESPONSE_EXCERPT_CAPTURE_ENABLED.load(Ordering::Relaxed)
}

fn is_sensitive_query_parameter(name: &str) -> bool {
    REDACTED_QUERY_PARAMETERS
        .iter()
        .any(|sensitive| name.eq_ignore_ascii_case(sensitive))
}

fn is_sensitive_response_field(name: &str) -> bool {
    REDACTED_RESPONSE_FIELD_NAMES
        .iter()
        .any(|sensitive| name.eq_ignore_ascii_case(sensitive))
}

/// Whether a header carries a credential and must not have its value logged.
pub fn is_sensitive_header(name: &str) -> bool {
    REDACTED_HEADER_NAMES
        .iter()
        .any(|sensitive| name.eq_ignore_ascii_case(sensitive))
}

/// Replace credential-bearing query values in a URL with [`REDACTED_VALUE`].
///
/// The fragment and all non-sensitive query pairs are preserved. The function
/// is lossless for URLs without a query string.
pub fn redact_url(url: &str) -> String {
    let (without_fragment, fragment) = match url.split_once('#') {
        Some((without_fragment, fragment)) => (without_fragment, Some(fragment)),
        None => (url, None),
    };
    let (base, query) = match without_fragment.split_once('?') {
        Some((base, query)) => (base, Some(query)),
        None => (without_fragment, None),
    };

    let redacted_query = query.map(|query| {
        query
            .split('&')
            .map(|pair| match pair.split_once('=') {
                Some((name, _value)) if is_sensitive_query_parameter(name) => {
                    format!("{name}={REDACTED_VALUE}")
                }
                _ => pair.to_string(),
            })
            .collect::<Vec<_>>()
            .join("&")
    });

    match (redacted_query, fragment) {
        (Some(query), Some(fragment)) => format!("{base}?{query}#{fragment}"),
        (Some(query), None) => format!("{base}?{query}"),
        (None, Some(fragment)) => format!("{base}#{fragment}"),
        (None, None) => base.to_string(),
    }
}

/// Capture one response body as a bounded, redacted excerpt.
///
/// Returns `None` while capture is disabled or when the body is empty. The
/// excerpt is capped at [`RESPONSE_EXCERPT_MAXIMUM_BYTES`] on a character
/// boundary. String values of [`REDACTED_RESPONSE_FIELD_NAMES`] members and
/// assignments are replaced with [`REDACTED_VALUE`]; a credential that is not
/// named by that vocabulary cannot be recognized and should not be placed in a
/// response body by a provider.
pub fn capture_response_excerpt(body: &[u8]) -> Option<String> {
    if !response_excerpt_capture_enabled() || body.is_empty() {
        return None;
    }
    let text = String::from_utf8_lossy(body);
    let redacted = redact_assignments(&redact_json_string_members(&text));
    Some(truncate_on_character_boundary(
        &redacted,
        RESPONSE_EXCERPT_MAXIMUM_BYTES,
    ))
}

/// Whether a byte may appear in an assignment field name.
fn is_assignment_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

/// Whether a byte ends an assignment value.
fn is_assignment_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b'&' | b'"' | b'\'' | b',' | b'}' | b')' | b']' | b';' | b'|' | b'#' | b'<'
    ) || byte.is_ascii_whitespace()
}

/// The index of the closing quote of the string that opens at `opening`.
fn string_end(bytes: &[u8], opening: usize) -> Option<usize> {
    let mut index = opening + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'"' => return Some(index),
            _ => index += 1,
        }
    }
    None
}

/// Replaces the string values of sensitive JSON members with [`REDACTED_VALUE`].
fn redact_json_string_members(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'"' {
            index += 1;
            continue;
        }
        let Some(key_end) = string_end(bytes, index) else {
            break;
        };
        let key = &text[index + 1..key_end];
        let mut position = key_end + 1;
        while position < bytes.len() && bytes[position].is_ascii_whitespace() {
            position += 1;
        }
        if position >= bytes.len() || bytes[position] != b':' {
            index = key_end + 1;
            continue;
        }
        let mut value_start = position + 1;
        while value_start < bytes.len() && bytes[value_start].is_ascii_whitespace() {
            value_start += 1;
        }
        let Some(value_end) = (value_start < bytes.len() && bytes[value_start] == b'"')
            .then(|| string_end(bytes, value_start))
            .flatten()
        else {
            index = value_start.max(key_end + 1);
            continue;
        };
        if is_sensitive_response_field(key) {
            output.push_str(&text[cursor..value_start + 1]);
            output.push_str(REDACTED_VALUE);
            cursor = value_end;
        }
        index = value_end + 1;
    }
    output.push_str(&text[cursor..]);
    output
}

/// Replaces the value of every sensitive `name=value` assignment with
/// [`REDACTED_VALUE`].
fn redact_assignments(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'=' {
            index += 1;
            continue;
        }
        let mut name_start = index;
        while name_start > 0 && is_assignment_name_byte(bytes[name_start - 1]) {
            name_start -= 1;
        }
        let name = &text[name_start..index];
        if name.is_empty() || !is_sensitive_response_field(name) {
            index += 1;
            continue;
        }
        let mut value_start = index + 1;
        while value_start < bytes.len() && bytes[value_start].is_ascii_whitespace() {
            value_start += 1;
        }
        let mut value_end = value_start;
        while value_end < bytes.len() && !is_assignment_delimiter(bytes[value_end]) {
            value_end += 1;
        }
        if value_end == value_start {
            index += 1;
            continue;
        }
        output.push_str(&text[cursor..value_start]);
        output.push_str(REDACTED_VALUE);
        cursor = value_end;
        index = value_end;
    }
    output.push_str(&text[cursor..]);
    output
}

/// Caps `text` at `maximum_bytes`, never splitting a character.
fn truncate_on_character_boundary(text: &str, maximum_bytes: usize) -> String {
    if text.len() <= maximum_bytes {
        return text.to_string();
    }
    let mut end = maximum_bytes;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_string()
}

/// One request attempt as seen by a [`RequestObserver`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestObservation<'a> {
    /// Provider name passed to the client.
    pub provider: &'a str,
    /// HTTP method.
    pub method: Method,
    /// Request URL with credential values redacted.
    pub url: &'a str,
    /// One-based attempt number.
    pub attempt: u32,
}

/// The outcome of one request attempt as seen by a [`RequestObserver`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResponseObservation<'a> {
    /// Provider name passed to the client.
    pub provider: &'a str,
    /// HTTP status, or `None` when the transport failed.
    pub status: Option<u16>,
    /// Time spent in the transport.
    pub elapsed: Duration,
    /// One-based attempt number.
    pub attempt: u32,
    /// The bounded, redacted response excerpt, when capture is enabled and a
    /// response body arrived. See [`capture_response_excerpt`].
    pub response_excerpt: Option<&'a str>,
}

/// Receives every request attempt and its outcome.
///
/// Implementations must be cheap and non-blocking; they are called on the
/// request path. A `ProviderClient` holds at most one observer.
pub trait RequestObserver: Send + Sync {
    /// Called before the transport sends a request.
    fn on_request(&self, observation: &RequestObservation<'_>);

    /// Called after the transport returns or fails. The response excerpt is
    /// present only while response capture is enabled.
    fn on_response(&self, observation: &ResponseObservation<'_>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_every_known_credential_parameter() {
        let url = "https://example.com/route?apiKey=one&key=two&access_token=three&token=four&client_secret=five&api_key=six&subscription-key=seven&auth=eight&session=nine&sessionId=ten&signature=eleven&sig=twelve&q=berlin";
        let redacted = redact_url(url);
        for secret in [
            "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
            "eleven", "twelve",
        ] {
            assert!(
                !redacted.contains(secret),
                "secret {secret} leaked: {redacted}"
            );
        }
        assert!(redacted.contains("q=berlin"));
        assert_eq!(
            redacted,
            "https://example.com/route?apiKey=***&key=***&access_token=***&token=***&client_secret=***&api_key=***&subscription-key=***&auth=***&session=***&sessionId=***&signature=***&sig=***&q=berlin"
        );
    }

    #[test]
    fn redaction_preserves_fragments_and_is_case_insensitive() {
        let redacted = redact_url("https://example.com/a?APIKEY=secret&x=1#section");
        assert_eq!(redacted, "https://example.com/a?APIKEY=***&x=1#section");
    }

    #[test]
    fn redaction_is_lossless_without_credentials() {
        let url = "https://example.com/a?foo=bar&baz=qux#frag";
        assert_eq!(redact_url(url), url);
        assert_eq!(redact_url("https://example.com/a"), "https://example.com/a");
    }

    #[test]
    fn sensitive_headers_are_recognized_case_insensitively() {
        assert!(is_sensitive_header("Authorization"));
        assert!(is_sensitive_header("x-api-key"));
        assert!(!is_sensitive_header("content-type"));
    }

    #[test]
    fn a_response_excerpt_redacts_json_members_and_assignments() {
        let body = concat!(
            r#"{"apiKey":"secret-one","name":"Berlin","sessionId":"secret-two","#,
            r#""nested":{"access_token":"secret-three"},"url":"https://example.com/a?key=secret-four&q=1"}"#
        );
        let redacted = redact_assignments(&redact_json_string_members(body));
        for secret in ["secret-one", "secret-two", "secret-three", "secret-four"] {
            assert!(
                !redacted.contains(secret),
                "secret {secret} leaked: {redacted}"
            );
        }
        assert!(redacted.contains(r#""name":"Berlin""#));
        assert!(redacted.contains(&format!(r#""apiKey":"{REDACTED_VALUE}""#)));
        assert!(redacted.contains(&format!(r#""sessionId":"{REDACTED_VALUE}""#)));
    }

    #[test]
    fn capture_is_off_by_default_and_bounded_when_enabled() {
        set_response_excerpt_capture(false);
        assert!(capture_response_excerpt(br#"{"name":"Berlin"}"#).is_none());

        set_response_excerpt_capture(true);
        assert!(capture_response_excerpt(b"").is_none());
        let excerpt = capture_response_excerpt(br#"{"name":"Berlin"}"#).expect("captured");
        assert_eq!(excerpt, r#"{"name":"Berlin"}"#);

        // A multi-byte glyph at the boundary is never split.
        let long_body = format!(r#"{{"name":"{}"}}"#, "é".repeat(400));
        let bounded = capture_response_excerpt(long_body.as_bytes()).expect("captured");
        assert!(bounded.len() <= RESPONSE_EXCERPT_MAXIMUM_BYTES);
        assert!(bounded.is_char_boundary(bounded.len()));
        set_response_excerpt_capture(false);
    }
}
