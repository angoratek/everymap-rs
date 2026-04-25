//! Live integration tests for the HERE provider.
//!
//! These tests hit the real HERE API and require a valid API key.
//! Set the `EVERYMAP_HERE_API_KEY` environment variable to run them.
//! If the env var is not set, all tests skip silently (return early).

use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::attributes::{AttributeOptions, AttributeProvider};
use everymap_core::domains::imaging::{ImageOptions, MapImageProvider};
use everymap_core::domains::isoline::RangeType as CoreRangeType;
use everymap_core::domains::isoline::{IsolineOptions, IsolineProvider};
use everymap_core::domains::matching::{MatchingOptions, RouteMatcher};
use everymap_core::domains::positioning::{NetworkPositioner, PositioningOptions};
use everymap_core::domains::routing::{RouteOptions, Router, TransportMode};
use everymap_core::domains::search::{GeocodeOptions, Geocoder, ReverseGeocodeOptions};
use everymap_core::domains::tiling::{TileOptions, TileProvider};
use everymap_core::domains::tour::{TourOptions, TourPlanner};
use everymap_core::domains::traffic::{TrafficOptions, TrafficProvider};
use everymap_core::types::Coordinate;
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::attributes::HereAttributeProvider;
use everymap_providers_here::domain::imaging::HereMapImageProvider;
use everymap_providers_here::domain::isoline::HereIsoline;
use everymap_providers_here::domain::matching::HereRouteMatcher;
use everymap_providers_here::domain::positioning::HerePositioner;
use everymap_providers_here::domain::routing::HereRouter;
use everymap_providers_here::domain::search::HereGeocoder;
use everymap_providers_here::domain::tiling::HereTileProvider;
use everymap_providers_here::domain::tour::{
    Fleet, HereTourPlanner, Job, JobPlace, JobTask, JobTasks, Objective, Plan, Profile, ShiftStart,
    TourLocation, TourProblem, VehicleCosts, VehicleShift, VehicleType,
};
use everymap_providers_here::domain::traffic::HereTraffic;
use std::sync::Arc;

/// Helper to get the API key from the environment. Returns empty string if not set.
fn get_api_key() -> String {
    std::env::var("EVERYMAP_HERE_API_KEY").unwrap_or_default()
}

/// Helper to create a HereClient with the API key.
fn make_client(api_key: &str) -> Arc<HereClient> {
    let auth = Arc::new(ApiKeyProvider::new(
        api_key.to_string(),
        "apiKey".to_string(),
    ));
    Arc::new(HereClient::new(auth))
}

// ---------------------------------------------------------------------------
// Geocoder
// ---------------------------------------------------------------------------

#[tokio::test]
async fn live_geocode_berlin() {
    let api_key = get_api_key();
    if api_key.is_empty() {
        return;
    }

    let client = make_client(&api_key);
    let geocoder = HereGeocoder::new(client);

    let opts = GeocodeOptions::default();
    let res = geocoder.geocode("Berlin", &opts).await;

    match res {
        Ok(res) => {
            println!("[geocode] items: {}", res.items.len());
            assert!(
                !res.items.is_empty(),
                "Geocode should return at least one result"
            );
            let first = &res.items[0];
            assert!(
                first.coordinate.lat > 52.0 && first.coordinate.lat < 53.0,
                "Berlin latitude should be around 52.5, got {}",
                first.coordinate.lat
            );
            assert!(
                first.coordinate.lng > 13.0 && first.coordinate.lng < 14.0,
                "Berlin longitude should be around 13.4, got {}",
                first.coordinate.lng
            );
            println!(
                "[geocode] first result: {:?} at {}",
                first.title, first.coordinate
            );
        }
        Err(e) => {
            eprintln!("[geocode] ERROR: {:?}", e);
            panic!("Geocode request failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn live_reverse_geocode_berlin() {
    let api_key = get_api_key();
    if api_key.is_empty() {
        return;
    }

    let client = make_client(&api_key);
    let geocoder = HereGeocoder::new(client);

    let coord = Coordinate::new(52.51604, 13.37691).unwrap();
    let opts = ReverseGeocodeOptions::default();
    let res = geocoder.reverse_geocode(&coord, &opts).await;

    match res {
        Ok(res) => {
            println!("[reverse-geocode] items: {}", res.items.len());
            assert!(
                !res.items.is_empty(),
                "Reverse geocode should return at least one result"
            );
            let first = &res.items[0];
            println!(
                "[reverse-geocode] first result: {:?} at {}",
                first.address.label.as_deref().unwrap_or("(no label)"),
                first.coordinate
            );
        }
        Err(e) => {
            eprintln!("[reverse-geocode] ERROR: {:?}", e);
            panic!("Reverse geocode request failed: {:?}", e);
        }
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

#[tokio::test]
async fn live_route_berlin_to_munich() {
    let api_key = get_api_key();
    if api_key.is_empty() {
        return;
    }

    let client = make_client(&api_key);
    let router = HereRouter::new(client);

    let berlin = Coordinate::new(52.5200, 13.4050).unwrap();
    let munich = Coordinate::new(48.1351, 11.5820).unwrap();
    let opts = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        ..Default::default()
    };

    let res = router.calculate_route(&berlin, &munich, &opts).await;

    match res {
        Ok(res) => {
            println!("[route] routes: {}", res.routes.len());
            assert!(
                !res.routes.is_empty(),
                "Route calculation should return at least one route"
            );
            let route = &res.routes[0];
            assert!(
                route.distance > 0.0,
                "Route distance should be > 0, got {}",
                route.distance
            );
            assert!(
                route.duration > 0.0,
                "Route duration should be > 0, got {}",
                route.duration
            );
            // Berlin to Munich is roughly 580-600 km by car
            assert!(
                route.distance > 400_000.0 && route.distance < 800_000.0,
                "Berlin-Munich distance should be roughly 500-700 km, got {} m",
                route.distance
            );
            println!(
                "[route] distance: {:.1} km, duration: {:.0} s ({:.1} h)",
                route.distance / 1000.0,
                route.duration,
                route.duration / 3600.0
            );
        }
        Err(e) => {
            eprintln!("[route] ERROR: {:?}", e);
            panic!("Route calculation failed: {:?}", e);
        }
    }
}

// ---------------------------------------------------------------------------
// Isoline
// ---------------------------------------------------------------------------

#[tokio::test]
async fn live_isoline_berlin_5km() {
    let api_key = get_api_key();
    if api_key.is_empty() {
        return;
    }

    let client = make_client(&api_key);
    let isoline_provider = HereIsoline::new(client);

    let center = Coordinate::new(52.5200, 13.4050).unwrap();
    let opts = IsolineOptions {
        range_type: Some(CoreRangeType::Distance),
        ..Default::default()
    };

    let res = isoline_provider.get_isoline(&center, 5000.0, &opts).await;

    match res {
        Ok(res) => {
            println!("[isoline] isolines: {}", res.isolines.len());
            assert!(
                !res.isolines.is_empty(),
                "Isoline should return at least one result"
            );
            let iso = &res.isolines[0];
            assert!(
                !iso.polygon.is_empty(),
                "Isoline polygon should not be empty"
            );
            println!(
                "[isoline] polygon points: {}, range: {:?}",
                iso.polygon.len(),
                iso.range
            );
        }
        Err(e) => {
            eprintln!("[isoline] ERROR: {:?}", e);
            panic!("Isoline request failed: {:?}", e);
        }
    }
}

// ---------------------------------------------------------------------------
// Matching
// ---------------------------------------------------------------------------

#[tokio::test]
async fn live_matching_gps_trace() {
    let api_key = get_api_key();
    if api_key.is_empty() {
        return;
    }

    let client = make_client(&api_key);
    let matcher = HereRouteMatcher::new(client);

    // Simple GPS trace along a road in Berlin (Unter den Linden area)
    let points = vec![
        Coordinate::new(52.5164, 13.3777).unwrap(),
        Coordinate::new(52.5170, 13.3900).unwrap(),
        Coordinate::new(52.5180, 13.4000).unwrap(),
    ];
    let opts = MatchingOptions {
        provider_extra: Some(serde_json::json!({
            "mode": "car",
            "map_match_radius": 50
        })),
        ..Default::default()
    };

    let res = matcher.match_route(&points, &opts).await;

    match res {
        Ok(res) => {
            println!(
                "[matching] matched points: {}, distance: {:.1} m",
                res.matched_points.len(),
                res.distance
            );
            assert!(
                !res.matched_points.is_empty(),
                "Matching should return matched points"
            );
            assert!(res.distance > 0.0, "Matched distance should be > 0");
        }
        Err(e) => {
            eprintln!("[matching] ERROR: {:?}", e);
            panic!("Route matching failed: {:?}", e);
        }
    }
}

// ---------------------------------------------------------------------------
// Traffic
// ---------------------------------------------------------------------------

#[tokio::test]
async fn live_traffic_berlin() {
    let api_key = get_api_key();
    if api_key.is_empty() {
        return;
    }

    let client = make_client(&api_key);
    let traffic = HereTraffic::new(client);

    let coord = Coordinate::new(52.5200, 13.4050).unwrap();
    let opts = TrafficOptions::default();

    let res = traffic.get_traffic(&coord, &opts).await;

    match res {
        Ok(res) => {
            println!(
                "[traffic] flows: {}, incidents: {}",
                res.flows.len(),
                res.incidents.len()
            );
            // Traffic may return empty flows for small bbox, but the call itself should succeed
            if !res.flows.is_empty() {
                let flow = &res.flows[0];
                println!(
                    "[traffic] first flow - speed: {:?}, jam_factor: {:?}",
                    flow.speed, flow.jam_factor
                );
            }
        }
        Err(e) => {
            eprintln!("[traffic] ERROR: {:?}", e);
            panic!("Traffic request failed: {:?}", e);
        }
    }
}

// ---------------------------------------------------------------------------
// Positioning
// ---------------------------------------------------------------------------

#[tokio::test]
async fn live_positioning_empty_observations() {
    let api_key = get_api_key();
    if api_key.is_empty() {
        return;
    }

    let client = make_client(&api_key);
    let positioner = HerePositioner::new(client);

    let opts = PositioningOptions::default();

    let res = positioner.get_position(&opts).await;

    match res {
        Ok(res) => {
            println!(
                "[positioning] coordinate: {}, accuracy: {:?}",
                res.coordinate, res.accuracy
            );
            // With empty observations the API may still return an IP-based position
            // or return an error. Both are acceptable.
        }
        Err(e) => {
            // Positioning without any observations may return an error.
            // This is expected and acceptable.
            eprintln!(
                "[positioning] ERROR (expected with empty observations): {:?}",
                e
            );
            println!("[positioning] Error is acceptable when no observations provided.");
        }
    }
}

// ---------------------------------------------------------------------------
// Tour
// ---------------------------------------------------------------------------

#[tokio::test]
async fn live_tour_3_stops_berlin() {
    let api_key = get_api_key();
    if api_key.is_empty() {
        return;
    }

    let client = make_client(&api_key);
    let planner = HereTourPlanner::new(client);

    let stops = vec![
        Coordinate::new(52.5200, 13.4050).unwrap(), // Berlin center
        Coordinate::new(52.5300, 13.3800).unwrap(), // West Berlin
        Coordinate::new(52.5100, 13.4200).unwrap(), // East Berlin
    ];

    // Build a proper TourProblem with fleet and jobs
    let problem = TourProblem {
        fleet: Fleet {
            types: vec![VehicleType {
                id: "vehicle_1".to_string(),
                profile: "car_profile".to_string(),
                costs: VehicleCosts {
                    fixed: Some(0.0),
                    distance: Some(1.0),
                    time: Some(0.0),
                    job: None,
                },
                shifts: vec![VehicleShift {
                    start: ShiftStart {
                        time: Some("2026-04-15T08:00:00Z".to_string()),
                        earliest: None,
                        location: Some(TourLocation {
                            lat: 52.5200,
                            lng: 13.4050,
                        }),
                    },
                    ..Default::default()
                }],
                capacity: Some(vec![10]),
                amount: Some(1),
                ..Default::default()
            }],
            profiles: vec![Profile::Car {
                name: "car_profile".to_string(),
                departure_time: None,
                traffic: None,
            }],
            traffic: None,
        },
        plan: Plan {
            jobs: stops
                .iter()
                .enumerate()
                .map(|(i, coord)| Job {
                    id: format!("stop_{}", i),
                    tasks: JobTasks {
                        deliveries: Some(vec![JobTask {
                            places: vec![JobPlace {
                                location: TourLocation {
                                    lat: coord.lat,
                                    lng: coord.lng,
                                },
                                duration: 60,
                                ..Default::default()
                            }],
                            demand: vec![1],
                            ..Default::default()
                        }]),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        },
        objectives: Some(vec![Objective::MinimizeUnassigned, Objective::MinimizeCost]),
        ..Default::default()
    };

    let opts = TourOptions {
        provider_extra: Some(serde_json::to_value(problem).unwrap()),
    };

    let res = planner.optimize_tour(&stops, &opts).await;

    match res {
        Ok(res) => {
            println!(
                "[tour] stops: {}, total_distance: {:?}, total_duration: {:?}",
                res.stops.len(),
                res.total_distance,
                res.total_duration
            );
            assert!(
                !res.stops.is_empty(),
                "Tour should return at least one stop"
            );
            if let Some(dist) = res.total_distance {
                assert!(
                    dist > 0.0,
                    "Tour total distance should be > 0, got {}",
                    dist
                );
            }
        }
        Err(e) => {
            eprintln!("[tour] ERROR: {:?}", e);
            panic!("Tour planning failed: {:?}", e);
        }
    }
}

// ---------------------------------------------------------------------------
// Tiling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn live_tiling_zoom14_berlin() {
    let api_key = get_api_key();
    if api_key.is_empty() {
        return;
    }

    let client = make_client(&api_key);
    let provider = HereTileProvider::new(client);

    // Zoom 14, tile coords around Berlin
    // At zoom 14, Berlin center (52.52, 13.405) maps to roughly x=4494, y=2832
    let opts = TileOptions {
        provider_extra: Some(serde_json::json!({
            "layer": "base"
        })),
        ..Default::default()
    };

    let res = provider.get_tile(14, 4494, 2832, &opts).await;

    match res {
        Ok(res) => {
            println!(
                "[tiling] data bytes: {}, content_type: {:?}",
                res.data.len(),
                res.content_type
            );
            assert!(!res.data.is_empty(), "Tile data should not be empty");
            assert!(
                res.content_type.is_some(),
                "Tile should have a content-type"
            );
        }
        Err(e) => {
            eprintln!("[tiling] ERROR: {:?}", e);
            panic!("Tiling request failed: {:?}", e);
        }
    }
}

// ---------------------------------------------------------------------------
// Attributes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn live_attributes_roads_berlin() {
    let api_key = get_api_key();
    if api_key.is_empty() {
        return;
    }

    let client = make_client(&api_key);
    let provider = HereAttributeProvider::new(client);

    // Bounding box around Berlin center with layers parameter
    // The real API requires the 'layers' parameter
    let opts = AttributeOptions {
        bbox: Some("52.51,13.37;52.53,13.41".to_string()),
        provider_extra: Some(serde_json::json!({
            "layers": ["ROAD_GEOM_FCn", "SPEED_LIMITS_FCn"]
        })),
        ..Default::default()
    };

    let res = provider.get_attributes(&opts).await;

    match res {
        Ok(res) => {
            println!(
                "[attributes] response keys: {:?}",
                res.data.as_object().map(|o| o.keys().collect::<Vec<_>>())
            );
            // The response should be a valid JSON object.
            // The real HERE v8 API returns {"geometries": [...], "meta": {...}},
            // not GeoJSON FeatureCollection format.
            assert!(
                res.data.is_object(),
                "Attributes response should be a JSON object"
            );
            if let Some(obj) = res.data.as_object() {
                assert!(
                    obj.contains_key("geometries")
                        || obj.contains_key("features")
                        || obj.contains_key("type"),
                    "Attributes response should contain 'geometries', 'features', or 'type' key"
                );
            }
        }
        Err(e) => {
            eprintln!("[attributes] ERROR: {:?}", e);
            panic!("Attributes request failed: {:?}", e);
        }
    }
}

// ---------------------------------------------------------------------------
// Imaging
// ---------------------------------------------------------------------------

#[tokio::test]
async fn live_imaging_static_map_berlin() {
    let api_key = get_api_key();
    if api_key.is_empty() {
        return;
    }

    let client = make_client(&api_key);
    let provider = HereMapImageProvider::new(client);

    let center = Coordinate::new(52.5200, 13.4050).unwrap();
    let opts = ImageOptions::default();

    let res = provider.get_image(&center, 10, (512, 512), &opts).await;

    match res {
        Ok(res) => {
            println!(
                "[imaging] data bytes: {}, content_type: {:?}",
                res.data.len(),
                res.content_type
            );
            assert!(!res.data.is_empty(), "Image data should not be empty");
            assert!(
                res.content_type.is_some(),
                "Image should have a content-type"
            );
            let ct = res.content_type.as_deref().unwrap_or("");
            assert!(
                ct.starts_with("image/"),
                "Content type should be an image type, got: {}",
                ct
            );
        }
        Err(e) => {
            eprintln!("[imaging] ERROR: {:?}", e);
            panic!("Imaging request failed: {:?}", e);
        }
    }
}
