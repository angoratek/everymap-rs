use everymap_core::types::Coordinate;

/// Typed parameters for a benchmark scenario.
#[derive(Debug, Clone)]
pub enum ScenarioParams {
    Geocode { query: String },
    ReverseGeocode { coord: Coordinate },
    Route { start: Coordinate, end: Coordinate },
    Isoline { center: Coordinate, range: f64 },
    Matching { points: Vec<Coordinate> },
    Tour { stops: Vec<Coordinate> },
    Traffic { location: Coordinate },
    Tile { z: u32, x: u32, y: u32 },
    Positioning { provider_extra: Option<serde_json::Value> },
    Attributes { bbox: String },
    Image { center: Coordinate, zoom: u32 },
    GeofenceSearch { near: Coordinate, radius: f64 },
    TripCreate { origin: Coordinate, destination: Coordinate },
    FraudCheck { lat: f64, lng: f64 },
}

impl ScenarioParams {
    /// Returns the domain name for this scenario variant.
    pub fn domain(&self) -> &'static str {
        match self {
            ScenarioParams::Geocode { .. } | ScenarioParams::ReverseGeocode { .. } => "geocoder",
            ScenarioParams::Route { .. } => "routing",
            ScenarioParams::Isoline { .. } => "isoline",
            ScenarioParams::Matching { .. } => "matching",
            ScenarioParams::Tour { .. } => "tour",
            ScenarioParams::Traffic { .. } => "traffic",
            ScenarioParams::Tile { .. } => "tiling",
            ScenarioParams::Positioning { .. } => "positioning",
            ScenarioParams::Attributes { .. } => "attributes",
            ScenarioParams::Image { .. } => "imaging",
            ScenarioParams::GeofenceSearch { .. } => "geofencing",
            ScenarioParams::TripCreate { .. } => "tracking",
            ScenarioParams::FraudCheck { .. } => "fraud",
        }
    }
}

/// A benchmark scenario defining a test case.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BenchmarkScenario {
    pub name: String,
    pub description: String,
    pub params: ScenarioParams,
}

/// Predefined benchmark scenarios for all 13 domains.
pub fn get_scenarios(domain: Option<&str>) -> Vec<BenchmarkScenario> {
    let all = vec![
        // Geocoder
        BenchmarkScenario {
            name: "Berlin Brandenburg Gate".to_string(),
            description: "Forward geocode for 'Brandenburg Gate, Berlin'".to_string(),
            params: ScenarioParams::Geocode {
                query: "Brandenburg Gate, Berlin".to_string(),
            },
        },
        BenchmarkScenario {
            name: "Reverse Berlin center".to_string(),
            description: "Reverse geocode at Berlin center (52.52,13.38)".to_string(),
            params: ScenarioParams::ReverseGeocode {
                coord: Coordinate::new(52.5163, 13.3777).unwrap(),
            },
        },
        // Routing
        BenchmarkScenario {
            name: "Berlin to Paris".to_string(),
            description: "Route from Berlin (52.52,13.38) to Paris (48.86,2.35)".to_string(),
            params: ScenarioParams::Route {
                start: Coordinate::new(52.5163, 13.3777).unwrap(),
                end: Coordinate::new(48.8566, 2.3522).unwrap(),
            },
        },
        BenchmarkScenario {
            name: "NYC to LA".to_string(),
            description: "Route from NYC (40.71,-74.01) to LA (34.05,-118.24)".to_string(),
            params: ScenarioParams::Route {
                start: Coordinate::new(40.7128, -74.0060).unwrap(),
                end: Coordinate::new(34.0522, -118.2437).unwrap(),
            },
        },
        // Isoline
        BenchmarkScenario {
            name: "30min drive from Berlin".to_string(),
            description: "30-minute driving isoline from Berlin center".to_string(),
            params: ScenarioParams::Isoline {
                center: Coordinate::new(52.5163, 13.3777).unwrap(),
                range: 1800.0,
            },
        },
        // Matching
        BenchmarkScenario {
            name: "Berlin straight line".to_string(),
            description: "Match 4 GPS points along Berlin roads".to_string(),
            params: ScenarioParams::Matching {
                points: vec![
                    Coordinate::new(52.5200, 13.4050).unwrap(),
                    Coordinate::new(52.5250, 13.4100).unwrap(),
                    Coordinate::new(52.5300, 13.4150).unwrap(),
                    Coordinate::new(52.5350, 13.4200).unwrap(),
                ],
            },
        },
        // Tour
        BenchmarkScenario {
            name: "5 Berlin landmarks".to_string(),
            description: "Optimize tour through 5 Berlin landmarks".to_string(),
            params: ScenarioParams::Tour {
                stops: vec![
                    Coordinate::new(52.5163, 13.3777).unwrap(),
                    Coordinate::new(52.5162, 13.3739).unwrap(),
                    Coordinate::new(52.5200, 13.3966).unwrap(),
                    Coordinate::new(52.5076, 13.3456).unwrap(),
                    Coordinate::new(52.5145, 13.3500).unwrap(),
                ],
            },
        },
        // Traffic
        BenchmarkScenario {
            name: "Berlin traffic".to_string(),
            description: "Traffic flow near Berlin center".to_string(),
            params: ScenarioParams::Traffic {
                location: Coordinate::new(52.5163, 13.3777).unwrap(),
            },
        },
        // Tiling
        BenchmarkScenario {
            name: "Berlin tile z10".to_string(),
            description: "Fetch map tile at zoom 10 covering Berlin".to_string(),
            params: ScenarioParams::Tile { z: 10, x: 550, y: 335 },
        },
        // Positioning
        BenchmarkScenario {
            name: "WiFi positioning".to_string(),
            description: "Network positioning with WiFi observations".to_string(),
            params: ScenarioParams::Positioning {
                provider_extra: Some(serde_json::json!({
                    "wlan": [
                        {"mac": "00:11:22:33:44:55", "signalStrength": -65},
                        {"mac": "aa:bb:cc:dd:ee:ff", "signalStrength": -70}
                    ]
                })),
            },
        },
        // Attributes
        BenchmarkScenario {
            name: "Berlin attributes".to_string(),
            description: "Road attributes near Berlin center (bbox)".to_string(),
            params: ScenarioParams::Attributes {
                bbox: "13.3,52.5,13.4,52.52".to_string(),
            },
        },
        // Imaging
        BenchmarkScenario {
            name: "Berlin map image".to_string(),
            description: "Static map image of Berlin center at zoom 12".to_string(),
            params: ScenarioParams::Image {
                center: Coordinate::new(52.5163, 13.3777).unwrap(),
                zoom: 12,
            },
        },
        // Geofencing (Radar-only)
        BenchmarkScenario {
            name: "Geofence search near NYC".to_string(),
            description: "Search geofences near NYC (40.71,-74.01) r=1000m".to_string(),
            params: ScenarioParams::GeofenceSearch {
                near: Coordinate::new(40.7128, -74.0060).unwrap(),
                radius: 1000.0,
            },
        },
        // Tracking (Radar-only)
        BenchmarkScenario {
            name: "Trip NYC to Boston".to_string(),
            description: "Create a trip from NYC to Boston via Radar".to_string(),
            params: ScenarioParams::TripCreate {
                origin: Coordinate::new(40.7128, -74.0060).unwrap(),
                destination: Coordinate::new(42.3601, -71.0589).unwrap(),
            },
        },
        // Fraud (Radar-only)
        BenchmarkScenario {
            name: "Fraud check NYC".to_string(),
            description: "Fraud check at NYC coordinates via Radar".to_string(),
            params: ScenarioParams::FraudCheck { lat: 40.7128, lng: -74.0060 },
        },
    ];

    match domain {
        Some(d) => all.into_iter().filter(|s| s.params.domain() == d).collect(),
        None => all,
    }
}