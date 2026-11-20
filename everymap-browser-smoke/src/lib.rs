//! One request per provider through the browser's `fetch` transport.
//!
//! The module is built for `wasm32-unknown-unknown`, loaded by
//! `scripts/browser-smoke.mjs` in a headless Chromium page, and served by the
//! same origin as the fixture responses. Each provider is pointed at its
//! fixture root, one domain call is made, and the outcome is classified:
//! `transport_error` means the browser never received a response, while any
//! other non-success outcome means the request reached the fixture server.
//!
//! The clock check reads `web-time`, the same clock the shared client's rate
//! limiter uses.

#![cfg(target_arch = "wasm32")]

use everymap_core::auth::header::HeaderAuthProvider;
use everymap_core::auth::{ApiKeyProvider, AuthProvider};
use everymap_core::domains::routing::Router;
use everymap_core::domains::search::{GeocodeOptions, Geocoder};
use everymap_core::domains::tiling::{
    RasterImageFormat, RasterTileProvider, RasterTileRequest, RasterTileResolution,
    VectorTileEncoding, VectorTileProvider, VectorTileSourceDescriptor,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Coordinate;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

/// Seconds since the Unix epoch before which a browser clock is implausible.
const IMPLAUSIBLE_CLOCK_EPOCH_SECONDS: u64 = 1_600_000_000;

/// Classifies a provider call: only a transport failure proves the browser
/// never received a response.
fn outcome<T>(result: EveryMapResult<T>) -> &'static str {
    match result {
        Ok(_) => "ok",
        Err(EveryMapError::TransportError(_)) => "transport_error",
        Err(_) => "provider_response",
    }
}

fn api_key_auth(parameter: &str) -> Arc<dyn AuthProvider> {
    Arc::new(ApiKeyProvider::new(
        "browser-smoke".to_string(),
        parameter.to_string(),
    ))
}

async fn here(base_url: &str) -> &'static str {
    let client = Arc::new(everymap_providers_here::HereClient::new(api_key_auth(
        "apiKey",
    )));
    let geocoder = everymap_providers_here::HereGeocoder::with_base_url(
        client,
        format!("{base_url}/fixtures/here"),
    );
    outcome(geocoder.geocode("Berlin", &GeocodeOptions::default()).await)
}

async fn google(base_url: &str) -> &'static str {
    let client = Arc::new(everymap_providers_google::GoogleClient::new(api_key_auth(
        "key",
    )));
    let geocoder = everymap_providers_google::GoogleGeocoder::with_base_url(
        client,
        format!("{base_url}/fixtures/google"),
    );
    outcome(geocoder.geocode("Berlin", &GeocodeOptions::default()).await)
}

async fn tomtom(base_url: &str) -> &'static str {
    let client = Arc::new(everymap_providers_tomtom::TomTomClient::new(api_key_auth(
        "key",
    )));
    let geocoder = everymap_providers_tomtom::TomTomGeocoder::with_base_url(
        client,
        format!("{base_url}/fixtures/tomtom"),
    );
    outcome(geocoder.geocode("Berlin", &GeocodeOptions::default()).await)
}

async fn mapbox(base_url: &str) -> &'static str {
    let client = Arc::new(everymap_providers_mapbox::MapBoxClient::new(api_key_auth(
        "access_token",
    )));
    let geocoder = everymap_providers_mapbox::MapBoxGeocoder::with_base_url(
        client,
        format!("{base_url}/fixtures/mapbox"),
    );
    outcome(geocoder.geocode("Berlin", &GeocodeOptions::default()).await)
}

async fn radar(base_url: &str) -> &'static str {
    let auth: Arc<dyn AuthProvider> =
        Arc::new(HeaderAuthProvider::new("browser-smoke".to_string()));
    let client = Arc::new(everymap_providers_radar::RadarClient::new(auth));
    let geocoder = everymap_providers_radar::RadarGeocoder::with_base_url(
        client,
        format!("{base_url}/fixtures/radar"),
    );
    outcome(geocoder.geocode("Berlin", &GeocodeOptions::default()).await)
}

async fn mapquest(base_url: &str) -> &'static str {
    let client = Arc::new(everymap_providers_mapquest::MapQuestClient::new(
        everymap_providers_mapquest::MapQuestProviderConfiguration {
            base_url: format!("{base_url}/fixtures/mapquest"),
            api_key: "browser-smoke".to_string(),
            ..Default::default()
        },
    ));
    let geocoder = everymap_providers_mapquest::MapQuestGeocoder::new(client);
    outcome(geocoder.geocode("Berlin", &GeocodeOptions::default()).await)
}

async fn photon(base_url: &str) -> &'static str {
    let client = Arc::new(everymap_providers_photon::PhotonClient::new(
        everymap_providers_photon::PhotonProviderConfiguration {
            base_url: format!("{base_url}/fixtures/photon"),
            ..Default::default()
        },
    ));
    let geocoder = everymap_providers_photon::PhotonGeocoder::new(client);
    outcome(geocoder.geocode("Berlin", &GeocodeOptions::default()).await)
}

async fn osrm(base_url: &str) -> &'static str {
    let router = everymap_providers_osrm::OsrmRouter::new(
        everymap_providers_osrm::OsrmProviderConfiguration {
            base_url: format!("{base_url}/fixtures/osrm"),
            ..Default::default()
        },
    );
    let start = Coordinate::new(52.5, 13.4).expect("a valid coordinate");
    let end = Coordinate::new(48.8, 2.3).expect("a valid coordinate");
    let options = everymap_core::domains::routing::RouteOptions::default();
    outcome(router.calculate_route(&start, &end, &options).await)
}

async fn valhalla(base_url: &str) -> &'static str {
    let router = everymap_providers_valhalla::ValhallaRouter::new(
        everymap_providers_valhalla::ValhallaProviderConfiguration {
            base_url: format!("{base_url}/fixtures/valhalla"),
            ..Default::default()
        },
    );
    let start = Coordinate::new(52.5, 13.4).expect("a valid coordinate");
    let end = Coordinate::new(48.8, 2.3).expect("a valid coordinate");
    let options = everymap_core::domains::routing::RouteOptions::default();
    outcome(router.calculate_route(&start, &end, &options).await)
}

async fn raster_tiles(base_url: &str) -> &'static str {
    let source = everymap_providers_url_template_raster::UrlTemplateRasterTileSource {
        url_template: format!("{base_url}/fixtures/tiles/raster/{{z}}/{{x}}/{{y}}.png"),
        source_name: "browser-smoke/raster".to_string(),
        attribution_text: String::new(),
        terms_text: String::new(),
        minimum_zoom_level: 0,
        maximum_zoom_level: 18,
        image_format: RasterImageFormat::Png,
        tile_resolution: RasterTileResolution::Standard,
        subdomains: Vec::new(),
        headers: Vec::new(),
        pacing: None,
    };
    let provider =
        match everymap_providers_url_template_raster::UrlTemplateRasterTileProvider::new(source) {
            Ok(provider) => provider,
            Err(error) => return outcome::<()>(Err(error)),
        };
    let request = RasterTileRequest::new(
        1,
        2,
        3,
        RasterImageFormat::Png,
        RasterTileResolution::Standard,
    );
    outcome(provider.fetch_raster_tile(&request).await)
}

async fn vector_tiles(base_url: &str) -> &'static str {
    let source = everymap_providers_url_template_vector::UrlTemplateVectorTileSource {
        url_template: format!("{base_url}/fixtures/tiles/vector/{{z}}/{{x}}/{{y}}.mvt"),
        source_descriptor: VectorTileSourceDescriptor {
            source_name: "browser-smoke/vector".to_string(),
            attribution_text: String::new(),
            minimum_zoom_level: 0,
            maximum_zoom_level: 18,
            tile_encoding: VectorTileEncoding::MapboxVectorTile,
            style_hints: Vec::new(),
        },
        terms_text: String::new(),
        subdomains: Vec::new(),
        headers: Vec::new(),
        pacing: None,
    };
    let provider =
        match everymap_providers_url_template_vector::UrlTemplateVectorTileProvider::new(source) {
            Ok(provider) => provider,
            Err(error) => return outcome::<()>(Err(error)),
        };
    outcome(provider.fetch_vector_tile(1, 2, 3).await)
}

/// Drives one request per provider and reports the outcomes as JSON.
///
/// # Parameters
/// - `base_url` — the origin the harness page was served from; each provider
///   is rooted at `${base_url}/fixtures/{provider}`.
///
/// # Returns
/// A JSON object with `providers` (an array of `{key, outcome}`) and
/// `clock_ok`.
#[wasm_bindgen]
pub async fn run_smoke(base_url: String) -> Result<String, JsValue> {
    let providers = [
        (
            everymap_providers_here::PROVIDER_DESCRIPTOR.key,
            here(&base_url).await,
        ),
        (
            everymap_providers_google::PROVIDER_DESCRIPTOR.key,
            google(&base_url).await,
        ),
        (
            everymap_providers_tomtom::PROVIDER_DESCRIPTOR.key,
            tomtom(&base_url).await,
        ),
        (
            everymap_providers_mapbox::PROVIDER_DESCRIPTOR.key,
            mapbox(&base_url).await,
        ),
        (
            everymap_providers_radar::PROVIDER_DESCRIPTOR.key,
            radar(&base_url).await,
        ),
        (
            everymap_providers_mapquest::PROVIDER_DESCRIPTOR.key,
            mapquest(&base_url).await,
        ),
        (
            everymap_providers_photon::PROVIDER_DESCRIPTOR.key,
            photon(&base_url).await,
        ),
        (
            everymap_providers_osrm::PROVIDER_DESCRIPTOR.key,
            osrm(&base_url).await,
        ),
        (
            everymap_providers_valhalla::PROVIDER_DESCRIPTOR.key,
            valhalla(&base_url).await,
        ),
        (
            everymap_providers_url_template_raster::PROVIDER_DESCRIPTOR.key,
            raster_tiles(&base_url).await,
        ),
        (
            everymap_providers_url_template_vector::PROVIDER_DESCRIPTOR.key,
            vector_tiles(&base_url).await,
        ),
    ];

    let clock_ok = web_time::SystemTime::now()
        .duration_since(web_time::SystemTime::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() > IMPLAUSIBLE_CLOCK_EPOCH_SECONDS)
        .unwrap_or(false);

    let report = serde_json::json!({
        "providers": providers
            .iter()
            .map(|(key, outcome)| serde_json::json!({ "key": key, "outcome": outcome }))
            .collect::<Vec<_>>(),
        "clock_ok": clock_ok,
    });

    Ok(report.to_string())
}
