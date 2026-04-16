mod benchmark;
mod report;
mod scenarios;

use clap::Parser;
use everymap_core::auth::{AuthProvider, ApiKeyProvider};
use everymap_core::auth::header::HeaderAuthProvider;
use scenarios::ScenarioParams;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "everymap-bench", version, about = "Cross-provider benchmark framework for EveryMap")]
struct Cli {
    /// Run all benchmarks for all configured providers
    #[arg(long)]
    all: bool,

    /// Benchmark a specific domain (geocoder, routing, isoline, matching, tour, traffic, tiling, positioning, attributes, imaging, geofencing, tracking, fraud)
    #[arg(long)]
    domain: Option<String>,

    /// Comma-separated list of providers to benchmark (here,google,tomtom,mapbox,radar)
    #[arg(long, default_value = "here,google,tomtom,mapbox,radar")]
    providers: String,

    /// Output format (json, table, markdown)
    #[arg(long, default_value = "table")]
    output: String,

    /// HERE API key
    #[arg(long, env = "EVERYMAP_HERE_API_KEY")]
    here_key: Option<String>,

    /// Google API key
    #[arg(long, env = "EVERYMAP_GOOGLE_API_KEY")]
    google_key: Option<String>,

    /// TomTom API key
    #[arg(long, env = "EVERYMAP_TOMTOM_API_KEY")]
    tomtom_key: Option<String>,

    /// MapBox API key
    #[arg(long, env = "EVERYMAP_MAPBOX_API_KEY")]
    mapbox_key: Option<String>,

    /// Radar API key
    #[arg(long, env = "EVERYMAP_RADAR_API_KEY")]
    radar_key: Option<String>,

    /// Fallback API key for any provider
    #[arg(long, env = "EVERYMAP_API_KEY")]
    api_key: Option<String>,
}

fn get_provider_key(provider: &str, cli: &Cli) -> Option<String> {
    match provider {
        "here" => cli.here_key.clone().or(cli.api_key.clone()),
        "google" => cli.google_key.clone().or(cli.api_key.clone()),
        "tomtom" => cli.tomtom_key.clone().or(cli.api_key.clone()),
        "mapbox" => cli.mapbox_key.clone().or(cli.api_key.clone()),
        "radar" => cli.radar_key.clone().or(cli.api_key.clone()),
        _ => None,
    }
}

fn get_key_param(provider: &str) -> &'static str {
    match provider {
        "here" => "apiKey",
        "google" | "tomtom" => "key",
        "mapbox" => "access_token",
        _ => "key",
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let provider_names: Vec<&str> = cli.providers.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let scenarios = scenarios::get_scenarios(cli.domain.as_deref());

    if scenarios.is_empty() {
        eprintln!("No scenarios match the given domain filter.");
        std::process::exit(1);
    }

    // Create BenchProviders for each configured provider
    let mut provider_instances: Vec<benchmark::BenchProviders> = Vec::new();
    for provider in &provider_names {
        let key = match get_provider_key(provider, &cli) {
            Some(k) => k,
            None => {
                eprintln!(
                    "Skipping {}: no API key (set EVERYMAP_{}_API_KEY or EVERYMAP_API_KEY)",
                    provider,
                    provider.to_uppercase()
                );
                continue;
            }
        };

        let auth: Arc<dyn AuthProvider> = if *provider == "radar" {
            Arc::new(HeaderAuthProvider::new(key))
        } else {
            Arc::new(ApiKeyProvider::new(key, get_key_param(provider).to_string()))
        };

        match benchmark::BenchProviders::new(provider, auth) {
            Ok(bp) => provider_instances.push(bp),
            Err(e) => eprintln!("Failed to create provider '{}': {}", provider, e),
        }
    }

    if provider_instances.is_empty() {
        eprintln!("No providers available. Configure API keys and try again.");
        std::process::exit(1);
    }

    // Run each scenario against each provider
    for scenario in &scenarios {
        let mut results = Vec::new();

        for providers in &provider_instances {
            let result = match &scenario.params {
                ScenarioParams::Geocode { query } => {
                    benchmark::bench_geocode(providers, query).await
                }
                ScenarioParams::ReverseGeocode { coord } => {
                    benchmark::bench_reverse_geocode(providers, coord).await
                }
                ScenarioParams::Route { start, end } => {
                    benchmark::bench_route(providers, start, end).await
                }
                ScenarioParams::Isoline { center, range } => {
                    benchmark::bench_isoline(providers, center, *range).await
                }
                ScenarioParams::Matching { points } => {
                    benchmark::bench_matching(providers, points).await
                }
                ScenarioParams::Tour { stops } => {
                    benchmark::bench_tour(providers, stops).await
                }
                ScenarioParams::Traffic { location } => {
                    benchmark::bench_traffic(providers, location).await
                }
                ScenarioParams::Tile { z, x, y } => {
                    benchmark::bench_tile(providers, *z, *x, *y).await
                }
                ScenarioParams::Positioning => {
                    benchmark::bench_positioning(providers).await
                }
                ScenarioParams::Attributes { bbox } => {
                    benchmark::bench_attributes(providers, bbox).await
                }
                ScenarioParams::Image { center, zoom } => {
                    benchmark::bench_image(providers, center, *zoom).await
                }
                ScenarioParams::GeofenceSearch { near, radius } => {
                    benchmark::bench_geofence_search(providers, near, *radius).await
                }
                ScenarioParams::TripCreate { origin, destination } => {
                    benchmark::bench_trip_create(providers, origin, destination).await
                }
                ScenarioParams::FraudCheck { lat, lng } => {
                    benchmark::bench_fraud_check(providers, *lat, *lng).await
                }
            };
            results.push(result);
        }

        let report = benchmark::build_report(scenario.params.domain(), &scenario.name, results);

        let output = match cli.output.as_str() {
            "json" => report::format_json(&report),
            "markdown" | "md" => report::format_markdown(&report),
            _ => report::format_table(&report),
        };

        println!("{}", output);
        println!();
    }
}