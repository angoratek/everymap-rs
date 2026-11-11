use crate::auth::header::{is_valid_header_value, AUTHORIZATION_HEADER};
use crate::auth::provider::AuthProvider;
use crate::client::truncate_str;
use crate::error::{EveryMapError, EveryMapResult};
use crate::http::{default_transport, HttpRequest, HttpTransport, Method};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use web_time::Instant;
use zeroize::Zeroize;

/// Seconds shaved off the token lifetime so tokens refresh before actual expiry.
const TOKEN_EXPIRY_SKEW_SECS: u64 = 30;

/// Fallback token lifetime (seconds) when the token response omits `expires_in`.
const DEFAULT_EXPIRES_IN_SECS: u64 = 3600;

/// Maximum bytes of error body included in auth error messages.
const ERROR_BODY_LIMIT: usize = 256;

/// Provider name used in error messages.
const OAUTH2_PROVIDER_NAME: &str = "oauth2";

/// Provider for OAuth 2.0 client-credentials authentication.
///
/// On each request, ensures a valid cached access token (fetching one from the
/// token endpoint via the `client_credentials` grant when missing or expired)
/// and injects it as an `Authorization: Bearer <token>` header.
///
/// Token requests travel through the same [`HttpTransport`] seam as provider
/// requests, so the provider works on native and WebAssembly targets alike.
///
/// The client secret and cached tokens are zeroized on drop to prevent
/// lingering sensitive data in memory.
pub struct OAuth2Provider {
    credentials: ClientCredentials,
    token_endpoint: String,
    transport: Arc<dyn HttpTransport>,
    cached_token: Mutex<Option<CachedToken>>,
    /// Timeout applied to each token request, so an unresponsive endpoint
    /// cannot hold a caller indefinitely.
    request_timeout: Duration,
}

/// Timeout applied to a token request unless the caller sets another one.
const DEFAULT_TOKEN_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// OAuth 2.0 client credentials (zeroized on drop).
#[derive(Zeroize)]
#[zeroize(drop)]
struct ClientCredentials {
    client_id: String,
    client_secret: String,
    scope: Option<String>,
}

/// A cached access token with its expiry instant (zeroized on drop).
#[derive(Zeroize)]
#[zeroize(drop)]
struct CachedToken {
    access_token: String,
    #[zeroize(skip)]
    expires_at: Instant,
}

/// Token endpoint response (`client_credentials` grant).
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    expires_in: Option<u64>,
}

impl OAuth2Provider {
    /// Create a new OAuth 2.0 client-credentials auth provider.
    ///
    /// `token_endpoint` is the full URL of the OAuth 2.0 token endpoint.
    /// `scope` is optional and sent as the `scope` form parameter when present.
    pub fn new(
        token_endpoint: String,
        client_id: String,
        client_secret: String,
        scope: Option<String>,
    ) -> Self {
        Self::with_transport(
            default_transport(),
            token_endpoint,
            client_id,
            client_secret,
            scope,
        )
    }

    /// Create a provider that fetches tokens through an explicit transport.
    pub fn with_transport(
        transport: Arc<dyn HttpTransport>,
        token_endpoint: String,
        client_id: String,
        client_secret: String,
        scope: Option<String>,
    ) -> Self {
        Self {
            credentials: ClientCredentials {
                client_id,
                client_secret,
                scope,
            },
            token_endpoint,
            transport,
            cached_token: Mutex::new(None),
            request_timeout: DEFAULT_TOKEN_REQUEST_TIMEOUT,
        }
    }

    /// Overrides the timeout applied to each token request.
    pub fn set_request_timeout(&mut self, request_timeout: Duration) {
        self.request_timeout = request_timeout;
    }

    /// Return a valid cached access token, fetching a new one when missing or expired.
    ///
    /// The lock is held only to read or replace the cached token, never across
    /// the token request: a slow endpoint cannot block every other capability
    /// that needs the same credential. Two callers that race an expired token
    /// may each request one; the last response wins, which is harmless because
    /// both tokens are valid for the same client.
    async fn access_token(&self) -> EveryMapResult<String> {
        let cached_token = {
            let cached = self.cached_token.lock().await;
            cached
                .as_ref()
                .filter(|token| Instant::now() < token.expires_at)
                .map(|token| token.access_token.clone())
        };
        if let Some(access_token) = cached_token {
            return Ok(access_token);
        }
        let new_token = self.fetch_token().await?;
        let access_token = new_token.access_token.clone();
        *self.cached_token.lock().await = Some(new_token);
        Ok(access_token)
    }

    /// Fetch a fresh access token from the token endpoint via the
    /// `client_credentials` grant (`POST`, `application/x-www-form-urlencoded`).
    async fn fetch_token(&self) -> EveryMapResult<CachedToken> {
        let mut form_pairs: Vec<(&str, &str)> = vec![
            ("grant_type", "client_credentials"),
            ("client_id", &self.credentials.client_id),
            ("client_secret", &self.credentials.client_secret),
        ];
        if let Some(scope) = &self.credentials.scope {
            form_pairs.push(("scope", scope));
        }

        let request = HttpRequest::new(Method::Post, &self.token_endpoint)
            .form(&form_pairs)
            .timeout(self.request_timeout);
        let response = self.transport.send(request).await?;

        let status = response.status();
        let body = response.text();
        if !response.is_success() {
            return Err(EveryMapError::auth(
                OAUTH2_PROVIDER_NAME,
                format!(
                    "Token endpoint returned HTTP {}: {}",
                    status,
                    truncate_str(&body, ERROR_BODY_LIMIT)
                ),
            ));
        }

        let token_response: TokenResponse = serde_json::from_str(&body).map_err(|error| {
            EveryMapError::auth(
                OAUTH2_PROVIDER_NAME,
                format!(
                    "Failed to parse token response: {} (body: {})",
                    error,
                    truncate_str(&body, ERROR_BODY_LIMIT)
                ),
            )
        })?;

        Ok(CachedToken {
            access_token: token_response.access_token,
            expires_at: expires_at_from(
                token_response.expires_in.unwrap_or(DEFAULT_EXPIRES_IN_SECS),
            ),
        })
    }
}

#[async_trait]
impl AuthProvider for OAuth2Provider {
    async fn apply(&self, request: HttpRequest) -> EveryMapResult<HttpRequest> {
        let access_token = self.access_token().await?;
        let header_value = format!("Bearer {}", access_token);
        if !is_valid_header_value(&header_value) {
            return Err(EveryMapError::auth(
                OAUTH2_PROVIDER_NAME,
                "Invalid bearer token header value",
            ));
        }
        Ok(request.header(AUTHORIZATION_HEADER, header_value))
    }
}

/// Compute the cached-token expiry instant: `now + expires_in - safety skew`.
fn expires_at_from(expires_in_secs: u64) -> Instant {
    Instant::now() + Duration::from_secs(expires_in_secs.saturating_sub(TOKEN_EXPIRY_SKEW_SECS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expires_at_respects_skew() {
        let expires_at = expires_at_from(3600);
        let remaining = expires_at.checked_duration_since(Instant::now());
        let remaining = remaining.expect("token must not be expired");
        assert!(remaining <= Duration::from_secs(3600 - TOKEN_EXPIRY_SKEW_SECS));
        assert!(remaining >= Duration::from_secs(3600 - TOKEN_EXPIRY_SKEW_SECS - 5));
    }

    #[test]
    fn test_expires_at_zero_lifetime_is_immediately_expired() {
        let expires_at = expires_at_from(0);
        assert!(Instant::now() >= expires_at);
    }

    #[test]
    fn test_expires_at_lifetime_below_skew_is_immediately_expired() {
        let expires_at = expires_at_from(TOKEN_EXPIRY_SKEW_SECS - 1);
        assert!(Instant::now() >= expires_at);
    }

    #[test]
    fn test_token_response_full() {
        let token_response: TokenResponse =
            serde_json::from_str(r#"{"access_token":"tok","expires_in":7200}"#).unwrap();
        assert_eq!(token_response.access_token, "tok");
        assert_eq!(token_response.expires_in, Some(7200));
    }

    #[test]
    fn test_token_response_without_expires_in() {
        let token_response: TokenResponse =
            serde_json::from_str(r#"{"access_token":"tok"}"#).unwrap();
        assert_eq!(token_response.access_token, "tok");
        assert_eq!(token_response.expires_in, None);
    }

    #[test]
    fn test_token_response_missing_access_token_fails() {
        let parse_result = serde_json::from_str::<TokenResponse>(r#"{"expires_in":60}"#);
        assert!(parse_result.is_err());
    }

    #[test]
    fn header_value_validation_rejects_controls() {
        assert!(is_valid_header_value("Bearer token-abc"));
        assert!(!is_valid_header_value("Bearer token\nabc"));
        assert!(!is_valid_header_value(""));
    }

    /// Records the token requests and answers one fixed access token.
    struct RecordingTokenTransport {
        requests: std::sync::Mutex<Vec<crate::http::HttpRequest>>,
    }

    #[async_trait]
    impl crate::http::HttpTransport for RecordingTokenTransport {
        async fn send(
            &self,
            request: crate::http::HttpRequest,
        ) -> EveryMapResult<crate::http::HttpResponse> {
            self.requests
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(request);
            Ok(crate::http::HttpResponse::new(
                200,
                crate::http::HttpHeaders::default(),
                br#"{"access_token":"tok","expires_in":3600}"#.to_vec(),
            ))
        }
    }

    #[tokio::test]
    async fn a_token_request_carries_the_configured_timeout_and_the_token_is_cached() {
        let transport = std::sync::Arc::new(RecordingTokenTransport {
            requests: std::sync::Mutex::new(Vec::new()),
        });
        let mut provider = OAuth2Provider::with_transport(
            transport.clone(),
            "https://auth.example.test/token".to_string(),
            "client-id".to_string(),
            "client-secret".to_string(),
            None,
        );
        provider.set_request_timeout(Duration::from_secs(7));

        let first = provider.access_token().await.expect("the token answers");
        let second = provider
            .access_token()
            .await
            .expect("the cached token answers");
        assert_eq!(first, "tok");
        assert_eq!(second, "tok");

        let requests = transport
            .requests
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert_eq!(requests.len(), 1, "the second call serves the cached token");
        assert_eq!(requests[0].timeout_value(), Some(Duration::from_secs(7)));
    }
}
