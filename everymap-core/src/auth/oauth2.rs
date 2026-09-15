use crate::auth::provider::AuthProvider;
use crate::client::truncate_str;
use crate::error::{EveryMapError, EveryMapResult};
use async_trait::async_trait;
use reqwest::header::{HeaderValue, AUTHORIZATION};
use reqwest::RequestBuilder;
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
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
/// The client secret and cached tokens are zeroized on drop to prevent
/// lingering sensitive data in memory.
pub struct OAuth2Provider {
    credentials: ClientCredentials,
    token_endpoint: String,
    http_client: reqwest::Client,
    cached_token: Mutex<Option<CachedToken>>,
}

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
        Self {
            credentials: ClientCredentials {
                client_id,
                client_secret,
                scope,
            },
            token_endpoint,
            http_client: reqwest::Client::new(),
            cached_token: Mutex::new(None),
        }
    }

    /// Return a valid cached access token, fetching a new one when missing or expired.
    ///
    /// The fetch happens while holding the token lock, so concurrent callers
    /// share a single token request instead of stampeding the token endpoint.
    async fn access_token(&self) -> EveryMapResult<String> {
        let mut cached = self.cached_token.lock().await;
        if let Some(token) = cached.as_ref() {
            if Instant::now() < token.expires_at {
                return Ok(token.access_token.clone());
            }
        }
        let new_token = self.fetch_token().await?;
        let access_token = new_token.access_token.clone();
        *cached = Some(new_token);
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

        let response = self
            .http_client
            .post(&self.token_endpoint)
            .form(&form_pairs)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(EveryMapError::auth(
                OAUTH2_PROVIDER_NAME,
                format!(
                    "Token endpoint returned HTTP {}: {}",
                    status.as_u16(),
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
    async fn apply(&self, request: RequestBuilder) -> EveryMapResult<RequestBuilder> {
        let access_token = self.access_token().await?;
        let header_value =
            HeaderValue::from_str(&format!("Bearer {}", access_token)).map_err(|error| {
                EveryMapError::auth(
                    OAUTH2_PROVIDER_NAME,
                    format!("Invalid bearer token header value: {}", error),
                )
            })?;
        Ok(request.header(AUTHORIZATION, header_value))
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
}
