mod config;
mod output;

use clap::{Parser, Subcommand};
use everymap_core::auth::{AuthProvider, ApiKeyProvider};
use everymap_core::auth::header::HeaderAuthProvider;
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
use everymap_core::domains::geofencing::{GeofenceProvider, GeofenceOptions, GeofenceCreateOptions, GeofenceType};
use everymap_core::domains::tracking::{TripTracker, TripCreateOptions, TripUpdateOptions, TripStatus};
use everymap_core::domains::fraud::{FraudDetector, FraudCheckOptions};
use everymap_core::types::Coordinate;
use everymap_providers_here::client::HereClient;
use everymap_providers_google::client::GoogleClient;
use everymap_providers_tomtom::client::TomTomClient;
use everymap_providers_mapbox::client::MapBoxClient;
use everymap_providers_radar::client::RadarClient;
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

// Factory functions for TomTom provider
fn create_tomtom_geocoder(client: Arc<TomTomClient>) -> Box<dyn Geocoder> {
    Box::new(everymap_providers_tomtom::TomTomGeocoder::new(client))
}

fn create_tomtom_router(client: Arc<TomTomClient>) -> Box<dyn Router> {
    Box::new(everymap_providers_tomtom::TomTomRouter::new(client))
}

fn create_tomtom_traffic_provider(client: Arc<TomTomClient>) -> Box<dyn TrafficProvider> {
    Box::new(everymap_providers_tomtom::TomTomTraffic::new(client))
}

fn create_tomtom_isoline_provider(client: Arc<TomTomClient>) -> Box<dyn IsolineProvider> {
    Box::new(everymap_providers_tomtom::TomTomIsoline::new(client))
}

fn create_tomtom_route_matcher(client: Arc<TomTomClient>) -> Box<dyn RouteMatcher> {
    Box::new(everymap_providers_tomtom::TomTomRouteMatcher::new(client))
}

fn create_tomtom_tour_planner(client: Arc<TomTomClient>) -> Box<dyn TourPlanner> {
    Box::new(everymap_providers_tomtom::TomTomTourPlanner::new(client))
}

fn create_tomtom_tile_provider(client: Arc<TomTomClient>) -> Box<dyn TileProvider> {
    Box::new(everymap_providers_tomtom::TomTomTileProvider::new(client))
}

fn create_tomtom_image_provider(client: Arc<TomTomClient>) -> Box<dyn MapImageProvider> {
    Box::new(everymap_providers_tomtom::TomTomMapImageProvider::new(client))
}

fn create_tomtom_positioner() -> Box<dyn NetworkPositionerTrait> {
    Box::new(everymap_providers_tomtom::TomTomPositioner)
}

fn create_tomtom_attribute_provider() -> Box<dyn AttributeProvider> {
    Box::new(everymap_providers_tomtom::TomTomAttributeProvider)
}

// Factory functions for MapBox provider
fn create_mapbox_geocoder(client: Arc<MapBoxClient>) -> Box<dyn Geocoder> {
    Box::new(everymap_providers_mapbox::MapBoxGeocoder::new(client))
}

fn create_mapbox_router(client: Arc<MapBoxClient>) -> Box<dyn Router> {
    Box::new(everymap_providers_mapbox::MapBoxRouter::new(client))
}

fn create_mapbox_isoline_provider(client: Arc<MapBoxClient>) -> Box<dyn IsolineProvider> {
    Box::new(everymap_providers_mapbox::MapBoxIsoline::new(client))
}

fn create_mapbox_route_matcher(client: Arc<MapBoxClient>) -> Box<dyn RouteMatcher> {
    Box::new(everymap_providers_mapbox::MapBoxRouteMatcher::new(client))
}

fn create_mapbox_tour_planner(client: Arc<MapBoxClient>) -> Box<dyn TourPlanner> {
    Box::new(everymap_providers_mapbox::MapBoxTourPlanner::new(client))
}

fn create_mapbox_tile_provider(client: Arc<MapBoxClient>) -> Box<dyn TileProvider> {
    Box::new(everymap_providers_mapbox::MapBoxTileProvider::new(client))
}

fn create_mapbox_image_provider(client: Arc<MapBoxClient>) -> Box<dyn MapImageProvider> {
    Box::new(everymap_providers_mapbox::MapBoxMapImageProvider::new(client))
}

fn create_mapbox_traffic() -> Box<dyn TrafficProvider> {
    Box::new(everymap_providers_mapbox::MapBoxTraffic)
}

fn create_mapbox_positioner() -> Box<dyn NetworkPositionerTrait> {
    Box::new(everymap_providers_mapbox::MapBoxPositioner)
}

fn create_mapbox_attribute_provider() -> Box<dyn AttributeProvider> {
    Box::new(everymap_providers_mapbox::MapBoxAttributeProvider)
}

// Factory functions for Radar provider
fn create_radar_geocoder(client: Arc<RadarClient>) -> Box<dyn Geocoder> {
    Box::new(everymap_providers_radar::RadarGeocoder::new(client))
}

fn create_radar_router(client: Arc<RadarClient>) -> Box<dyn Router> {
    Box::new(everymap_providers_radar::RadarRouter::new(client))
}

fn create_radar_route_matcher(client: Arc<RadarClient>) -> Box<dyn RouteMatcher> {
    Box::new(everymap_providers_radar::RadarRouteMatcher::new(client))
}

fn create_radar_tour_planner(client: Arc<RadarClient>) -> Box<dyn TourPlanner> {
    Box::new(everymap_providers_radar::RadarTourPlanner::new(client))
}

fn create_radar_geofence_provider(client: Arc<RadarClient>) -> Box<dyn GeofenceProvider> {
    Box::new(everymap_providers_radar::RadarGeofenceProvider::new(client))
}

fn create_radar_trip_tracker(client: Arc<RadarClient>) -> Box<dyn TripTracker> {
    Box::new(everymap_providers_radar::RadarTripTracker::new(client))
}

fn create_radar_fraud_detector(client: Arc<RadarClient>) -> Box<dyn FraudDetector> {
    Box::new(everymap_providers_radar::RadarFraudDetector::new(client))
}

fn create_radar_traffic() -> Box<dyn TrafficProvider> {
    Box::new(everymap_providers_radar::RadarTraffic)
}

fn create_radar_positioner() -> Box<dyn NetworkPositionerTrait> {
    Box::new(everymap_providers_radar::RadarPositioner)
}

fn create_radar_isoline() -> Box<dyn IsolineProvider> {
    Box::new(everymap_providers_radar::RadarIsoline)
}

fn create_radar_tile_provider() -> Box<dyn TileProvider> {
    Box::new(everymap_providers_radar::RadarTileProvider)
}

fn create_radar_attribute_provider() -> Box<dyn AttributeProvider> {
    Box::new(everymap_providers_radar::RadarAttributeProvider)
}

fn create_radar_image_provider() -> Box<dyn MapImageProvider> {
    Box::new(everymap_providers_radar::RadarMapImageProvider)
}

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

    if !["here", "google", "tomtom", "mapbox", "radar"].contains(&cli.provider.as_str()) {
        eprintln!("Error: Unsupported provider '{}'. Supported providers: here, google, tomtom, mapbox, radar", cli.provider);
        std::process::exit(1);
    }

    // Default API key param name varies by provider
    let key_param = cli.api_key_param.clone().unwrap_or_else(|| {
        match cli.provider.as_str() {
            "google" | "tomtom" => "key".to_string(),
            "mapbox" => "access_token".to_string(),
            _ => "apiKey".to_string(),
        }
    });

    let fmt = output::OutputFormat::from_str(&cli.output);

    // Radar uses HeaderAuthProvider (Authorization header), all others use ApiKeyProvider (query param)
    if cli.provider.as_str() == "radar" {
        let auth: Arc<dyn AuthProvider> = Arc::new(HeaderAuthProvider::new(api_key));
        let mut client = RadarClient::new(auth);
        client.set_verbose(cli.verbose);
        let client = Arc::new(client);
        run_radar_commands(&cli, client, &fmt).await;
    } else {
        let auth = Arc::new(ApiKeyProvider::new(api_key, key_param));

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
        "tomtom" => {
            let mut client = TomTomClient::new(auth);
            client.set_verbose(cli.verbose);
            let client = Arc::new(client);
            run_tomtom_commands(&cli, client, &fmt).await;
        }
        "mapbox" => {
            let mut client = MapBoxClient::new(auth);
            client.set_verbose(cli.verbose);
            let client = Arc::new(client);
            run_mapbox_commands(&cli, client, &fmt).await;
        }
        _ => unreachable!(),
    }
    } // end else (non-Radar providers)
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
        Commands::GeofenceSearch { .. } => {
            eprintln!("Error: Geofence search is not supported by this provider. Use --provider radar");
        }
        Commands::GeofenceCreate { .. } => {
            eprintln!("Error: Geofence creation is not supported by this provider. Use --provider radar");
        }
        Commands::GeofenceGet { .. } => {
            eprintln!("Error: Geofence get is not supported by this provider. Use --provider radar");
        }
        Commands::GeofenceDelete { .. } => {
            eprintln!("Error: Geofence deletion is not supported by this provider. Use --provider radar");
        }
        Commands::TripCreate { .. } => {
            eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar");
        }
        Commands::TripUpdate { .. } => {
            eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar");
        }
        Commands::TripGet { .. } => {
            eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar");
        }
        Commands::FraudCheck { .. } => {
            eprintln!("Error: Fraud detection is not supported by this provider. Use --provider radar");
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
            match traffic.get_traffic(&Coordinate::ORIGIN, &opts).await {
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
            match isoline.get_isoline(&Coordinate::ORIGIN, 1000.0, &opts).await {
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
        Commands::GeofenceSearch { .. } => {
            eprintln!("Error: Geofence search is not supported by this provider. Use --provider radar");
        }
        Commands::GeofenceCreate { .. } => {
            eprintln!("Error: Geofence creation is not supported by this provider. Use --provider radar");
        }
        Commands::GeofenceGet { .. } => {
            eprintln!("Error: Geofence get is not supported by this provider. Use --provider radar");
        }
        Commands::GeofenceDelete { .. } => {
            eprintln!("Error: Geofence deletion is not supported by this provider. Use --provider radar");
        }
        Commands::TripCreate { .. } => {
            eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar");
        }
        Commands::TripUpdate { .. } => {
            eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar");
        }
        Commands::TripGet { .. } => {
            eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar");
        }
        Commands::FraudCheck { .. } => {
            eprintln!("Error: Fraud detection is not supported by this provider. Use --provider radar");
        }
    }
}

async fn run_tomtom_commands(cli: &Cli, client: Arc<TomTomClient>, fmt: &output::OutputFormat) {
    match &cli.command {
        Commands::Geocode { query } => {
            let geocoder = create_tomtom_geocoder(client);
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
                        })
                    }).collect();
                    print_output(&serde_json::json!({ "results": output }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::ReverseGeocode { lat, lng } => {
            let geocoder = create_tomtom_geocoder(client);
            let coord = Coordinate::new(*lat, *lng).unwrap_or_else(|e| {
                eprintln!("Invalid coordinates: {}", e);
                std::process::exit(1);
            });
            let opts = ReverseGeocodeOptions::default();
            match geocoder.reverse_geocode(&coord, &opts).await {
                Ok(res) => {
                    let output: Vec<serde_json::Value> = res.items.iter().map(|item| {
                        serde_json::json!({
                            "id": item.id,
                            "title": item.title,
                            "coordinate": { "lat": item.coordinate.lat, "lng": item.coordinate.lng },
                        })
                    }).collect();
                    print_output(&serde_json::json!({ "results": output }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Route { origin, destination, transport } => {
            let router = create_tomtom_router(client);
            let start = parse_coordinate(origin).unwrap_or_else(|e| {
                eprintln!("Invalid origin: {}", e);
                std::process::exit(1);
            });
            let end = parse_coordinate(destination).unwrap_or_else(|e| {
                eprintln!("Invalid destination: {}", e);
                std::process::exit(1);
            });
            let transport_mode = parse_transport_mode(transport);
            let opts = RouteOptions { transport_mode: Some(transport_mode), ..Default::default() };
            match router.calculate_route(&start, &end, &opts).await {
                Ok(res) => {
                    let routes: Vec<serde_json::Value> = res.routes.iter().map(|r| {
                        serde_json::json!({ "distance_m": r.distance, "duration_s": r.duration, "points": r.geometry.points.len() })
                    }).collect();
                    print_output(&serde_json::json!({ "routes": routes }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Traffic { lat, lng } => {
            let traffic = create_tomtom_traffic_provider(client);
            let coord = Coordinate::new(*lat, *lng).unwrap_or_else(|e| {
                eprintln!("Invalid coordinates: {}", e);
                std::process::exit(1);
            });
            let opts = TrafficOptions::default();
            match traffic.get_traffic(&coord, &opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({ "flows": res.flows.len(), "incidents": res.incidents.len() }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Position => {
            let positioner = create_tomtom_positioner();
            let opts = PositioningOptions::default();
            match positioner.get_position(&opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Isoline { lat, lng, range } => {
            let isoline = create_tomtom_isoline_provider(client);
            let center = Coordinate::new(*lat, *lng).unwrap_or_else(|e| {
                eprintln!("Invalid coordinates: {}", e);
                std::process::exit(1);
            });
            let opts = IsolineOptions { range_type: Some(CoreRangeType::Distance), ..Default::default() };
            match isoline.get_isoline(&center, *range, &opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({ "isolines": res.isolines.len() }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::MatchRoute { trace } => {
            let matcher = create_tomtom_route_matcher(client);
            let points: Vec<Coordinate> = trace.split(';')
                .filter_map(|p| parse_coordinate(p.trim()).ok())
                .collect();
            if points.is_empty() {
                eprintln!("Error: No valid coordinates in trace");
                std::process::exit(1);
            }
            let opts = MatchingOptions::default();
            match matcher.match_route(&points, &opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({ "matched_points": res.matched_points.len(), "distance_m": res.distance }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Tour { stops } => {
            let planner = create_tomtom_tour_planner(client);
            let coordinates: Vec<Coordinate> = stops.iter()
                .filter_map(|s| parse_coordinate(s).ok())
                .collect();
            if coordinates.len() < 2 {
                eprintln!("Error: At least 2 stops required");
                std::process::exit(1);
            }
            let opts = TourOptions::default();
            match planner.optimize_tour(&coordinates, &opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({ "stops": res.stops.len(), "total_distance": res.total_distance, "total_duration": res.total_duration }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Tile { z, x, y } => {
            let tile_provider = create_tomtom_tile_provider(client);
            let opts = TileOptions::default();
            match tile_provider.get_tile(*z, *x, *y, &opts).await {
                Ok(res) => {
                    println!("Retrieved tile ({} bytes, content_type: {:?})", res.data.len(), res.content_type);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Attributes { .. } => {
            let attr_provider = create_tomtom_attribute_provider();
            let opts = AttributeOptions::default();
            match attr_provider.get_attributes(&opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::MapImage { lat, lng, zoom } => {
            let image_provider = create_tomtom_image_provider(client);
            let center = Coordinate::new(*lat, *lng).unwrap_or_else(|e| {
                eprintln!("Invalid coordinates: {}", e);
                std::process::exit(1);
            });
            let opts = ImageOptions::default();
            match image_provider.get_image(&center, *zoom, (800, 600), &opts).await {
                Ok(res) => {
                    println!("Retrieved map image ({} bytes, content_type: {:?})", res.data.len(), res.content_type);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::GeofenceSearch { .. } => {
            eprintln!("Error: Geofence search is not supported by this provider. Use --provider radar");
        }
        Commands::GeofenceCreate { .. } => {
            eprintln!("Error: Geofence creation is not supported by this provider. Use --provider radar");
        }
        Commands::GeofenceGet { .. } => {
            eprintln!("Error: Geofence get is not supported by this provider. Use --provider radar");
        }
        Commands::GeofenceDelete { .. } => {
            eprintln!("Error: Geofence deletion is not supported by this provider. Use --provider radar");
        }
        Commands::TripCreate { .. } => {
            eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar");
        }
        Commands::TripUpdate { .. } => {
            eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar");
        }
        Commands::TripGet { .. } => {
            eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar");
        }
        Commands::FraudCheck { .. } => {
            eprintln!("Error: Fraud detection is not supported by this provider. Use --provider radar");
        }
    }
}

async fn run_mapbox_commands(cli: &Cli, client: Arc<MapBoxClient>, fmt: &output::OutputFormat) {
    match &cli.command {
        Commands::Geocode { query } => {
            let geocoder = create_mapbox_geocoder(client);
            let opts = GeocodeOptions::default();
            match geocoder.geocode(query, &opts).await {
                Ok(res) => {
                    let output: Vec<serde_json::Value> = res.items.iter().map(|item| {
                        serde_json::json!({
                            "id": item.id,
                            "title": item.title,
                            "coordinate": { "lat": item.coordinate.lat, "lng": item.coordinate.lng },
                            "result_type": format!("{:?}", item.result_type),
                        })
                    }).collect();
                    print_output(&serde_json::json!({ "results": output }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::ReverseGeocode { lat, lng } => {
            let geocoder = create_mapbox_geocoder(client);
            let coord = Coordinate::new(*lat, *lng).unwrap_or_else(|e| {
                eprintln!("Invalid coordinates: {}", e);
                std::process::exit(1);
            });
            let opts = ReverseGeocodeOptions::default();
            match geocoder.reverse_geocode(&coord, &opts).await {
                Ok(res) => {
                    let output: Vec<serde_json::Value> = res.items.iter().map(|item| {
                        serde_json::json!({
                            "id": item.id,
                            "title": item.title,
                            "coordinate": { "lat": item.coordinate.lat, "lng": item.coordinate.lng },
                        })
                    }).collect();
                    print_output(&serde_json::json!({ "results": output }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Route { origin, destination, transport } => {
            let router = create_mapbox_router(client);
            let start = parse_coordinate(origin).unwrap_or_else(|e| {
                eprintln!("Invalid origin: {}", e);
                std::process::exit(1);
            });
            let end = parse_coordinate(destination).unwrap_or_else(|e| {
                eprintln!("Invalid destination: {}", e);
                std::process::exit(1);
            });
            let transport_mode = parse_transport_mode(transport);
            let opts = RouteOptions { transport_mode: Some(transport_mode), ..Default::default() };
            match router.calculate_route(&start, &end, &opts).await {
                Ok(res) => {
                    let routes: Vec<serde_json::Value> = res.routes.iter().map(|r| {
                        serde_json::json!({ "distance_m": r.distance, "duration_s": r.duration })
                    }).collect();
                    print_output(&serde_json::json!({ "routes": routes }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Traffic { .. } => {
            let traffic = create_mapbox_traffic();
            let opts = TrafficOptions::default();
            match traffic.get_traffic(&Coordinate::ORIGIN, &opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Position => {
            let positioner = create_mapbox_positioner();
            let opts = PositioningOptions::default();
            match positioner.get_position(&opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Isoline { lat, lng, range } => {
            let isoline = create_mapbox_isoline_provider(client);
            let center = Coordinate::new(*lat, *lng).unwrap_or_else(|e| {
                eprintln!("Invalid coordinates: {}", e);
                std::process::exit(1);
            });
            let opts = IsolineOptions { range_type: Some(CoreRangeType::Time), ..Default::default() };
            match isoline.get_isoline(&center, *range, &opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({ "isolines": res.isolines.len() }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::MatchRoute { trace } => {
            let matcher = create_mapbox_route_matcher(client);
            let points: Vec<Coordinate> = trace.split(';')
                .filter_map(|p| parse_coordinate(p.trim()).ok())
                .collect();
            if points.is_empty() {
                eprintln!("Error: No valid coordinates in trace");
                std::process::exit(1);
            }
            let opts = MatchingOptions::default();
            match matcher.match_route(&points, &opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({ "matched_points": res.matched_points.len(), "distance_m": res.distance }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Tour { stops } => {
            let planner = create_mapbox_tour_planner(client);
            let coordinates: Vec<Coordinate> = stops.iter()
                .filter_map(|s| parse_coordinate(s).ok())
                .collect();
            if coordinates.len() < 2 {
                eprintln!("Error: At least 2 stops required");
                std::process::exit(1);
            }
            let opts = TourOptions::default();
            match planner.optimize_tour(&coordinates, &opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({ "stops": res.stops.len(), "total_distance": res.total_distance, "total_duration": res.total_duration }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Tile { z, x, y } => {
            let tile_provider = create_mapbox_tile_provider(client);
            let opts = TileOptions::default();
            match tile_provider.get_tile(*z, *x, *y, &opts).await {
                Ok(res) => {
                    println!("Retrieved tile ({} bytes, content_type: {:?})", res.data.len(), res.content_type);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Attributes { .. } => {
            let attr_provider = create_mapbox_attribute_provider();
            let opts = AttributeOptions::default();
            match attr_provider.get_attributes(&opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::MapImage { lat, lng, zoom } => {
            let image_provider = create_mapbox_image_provider(client);
            let center = Coordinate::new(*lat, *lng).unwrap_or_else(|e| {
                eprintln!("Invalid coordinates: {}", e);
                std::process::exit(1);
            });
            let opts = ImageOptions::default();
            match image_provider.get_image(&center, *zoom, (800, 600), &opts).await {
                Ok(res) => {
                    println!("Retrieved map image ({} bytes, content_type: {:?})", res.data.len(), res.content_type);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::GeofenceSearch { .. } => {
            eprintln!("Error: Geofence search is not supported by this provider. Use --provider radar");
        }
        Commands::GeofenceCreate { .. } => {
            eprintln!("Error: Geofence creation is not supported by this provider. Use --provider radar");
        }
        Commands::GeofenceGet { .. } => {
            eprintln!("Error: Geofence get is not supported by this provider. Use --provider radar");
        }
        Commands::GeofenceDelete { .. } => {
            eprintln!("Error: Geofence deletion is not supported by this provider. Use --provider radar");
        }
        Commands::TripCreate { .. } => {
            eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar");
        }
        Commands::TripUpdate { .. } => {
            eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar");
        }
        Commands::TripGet { .. } => {
            eprintln!("Error: Trip tracking is not supported by this provider. Use --provider radar");
        }
        Commands::FraudCheck { .. } => {
            eprintln!("Error: Fraud detection is not supported by this provider. Use --provider radar");
        }
    }
}

async fn run_radar_commands(cli: &Cli, client: Arc<RadarClient>, fmt: &output::OutputFormat) {
    match &cli.command {
        Commands::Geocode { query } => {
            let geocoder = create_radar_geocoder(client);
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
                        })
                    }).collect();
                    print_output(&serde_json::json!({ "results": output }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::ReverseGeocode { lat, lng } => {
            let geocoder = create_radar_geocoder(client);
            let coord = Coordinate::new(*lat, *lng).unwrap_or_else(|e| {
                eprintln!("Invalid coordinates: {}", e);
                std::process::exit(1);
            });
            let opts = ReverseGeocodeOptions::default();
            match geocoder.reverse_geocode(&coord, &opts).await {
                Ok(res) => {
                    let output: Vec<serde_json::Value> = res.items.iter().map(|item| {
                        serde_json::json!({
                            "id": item.id,
                            "title": item.title,
                            "coordinate": { "lat": item.coordinate.lat, "lng": item.coordinate.lng },
                            "address": item.address,
                        })
                    }).collect();
                    print_output(&serde_json::json!({ "results": output }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Route { origin, destination, transport } => {
            let router = create_radar_router(client);
            let start = parse_coordinate(origin).unwrap_or_else(|e| {
                eprintln!("Invalid origin: {}", e);
                std::process::exit(1);
            });
            let end = parse_coordinate(destination).unwrap_or_else(|e| {
                eprintln!("Invalid destination: {}", e);
                std::process::exit(1);
            });
            let transport_mode = parse_transport_mode(transport);
            let opts = RouteOptions { transport_mode: Some(transport_mode), ..Default::default() };
            match router.calculate_route(&start, &end, &opts).await {
                Ok(res) => {
                    let routes: Vec<serde_json::Value> = res.routes.iter().map(|r| {
                        serde_json::json!({ "distance_m": r.distance, "duration_s": r.duration, "points": r.geometry.points.len() })
                    }).collect();
                    print_output(&serde_json::json!({ "routes": routes }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Traffic { .. } => {
            let traffic = create_radar_traffic();
            let opts = TrafficOptions::default();
            match traffic.get_traffic(&Coordinate::ORIGIN, &opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Position => {
            let positioner = create_radar_positioner();
            let opts = PositioningOptions::default();
            match positioner.get_position(&opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Isoline { .. } => {
            let isoline = create_radar_isoline();
            let opts = IsolineOptions::default();
            match isoline.get_isoline(&Coordinate::ORIGIN, 1000.0, &opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::MatchRoute { trace } => {
            let matcher = create_radar_route_matcher(client);
            let points: Vec<Coordinate> = trace.split(';')
                .filter_map(|p| parse_coordinate(p.trim()).ok())
                .collect();
            if points.is_empty() {
                eprintln!("Error: No valid coordinates in trace");
                std::process::exit(1);
            }
            let opts = MatchingOptions::default();
            match matcher.match_route(&points, &opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({ "matched_points": res.matched_points.len(), "distance_m": res.distance }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Tour { stops } => {
            let planner = create_radar_tour_planner(client);
            let coordinates: Vec<Coordinate> = stops.iter()
                .filter_map(|s| parse_coordinate(s).ok())
                .collect();
            if coordinates.len() < 2 {
                eprintln!("Error: At least 2 stops required");
                std::process::exit(1);
            }
            let opts = TourOptions::default();
            match planner.optimize_tour(&coordinates, &opts).await {
                Ok(res) => {
                    print_output(&serde_json::json!({ "stops": res.stops.len(), "total_distance": res.total_distance, "total_duration": res.total_duration }), fmt);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Tile { z, x, y } => {
            let tile_provider = create_radar_tile_provider();
            let opts = TileOptions::default();
            match tile_provider.get_tile(*z, *x, *y, &opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Attributes { .. } => {
            let attr_provider = create_radar_attribute_provider();
            let opts = AttributeOptions::default();
            match attr_provider.get_attributes(&opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::MapImage { .. } => {
            let image_provider = create_radar_image_provider();
            let opts = ImageOptions::default();
            match image_provider.get_image(&Coordinate::ORIGIN, 14, (800, 600), &opts).await {
                Ok(_) => unreachable!(),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::GeofenceSearch { lat, lng, radius, tags, limit } => {
            let provider = create_radar_geofence_provider(client);
            let near = Coordinate::new(*lat, *lng).unwrap_or_else(|e| {
                eprintln!("Invalid coordinates: {}", e);
                std::process::exit(1);
            });
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
            let provider = create_radar_geofence_provider(client);
            let center = Coordinate::new(*lat, *lng).unwrap_or_else(|e| {
                eprintln!("Invalid coordinates: {}", e);
                std::process::exit(1);
            });
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
            let provider = create_radar_geofence_provider(client);
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
            let provider = create_radar_geofence_provider(client);
            match provider.delete_geofence(id).await {
                Ok(()) => println!("Geofence '{}' deleted", id),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::TripCreate { origin, destination, mode, external_id, tag } => {
            let tracker = create_radar_trip_tracker(client);
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
            let tracker = create_radar_trip_tracker(client);
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
            let tracker = create_radar_trip_tracker(client);
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
            let detector = create_radar_fraud_detector(client);
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