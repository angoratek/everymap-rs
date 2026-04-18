use serde::{Deserialize, Serialize};
use std::time::Instant;

use everymap_core::auth::AuthProvider;
use everymap_core::domains::search::{Geocoder, GeocodeOptions, ReverseGeocodeOptions};
use everymap_core::domains::routing::{Router, RouteOptions, TransportMode};
use everymap_core::domains::isoline::{IsolineProvider, IsolineOptions, RangeType};
use everymap_core::domains::matching::{RouteMatcher, MatchingOptions};
use everymap_core::domains::tour::{TourPlanner, TourOptions};
use everymap_core::domains::traffic::{TrafficProvider, TrafficOptions};
use everymap_core::domains::tiling::{TileProvider, TileOptions};
use everymap_core::domains::positioning::{NetworkPositioner, PositioningOptions};
use everymap_core::domains::attributes::{AttributeProvider, AttributeOptions};
use everymap_core::domains::imaging::{MapImageProvider, ImageOptions};
use everymap_core::domains::geofencing::{GeofenceProvider, GeofenceOptions};
use everymap_core::domains::tracking::{TripTracker, TripCreateOptions};
use everymap_core::domains::fraud::{FraudDetector, FraudCheckOptions};
use everymap_core::types::Coordinate;
use everymap_core::error::{EveryMapError, EveryMapResult};
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
    pub geofence: Option<Box<dyn GeofenceProvider>>,
    pub trip_tracker: Option<Box<dyn TripTracker>>,
    pub fraud_detector: Option<Box<dyn FraudDetector>>,
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

impl BenchProviders {
    pub fn new(provider: &str, auth: Arc<dyn AuthProvider>) -> EveryMapResult<Self> {
        match provider {
            "here" => {
                let client = Arc::new(everymap_providers_here::client::HereClient::new(auth));
                Ok(Self {
                    provider_name: provider.to_string(),
                    geocoder: Some(Box::new(everymap_providers_here::domain::search::HereGeocoder::new(client.clone()))),
                    router: Some(Box::new(everymap_providers_here::domain::routing::HereRouter::new(client.clone()))),
                    isoline: Some(Box::new(everymap_providers_here::domain::isoline::HereIsoline::new(client.clone()))),
                    route_matcher: Some(Box::new(everymap_providers_here::domain::matching::HereRouteMatcher::new(client.clone()))),
                    tour_planner: Some(Box::new(everymap_providers_here::domain::tour::HereTourPlanner::new(client.clone()))),
                    traffic: Some(Box::new(everymap_providers_here::domain::traffic::HereTraffic::new(client.clone()))),
                    tile: Some(Box::new(everymap_providers_here::domain::tiling::HereTileProvider::new(client.clone()))),
                    positioner: Some(Box::new(everymap_providers_here::domain::positioning::HerePositioner::new(client.clone()))),
                    attributes: Some(Box::new(everymap_providers_here::domain::attributes::HereAttributeProvider::new(client.clone()))),
                    image: Some(Box::new(everymap_providers_here::domain::imaging::HereMapImageProvider::new(client))),
                    geofence: None,
                    trip_tracker: None,
                    fraud_detector: None,
                })
            }
            "google" => {
                let client = Arc::new(everymap_providers_google::client::GoogleClient::new(auth));
                Ok(Self {
                    provider_name: provider.to_string(),
                    geocoder: Some(Box::new(everymap_providers_google::GoogleGeocoder::new(client.clone()))),
                    router: Some(Box::new(everymap_providers_google::GoogleRouter::new(client.clone()))),
                    isoline: None,
                    route_matcher: Some(Box::new(everymap_providers_google::GoogleRouteMatcher::new(client.clone()))),
                    tour_planner: None,
                    traffic: None,
                    tile: None,
                    positioner: Some(Box::new(everymap_providers_google::GooglePositioner::new(client.clone()))),
                    attributes: Some(Box::new(everymap_providers_google::GoogleAttributeProvider::new(client.clone()))),
                    image: Some(Box::new(everymap_providers_google::GoogleMapImageProvider::new(client))),
                    geofence: None,
                    trip_tracker: None,
                    fraud_detector: None,
                })
            }
            "tomtom" => {
                let client = Arc::new(everymap_providers_tomtom::client::TomTomClient::new(auth));
                Ok(Self {
                    provider_name: provider.to_string(),
                    geocoder: Some(Box::new(everymap_providers_tomtom::TomTomGeocoder::new(client.clone()))),
                    router: Some(Box::new(everymap_providers_tomtom::TomTomRouter::new(client.clone()))),
                    isoline: Some(Box::new(everymap_providers_tomtom::TomTomIsoline::new(client.clone()))),
                    route_matcher: Some(Box::new(everymap_providers_tomtom::TomTomRouteMatcher::new(client.clone()))),
                    tour_planner: Some(Box::new(everymap_providers_tomtom::TomTomTourPlanner::new(client.clone()))),
                    traffic: Some(Box::new(everymap_providers_tomtom::TomTomTraffic::new(client.clone()))),
                    tile: Some(Box::new(everymap_providers_tomtom::TomTomTileProvider::new(client.clone()))),
                    positioner: None,
                    attributes: None,
                    image: Some(Box::new(everymap_providers_tomtom::TomTomMapImageProvider::new(client))),
                    geofence: None,
                    trip_tracker: None,
                    fraud_detector: None,
                })
            }
            "mapbox" => {
                let client = Arc::new(everymap_providers_mapbox::client::MapBoxClient::new(auth));
                Ok(Self {
                    provider_name: provider.to_string(),
                    geocoder: Some(Box::new(everymap_providers_mapbox::MapBoxGeocoder::new(client.clone()))),
                    router: Some(Box::new(everymap_providers_mapbox::MapBoxRouter::new(client.clone()))),
                    isoline: Some(Box::new(everymap_providers_mapbox::MapBoxIsoline::new(client.clone()))),
                    route_matcher: Some(Box::new(everymap_providers_mapbox::MapBoxRouteMatcher::new(client.clone()))),
                    tour_planner: Some(Box::new(everymap_providers_mapbox::MapBoxTourPlanner::new(client.clone()))),
                    traffic: None,
                    tile: Some(Box::new(everymap_providers_mapbox::MapBoxTileProvider::new(client.clone()))),
                    positioner: None,
                    attributes: None,
                    image: Some(Box::new(everymap_providers_mapbox::MapBoxMapImageProvider::new(client))),
                    geofence: None,
                    trip_tracker: None,
                    fraud_detector: None,
                })
            }
            "radar" => {
                let client = Arc::new(everymap_providers_radar::client::RadarClient::new(auth));
                Ok(Self {
                    provider_name: provider.to_string(),
                    geocoder: Some(Box::new(everymap_providers_radar::RadarGeocoder::new(client.clone()))),
                    router: Some(Box::new(everymap_providers_radar::RadarRouter::new(client.clone()))),
                    isoline: None,
                    route_matcher: Some(Box::new(everymap_providers_radar::RadarRouteMatcher::new(client.clone()))),
                    tour_planner: Some(Box::new(everymap_providers_radar::RadarTourPlanner::new(client.clone()))),
                    traffic: None,
                    tile: None,
                    positioner: None,
                    attributes: None,
                    image: None,
                    geofence: Some(Box::new(everymap_providers_radar::RadarGeofenceProvider::new(client.clone()))),
                    trip_tracker: Some(Box::new(everymap_providers_radar::RadarTripTracker::new(client.clone()))),
                    fraud_detector: Some(Box::new(everymap_providers_radar::RadarFraudDetector::new(client))),
                })
            }
            _ => Err(EveryMapError::provider("bench", "UNKNOWN_PROVIDER", format!("Unknown provider: {}", provider))),
        }
    }
}

// --- Domain-specific benchmark functions ---

pub async fn bench_geocode(providers: &BenchProviders, query: &str) -> BenchmarkResult {
    let geocoder = match providers.geocoder.as_ref() {
        Some(g) => g,
        None => return unsupported_result(&providers.provider_name, "geocoder", &format!("Geocode: {}", query)),
    };
    let start = Instant::now();
    let result = geocoder.geocode(query, &GeocodeOptions::default()).await;
    let elapsed = start.elapsed().as_millis() as u64;
    match result {
        Ok(res) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "geocoder".to_string(),
            scenario: format!("Geocode: {}", query), duration_ms: elapsed,
            success: true, error: None, result_count: res.items.len(), raw_response_size: None,
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "geocoder".to_string(),
            scenario: format!("Geocode: {}", query), duration_ms: elapsed,
            success: false, error: Some(e.to_string()), result_count: 0, raw_response_size: None,
        },
    }
}

pub async fn bench_reverse_geocode(providers: &BenchProviders, coord: &Coordinate) -> BenchmarkResult {
    let geocoder = match providers.geocoder.as_ref() {
        Some(g) => g,
        None => return unsupported_result(&providers.provider_name, "geocoder", &format!("Reverse geocode: ({},{})", coord.lat, coord.lng)),
    };
    let start = Instant::now();
    let result = geocoder.reverse_geocode(coord, &ReverseGeocodeOptions::default()).await;
    let elapsed = start.elapsed().as_millis() as u64;
    match result {
        Ok(res) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "geocoder".to_string(),
            scenario: format!("Reverse geocode: ({},{})", coord.lat, coord.lng), duration_ms: elapsed,
            success: true, error: None, result_count: res.items.len(), raw_response_size: None,
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "geocoder".to_string(),
            scenario: format!("Reverse geocode: ({},{})", coord.lat, coord.lng), duration_ms: elapsed,
            success: false, error: Some(e.to_string()), result_count: 0, raw_response_size: None,
        },
    }
}

pub async fn bench_route(providers: &BenchProviders, start: &Coordinate, end: &Coordinate) -> BenchmarkResult {
    let router = match providers.router.as_ref() {
        Some(r) => r,
        None => return unsupported_result(&providers.provider_name, "routing", &format!("Route: ({},{}) -> ({},{})", start.lat, start.lng, end.lat, end.lng)),
    };
    let opts = RouteOptions { transport_mode: Some(TransportMode::Car), ..Default::default() };
    let t = Instant::now();
    let result = router.calculate_route(start, end, &opts).await;
    let elapsed = t.elapsed().as_millis() as u64;
    match result {
        Ok(res) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "routing".to_string(),
            scenario: format!("Route: ({},{}) -> ({},{})", start.lat, start.lng, end.lat, end.lng), duration_ms: elapsed,
            success: true, error: None, result_count: res.routes.len(), raw_response_size: None,
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "routing".to_string(),
            scenario: format!("Route: ({},{}) -> ({},{})", start.lat, start.lng, end.lat, end.lng), duration_ms: elapsed,
            success: false, error: Some(e.to_string()), result_count: 0, raw_response_size: None,
        },
    }
}

pub async fn bench_isoline(providers: &BenchProviders, center: &Coordinate, range: f64) -> BenchmarkResult {
    let isoline = match providers.isoline.as_ref() {
        Some(i) => i,
        None => return unsupported_result(&providers.provider_name, "isoline", &format!("Isoline: {}m from ({},{})", range as i64, center.lat, center.lng)),
    };
    let opts = IsolineOptions { range_type: Some(RangeType::Time), ..Default::default() };
    let t = Instant::now();
    let result = isoline.get_isoline(center, range, &opts).await;
    let elapsed = t.elapsed().as_millis() as u64;
    match result {
        Ok(res) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "isoline".to_string(),
            scenario: format!("Isoline: {}m from ({},{})", range as i64, center.lat, center.lng), duration_ms: elapsed,
            success: true, error: None, result_count: res.isolines.len(), raw_response_size: None,
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "isoline".to_string(),
            scenario: format!("Isoline: {}m from ({},{})", range as i64, center.lat, center.lng), duration_ms: elapsed,
            success: false, error: Some(e.to_string()), result_count: 0, raw_response_size: None,
        },
    }
}

pub async fn bench_matching(providers: &BenchProviders, points: &[Coordinate]) -> BenchmarkResult {
    let matcher = match providers.route_matcher.as_ref() {
        Some(m) => m,
        None => return unsupported_result(&providers.provider_name, "matching", &format!("Match: {} points", points.len())),
    };
    let opts = MatchingOptions::default();
    let t = Instant::now();
    let result = matcher.match_route(points, &opts).await;
    let elapsed = t.elapsed().as_millis() as u64;
    match result {
        Ok(res) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "matching".to_string(),
            scenario: format!("Match: {} points", points.len()), duration_ms: elapsed,
            success: true, error: None, result_count: res.matched_points.len(), raw_response_size: None,
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "matching".to_string(),
            scenario: format!("Match: {} points", points.len()), duration_ms: elapsed,
            success: false, error: Some(e.to_string()), result_count: 0, raw_response_size: None,
        },
    }
}

pub async fn bench_tour(providers: &BenchProviders, stops: &[Coordinate]) -> BenchmarkResult {
    let planner = match providers.tour_planner.as_ref() {
        Some(p) => p,
        None => return unsupported_result(&providers.provider_name, "tour", &format!("Tour: {} stops", stops.len())),
    };
    let opts = TourOptions::default();
    let t = Instant::now();
    let result = planner.optimize_tour(stops, &opts).await;
    let elapsed = t.elapsed().as_millis() as u64;
    match result {
        Ok(res) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "tour".to_string(),
            scenario: format!("Tour: {} stops", stops.len()), duration_ms: elapsed,
            success: true, error: None, result_count: res.stops.len(), raw_response_size: None,
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "tour".to_string(),
            scenario: format!("Tour: {} stops", stops.len()), duration_ms: elapsed,
            success: false, error: Some(e.to_string()), result_count: 0, raw_response_size: None,
        },
    }
}

pub async fn bench_traffic(providers: &BenchProviders, location: &Coordinate) -> BenchmarkResult {
    let traffic = match providers.traffic.as_ref() {
        Some(t) => t,
        None => return unsupported_result(&providers.provider_name, "traffic", &format!("Traffic: ({},{})", location.lat, location.lng)),
    };
    let opts = TrafficOptions::default();
    let t = Instant::now();
    let result = traffic.get_traffic(location, &opts).await;
    let elapsed = t.elapsed().as_millis() as u64;
    match result {
        Ok(res) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "traffic".to_string(),
            scenario: format!("Traffic: ({},{})", location.lat, location.lng), duration_ms: elapsed,
            success: true, error: None, result_count: res.flows.len(), raw_response_size: None,
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "traffic".to_string(),
            scenario: format!("Traffic: ({},{})", location.lat, location.lng), duration_ms: elapsed,
            success: false, error: Some(e.to_string()), result_count: 0, raw_response_size: None,
        },
    }
}

pub async fn bench_tile(providers: &BenchProviders, z: u32, x: u32, y: u32) -> BenchmarkResult {
    let tile = match providers.tile.as_ref() {
        Some(t) => t,
        None => return unsupported_result(&providers.provider_name, "tiling", &format!("Tile: z={} x={} y={}", z, x, y)),
    };
    let opts = TileOptions::default();
    let t = Instant::now();
    let result = tile.get_tile(z, x, y, &opts).await;
    let elapsed = t.elapsed().as_millis() as u64;
    match result {
        Ok(res) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "tiling".to_string(),
            scenario: format!("Tile: z={} x={} y={}", z, x, y), duration_ms: elapsed,
            success: true, error: None, result_count: 1, raw_response_size: Some(res.data.len()),
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "tiling".to_string(),
            scenario: format!("Tile: z={} x={} y={}", z, x, y), duration_ms: elapsed,
            success: false, error: Some(e.to_string()), result_count: 0, raw_response_size: None,
        },
    }
}

pub async fn bench_positioning(providers: &BenchProviders, provider_extra: &Option<serde_json::Value>) -> BenchmarkResult {
    let positioner = match providers.positioner.as_ref() {
        Some(p) => p,
        None => return unsupported_result(&providers.provider_name, "positioning", "Positioning: default"),
    };
    let opts = PositioningOptions {
        provider_extra: provider_extra.clone(),
    };
    let t = Instant::now();
    let result = positioner.get_position(&opts).await;
    let elapsed = t.elapsed().as_millis() as u64;
    match result {
        Ok(_) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "positioning".to_string(),
            scenario: "Positioning: default".to_string(), duration_ms: elapsed,
            success: true, error: None, result_count: 1, raw_response_size: None,
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "positioning".to_string(),
            scenario: "Positioning: default".to_string(), duration_ms: elapsed,
            success: false, error: Some(e.to_string()), result_count: 0, raw_response_size: None,
        },
    }
}

pub async fn bench_attributes(providers: &BenchProviders, bbox: &str) -> BenchmarkResult {
    let attrs = match providers.attributes.as_ref() {
        Some(a) => a,
        None => return unsupported_result(&providers.provider_name, "attributes", &format!("Attributes: bbox={}", bbox)),
    };
    let opts = AttributeOptions { bbox: Some(bbox.to_string()), ..Default::default() };
    let t = Instant::now();
    let result = attrs.get_attributes(&opts).await;
    let elapsed = t.elapsed().as_millis() as u64;
    match result {
        Ok(_) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "attributes".to_string(),
            scenario: format!("Attributes: bbox={}", bbox), duration_ms: elapsed,
            success: true, error: None, result_count: 1, raw_response_size: None,
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "attributes".to_string(),
            scenario: format!("Attributes: bbox={}", bbox), duration_ms: elapsed,
            success: false, error: Some(e.to_string()), result_count: 0, raw_response_size: None,
        },
    }
}

pub async fn bench_image(providers: &BenchProviders, center: &Coordinate, zoom: u32) -> BenchmarkResult {
    let img = match providers.image.as_ref() {
        Some(i) => i,
        None => return unsupported_result(&providers.provider_name, "imaging", &format!("Image: ({},{}) z={}", center.lat, center.lng, zoom)),
    };
    let opts = ImageOptions::default();
    let t = Instant::now();
    let result = img.get_image(center, zoom, (800, 600), &opts).await;
    let elapsed = t.elapsed().as_millis() as u64;
    match result {
        Ok(res) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "imaging".to_string(),
            scenario: format!("Image: ({},{}) z={}", center.lat, center.lng, zoom), duration_ms: elapsed,
            success: true, error: None, result_count: 1, raw_response_size: Some(res.data.len()),
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "imaging".to_string(),
            scenario: format!("Image: ({},{}) z={}", center.lat, center.lng, zoom), duration_ms: elapsed,
            success: false, error: Some(e.to_string()), result_count: 0, raw_response_size: None,
        },
    }
}

pub async fn bench_geofence_search(providers: &BenchProviders, near: &Coordinate, radius: f64) -> BenchmarkResult {
    let gf = match providers.geofence.as_ref() {
        Some(g) => g,
        None => return unsupported_result(&providers.provider_name, "geofencing", &format!("Geofence search: ({},{}) r={}m", near.lat, near.lng, radius as i64)),
    };
    let opts = GeofenceOptions { near: Some(*near), radius: Some(radius), ..Default::default() };
    let t = Instant::now();
    let result = gf.search_geofences(&opts).await;
    let elapsed = t.elapsed().as_millis() as u64;
    match result {
        Ok(res) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "geofencing".to_string(),
            scenario: format!("Geofence search: ({},{}) r={}m", near.lat, near.lng, radius as i64), duration_ms: elapsed,
            success: true, error: None, result_count: res.geofences.len(), raw_response_size: None,
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "geofencing".to_string(),
            scenario: format!("Geofence search: ({},{}) r={}m", near.lat, near.lng, radius as i64), duration_ms: elapsed,
            success: false, error: Some(e.to_string()), result_count: 0, raw_response_size: None,
        },
    }
}

pub async fn bench_trip_create(providers: &BenchProviders, origin: &Coordinate, dest: &Coordinate) -> BenchmarkResult {
    let tracker = match providers.trip_tracker.as_ref() {
        Some(t) => t,
        None => return unsupported_result(&providers.provider_name, "tracking", "Trip create"),
    };
    let opts = TripCreateOptions { origin: Some(*origin), destination: Some(*dest), mode: Some("car".to_string()), ..Default::default() };
    let t = Instant::now();
    let result = tracker.create_trip(&opts).await;
    let elapsed = t.elapsed().as_millis() as u64;
    match result {
        Ok(_) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "tracking".to_string(),
            scenario: "Trip create".to_string(), duration_ms: elapsed,
            success: true, error: None, result_count: 1, raw_response_size: None,
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "tracking".to_string(),
            scenario: "Trip create".to_string(), duration_ms: elapsed,
            success: false, error: Some(e.to_string()), result_count: 0, raw_response_size: None,
        },
    }
}

pub async fn bench_fraud_check(providers: &BenchProviders, lat: f64, lng: f64) -> BenchmarkResult {
    let detector = match providers.fraud_detector.as_ref() {
        Some(d) => d,
        None => return unsupported_result(&providers.provider_name, "fraud", &format!("Fraud check: ({},{})", lat, lng)),
    };
    let opts = FraudCheckOptions { device_id: "bench_device".to_string(), latitude: lat, longitude: lng, accuracy: 10.0, ..Default::default() };
    let t = Instant::now();
    let result = detector.check_fraud(&opts).await;
    let elapsed = t.elapsed().as_millis() as u64;
    match result {
        Ok(_) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "fraud".to_string(),
            scenario: format!("Fraud check: ({},{})", lat, lng), duration_ms: elapsed,
            success: true, error: None, result_count: 1, raw_response_size: None,
        },
        Err(e) => BenchmarkResult {
            provider: providers.provider_name.clone(), domain: "fraud".to_string(),
            scenario: format!("Fraud check: ({},{})", lat, lng), duration_ms: elapsed,
            success: false, error: Some(e.to_string()), result_count: 0, raw_response_size: None,
        },
    }
}

fn percentile(sorted: &[u64], p: f64) -> u64 {
    if sorted.is_empty() { return 0; }
    let idx = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
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

    // Compute percentiles when we have multiple iterations per provider
    let mut provider_durations: std::collections::HashMap<String, Vec<u64>> = std::collections::HashMap::new();
    let mut provider_success: std::collections::HashMap<String, (u32, u32)> = std::collections::HashMap::new();
    for r in &results {
        provider_durations.entry(r.provider.clone()).or_default();
        provider_success.entry(r.provider.clone()).or_insert((0, 0));
        let (ok, err) = provider_success.get_mut(&r.provider).unwrap();
        if r.success {
            provider_durations.get_mut(&r.provider).unwrap().push(r.duration_ms);
            *ok += 1;
        } else {
            *err += 1;
        }
    }

    let percentiles = if provider_durations.values().any(|v| v.len() > 1) {
        let mut pcts: Vec<ProviderPercentiles> = Vec::new();
        let mut providers: Vec<String> = provider_durations.keys().cloned().collect();
        providers.sort();
        for provider in &providers {
            let mut durations = provider_durations[provider].clone();
            durations.sort();
            let (ok, err) = provider_success[provider];
            pcts.push(ProviderPercentiles {
                provider: provider.clone(),
                p50_ms: percentile(&durations, 50.0),
                p95_ms: percentile(&durations, 95.0),
                p99_ms: percentile(&durations, 99.0),
                min_ms: durations.first().copied().unwrap_or(0),
                max_ms: durations.last().copied().unwrap_or(0),
                success_count: ok,
                error_count: err,
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