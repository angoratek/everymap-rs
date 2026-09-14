use serde::{Deserialize, Serialize};
use std::time::Instant;

use everymap_core::auth::AuthProvider;
use everymap_core::domains::attributes::{AttributeOptions, AttributeProvider};
use everymap_core::domains::imaging::{ImageOptions, MapImageProvider};
use everymap_core::domains::isoline::{IsolineOptions, IsolineProvider, RangeType};
use everymap_core::domains::matching::{MatchingOptions, RouteMatcher};
use everymap_core::domains::positioning::{NetworkPositioner, PositioningOptions};
use everymap_core::domains::routing::{RouteOptions, Router, TransportMode};
use everymap_core::domains::search::{GeocodeOptions, Geocoder, ReverseGeocodeOptions};
use everymap_core::domains::tiling::{TileOptions, TileProvider};
use everymap_core::domains::tour::{TourOptions, TourPlanner};
use everymap_core::domains::traffic::{TrafficOptions, TrafficProvider};
use everymap_core::error::{EveryMapError, EveryMapResult};
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

/// Per-provider percentile statistics (computed when iterations > 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPercentiles {
    pub provider: String,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
    pub min_ms: u64,
    pub max_ms: u64,
    pub success_count: u32,
    pub error_count: u32,
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
    pub percentiles: Option<Vec<ProviderPercentiles>>,
}

/// Container for all domain trait objects for a single provider.
/// All fields use Option<> — None means the provider does not support this domain.
pub struct BenchProviders {
    pub provider_name: String,
    pub geocoder: Option<Box<dyn Geocoder>>,
    pub router: Option<Box<dyn Router>>,
    pub isoline: Option<Box<dyn IsolineProvider>>,
    pub route_matcher: Option<Box<dyn RouteMatcher>>,
    pub tour_planner: Option<Box<dyn TourPlanner>>,
    pub traffic: Option<Box<dyn TrafficProvider>>,
    pub tile: Option<Box<dyn TileProvider>>,
    pub positioner: Option<Box<dyn NetworkPositioner>>,
    pub attributes: Option<Box<dyn AttributeProvider>>,
    pub image: Option<Box<dyn MapImageProvider>>,
}

fn unsupported_result(provider: &str, domain: &str, scenario: &str) -> BenchmarkResult {
    BenchmarkResult {
        provider: provider.to_string(),
        domain: domain.to_string(),
        scenario: scenario.to_string(),
        duration_ms: 0,
        success: false,
        error: Some("Not supported by this provider".to_string()),
        result_count: 0,
        raw_response_size: None,
    }
}

/// Compute the approximate size of a raw JSON response value.
fn raw_size(raw: &Option<serde_json::Value>) -> Option<usize> {
    raw.as_ref().map(|v| v.to_string().len())
}

impl BenchProviders {
    pub fn new(provider: &str, auth: Arc<dyn AuthProvider>) -> EveryMapResult<Self> {
        match provider {
            "here" => {
                let client = Arc::new(everymap_providers_here::client::HereClient::new(auth));
                Ok(Self {
                    provider_name: provider.to_string(),
                    geocoder: Some(Box::new(
                        everymap_providers_here::domain::search::HereGeocoder::new(client.clone()),
                    )),
                    router: Some(Box::new(
                        everymap_providers_here::domain::routing::HereRouter::new(client.clone()),
                    )),
                    isoline: Some(Box::new(
                        everymap_providers_here::domain::isoline::HereIsoline::new(client.clone()),
                    )),
                    route_matcher: Some(Box::new(
                        everymap_providers_here::domain::matching::HereRouteMatcher::new(
                            client.clone(),
                        ),
                    )),
                    tour_planner: Some(Box::new(
                        everymap_providers_here::domain::tour::HereTourPlanner::new(client.clone()),
                    )),
                    traffic: Some(Box::new(
                        everymap_providers_here::domain::traffic::HereTraffic::new(client.clone()),
                    )),
                    tile: Some(Box::new(
                        everymap_providers_here::domain::tiling::HereTileProvider::new(
                            client.clone(),
                        ),
                    )),
                    positioner: Some(Box::new(
                        everymap_providers_here::domain::positioning::HerePositioner::new(
                            client.clone(),
                        ),
                    )),
                    attributes: Some(Box::new(
                        everymap_providers_here::domain::attributes::HereAttributeProvider::new(
                            client.clone(),
                        ),
                    )),
                    image: Some(Box::new(
                        everymap_providers_here::domain::imaging::HereMapImageProvider::new(client),
                    )),
                })
            }
            "google" => {
                let client = Arc::new(everymap_providers_google::client::GoogleClient::new(auth));
                Ok(Self {
                    provider_name: provider.to_string(),
                    geocoder: Some(Box::new(everymap_providers_google::GoogleGeocoder::new(
                        client.clone(),
                    ))),
                    router: Some(Box::new(everymap_providers_google::GoogleRouter::new(
                        client.clone(),
                    ))),
                    isoline: None,
                    route_matcher: Some(Box::new(
                        everymap_providers_google::GoogleRouteMatcher::new(client.clone()),
                    )),
                    tour_planner: None,
                    traffic: None,
                    tile: None,
                    positioner: Some(Box::new(everymap_providers_google::GooglePositioner::new(
                        client.clone(),
                    ))),
                    attributes: Some(Box::new(
                        everymap_providers_google::GoogleAttributeProvider::new(client.clone()),
                    )),
                    image: Some(Box::new(
                        everymap_providers_google::GoogleMapImageProvider::new(client),
                    )),
                })
            }
            "tomtom" => {
                let client = Arc::new(everymap_providers_tomtom::client::TomTomClient::new(auth));
                Ok(Self {
                    provider_name: provider.to_string(),
                    geocoder: Some(Box::new(everymap_providers_tomtom::TomTomGeocoder::new(
                        client.clone(),
                    ))),
                    router: Some(Box::new(everymap_providers_tomtom::TomTomRouter::new(
                        client.clone(),
                    ))),
                    isoline: Some(Box::new(everymap_providers_tomtom::TomTomIsoline::new(
                        client.clone(),
                    ))),
                    route_matcher: Some(Box::new(
                        everymap_providers_tomtom::TomTomRouteMatcher::new(client.clone()),
                    )),
                    tour_planner: Some(Box::new(
                        everymap_providers_tomtom::TomTomTourPlanner::new(client.clone()),
                    )),
                    traffic: Some(Box::new(everymap_providers_tomtom::TomTomTraffic::new(
                        client.clone(),
                    ))),
                    tile: Some(Box::new(
                        everymap_providers_tomtom::TomTomTileProvider::new(client.clone()),
                    )),
                    positioner: None,
                    attributes: None,
                    image: Some(Box::new(
                        everymap_providers_tomtom::TomTomMapImageProvider::new(client),
                    )),
                })
            }
            "mapbox" => {
                let client = Arc::new(everymap_providers_mapbox::client::MapBoxClient::new(auth));
                Ok(Self {
                    provider_name: provider.to_string(),
                    geocoder: Some(Box::new(everymap_providers_mapbox::MapBoxGeocoder::new(
                        client.clone(),
                    ))),
                    router: Some(Box::new(everymap_providers_mapbox::MapBoxRouter::new(
                        client.clone(),
                    ))),
                    isoline: Some(Box::new(everymap_providers_mapbox::MapBoxIsoline::new(
                        client.clone(),
                    ))),
                    route_matcher: Some(Box::new(
                        everymap_providers_mapbox::MapBoxRouteMatcher::new(client.clone()),
                    )),
                    tour_planner: Some(Box::new(
                        everymap_providers_mapbox::MapBoxTourPlanner::new(client.clone()),
                    )),
                    traffic: None,
                    tile: Some(Box::new(
                        everymap_providers_mapbox::MapBoxTileProvider::new(client.clone()),
                    )),
                    positioner: None,
                    attributes: None,
                    image: Some(Box::new(
                        everymap_providers_mapbox::MapBoxMapImageProvider::new(client),
                    )),
                })
            }
            "radar" => {
                let client = Arc::new(everymap_providers_radar::client::RadarClient::new(auth));
                Ok(Self {
                    provider_name: provider.to_string(),
                    geocoder: Some(Box::new(everymap_providers_radar::RadarGeocoder::new(
                        client.clone(),
                    ))),
                    router: Some(Box::new(everymap_providers_radar::RadarRouter::new(
                        client.clone(),
                    ))),
                    isoline: None,
                    route_matcher: Some(Box::new(
                        everymap_providers_radar::RadarRouteMatcher::new(client.clone()),
                    )),
                    tour_planner: Some(Box::new(everymap_providers_radar::RadarTourPlanner::new(
                        client.clone(),
                    ))),
                    traffic: None,
                    tile: None,
                    positioner: None,
                    attributes: None,
                    image: None,
                })
            }
            _ => Err(EveryMapError::provider(
                "bench",
                "UNKNOWN_PROVIDER",
                format!("Unknown provider: {}", provider),
            )),
        }
    }
}

// --- Domain-specific benchmark functions ---

pub async fn bench_geocode(providers: &BenchProviders, query: &str) -> BenchmarkResult {
    let geocoder = match providers.geocoder.as_ref() {
        Some(g) => g,
        None => {
            return unsupported_result(
                &providers.provider_name,
                "geocoder",
                &format!("Geocode: {}", query),
            )
        }
    };
    let start_time = Instant::now();
    let result = geocoder.geocode(query, &GeocodeOptions::default()).await;
    let elapsed = start_time.elapsed().as_millis() as u64;
    match result {
        Ok(response) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "geocoder".to_string(),
            scenario: format!("Geocode: {}", query),
            duration_ms: elapsed,
            success: true,
            error: None,
            result_count: response.items.len(),
            raw_response_size: response
                .items
                .first()
                .and_then(|i| i.raw.as_ref())
                .map(|v| v.to_string().len()),
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "geocoder".to_string(),
            scenario: format!("Geocode: {}", query),
            duration_ms: elapsed,
            success: false,
            error: Some(e.to_string()),
            result_count: 0,
            raw_response_size: None,
        },
    }
}

pub async fn bench_reverse_geocode(
    providers: &BenchProviders,
    coordinate: &Coordinate,
) -> BenchmarkResult {
    let geocoder = match providers.geocoder.as_ref() {
        Some(g) => g,
        None => {
            return unsupported_result(
                &providers.provider_name,
                "geocoder",
                &format!("Reverse geocode: ({},{})", coordinate.lat, coordinate.lng),
            )
        }
    };
    let start_time = Instant::now();
    let result = geocoder
        .reverse_geocode(coordinate, &ReverseGeocodeOptions::default())
        .await;
    let elapsed = start_time.elapsed().as_millis() as u64;
    match result {
        Ok(response) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "geocoder".to_string(),
            scenario: format!("Reverse geocode: ({},{})", coordinate.lat, coordinate.lng),
            duration_ms: elapsed,
            success: true,
            error: None,
            result_count: response.items.len(),
            raw_response_size: response
                .items
                .first()
                .and_then(|i| i.raw.as_ref())
                .map(|v| v.to_string().len()),
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "geocoder".to_string(),
            scenario: format!("Reverse geocode: ({},{})", coordinate.lat, coordinate.lng),
            duration_ms: elapsed,
            success: false,
            error: Some(e.to_string()),
            result_count: 0,
            raw_response_size: None,
        },
    }
}

pub async fn bench_route(
    providers: &BenchProviders,
    start: &Coordinate,
    end: &Coordinate,
) -> BenchmarkResult {
    let router = match providers.router.as_ref() {
        Some(r) => r,
        None => {
            return unsupported_result(
                &providers.provider_name,
                "routing",
                &format!(
                    "Route: ({},{}) -> ({},{})",
                    start.lat, start.lng, end.lat, end.lng
                ),
            )
        }
    };
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        ..Default::default()
    };
    let start_time = Instant::now();
    let result = router.calculate_route(start, end, &options).await;
    let elapsed = start_time.elapsed().as_millis() as u64;
    match result {
        Ok(response) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "routing".to_string(),
            scenario: format!(
                "Route: ({},{}) -> ({},{})",
                start.lat, start.lng, end.lat, end.lng
            ),
            duration_ms: elapsed,
            success: true,
            error: None,
            result_count: response.routes.len(),
            raw_response_size: response
                .routes
                .first()
                .and_then(|r| r.raw.as_ref())
                .map(|v| v.to_string().len()),
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "routing".to_string(),
            scenario: format!(
                "Route: ({},{}) -> ({},{})",
                start.lat, start.lng, end.lat, end.lng
            ),
            duration_ms: elapsed,
            success: false,
            error: Some(e.to_string()),
            result_count: 0,
            raw_response_size: None,
        },
    }
}

pub async fn bench_isoline(
    providers: &BenchProviders,
    center: &Coordinate,
    range: f64,
) -> BenchmarkResult {
    let isoline = match providers.isoline.as_ref() {
        Some(i) => i,
        None => {
            return unsupported_result(
                &providers.provider_name,
                "isoline",
                &format!(
                    "Isoline: {}m from ({},{})",
                    range as i64, center.lat, center.lng
                ),
            )
        }
    };
    let options = IsolineOptions {
        range_type: Some(RangeType::Time),
        ..Default::default()
    };
    let start_time = Instant::now();
    let result = isoline.get_isoline(center, range, &options).await;
    let elapsed = start_time.elapsed().as_millis() as u64;
    match result {
        Ok(response) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "isoline".to_string(),
            scenario: format!(
                "Isoline: {}m from ({},{})",
                range as i64, center.lat, center.lng
            ),
            duration_ms: elapsed,
            success: true,
            error: None,
            result_count: response.isolines.len(),
            raw_response_size: raw_size(&response.raw),
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "isoline".to_string(),
            scenario: format!(
                "Isoline: {}m from ({},{})",
                range as i64, center.lat, center.lng
            ),
            duration_ms: elapsed,
            success: false,
            error: Some(e.to_string()),
            result_count: 0,
            raw_response_size: None,
        },
    }
}

pub async fn bench_matching(providers: &BenchProviders, points: &[Coordinate]) -> BenchmarkResult {
    let matcher = match providers.route_matcher.as_ref() {
        Some(m) => m,
        None => {
            return unsupported_result(
                &providers.provider_name,
                "matching",
                &format!("Match: {} points", points.len()),
            )
        }
    };
    let options = MatchingOptions::default();
    let start_time = Instant::now();
    let result = matcher.match_route(points, &options).await;
    let elapsed = start_time.elapsed().as_millis() as u64;
    match result {
        Ok(response) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "matching".to_string(),
            scenario: format!("Match: {} points", points.len()),
            duration_ms: elapsed,
            success: true,
            error: None,
            result_count: response.matched_points.len(),
            raw_response_size: raw_size(&response.raw),
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "matching".to_string(),
            scenario: format!("Match: {} points", points.len()),
            duration_ms: elapsed,
            success: false,
            error: Some(e.to_string()),
            result_count: 0,
            raw_response_size: None,
        },
    }
}

pub async fn bench_tour(providers: &BenchProviders, stops: &[Coordinate]) -> BenchmarkResult {
    let planner = match providers.tour_planner.as_ref() {
        Some(p) => p,
        None => {
            return unsupported_result(
                &providers.provider_name,
                "tour",
                &format!("Tour: {} stops", stops.len()),
            )
        }
    };
    let options = TourOptions {
        transport_mode: Some(TransportMode::Car),
        ..Default::default()
    };
    let start_time = Instant::now();
    let result = planner.optimize_tour(stops, &options).await;
    let elapsed = start_time.elapsed().as_millis() as u64;
    match result {
        Ok(response) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "tour".to_string(),
            scenario: format!("Tour: {} stops", stops.len()),
            duration_ms: elapsed,
            success: true,
            error: None,
            result_count: response.stops.len(),
            raw_response_size: raw_size(&response.raw),
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "tour".to_string(),
            scenario: format!("Tour: {} stops", stops.len()),
            duration_ms: elapsed,
            success: false,
            error: Some(e.to_string()),
            result_count: 0,
            raw_response_size: None,
        },
    }
}

pub async fn bench_traffic(providers: &BenchProviders, location: &Coordinate) -> BenchmarkResult {
    let traffic = match providers.traffic.as_ref() {
        Some(t) => t,
        None => {
            return unsupported_result(
                &providers.provider_name,
                "traffic",
                &format!("Traffic: ({},{})", location.lat, location.lng),
            )
        }
    };
    let options = TrafficOptions::default();
    let start_time = Instant::now();
    let result = traffic.get_traffic(location, &options).await;
    let elapsed = start_time.elapsed().as_millis() as u64;
    match result {
        Ok(response) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "traffic".to_string(),
            scenario: format!("Traffic: ({},{})", location.lat, location.lng),
            duration_ms: elapsed,
            success: true,
            error: None,
            result_count: response.flows.len(),
            raw_response_size: raw_size(&response.raw),
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "traffic".to_string(),
            scenario: format!("Traffic: ({},{})", location.lat, location.lng),
            duration_ms: elapsed,
            success: false,
            error: Some(e.to_string()),
            result_count: 0,
            raw_response_size: None,
        },
    }
}

pub async fn bench_tile(providers: &BenchProviders, z: u32, x: u32, y: u32) -> BenchmarkResult {
    let tile = match providers.tile.as_ref() {
        Some(t) => t,
        None => {
            return unsupported_result(
                &providers.provider_name,
                "tiling",
                &format!("Tile: z={} x={} y={}", z, x, y),
            )
        }
    };
    let options = TileOptions::default();
    let start_time = Instant::now();
    let result = tile.get_tile(z, x, y, &options).await;
    let elapsed = start_time.elapsed().as_millis() as u64;
    match result {
        Ok(response) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "tiling".to_string(),
            scenario: format!("Tile: z={} x={} y={}", z, x, y),
            duration_ms: elapsed,
            success: true,
            error: None,
            result_count: 1,
            raw_response_size: Some(response.data.len()),
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "tiling".to_string(),
            scenario: format!("Tile: z={} x={} y={}", z, x, y),
            duration_ms: elapsed,
            success: false,
            error: Some(e.to_string()),
            result_count: 0,
            raw_response_size: None,
        },
    }
}

pub async fn bench_positioning(
    providers: &BenchProviders,
    provider_extra: &Option<serde_json::Value>,
) -> BenchmarkResult {
    let positioner = match providers.positioner.as_ref() {
        Some(p) => p,
        None => {
            return unsupported_result(
                &providers.provider_name,
                "positioning",
                "Positioning: default",
            )
        }
    };
    let options = PositioningOptions {
        provider_extra: provider_extra.clone(),
    };
    let start_time = Instant::now();
    let result = positioner.get_position(&options).await;
    let elapsed = start_time.elapsed().as_millis() as u64;
    match result {
        Ok(response) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "positioning".to_string(),
            scenario: "Positioning: default".to_string(),
            duration_ms: elapsed,
            success: true,
            error: None,
            result_count: 1,
            raw_response_size: raw_size(&response.raw),
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "positioning".to_string(),
            scenario: "Positioning: default".to_string(),
            duration_ms: elapsed,
            success: false,
            error: Some(e.to_string()),
            result_count: 0,
            raw_response_size: None,
        },
    }
}

pub async fn bench_attributes(
    providers: &BenchProviders,
    bbox: &str,
    provider_extra: &Option<serde_json::Value>,
) -> BenchmarkResult {
    let attrs = match providers.attributes.as_ref() {
        Some(a) => a,
        None => {
            return unsupported_result(
                &providers.provider_name,
                "attributes",
                &format!("Attributes: bbox={}", bbox),
            )
        }
    };
    let options = AttributeOptions {
        bbox: Some(bbox.to_string()),
        provider_extra: provider_extra.clone(),
        ..Default::default()
    };
    let start_time = Instant::now();
    let result = attrs.get_attributes(&options).await;
    let elapsed = start_time.elapsed().as_millis() as u64;
    match result {
        Ok(response) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "attributes".to_string(),
            scenario: format!("Attributes: bbox={}", bbox),
            duration_ms: elapsed,
            success: true,
            error: None,
            result_count: 1,
            raw_response_size: Some(response.data.to_string().len()),
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "attributes".to_string(),
            scenario: format!("Attributes: bbox={}", bbox),
            duration_ms: elapsed,
            success: false,
            error: Some(e.to_string()),
            result_count: 0,
            raw_response_size: None,
        },
    }
}

pub async fn bench_image(
    providers: &BenchProviders,
    center: &Coordinate,
    zoom: u32,
) -> BenchmarkResult {
    let img = match providers.image.as_ref() {
        Some(i) => i,
        None => {
            return unsupported_result(
                &providers.provider_name,
                "imaging",
                &format!("Image: ({},{}) z={}", center.lat, center.lng, zoom),
            )
        }
    };
    let options = ImageOptions::default();
    let start_time = Instant::now();
    let result = img.get_image(center, zoom, (800, 600), &options).await;
    let elapsed = start_time.elapsed().as_millis() as u64;
    match result {
        Ok(response) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "imaging".to_string(),
            scenario: format!("Image: ({},{}) z={}", center.lat, center.lng, zoom),
            duration_ms: elapsed,
            success: true,
            error: None,
            result_count: 1,
            raw_response_size: Some(response.data.len()),
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(),
            domain: "imaging".to_string(),
            scenario: format!("Image: ({},{}) z={}", center.lat, center.lng, zoom),
            duration_ms: elapsed,
            success: false,
            error: Some(e.to_string()),
            result_count: 0,
            raw_response_size: None,
        },
    }
}

fn percentile(sorted: &[u64], p: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let index = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}
/// Build a benchmark report from a set of results.
pub fn build_report(
    domain: &str,
    scenario: &str,
    results: Vec<BenchmarkResult>,
) -> BenchmarkReport {
    let fastest = results
        .iter()
        .filter(|r| r.success)
        .min_by_key(|r| r.duration_ms)
        .map(|r| r.provider.clone());

    let most_results = results
        .iter()
        .filter(|r| r.success)
        .max_by_key(|r| r.result_count)
        .map(|r| r.provider.clone());

    // Compute percentiles when we have multiple iterations per provider
    let mut provider_durations: std::collections::HashMap<String, Vec<u64>> =
        std::collections::HashMap::new();
    let mut provider_success: std::collections::HashMap<String, (u32, u32)> =
        std::collections::HashMap::new();
    for r in &results {
        provider_durations.entry(r.provider.clone()).or_default();
        provider_success.entry(r.provider.clone()).or_insert((0, 0));
        let (success_count, error_count) = provider_success.get_mut(&r.provider).unwrap();
        if r.success {
            provider_durations
                .get_mut(&r.provider)
                .unwrap()
                .push(r.duration_ms);
            *success_count += 1;
        } else {
            *error_count += 1;
        }
    }

    let percentiles = if provider_durations.values().any(|v| v.len() > 1) {
        let mut pcts: Vec<ProviderPercentiles> = Vec::new();
        let mut providers: Vec<String> = provider_durations.keys().cloned().collect();
        providers.sort();
        for provider in &providers {
            let mut durations = provider_durations[provider].clone();
            durations.sort();
            let (success_count, error_count) = provider_success[provider];
            pcts.push(ProviderPercentiles {
                provider: provider.clone(),
                p50_ms: percentile(&durations, 50.0),
                p95_ms: percentile(&durations, 95.0),
                p99_ms: percentile(&durations, 99.0),
                min_ms: durations.first().copied().unwrap_or(0),
                max_ms: durations.last().copied().unwrap_or(0),
                success_count,
                error_count,
            });
        }
        Some(pcts)
    } else {
        None
    };

    BenchmarkReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        domain: domain.to_string(),
        scenario: scenario.to_string(),
        results,
        fastest,
        most_results,
        percentiles,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use everymap_core::auth::ApiKeyProvider;
    use std::sync::Arc;

    fn test_auth() -> Arc<dyn AuthProvider> {
        Arc::new(ApiKeyProvider::new(
            "test-key".to_string(),
            "key".to_string(),
        ))
    }

    #[test]
    fn test_bench_providers_here() {
        let providers = BenchProviders::new("here", test_auth()).unwrap();
        assert_eq!(providers.provider_name, "here");
        assert!(providers.geocoder.is_some());
        assert!(providers.router.is_some());
        assert!(providers.isoline.is_some());
        assert!(providers.route_matcher.is_some());
        assert!(providers.tour_planner.is_some());
        assert!(providers.traffic.is_some());
        assert!(providers.tile.is_some());
        assert!(providers.positioner.is_some());
        assert!(providers.attributes.is_some());
        assert!(providers.image.is_some());
    }

    #[test]
    fn test_bench_providers_google() {
        let providers = BenchProviders::new("google", test_auth()).unwrap();
        assert_eq!(providers.provider_name, "google");
        assert!(providers.geocoder.is_some());
        assert!(providers.router.is_some());
        assert!(providers.isoline.is_none());
        assert!(providers.route_matcher.is_some());
        assert!(providers.tour_planner.is_none());
        assert!(providers.traffic.is_none());
        assert!(providers.tile.is_none());
        assert!(providers.positioner.is_some());
        assert!(providers.attributes.is_some());
        assert!(providers.image.is_some());
    }

    #[test]
    fn test_bench_providers_tomtom() {
        let providers = BenchProviders::new("tomtom", test_auth()).unwrap();
        assert_eq!(providers.provider_name, "tomtom");
        assert!(providers.geocoder.is_some());
        assert!(providers.router.is_some());
        assert!(providers.isoline.is_some());
        assert!(providers.route_matcher.is_some());
        assert!(providers.tour_planner.is_some());
        assert!(providers.traffic.is_some());
        assert!(providers.tile.is_some());
        assert!(providers.positioner.is_none());
        assert!(providers.attributes.is_none());
        assert!(providers.image.is_some());
    }

    #[test]
    fn test_bench_providers_unknown() {
        let result = BenchProviders::new("unknown", test_auth());
        assert!(result.is_err());
    }

    #[test]
    fn test_percentile_empty() {
        assert_eq!(percentile(&[], 50.0), 0);
    }

    #[test]
    fn test_percentile_single() {
        assert_eq!(percentile(&[100], 50.0), 100);
        assert_eq!(percentile(&[100], 95.0), 100);
    }

    #[test]
    fn test_percentile_multiple() {
        let values: Vec<u64> = (0..100).collect();
        assert_eq!(percentile(&values, 50.0), 50);
        assert_eq!(percentile(&values, 95.0), 94);
        assert_eq!(percentile(&values, 0.0), 0);
        assert_eq!(percentile(&values, 100.0), 99);
    }

    #[test]
    fn test_build_report_single_provider() {
        let results = vec![BenchmarkResult {
            provider: "here".to_string(),
            domain: "routing".to_string(),
            scenario: "Berlin to Paris".to_string(),
            duration_ms: 1234,
            success: true,
            error: None,
            result_count: 1,
            raw_response_size: Some(5000),
        }];
        let report = build_report("routing", "Berlin to Paris", results);
        assert_eq!(report.domain, "routing");
        assert_eq!(report.scenario, "Berlin to Paris");
        assert_eq!(report.results.len(), 1);
        assert_eq!(report.fastest.as_deref(), Some("here"));
        assert_eq!(report.most_results.as_deref(), Some("here"));
        assert!(report.percentiles.is_none());
    }

    #[test]
    fn test_build_report_with_failure() {
        let results = vec![
            BenchmarkResult {
                provider: "here".to_string(),
                domain: "routing".to_string(),
                scenario: "Berlin to Paris".to_string(),
                duration_ms: 100,
                success: true,
                error: None,
                result_count: 1,
                raw_response_size: Some(5000),
            },
            BenchmarkResult {
                provider: "google".to_string(),
                domain: "routing".to_string(),
                scenario: "Berlin to Paris".to_string(),
                duration_ms: 0,
                success: false,
                error: Some("API error".to_string()),
                result_count: 0,
                raw_response_size: None,
            },
        ];
        let report = build_report("routing", "Berlin to Paris", results);
        assert_eq!(report.results.len(), 2);
        assert_eq!(report.fastest.as_deref(), Some("here"));
        assert_eq!(report.most_results.as_deref(), Some("here"));
    }
}
