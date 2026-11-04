//! Redaction and the request/response observer seam.
//!
//! Observers are called by [`crate::client::ProviderClient`] for every attempt.
//! They receive the request URL after credential-bearing query values have been
//! replaced with `***`. Request and response bodies are never passed to an
//! observer, and header values are omitted entirely.

use crate::http::Method;
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

fn is_sensitive_query_parameter(name: &str) -> bool {
    REDACTED_QUERY_PARAMETERS
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
}

/// Receives every request attempt and its outcome.
///
/// Implementations must be cheap and non-blocking; they are called on the
/// request path. A `ProviderClient` holds at most one observer.
pub trait RequestObserver: Send + Sync {
    /// Called before the transport sends a request.
    fn on_request(&self, observation: &RequestObservation<'_>);

    /// Called after the transport returns or fails.
    fn on_response(&self, observation: &ResponseObservation<'_>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_every_known_credential_parameter() {
        let url = "https://example.com/route?apiKey=one&key=two&access_token=three&token=four&client_secret=five&api_key=six&subscription-key=seven&auth=eight&q=berlin";
        let redacted = redact_url(url);
        for secret in [
            "one", "two", "three", "four", "five", "six", "seven", "eight",
        ] {
            assert!(
                !redacted.contains(secret),
                "secret {secret} leaked: {redacted}"
            );
        }
        assert!(redacted.contains("q=berlin"));
        assert_eq!(
            redacted,
            "https://example.com/route?apiKey=***&key=***&access_token=***&token=***&client_secret=***&api_key=***&subscription-key=***&auth=***&q=berlin"
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
}
