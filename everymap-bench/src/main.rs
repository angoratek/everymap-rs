mod benchmark;
mod report;
mod scenarios;

use clap::Parser;
use everymap_core::auth::{AuthProvider, ApiKeyProvider};
use everymap_core::auth::header::HeaderAuthProvider;
use everymap_core::types::Coordinate;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "everymap-bench", version, about = "Cross-provider benchmark framework for EveryMap")]
struct Cli {
    /// Run all benchmarks for all configured providers
    #[arg(long)]
    all: bool,

    /// Benchmark a specific domain (geocoder, routing, isoline, matching, tour)
    #[arg(long)]
    domain: Option<String>,

    /// Comma-separated list of providers to benchmark (here,google,tomtom,mapbox,radar)
    #[arg(long, default_value = "here,google,tomtom,mapbox")]
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

    let providers: Vec<&str> = cli.providers.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let scenarios = scenarios::get_scenarios(cli.domain.as_deref());

    if scenarios.is_empty() {
        eprintln!("No scenarios match the given domain filter.");
        std::process::exit(1);
    }

    for scenario in &scenarios {
        let mut results = Vec::new();

        for provider in &providers {
            let key = match get_provider_key(provider, &cli) {
                Some(k) => k,
                None => {
                    eprintln!("Skipping {}: no API key configured (set EVERYMAP_{}_API_KEY or EVERYMAP_API_KEY)", provider, provider.to_uppercase());
                    results.push(benchmark::BenchmarkResult {
                        provider: provider.to_string(),
                        domain: scenario.domain.clone(),
                        scenario: scenario.name.clone(),
                        duration_ms: 0,
                        success: false,
                        error: Some("No API key".to_string()),
                        result_count: 0,
                        raw_response_size: None,
                    });
                    continue;
                }
            };

            let auth: Arc<dyn AuthProvider> = if *provider == "radar" {
                Arc::new(HeaderAuthProvider::new(key))
            } else {
                Arc::new(ApiKeyProvider::new(key, get_key_param(provider).to_string()))
            };

            let result = match scenario.domain.as_str() {
                "geocoder" if scenario.name.starts_with("Reverse") => {
                    // Parse coordinates from scenario name
                    let coords: Vec<&str> = scenario.name.split_whitespace()
                        .filter(|s| s.contains(','))
                        .collect();
                    if let Some(coord_str) = coords.first() {
                        let parts: Vec<&str> = coord_str.split(',').collect();
                        if parts.len() == 2 {
                            if let (Ok(lat), Ok(lng)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                                benchmark::bench_geocode(provider, auth, &format!("{},{}", lat, lng)).await
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                "geocoder" => {
                    let query = match scenario.name.as_str() {
                        "Berlin Brandenburg Gate" => "Brandenburg Gate, Berlin",
                        "NYC Empire State Building (Radar)" => "Empire State Building, NYC",
                        _ => "Berlin",
                    };
                    benchmark::bench_geocode(provider, auth, query).await
                }
                "routing" => {
                    let (start, end) = match scenario.name.as_str() {
                        "Berlin to Paris" => (
                            Coordinate::new(52.5163, 13.3777).unwrap(),
                            Coordinate::new(48.8566, 2.3522).unwrap(),
                        ),
                        "NYC to LA" => (
                            Coordinate::new(40.7128, -74.0060).unwrap(),
                            Coordinate::new(34.0522, -118.2437).unwrap(),
                        ),
                        "NYC to Boston (Radar)" => (
                            Coordinate::new(40.7128, -74.0060).unwrap(),
                            Coordinate::new(42.3601, -71.0589).unwrap(),
                        ),
                        _ => continue,
                    };
                    benchmark::bench_route(provider, auth, &start, &end).await
                }
                _ => {
                    eprintln!("Benchmark domain '{}' not yet implemented", scenario.domain);
                    continue;
                }
            };

            results.push(result);
        }

        let report = benchmark::build_report(&scenario.domain, &scenario.name, results);

        let output = match cli.output.as_str() {
            "json" => report::format_json(&report),
            "markdown" | "md" => report::format_markdown(&report),
            _ => report::format_table(&report),
        };

        println!("{}", output);
        println!();
    }
}