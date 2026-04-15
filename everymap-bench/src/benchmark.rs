use serde::{Deserialize, Serialize};
use std::time::Instant;

use everymap_core::auth::AuthProvider;
use everymap_core::domains::search::{Geocoder, GeocodeOptions};
use everymap_core::domains::routing::{Router, RouteOptions, TransportMode};
use everymap_core::types::Coordinate;
use std::sync::Arc;

/// Result of a single benchmark run against one provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub provider: String,
    pub domain: String,
    pub scenario: String,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
    pub result_count: usize,
    pub raw_response_size: Option<usize>,
}

/// Summary analysis across providers for a scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub timestamp: String,
    pub domain: String,
    pub scenario: String,
    pub results: Vec<BenchmarkResult>,
    pub fastest: Option<String>,
    pub most_results: Option<String>,
}

/// Run a geocode benchmark for a provider.
pub async fn bench_geocode(
    provider: &str,
    auth: Arc<dyn AuthProvider>,
    query: &str,
) -> BenchmarkResult {
    let start = Instant::now();
    let result = run_geocode(provider, auth, query).await;
    let elapsed = start.elapsed().as_millis() as u64;

    match result {
        Ok(count) => BenchmarkResult {
            provider: provider.to_string(),
            domain: "geocoder".to_string(),
            scenario: format!("Geocode: {}", query),
            duration_ms: elapsed,
            success: true,
            error: None,
            result_count: count,
            raw_response_size: None,
        },
        Err(e) => BenchmarkResult {
            provider: provider.to_string(),
            domain: "geocoder".to_string(),
            scenario: format!("Geocode: {}", query),
            duration_ms: elapsed,
            success: false,
            error: Some(format!("{}", e)),
            result_count: 0,
            raw_response_size: None,
        },
    }
}

/// Run a routing benchmark for a provider.
pub async fn bench_route(
    provider: &str,
    auth: Arc<dyn AuthProvider>,
    start_coord: &Coordinate,
    end_coord: &Coordinate,
) -> BenchmarkResult {
    let start = Instant::now();
    let result = run_route(provider, auth, start_coord, end_coord).await;
    let elapsed = start.elapsed().as_millis() as u64;

    match result {
        Ok(count) => BenchmarkResult {
            provider: provider.to_string(),
            domain: "routing".to_string(),
            scenario: format!("Route: ({},{}) -> ({},{})", start_coord.lat, start_coord.lng, end_coord.lat, end_coord.lng),
            duration_ms: elapsed,
            success: true,
            error: None,
            result_count: count,
            raw_response_size: None,
        },
        Err(e) => BenchmarkResult {
            provider: provider.to_string(),
            domain: "routing".to_string(),
            scenario: format!("Route: ({},{}) -> ({},{})", start_coord.lat, start_coord.lng, end_coord.lat, end_coord.lng),
            duration_ms: elapsed,
            success: false,
            error: Some(format!("{}", e)),
            result_count: 0,
            raw_response_size: None,
        },
    }
}

// --- Internal dispatch functions ---

async fn run_geocode(provider: &str, auth: Arc<dyn AuthProvider>, query: &str) -> everymap_core::error::EveryMapResult<usize> {
    match provider {
        "here" => {
            let client = Arc::new(everymap_providers_here::client::HereClient::new(auth));
            let geocoder = everymap_providers_here::domain::search::HereGeocoder::new(client);
            let res = geocoder.geocode(query, &GeocodeOptions::default()).await?;
            Ok(res.items.len())
        }
        "google" => {
            let client = Arc::new(everymap_providers_google::client::GoogleClient::new(auth));
            let geocoder = everymap_providers_google::GoogleGeocoder::new(client);
            let res = geocoder.geocode(query, &GeocodeOptions::default()).await?;
            Ok(res.items.len())
        }
        "tomtom" => {
            let client = Arc::new(everymap_providers_tomtom::client::TomTomClient::new(auth));
            let geocoder = everymap_providers_tomtom::TomTomGeocoder::new(client);
            let res = geocoder.geocode(query, &GeocodeOptions::default()).await?;
            Ok(res.items.len())
        }
        "mapbox" => {
            let client = Arc::new(everymap_providers_mapbox::client::MapBoxClient::new(auth));
            let geocoder = everymap_providers_mapbox::MapBoxGeocoder::new(client);
            let res = geocoder.geocode(query, &GeocodeOptions::default()).await?;
            Ok(res.items.len())
        }
        "radar" => {
            let client = Arc::new(everymap_providers_radar::client::RadarClient::new(auth));
            let geocoder = everymap_providers_radar::RadarGeocoder::new(client);
            let res = geocoder.geocode(query, &GeocodeOptions::default()).await?;
            Ok(res.items.len())
        }
        _ => Err(everymap_core::error::EveryMapError::provider(
            "bench", "UNKNOWN_PROVIDER", format!("Unknown provider: {}", provider)
        )),
    }
}

async fn run_route(provider: &str, auth: Arc<dyn AuthProvider>, start: &Coordinate, end: &Coordinate) -> everymap_core::error::EveryMapResult<usize> {
    let opts = RouteOptions { transport_mode: Some(TransportMode::Car), ..Default::default() };
    match provider {
        "here" => {
            let client = Arc::new(everymap_providers_here::client::HereClient::new(auth));
            let router = everymap_providers_here::domain::routing::HereRouter::new(client);
            let res = router.calculate_route(start, end, &opts).await?;
            Ok(res.routes.len())
        }
        "google" => {
            let client = Arc::new(everymap_providers_google::client::GoogleClient::new(auth));
            let router = everymap_providers_google::GoogleRouter::new(client);
            let res = router.calculate_route(start, end, &opts).await?;
            Ok(res.routes.len())
        }
        "tomtom" => {
            let client = Arc::new(everymap_providers_tomtom::client::TomTomClient::new(auth));
            let router = everymap_providers_tomtom::TomTomRouter::new(client);
            let res = router.calculate_route(start, end, &opts).await?;
            Ok(res.routes.len())
        }
        "mapbox" => {
            let client = Arc::new(everymap_providers_mapbox::client::MapBoxClient::new(auth));
            let router = everymap_providers_mapbox::MapBoxRouter::new(client);
            let res = router.calculate_route(start, end, &opts).await?;
            Ok(res.routes.len())
        }
        "radar" => {
            let client = Arc::new(everymap_providers_radar::client::RadarClient::new(auth));
            let router = everymap_providers_radar::RadarRouter::new(client);
            let res = router.calculate_route(start, end, &opts).await?;
            Ok(res.routes.len())
        }
        _ => Err(everymap_core::error::EveryMapError::provider(
            "bench", "UNKNOWN_PROVIDER", format!("Unknown provider: {}", provider)
        )),
    }
}

/// Build a benchmark report from a set of results.
pub fn build_report(domain: &str, scenario: &str, results: Vec<BenchmarkResult>) -> BenchmarkReport {
    let fastest = results.iter()
        .filter(|r| r.success)
        .min_by_key(|r| r.duration_ms)
        .map(|r| r.provider.clone());

    let most_results = results.iter()
        .filter(|r| r.success)
        .max_by_key(|r| r.result_count)
        .map(|r| r.provider.clone());

    BenchmarkReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        domain: domain.to_string(),
        scenario: scenario.to_string(),
        results,
        fastest,
        most_results,
    }
}