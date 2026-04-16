mod config;
mod output;
mod provider;

use clap::{Parser, Subcommand};
use everymap_core::domains::search::{GeocodeOptions, ReverseGeocodeOptions};
use everymap_core::domains::routing::{RouteOptions, TransportMode as CoreTransportMode};
use everymap_core::domains::traffic::TrafficOptions;
use everymap_core::domains::positioning::PositioningOptions;
use everymap_core::domains::isoline::{IsolineOptions, RangeType as CoreRangeType};
use everymap_core::domains::matching::MatchingOptions;
use everymap_core::domains::tour::TourOptions;
use everymap_core::domains::tiling::TileOptions;
use everymap_core::domains::attributes::AttributeOptions;
use everymap_core::domains::imaging::ImageOptions;
use everymap_core::domains::geofencing::{GeofenceOptions, GeofenceCreateOptions, GeofenceType};
use everymap_core::domains::tracking::{TripCreateOptions, TripUpdateOptions, TripStatus};
use everymap_core::domains::fraud::FraudCheckOptions;
use everymap_core::types::Coordinate;
use provider::ProviderRegistry;

#[derive(Parser)]
#[command(name = "everymap", version, about = "Geospatial API CLI - unified interface for map providers")]
struct Cli {
    /// Provider to use (here, google, tomtom, mapbox, radar)
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
    /// Search for geofences near a location
    GeofenceSearch {
        /// Latitude
        #[arg(long)]
        lat: f64,
        /// Longitude
        #[arg(long)]
        lng: f64,
        /// Search radius in meters
        #[arg(long)]
        radius: Option<f64>,
        /// Filter by tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
        /// Maximum number of results
        #[arg(long)]
        limit: Option<u32>,
    },
    /// Create a circle geofence
    GeofenceCreate {
        /// Center latitude
        #[arg(long)]
        lat: f64,
        /// Center longitude
        #[arg(long)]
        lng: f64,
        /// Radius in meters
        #[arg(long)]
        radius: f64,
        /// Tag for grouping
        #[arg(long)]
        tag: Option<String>,
        /// Description
        #[arg(long)]
        description: Option<String>,
    },
    /// Get a geofence by ID
    GeofenceGet {
        /// Geofence ID
        id: String,
    },
    /// Delete a geofence by ID
    GeofenceDelete {
        /// Geofence ID
        id: String,
    },
    /// Create a new trip for tracking
    TripCreate {
        /// Origin coordinates (lat,lng)
        #[arg(long)]
        origin: Option<String>,
        /// Destination coordinates (lat,lng)
        #[arg(long)]
        destination: Option<String>,
        /// Travel mode (car, foot, bike)
        #[arg(long, default_value = "car")]
        mode: String,
        /// External ID for linking
        #[arg(long)]
        external_id: Option<String>,
        /// Tag for grouping
        #[arg(long)]
        tag: Option<String>,
    },
    /// Update a trip's status
    TripUpdate {
        /// Trip ID to update
        #[arg(long)]
        trip_id: String,
        /// New status (pending, started, approaching, arrived, completed)
        #[arg(long)]
        status: String,
    },
    /// Get a trip by ID
    TripGet {
        /// Trip ID
        id: String,
    },
    /// Check for location fraud
    FraudCheck {
        /// Device ID
        #[arg(long)]
        device_id: String,
        /// Latitude
        #[arg(long)]
        lat: f64,
        /// Longitude
        #[arg(long)]
        lng: f64,
        /// Accuracy in meters
        #[arg(long, default_value = "10.0")]
        accuracy: f64,
        /// User ID (optional)
        #[arg(long)]
        user_id: Option<String>,
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

fn exit_with_coord_error(msg: &str) -> ! {
    eprintln!("Invalid coordinates: {}", msg);
    std::process::exit(1);
}

fn exit_with_error(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let api_key = match cli.api_key.clone() {
        Some(key) => key,
        None => {
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

    if !["here", "google", "tomtom", "mapbox", "radar"].contains(&cli.provider.as_str()) {
        eprintln!("Error: Unsupported provider '{}'. Supported providers: here, google, tomtom, mapbox, radar", cli.provider);
        std::process::exit(1);
    }

    let key_param = cli.api_key_param.clone().unwrap_or_else(|| {
        match cli.provider.as_str() {
            "google" | "tomtom" => "key".to_string(),
            "mapbox" => "access_token".to_string(),
            _ => "apiKey".to_string(),
        }
    });

    let fmt = output::OutputFormat::from_str(&cli.output);
    let registry = ProviderRegistry::new(&cli.provider, api_key, key_param, cli.verbose);
    run_commands(&cli, &registry, &fmt).await;
}

async fn run_commands(cli: &Cli, registry: &ProviderRegistry, fmt: &output::OutputFormat) {
    match &cli.command {
        Commands::Geocode { query } => {
            let geocoder = registry.geocoder();
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
            let geocoder = registry.geocoder();
            let coord = Coordinate::new(*lat, *lng).unwrap_or_else(|e| exit_with_coord_error(&e.to_string()));
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
            let router = registry.router();
            let start = parse_coordinate(origin).unwrap_or_else(|e| exit_with_error(&format!("Invalid origin: {}", e)));
            let end = parse_coordinate(destination).unwrap_or_else(|e| exit_with_error(&format!("Invalid destination: {}", e)));
            let transport_mode = parse_transport_mode(transport);
            let opts = RouteOptions { transport_mode: Some(transport_mode), ..Default::default() };
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
            let traffic = registry.traffic();
            let coord = Coordinate::new(*lat, *lng).unwrap_or_else(|e| exit_with_coord_error(&e.to_string()));
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
            let positioner = registry.positioner();
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
            let isoline = registry.isoline();
            let center = Coordinate::new(*lat, *lng).unwrap_or_else(|e| exit_with_coord_error(&e.to_string()));
            let opts = IsolineOptions { range_type: Some(CoreRangeType::Distance), ..Default::default() };
            match isoline.get_isoline(&center, *range, &opts).await {
                Ok(res) => {
                    let isolines: Vec<serde_json::Value> = res.isolines.iter().map(|iso| {
                        serde_json::json!({ "range": iso.range, "points": iso.polygon.len() })
                    }).collect();
                    print_output(&serde_json::json!({ "isolines": isolines }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::MatchRoute { trace } => {
            let matcher = registry.route_matcher();
            let points: Vec<Coordinate> = trace.split(';')
                .filter_map(|p| parse_coordinate(p.trim()).ok())
                .collect();
            if points.is_empty() {
                exit_with_error("No valid coordinates in trace. Use format: 'lat,lng;lat,lng'");
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
            let planner = registry.tour_planner();
            let coordinates: Vec<Coordinate> = stops.iter()
                .filter_map(|s| parse_coordinate(s).ok())
                .collect();
            if coordinates.len() < 2 {
                exit_with_error("At least 2 stops required for tour optimization");
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
            let tile_provider = registry.tile_provider();
            let opts = TileOptions::default();
            match tile_provider.get_tile(*z, *x, *y, &opts).await {
                Ok(res) => {
                    println!("Retrieved tile ({} bytes, content_type: {:?})", res.data.len(), res.content_type);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Attributes { bbox, layer, format, ids, include } => {
            let attr_provider = registry.attribute_provider();
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
            let image_provider = registry.image_provider();
            let center = Coordinate::new(*lat, *lng).unwrap_or_else(|e| exit_with_coord_error(&e.to_string()));
            let opts = ImageOptions::default();
            match image_provider.get_image(&center, *zoom, (800, 600), &opts).await {
                Ok(res) => {
                    println!("Retrieved map image ({} bytes, content_type: {:?})", res.data.len(), res.content_type);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::GeofenceSearch { lat, lng, radius, tags, limit } => {
            let provider = match registry.geofence() {
                Some(p) => p,
                None => { eprintln!("Error: Geofence search is not supported by this provider. Use --provider radar"); return; }
            };
            let near = Coordinate::new(*lat, *lng).unwrap_or_else(|e| exit_with_coord_error(&e.to_string()));
            let opts = GeofenceOptions {
                near: Some(near),
                radius: *radius,
                tags: tags.as_ref().map(|t| t.split(',').map(String::from).collect()).unwrap_or_default(),
                limit: *limit,
                ..Default::default()
            };
            match provider.search_geofences(&opts).await {
                Ok(res) => {
                    let geofences: Vec<serde_json::Value> = res.geofences.iter().map(|g| {
                        serde_json::json!({
                            "id": g.id,
                            "tag": g.tag,
                            "description": g.description,
                            "enabled": g.enabled,
                        })
                    }).collect();
                    print_output(&serde_json::json!({ "geofences": geofences }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::GeofenceCreate { lat, lng, radius, tag, description } => {
            let provider = match registry.geofence() {
                Some(p) => p,
                None => { eprintln!("Error: Geofence creation is not supported by this provider. Use --provider radar"); return; }
            };
            let center = Coordinate::new(*lat, *lng).unwrap_or_else(|e| exit_with_coord_error(&e.to_string()));
            let opts = GeofenceCreateOptions {
                center: Some(center),
                radius: Some(*radius),
                tag: tag.clone(),
                description: description.clone(),
                geofence_type: Some(GeofenceType::Circle),
                ..Default::default()
            };
            match provider.create_geofence(&opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({
                        "id": res.id,
                        "tag": res.tag,
                        "description": res.description,
                        "enabled": res.enabled,
                    }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::GeofenceGet { id } => {
            let provider = match registry.geofence() {
                Some(p) => p,
                None => { eprintln!("Error: Geofence get is not supported by this provider. Use --provider radar"); return; }
            };
            match provider.get_geofence(id).await {
                Ok(res) => {
                    print_output(&serde_json::json!({
                        "id": res.id,
                        "tag": res.tag,
                        "description": res.description,
                        "enabled": res.enabled,
                    }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::GeofenceDelete { id } => {
            let provider = match registry.geofence() {
                Some(p) => p,
                None => { eprintln!("Error: Geofence deletion is not supported by this provider. Use --provider radar"); return; }
            };
            match provider.delete_geofence(id).await {
                Ok(()) => println!("Geofence '{}' deleted", id),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::TripCreate { origin, destination, mode, external_id, tag } => {
            let tracker = match registry.trip_tracker() {
                Some(t) => t,
                None => { eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar"); return; }
            };
            let opts = TripCreateOptions {
                origin: origin.as_ref().and_then(|o| parse_coordinate(o).ok()),
                destination: destination.as_ref().and_then(|d| parse_coordinate(d).ok()),
                mode: Some(mode.clone()),
                external_id: external_id.clone(),
                tag: tag.clone(),
                ..Default::default()
            };
            match tracker.create_trip(&opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({
                        "id": res.id,
                        "status": format!("{:?}", res.status),
                        "mode": res.mode,
                        "eta": res.eta,
                    }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::TripUpdate { trip_id, status } => {
            let tracker = match registry.trip_tracker() {
                Some(t) => t,
                None => { eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar"); return; }
            };
            let trip_status = match status.as_str() {
                "pending" => Some(TripStatus::Pending),
                "started" => Some(TripStatus::Started),
                "approaching" => Some(TripStatus::Approaching),
                "arrived" => Some(TripStatus::Arrived),
                "completed" => Some(TripStatus::Completed),
                _ => None,
            };
            let opts = TripUpdateOptions {
                trip_id: trip_id.clone(),
                status: trip_status,
                ..Default::default()
            };
            match tracker.update_trip(&opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({
                        "id": res.id,
                        "status": format!("{:?}", res.status),
                    }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::TripGet { id } => {
            let tracker = match registry.trip_tracker() {
                Some(t) => t,
                None => { eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar"); return; }
            };
            match tracker.get_trip(id).await {
                Ok(res) => {
                    print_output(&serde_json::json!({
                        "id": res.id,
                        "status": format!("{:?}", res.status),
                        "mode": res.mode,
                        "eta": res.eta,
                    }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::FraudCheck { device_id, lat, lng, accuracy, user_id } => {
            let detector = match registry.fraud_detector() {
                Some(d) => d,
                None => { eprintln!("Error: Fraud detection is not supported by this provider. Use --provider radar"); return; }
            };
            let opts = FraudCheckOptions {
                device_id: device_id.clone(),
                latitude: *lat,
                longitude: *lng,
                accuracy: *accuracy,
                user_id: user_id.clone(),
                ..Default::default()
            };
            match detector.check_fraud(&opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({
                        "verified": res.verified,
                        "passed": res.passed,
                        "mocked": res.mocked,
                        "jumped": res.jumped,
                        "compromised": res.compromised,
                        "inaccurate": res.inaccurate,
                        "proxy": res.proxy,
                        "blocked": res.blocked,
                    }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
}