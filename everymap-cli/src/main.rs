use clap::{Parser, Subcommand};
use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::search::{Geocoder, GeocodeRequest};
use everymap_core::domains::routing::{Router, RouteRequest};
use everymap_core::domains::traffic::{TrafficProvider, TrafficRequest};
use everymap_core::domains::positioning::{NetworkPositioner, PositioningRequest};
use everymap_core::domains::isoline::{IsolineProvider, IsolineRequest};
use everymap_core::types::Coordinate;
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::search::HereGeocoder;
use everymap_providers_here::domain::search::HereGeocodeOptions;
use everymap_providers_here::domain::routing::HereRouter;
use everymap_providers_here::domain::routing::HereRouteOptions;
use everymap_providers_here::domain::routing::TransportMode;
use everymap_providers_here::domain::traffic::HereTraffic;
use everymap_providers_here::domain::traffic::HereFlowOptions;
use everymap_providers_here::domain::positioning::HerePositioner;
use everymap_providers_here::domain::positioning::HerePositioningOptions;
use everymap_providers_here::domain::isoline::HereIsoline;
use everymap_providers_here::domain::isoline::HereIsolineOptions;
use everymap_providers_here::domain::isoline::RangeType;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "everymap", version, about = "Geospatial API CLI - unified interface for map providers")]
struct Cli {
    /// Provider to use
    #[arg(long, default_value = "here")]
    provider: String,

    /// API key for authentication
    #[arg(long, env = "EVERYMAP_API_KEY")]
    api_key: Option<String>,

    /// API key parameter name (default: apiKey)
    #[arg(long, default_value = "apiKey")]
    api_key_param: String,

    /// Output format
    #[arg(long, default_value = "json")]
    output: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Geocode an address to coordinates
    Geocode {
        /// Address to geocode
        query: String,
    },
    /// Reverse geocode coordinates to an address
    ReverseGeocode {
        /// Latitude
        #[arg(long)]
        lat: f64,
        /// Longitude
        #[arg(long)]
        lng: f64,
    },
    /// Calculate a route between two points
    Route {
        /// Origin coordinates (lat,lng)
        #[arg(long)]
        origin: String,
        /// Destination coordinates (lat,lng)
        #[arg(long)]
        destination: String,
        /// Transport mode (car, truck, pedestrian, bicycle, scooter, bus, taxi)
        #[arg(long, default_value = "car")]
        transport: String,
    },
    /// Get traffic flow data for a location
    Traffic {
        /// Latitude
        #[arg(long)]
        lat: f64,
        /// Longitude
        #[arg(long)]
        lng: f64,
    },
    /// Get position estimate from network data
    Position,
    /// Calculate an isoline (reachability polygon)
    Isoline {
        /// Center latitude
        #[arg(long)]
        lat: f64,
        /// Center longitude
        #[arg(long)]
        lng: f64,
        /// Range value in meters
        #[arg(long, default_value = "1000")]
        range: f64,
    },
}

fn parse_coordinate(input: &str) -> Result<Coordinate, String> {
    let parts: Vec<&str> = input.split(',').collect();
    if parts.len() != 2 {
        return Err("Expected format: lat,lng".to_string());
    }
    let lat: f64 = parts[0].trim().parse().map_err(|_| "Invalid latitude")?;
    let lng: f64 = parts[1].trim().parse().map_err(|_| "Invalid longitude")?;
    Coordinate::new(lat, lng).map_err(|e| e.to_string())
}

fn print_json(value: &serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()));
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let api_key = match cli.api_key {
        Some(key) => key,
        None => {
            eprintln!("Error: API key required. Use --api-key or set EVERYMAP_API_KEY environment variable.");
            std::process::exit(1);
        }
    };

    if cli.provider != "here" {
        eprintln!("Error: Only 'here' provider is currently supported. Provider '{}' not found.", cli.provider);
        std::process::exit(1);
    }

    let auth = Arc::new(ApiKeyProvider::new(api_key, cli.api_key_param));
    let client = Arc::new(HereClient::new(auth));

    match cli.command {
        Commands::Geocode { query } => {
            let geocoder = HereGeocoder::new(client);
            let req = GeocodeRequest {
                query,
                options: HereGeocodeOptions::default(),
            };
            match geocoder.geocode(req).await {
                Ok(res) => {
                    let output: Vec<serde_json::Value> = res.items.iter().map(|item| {
                        serde_json::json!({
                            "id": item.id,
                            "title": item.title,
                            "coordinate": { "lat": item.coordinate.lat, "lng": item.coordinate.lng },
                            "address": item.address,
                            "result_type": format!("{:?}", item.result_type),
                            "distance": item.distance,
                        })
                    }).collect();
                    print_json(&serde_json::json!({ "results": output }));
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::ReverseGeocode { lat, lng } => {
            let geocoder = HereGeocoder::new(client);
            let coord = match Coordinate::new(lat, lng) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid coordinates: {}", e);
                    std::process::exit(1);
                }
            };
            let req = everymap_core::domains::search::ReverseGeocodeRequest {
                coordinate: coord,
                options: HereGeocodeOptions::default(),
            };
            match geocoder.reverse_geocode(req).await {
                Ok(res) => {
                    let output: Vec<serde_json::Value> = res.items.iter().map(|item| {
                        serde_json::json!({
                            "id": item.id,
                            "title": item.title,
                            "coordinate": { "lat": item.coordinate.lat, "lng": item.coordinate.lng },
                            "address": item.address,
                            "result_type": format!("{:?}", item.result_type),
                        })
                    }).collect();
                    print_json(&serde_json::json!({ "results": output }));
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Route { origin, destination, transport } => {
            let router = HereRouter::new(client);
            let start = match parse_coordinate(&origin) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid origin: {}", e);
                    std::process::exit(1);
                }
            };
            let end = match parse_coordinate(&destination) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid destination: {}", e);
                    std::process::exit(1);
                }
            };
            let transport_mode = match transport.as_str() {
                "car" => TransportMode::Car,
                "truck" => TransportMode::Truck,
                "pedestrian" => TransportMode::Pedestrian,
                "bicycle" => TransportMode::Bicycle,
                "scooter" => TransportMode::Scooter,
                "bus" => TransportMode::Bus,
                "taxi" => TransportMode::Taxi,
                _ => {
                    eprintln!("Unknown transport mode: {}. Using car.", transport);
                    TransportMode::Car
                }
            };
            let req = RouteRequest {
                start,
                end,
                options: HereRouteOptions {
                    transport_mode,
                    ..Default::default()
                },
            };
            match router.calculate_route(req).await {
                Ok(res) => {
                    let routes: Vec<serde_json::Value> = res.routes.iter().map(|r| {
                        serde_json::json!({
                            "distance_m": r.distance,
                            "duration_s": r.duration,
                            "points": r.geometry.points.len(),
                        })
                    }).collect();
                    print_json(&serde_json::json!({ "routes": routes }));
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Traffic { lat, lng } => {
            let traffic = HereTraffic::new(client);
            let coord = match Coordinate::new(lat, lng) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid coordinates: {}", e);
                    std::process::exit(1);
                }
            };
            let req = TrafficRequest {
                location: coord,
                options: HereFlowOptions::default(),
            };
            match traffic.get_traffic(req).await {
                Ok(res) => {
                    let flows: Vec<serde_json::Value> = res.flows.iter().map(|f| {
                        serde_json::json!({
                            "jam_factor": f.jam_factor,
                            "speed": f.speed,
                            "free_flow_speed": f.free_flow_speed,
                            "road_name": f.road_name,
                        })
                    }).collect();
                    print_json(&serde_json::json!({ "flows": flows, "incidents": res.incidents.len() }));
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Position => {
            let positioner = HerePositioner::new(client);
            let req = PositioningRequest {
                options: HerePositioningOptions::default(),
            };
            match positioner.get_position(req).await {
                Ok(res) => {
                    print_json(&serde_json::json!({
                        "coordinate": { "lat": res.coordinate.lat, "lng": res.coordinate.lng },
                        "accuracy": res.accuracy,
                        "altitude": res.altitude,
                    }));
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Isoline { lat, lng, range } => {
            let isoline = HereIsoline::new(client);
            let center = match Coordinate::new(lat, lng) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid coordinates: {}", e);
                    std::process::exit(1);
                }
            };
            let req = IsolineRequest {
                center,
                range,
                options: HereIsolineOptions {
                    range_type: RangeType::Distance,
                    ..Default::default()
                },
            };
            match isoline.get_isoline(req).await {
                Ok(res) => {
                    let isolines: Vec<serde_json::Value> = res.isolines.iter().map(|iso| {
                        serde_json::json!({
                            "range": iso.range,
                            "points": iso.polygon.len(),
                        })
                    }).collect();
                    print_json(&serde_json::json!({ "isolines": isolines }));
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
}