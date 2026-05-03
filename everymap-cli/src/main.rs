mod config;
mod output;
mod provider;

use clap::{Parser, Subcommand};
use everymap_core::domains::attributes::AttributeOptions;
use everymap_core::domains::imaging::ImageOptions;
use everymap_core::domains::isoline::{IsolineOptions, RangeType as CoreRangeType};
use everymap_core::domains::matching::MatchingOptions;
use everymap_core::domains::positioning::PositioningOptions;
use everymap_core::domains::routing::{RouteOptions, TransportMode as CoreTransportMode};
use everymap_core::domains::search::{GeocodeOptions, ReverseGeocodeOptions};
use everymap_core::domains::tiling::TileOptions;
use everymap_core::domains::tour::TourOptions;
use everymap_core::domains::traffic::TrafficOptions;
use everymap_core::types::Coordinate;
use provider::ProviderRegistry;

#[derive(Parser)]
#[command(
    name = "everymap",
    version,
    about = "Geospatial API CLI - unified interface for map providers"
)]
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
        /// Search radius in meters
        #[arg(long)]
        radius: Option<f64>,
        /// Include incident data (true/false)
        #[arg(long)]
        include_incidents: Option<bool>,
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
        /// Range value (meters for distance, seconds for time)
        #[arg(long, default_value = "1000")]
        range: f64,
        /// Transport mode (car, truck, pedestrian, bicycle) — default: car
        #[arg(long, default_value = "car")]
        transport: String,
        /// Range type (distance, time) — default: distance
        #[arg(long, default_value = "distance")]
        range_type: String,
    },
    /// Match a GPS trace to the road network
    MatchRoute {
        /// Trace points as semicolon-separated lat,lng pairs (e.g., "52.5,13.3;52.6,13.4")
        #[arg(long)]
        trace: String,
        /// Transport mode for matching (car, truck, pedestrian, bicycle) — default: car
        #[arg(long, default_value = "car")]
        transport: String,
    },
    /// Optimize a tour visiting multiple stops
    Tour {
        /// Stop coordinates as space-separated lat,lng pairs
        #[arg(long, num_args = 1..)]
        stops: Vec<String>,
        /// Transport mode (car, truck, pedestrian, bicycle) — default: car
        #[arg(long, default_value = "car")]
        transport: String,
        /// Departure time (ISO 8601, e.g., "2026-04-21T08:00:00Z") — default: now
        #[arg(long)]
        departure: Option<String>,
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
        /// Tile layer (provider-specific: HERE uses base/core/hybrid, TomTom uses basic/hybrid/labels)
        #[arg(long)]
        layer: Option<String>,
        /// Output file path (default: tile.omv)
        #[arg(long, default_value = "tile.omv")]
        output_file: String,
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
        /// Output file path (default: map.png)
        #[arg(long, default_value = "map.png")]
        output_file: String,
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
    use std::io::stdout;
    let mut out = stdout();
    if let Err(e) = output::write_output(&mut out, value, format) {
        eprintln!("Output error: {}", e);
    }
}

fn exit_with_coord_error(message: &str) -> ! {
    eprintln!("Invalid coordinates: {}", message);
    std::process::exit(1);
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("Error: {}", message);
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let api_key = match cli.api_key.clone() {
        Some(key) => key,
        None => {
            let config = config::Config::load();
            match config.resolve_api_key(&None, &cli.provider, "EVERYMAP_API_KEY") {
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

    let key_param = cli
        .api_key_param
        .clone()
        .unwrap_or_else(|| match cli.provider.as_str() {
            "google" | "tomtom" => "key".to_string(),
            "mapbox" => "access_token".to_string(),
            _ => "apiKey".to_string(),
        });

    let output_format = output::OutputFormat::from_str(&cli.output);
    let registry = ProviderRegistry::new(&cli.provider, api_key, key_param, cli.verbose);
    run_commands(&cli, &registry, &output_format).await;
}

async fn run_commands(
    cli: &Cli,
    registry: &ProviderRegistry,
    output_format: &output::OutputFormat,
) {
    match &cli.command {
        Commands::Geocode { query } => {
            let geocoder = registry.geocoder();
            let options = GeocodeOptions::default();
            match geocoder.geocode(query, &options).await {
                Ok(result) => {
                    let output: Vec<serde_json::Value> = result.items.iter().map(|item| {
                        serde_json::json!({
                            "id": item.id,
                            "title": item.title,
                            "coordinate": { "lat": item.coordinate.lat, "lng": item.coordinate.lng },
                            "address": item.address,
                            "result_type": format!("{:?}", item.result_type),
                            "distance": item.distance,
                        })
                    }).collect();
                    print_output(&serde_json::json!({ "results": output }), output_format);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::ReverseGeocode { lat, lng } => {
            let geocoder = registry.geocoder();
            let coordinate = Coordinate::new(*lat, *lng)
                .unwrap_or_else(|e| exit_with_coord_error(&e.to_string()));
            let options = ReverseGeocodeOptions::default();
            match geocoder.reverse_geocode(&coordinate, &options).await {
                Ok(result) => {
                    let output: Vec<serde_json::Value> = result.items.iter().map(|item| {
                        serde_json::json!({
                            "id": item.id,
                            "title": item.title,
                            "coordinate": { "lat": item.coordinate.lat, "lng": item.coordinate.lng },
                            "address": item.address,
                            "result_type": format!("{:?}", item.result_type),
                        })
                    }).collect();
                    print_output(&serde_json::json!({ "results": output }), output_format);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Route {
            origin,
            destination,
            transport,
        } => {
            let router = registry.router();
            let start = parse_coordinate(origin)
                .unwrap_or_else(|e| exit_with_error(&format!("Invalid origin: {}", e)));
            let end = parse_coordinate(destination)
                .unwrap_or_else(|e| exit_with_error(&format!("Invalid destination: {}", e)));
            let transport_mode = parse_transport_mode(transport);
            let options = RouteOptions {
                transport_mode: Some(transport_mode),
                ..Default::default()
            };
            match router.calculate_route(&start, &end, &options).await {
                Ok(result) => {
                    let routes: Vec<serde_json::Value> = result
                        .routes
                        .iter()
                        .map(|r| {
                            serde_json::json!({
                                "distance_m": r.distance,
                                "duration_s": r.duration,
                                "points": r.geometry.points.len(),
                            })
                        })
                        .collect();
                    print_output(&serde_json::json!({ "routes": routes }), output_format);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Traffic { lat, lng, radius, include_incidents } => {
            let traffic = registry.traffic();
            let coordinate = Coordinate::new(*lat, *lng)
                .unwrap_or_else(|e| exit_with_coord_error(&e.to_string()));
            let options = TrafficOptions {
                radius: *radius,
                include_incidents: *include_incidents,
                ..Default::default()
            };
            match traffic.get_traffic(&coordinate, &options).await {
                Ok(result) => {
                    let flows: Vec<serde_json::Value> = result
                        .flows
                        .iter()
                        .map(|f| {
                            serde_json::json!({
                                "jam_factor": f.jam_factor,
                                "speed": f.speed,
                                "free_flow_speed": f.free_flow_speed,
                                "road_name": f.road_name,
                            })
                        })
                        .collect();
                    print_output(
                        &serde_json::json!({ "flows": flows, "incidents": result.incidents.len() }),
                        output_format,
                    );
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Position => {
            let positioner = registry.positioner();
            let options = PositioningOptions::default();
            match positioner.get_position(&options).await {
                Ok(result) => {
                    print_output(
                        &serde_json::json!({
                            "coordinate": { "lat": result.coordinate.lat, "lng": result.coordinate.lng },
                            "accuracy": result.accuracy,
                            "altitude": result.altitude,
                        }),
                        output_format,
                    );
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Isoline { lat, lng, range, transport, range_type } => {
            let isoline = registry.isoline();
            let center = Coordinate::new(*lat, *lng)
                .unwrap_or_else(|e| exit_with_coord_error(&e.to_string()));
            let transport_mode = parse_transport_mode(transport);
            let core_range_type = match range_type.as_str() {
                "time" => CoreRangeType::Time,
                _ => CoreRangeType::Distance,
            };
            let options = IsolineOptions {
                range_type: Some(core_range_type),
                transport_mode: Some(transport_mode),
                ..Default::default()
            };
            match isoline.get_isoline(&center, *range, &options).await {
                Ok(result) => {
                    let isolines: Vec<serde_json::Value> = result.isolines.iter().map(|iso| {
                        serde_json::json!({ "range": iso.range, "points": iso.polygon.len() })
                    }).collect();
                    print_output(&serde_json::json!({ "isolines": isolines }), output_format);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::MatchRoute { trace, transport } => {
            let matcher = registry.route_matcher();
            let points: Vec<Coordinate> = trace
                .split(';')
                .filter_map(|p| parse_coordinate(p.trim()).ok())
                .collect();
            if points.is_empty() {
                exit_with_error("No valid coordinates in trace. Use format: 'lat,lng;lat,lng'");
            }
            let transport_mode = parse_transport_mode(transport);
            let options = MatchingOptions {
                transport_mode: Some(transport_mode),
                provider_extra: Some(serde_json::json!({
                    "transport_mode": transport
                })),
                ..Default::default()
            };
            match matcher.match_route(&points, &options).await {
                Ok(result) => {
                    print_output(
                        &serde_json::json!({
                            "matched_points": result.matched_points.len(),
                            "distance_m": result.distance,
                            "duration_s": result.duration,
                        }),
                        output_format,
                    );
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Tour { stops, transport, departure } => {
            let planner = registry.tour_planner();
            let coordinates: Vec<Coordinate> = stops
                .iter()
                .filter_map(|s| parse_coordinate(s).ok())
                .collect();
            if coordinates.len() < 2 {
                exit_with_error("At least 2 stops required for tour optimization");
            }
            let transport_mode = parse_transport_mode(transport);
            let mut provider_extra = serde_json::json!({});
            if let Some(departure_time) = departure {
                provider_extra["departure_time"] = serde_json::json!(departure_time);
            }
            let options = TourOptions {
                transport_mode: Some(transport_mode),
                provider_extra: Some(provider_extra),
            };
            match planner.optimize_tour(&coordinates, &options).await {
                Ok(result) => {
                    let stop_list: Vec<serde_json::Value> = result
                        .stops
                        .iter()
                        .map(|s| {
                            serde_json::json!({
                                "coordinate": { "lat": s.coordinate.lat, "lng": s.coordinate.lng },
                                "arrival_time": s.arrival_time,
                                "departure_time": s.departure_time,
                            })
                        })
                        .collect();
                    print_output(
                        &serde_json::json!({
                            "stops": stop_list,
                            "total_distance": result.total_distance,
                            "total_duration": result.total_duration,
                            "unassigned_count": result.unassigned_count,
                        }),
                        output_format,
                    );
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Tile {
            z,
            x,
            y,
            layer,
            output_file,
        } => {
            let tile_provider = registry.tile_provider();
            let mut provider_extra = serde_json::json!({});
            if let Some(ref layer_val) = layer {
                provider_extra["layer"] = serde_json::json!(layer_val);
            }
            let options = TileOptions {
                format: None,
                provider_extra: Some(provider_extra),
            };
            match tile_provider.get_tile(*z, *x, *y, &options).await {
                Ok(result) => match std::fs::write(output_file, &result.data) {
                    Ok(()) => println!(
                        "Saved tile to {} ({} bytes, {})",
                        output_file,
                        result.data.len(),
                        result.content_type.as_deref().unwrap_or("unknown")
                    ),
                    Err(e) => eprintln!("Error writing file {}: {}", output_file, e),
                },
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Attributes {
            bbox,
            layer,
            format,
            ids,
            include,
        } => {
            let attr_provider = registry.attribute_provider();
            let mut provider_extra = serde_json::json!({
                "layer": layer,
                "format": format,
            });
            if let Some(ids) = ids {
                provider_extra["ids"] = serde_json::json!(ids);
            }
            if let Some(include) = include {
                provider_extra["include"] =
                    serde_json::json!(include.split(',').collect::<Vec<_>>());
            }
            let options = AttributeOptions {
                bbox: bbox.clone(),
                provider_extra: Some(provider_extra),
                ..Default::default()
            };
            match attr_provider.get_attributes(&options).await {
                Ok(result) => {
                    print_output(&result.data, output_format);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::MapImage {
            lat,
            lng,
            zoom,
            output_file,
        } => {
            let image_provider = registry.image_provider();
            let center = Coordinate::new(*lat, *lng)
                .unwrap_or_else(|e| exit_with_coord_error(&e.to_string()));
            let options = ImageOptions::default();
            match image_provider
                .get_image(&center, *zoom, (800, 600), &options)
                .await
            {
                Ok(result) => match std::fs::write(output_file, &result.data) {
                    Ok(()) => println!(
                        "Saved map image to {} ({} bytes, {})",
                        output_file,
                        result.data.len(),
                        result.content_type.as_deref().unwrap_or("unknown")
                    ),
                    Err(e) => eprintln!("Error writing file {}: {}", output_file, e),
                },
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
}
