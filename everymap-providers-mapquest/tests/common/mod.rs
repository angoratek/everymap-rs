//! Shared helpers for the MapQuest contract tests.
//!
//! Every test drives the provider against an in-process wiremock server that
//! replays a committed fixture. The API key is an obvious placeholder: it is
//! present on the wire so the authentication path is exercised, and no test
//! asserts that it appears in an error message.

#![allow(dead_code)]

use everymap_providers_mapquest::{MapQuestClient, MapQuestProviderConfiguration};
use std::sync::Arc;
use wiremock::MockServer;

/// Obvious placeholder key used by every contract test.
pub const TEST_API_KEY: &str = "test-placeholder-api-key";

/// User agent every contract test configures and matches on the wire.
pub const TEST_USER_AGENT: &str = "everymap-mapquest-provider-tests";

/// Builds a client pointed at the in-process server.
pub fn client_for(server: &MockServer) -> Arc<MapQuestClient> {
    Arc::new(MapQuestClient::new(MapQuestProviderConfiguration {
        base_url: server.uri(),
        api_key: TEST_API_KEY.to_string(),
        user_agent: TEST_USER_AGENT.to_string(),
    }))
}

/// Reads and decodes a committed JSON fixture.
pub fn fixture(name: &str) -> serde_json::Value {
    let fixture_path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(
        &std::fs::read_to_string(fixture_path).expect("the fixture must be readable"),
    )
    .expect("the fixture must be valid JSON")
}

/// Reads a committed binary fixture.
pub fn fixture_bytes(name: &str) -> Vec<u8> {
    let fixture_path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(fixture_path).expect("the fixture must be readable")
}
