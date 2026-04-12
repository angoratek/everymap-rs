mod config;
mod output;

use clap::{Parser, Subcommand};
use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::search::{Geocoder, GeocodeOptions, ReverseGeocodeOptions};
use everymap_core::domains::routing::{Router, RouteOptions, TransportMode as CoreTransportMode};
use everymap_core::domains::traffic::{TrafficProvider, TrafficOptions};
use everymap_core::domains::positioning::{NetworkPositioner as NetworkPositionerTrait, PositioningOptions};
use everymap_core::domains::isoline::{IsolineProvider, IsolineOptions, RangeType as CoreRangeType};
use everymap_core::domains::matching::{RouteMatcher, MatchingOptions};
use everymap_core::domains::tour::{TourPlanner, TourOptions};
use everymap_core::domains::tiling::{TileProvider, TileOptions};
use everymap_core::domains::attributes::{AttributeProvider, AttributeOptions};
use everymap_core::domains::imaging::{MapImageProvider, ImageOptions};
use everymap_core::types::Coordinate;
use everymap_providers_here::client::HereClient;
use everymap_providers_google::client::GoogleClient;
use std::sync::Arc;

// Factory functions for HERE provider
fn create_here_geocoder(client: Arc<HereClient>) -> Box<dyn Geocoder> {
    Box::new(everymap_providers_here::domain::search::HereGeocoder::new(client))
}

fn create_here_router(client: Arc<HereClient>) -> Box<dyn Router> {
    Box::new(everymap_providers_here::domain::routing::HereRouter::new(client))
}

fn create_here_traffic_provider(client: Arc<HereClient>) -> Box<dyn TrafficProvider> {
    Box::new(everymap_providers_here::domain::traffic::HereTraffic::new(client))
}

fn create_here_positioner(client: Arc<HereClient>) -> Box<dyn NetworkPositionerTrait> {
    Box::new(everymap_providers_here::domain::positioning::HerePositioner::new(client))
}

fn create_here_isoline_provider(client: Arc<HereClient>) -> Box<dyn IsolineProvider> {
    Box::new(everymap_providers_here::domain::isoline::HereIsoline::new(client))
}

fn create_here_route_matcher(client: Arc<HereClient>) -> Box<dyn RouteMatcher> {
    Box::new(everymap_providers_here::domain::matching::HereRouteMatcher::new(client))
}

fn create_here_tour_planner(client: Arc<HereClient>) -> Box<dyn TourPlanner> {
    Box::new(everymap_providers_here::domain::tour::HereTourPlanner::new(client))
}

fn create_here_tile_provider(client: Arc<HereClient>) -> Box<dyn TileProvider> {
    Box::new(everymap_providers_here::domain::tiling::HereTileProvider::new(client))
}

fn create_here_attribute_provider(client: Arc<HereClient>) -> Box<dyn AttributeProvider> {
    Box::new(everymap_providers_here::domain::attributes::HereAttributeProvider::new(client))
}

fn create_here_image_provider(client: Arc<HereClient>) -> Box<dyn MapImageProvider> {
    Box::new(everymap_providers_here::domain::imaging::HereMapImageProvider::new(client))
}

// Factory functions for Google provider
fn create_google_geocoder(client: Arc<GoogleClient>) -> Box<dyn Geocoder> {
    Box::new(everymap_providers_google::GoogleGeocoder::new(client))
}

fn create_google_router(client: Arc<GoogleClient>) -> Box<dyn Router> {
    Box::new(everymap_providers_google::GoogleRouter::new(client))
}

fn create_google_isoline_provider(_client: Arc<GoogleClient>) -> Box<dyn IsolineProvider> {
    Box::new(everymap_providers_google::GoogleIsoline)
}

fn create_google_traffic_provider(_client: Arc<GoogleClient>) -> Box<dyn TrafficProvider> {
    Box::new(everymap_providers_google::GoogleTraffic)
}

fn create_google_positioner(client: Arc<GoogleClient>) -> Box<dyn NetworkPositionerTrait> {
    Box::new(everymap_providers_google::GooglePositioner::new(client))
}

fn create_google_route_matcher(client: Arc<GoogleClient>) -> Box<dyn RouteMatcher> {
    Box::new(everymap_providers_google::GoogleRouteMatcher::new(client))
}

fn create_google_tour_planner(_client: Arc<GoogleClient>) -> Box<dyn TourPlanner> {
    Box::new(everymap_providers_google::GoogleTourPlanner)
}

fn create_google_tile_provider(_client: Arc<GoogleClient>) -> Box<dyn TileProvider> {
    Box::new(everymap_providers_google::GoogleTileProvider)
}

fn create_google_attribute_provider(client: Arc<GoogleClient>) -> Box<dyn AttributeProvider> {
    Box::new(everymap_providers_google::GoogleAttributeProvider::new(client))
}

fn create_google_image_provider(client: Arc<GoogleClient>) -> Box<dyn MapImageProvider> {
    Box::new(everymap_providers_google::GoogleMapImageProvider::new(client))
}

#[derive(Parser)]
#[command(name = "everymap", version, about = "Geospatial API CLI - unified interface for map providers")]
struct Cli {
    /// Provider to use (here, google)
    #[arg(long, default_value = "here")]
    provider: String,

    /// API key for authentication
    #[arg(long, env = "EVERYMAP_API_KEY")]
    api_key: Option<String>,

    /// API key parameter name (default: key for Google, apiKey for HERE)
    #[arg(long)]
    api_key_param: Option<String>,

    /// Output format
    #[arg(long, default_value = "json")]
    output: String,

    /// Enable verbose output (show request URLs and response bodies on stderr)
    #[arg(long, short)]
    verbose: bool,

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
    /// Match a GPS trace to the road network
    MatchRoute {
        /// Trace points as semicolon-separated lat,lng pairs (e.g., "52.5,13.3;52.6,13.4")
        #[arg(long)]
        trace: String,
    },
    /// Optimize a tour visiting multiple stops
    Tour {
        /// Stop coordinates as space-separated lat,lng pairs
        #[arg(long, num_args = 1..)]
        stops: Vec<String>,
    },
    /// Get a map tile
    Tile {
        /// Zoom level
        #[arg(long)]
        z: u32,
        /// X coordinate
        #[arg(long)]
        x: u32,
        /// Y coordinate
        #[arg(long)]
        y: u32,
    },
    /// Query road attributes
    Attributes {
        /// Bounding box as "south,west;north,east" or "lat,lng;lat,lng"
        #[arg(long)]
        bbox: Option<String>,
        /// Attribute layer (roads, segments, adminAreas, buildings, landmarks)
        #[arg(long, default_value = "roads")]
        layer: String,
        /// Response format (json, geojson)
        #[arg(long, default_value = "json")]
        format: String,
        /// Specific feature IDs to retrieve
        #[arg(long)]
        ids: Option<Vec<String>>,
        /// Include specific attribute fields (comma-separated)
        #[arg(long)]
        include: Option<String>,
    },
    /// Get a static map image
    MapImage {
        /// Center latitude
        #[arg(long)]
        lat: f64,
        /// Center longitude
        #[arg(long)]
        lng: f64,
        /// Zoom level
        #[arg(long, default_value = "14")]
        zoom: u32,
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

fn parse_transport_mode(transport: &str) -> CoreTransportMode {
    match transport {
        "car" => CoreTransportMode::Car,
        "truck" => CoreTransportMode::Truck,
        "pedestrian" => CoreTransportMode::Pedestrian,
        "bicycle" => CoreTransportMode::Bicycle,
        "scooter" => CoreTransportMode::Scooter,
        "bus" => CoreTransportMode::Bus,
        "taxi" => CoreTransportMode::Taxi,
        _ => CoreTransportMode::Car,
    }
}

fn print_output(value: &serde_json::Value, format: &output::OutputFormat) {
    println!("{}", output::format_output(value, format));
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let api_key = match cli.api_key.clone() {
        Some(key) => key,
        None => {
            // Load config and try provider-specific key, then env var
            let cfg = config::Config::load();
            match cfg.resolve_api_key(&None, &cli.provider, "EVERYMAP_API_KEY") {
                Some(key) => key,
                None => {
                    eprintln!("Error: API key required. Use --api-key, set EVERYMAP_API_KEY env var, or configure ~/.everymap/config.toml");
                    std::process::exit(1);
                }
            }
        }
    };

    if cli.provider != "here" && cli.provider != "google" {
        eprintln!("Error: Unsupported provider '{}'. Supported providers: here, google", cli.provider);
        std::process::exit(1);
    }

    // Default API key param name varies by provider
    let key_param = cli.api_key_param.clone().unwrap_or_else(|| {
        if cli.provider == "google" { "key".to_string() } else { "apiKey".to_string() }
    });

    let auth = Arc::new(ApiKeyProvider::new(api_key, key_param));
    let fmt = output::OutputFormat::from_str(&cli.output);

    match cli.provider.as_str() {
        "here" => {
            let mut client = HereClient::new(auth);
            client.set_verbose(cli.verbose);
            let client = Arc::new(client);
            run_here_commands(&cli, client, &fmt).await;
        }
        "google" => {
            let mut client = GoogleClient::new(auth);
            client.set_verbose(cli.verbose);
            let client = Arc::new(client);
            run_google_commands(&cli, client, &fmt).await;
        }
        _ => unreachable!(),
    }
}

async fn run_here_commands(cli: &Cli, client: Arc<HereClient>, fmt: &output::OutputFormat) {
    match &cli.command {
        Commands::Geocode { query } => {
            let geocoder = create_here_geocoder(client);
            let opts = GeocodeOptions::default();
            match geocoder.geocode(query, &opts).await {
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
                    print_output(&serde_json::json!({ "results": output }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::ReverseGeocode { lat, lng } => {
            let geocoder = create_here_geocoder(client);
            let coord = match Coordinate::new(*lat, *lng) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid coordinates: {}", e);
                    std::process::exit(1);
                }
            };
            let opts = ReverseGeocodeOptions::default();
            match geocoder.reverse_geocode(&coord, &opts).await {
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
                    print_output(&serde_json::json!({ "results": output }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Route { origin, destination, transport } => {
            let router = create_here_router(client);
            let start = match parse_coordinate(origin) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid origin: {}", e);
                    std::process::exit(1);
                }
            };
            let end = match parse_coordinate(destination) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid destination: {}", e);
                    std::process::exit(1);
                }
            };
            let transport_mode = parse_transport_mode(transport);
            let opts = RouteOptions {
                transport_mode: Some(transport_mode),
                ..Default::default()
            };
            match router.calculate_route(&start, &end, &opts).await {
                Ok(res) => {
                    let routes: Vec<serde_json::Value> = res.routes.iter().map(|r| {
                        serde_json::json!({
                            "distance_m": r.distance,
                            "duration_s": r.duration,
                            "points": r.geometry.points.len(),
                        })
                    }).collect();
                    print_output(&serde_json::json!({ "routes": routes }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Traffic { lat, lng } => {
            let traffic = create_here_traffic_provider(client);
            let coord = match Coordinate::new(*lat, *lng) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid coordinates: {}", e);
                    std::process::exit(1);
                }
            };
            let opts = TrafficOptions::default();
            match traffic.get_traffic(&coord, &opts).await {
                Ok(res) => {
                    let flows: Vec<serde_json::Value> = res.flows.iter().map(|f| {
                        serde_json::json!({
                            "jam_factor": f.jam_factor,
                            "speed": f.speed,
                            "free_flow_speed": f.free_flow_speed,
                            "road_name": f.road_name,
                        })
                    }).collect();
                    print_output(&serde_json::json!({ "flows": flows, "incidents": res.incidents.len() }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Position => {
            let positioner = create_here_positioner(client);
            let opts = PositioningOptions::default();
            match positioner.get_position(&opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({
                        "coordinate": { "lat": res.coordinate.lat, "lng": res.coordinate.lng },
                        "accuracy": res.accuracy,
                        "altitude": res.altitude,
                    }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Isoline { lat, lng, range } => {
            let isoline = create_here_isoline_provider(client);
            let center = match Coordinate::new(*lat, *lng) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid coordinates: {}", e);
                    std::process::exit(1);
                }
            };
            let opts = IsolineOptions {
                range_type: Some(CoreRangeType::Distance),
                ..Default::default()
            };
            match isoline.get_isoline(&center, *range, &opts).await {
                Ok(res) => {
                    let isolines: Vec<serde_json::Value> = res.isolines.iter().map(|iso| {
                        serde_json::json!({
                            "range": iso.range,
                            "points": iso.polygon.len(),
                        })
                    }).collect();
                    print_output(&serde_json::json!({ "isolines": isolines }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::MatchRoute { trace } => {
            let matcher = create_here_route_matcher(client);
            let points: Vec<Coordinate> = trace.split(';')
                .filter_map(|p| parse_coordinate(p.trim()).ok())
                .collect();
            if points.is_empty() {
                eprintln!("Error: No valid coordinates in trace. Use format: 'lat,lng;lat,lng'");
                std::process::exit(1);
            }
            let opts = MatchingOptions::default();
            match matcher.match_route(&points, &opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({
                        "matched_points": res.matched_points.len(),
                        "distance_m": res.distance,
                        "duration_s": res.duration,
                    }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Tour { stops } => {
            let planner = create_here_tour_planner(client);
            let coordinates: Vec<Coordinate> = stops.iter()
                .filter_map(|s| parse_coordinate(s).ok())
                .collect();
            if coordinates.len() < 2 {
                eprintln!("Error: At least 2 stops required for tour optimization");
                std::process::exit(1);
            }
            let opts = TourOptions::default();
            match planner.optimize_tour(&coordinates, &opts).await {
                Ok(res) => {
                    let stop_list: Vec<serde_json::Value> = res.stops.iter().map(|s| {
                        serde_json::json!({
                            "coordinate": { "lat": s.coordinate.lat, "lng": s.coordinate.lng },
                            "arrival_time": s.arrival_time,
                            "departure_time": s.departure_time,
                        })
                    }).collect();
                    print_output(&serde_json::json!({
                        "stops": stop_list,
                        "total_distance": res.total_distance,
                        "total_duration": res.total_duration,
                        "unassigned_count": res.unassigned_count,
                    }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Tile { z, x, y } => {
            let tile_provider = create_here_tile_provider(client);
            let opts = TileOptions::default();
            match tile_provider.get_tile(*z, *x, *y, &opts).await {
                Ok(res) => {
                    println!("Retrieved tile ({} bytes, content_type: {:?})", res.data.len(), res.content_type);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Attributes { bbox, layer, format, ids, include } => {
            let attr_provider = create_here_attribute_provider(client);
            let mut provider_extra = serde_json::json!({
                "layer": layer,
                "format": format,
            });
            if let Some(ids) = ids {
                provider_extra["ids"] = serde_json::json!(ids);
            }
            if let Some(include) = include {
                provider_extra["include"] = serde_json::json!(include.split(',').collect::<Vec<_>>());
            }
            let opts = AttributeOptions {
                bbox: bbox.clone(),
                provider_extra: Some(provider_extra),
                ..Default::default()
            };
            match attr_provider.get_attributes(&opts).await {
                Ok(res) => {
                    print_output(&res.data, fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::MapImage { lat, lng, zoom } => {
            let image_provider = create_here_image_provider(client);
            let center = match Coordinate::new(*lat, *lng) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid coordinates: {}", e);
                    std::process::exit(1);
                }
            };
            let opts = ImageOptions::default();
            match image_provider.get_image(&center, *zoom, (800, 600), &opts).await {
                Ok(res) => {
                    println!("Retrieved map image ({} bytes, content_type: {:?})", res.data.len(), res.content_type);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
}

async fn run_google_commands(cli: &Cli, client: Arc<GoogleClient>, fmt: &output::OutputFormat) {
    match &cli.command {
        Commands::Geocode { query } => {
            let geocoder = create_google_geocoder(client);
            let opts = GeocodeOptions::default();
            match geocoder.geocode(query, &opts).await {
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
                    print_output(&serde_json::json!({ "results": output }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::ReverseGeocode { lat, lng } => {
            let geocoder = create_google_geocoder(client);
            let coord = match Coordinate::new(*lat, *lng) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid coordinates: {}", e);
                    std::process::exit(1);
                }
            };
            let opts = ReverseGeocodeOptions::default();
            match geocoder.reverse_geocode(&coord, &opts).await {
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
                    print_output(&serde_json::json!({ "results": output }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Route { origin, destination, transport } => {
            let router = create_google_router(client);
            let start = match parse_coordinate(origin) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid origin: {}", e);
                    std::process::exit(1);
                }
            };
            let end = match parse_coordinate(destination) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid destination: {}", e);
                    std::process::exit(1);
                }
            };
            let transport_mode = parse_transport_mode(transport);
            let opts = RouteOptions {
                transport_mode: Some(transport_mode),
                ..Default::default()
            };
            match router.calculate_route(&start, &end, &opts).await {
                Ok(res) => {
                    let routes: Vec<serde_json::Value> = res.routes.iter().map(|r| {
                        serde_json::json!({
                            "distance_m": r.distance,
                            "duration_s": r.duration,
                            "points": r.geometry.points.len(),
                        })
                    }).collect();
                    print_output(&serde_json::json!({ "routes": routes }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        // Unsupported domains return UnsupportedDomain error from the trait impl
        Commands::Traffic { .. } => {
            let traffic = create_google_traffic_provider(client);
            let opts = TrafficOptions::default();
            match traffic.get_traffic(&Coordinate::new(0.0, 0.0).unwrap(), &opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Position => {
            let positioner = create_google_positioner(client);
            let opts = PositioningOptions::default();
            match positioner.get_position(&opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({
                        "coordinate": { "lat": res.coordinate.lat, "lng": res.coordinate.lng },
                        "accuracy": res.accuracy,
                        "altitude": res.altitude,
                    }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Isoline { .. } => {
            let isoline = create_google_isoline_provider(client);
            let opts = IsolineOptions::default();
            match isoline.get_isoline(&Coordinate::new(0.0, 0.0).unwrap(), 1000.0, &opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::MatchRoute { trace } => {
            let matcher = create_google_route_matcher(client);
            let points: Vec<Coordinate> = trace.split(';')
                .filter_map(|p| parse_coordinate(p.trim()).ok())
                .collect();
            if points.is_empty() {
                eprintln!("Error: No valid coordinates in trace. Use format: 'lat,lng;lat,lng'");
                std::process::exit(1);
            }
            let opts = MatchingOptions::default();
            match matcher.match_route(&points, &opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({
                        "matched_points": res.matched_points.len(),
                        "distance_m": res.distance,
                        "duration_s": res.duration,
                    }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Tour { .. } => {
            let planner = create_google_tour_planner(client);
            let opts = TourOptions::default();
            match planner.optimize_tour(&[], &opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Tile { z, x, y } => {
            let tile_provider = create_google_tile_provider(client);
            let opts = TileOptions::default();
            match tile_provider.get_tile(*z, *x, *y, &opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Attributes { bbox, layer: _, format: _, ids, include: _ } => {
            let attr_provider = create_google_attribute_provider(client);
            let mut provider_extra = serde_json::json!({});
            if let Some(ids) = ids {
                provider_extra["place_ids"] = serde_json::json!(ids);
            }
            let opts = AttributeOptions {
                bbox: bbox.clone(),
                provider_extra: Some(provider_extra),
                ..Default::default()
            };
            match attr_provider.get_attributes(&opts).await {
                Ok(res) => {
                    print_output(&res.data, fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::MapImage { lat, lng, zoom } => {
            let image_provider = create_google_image_provider(client);
            let center = match Coordinate::new(*lat, *lng) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Invalid coordinates: {}", e);
                    std::process::exit(1);
                }
            };
            let opts = ImageOptions::default();
            match image_provider.get_image(&center, *zoom, (800, 600), &opts).await {
                Ok(res) => {
                    println!("Retrieved map image ({} bytes, content_type: {:?})", res.data.len(), res.content_type);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
}