use serde::{Deserialize, Serialize};

/// A benchmark scenario defining a test case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkScenario {
    pub domain: String,
    pub name: String,
    pub description: String,
}

/// Predefined benchmark scenarios per domain.
pub fn get_scenarios(domain: Option<&str>) -> Vec<BenchmarkScenario> {
    let all = vec![
        BenchmarkScenario {
            domain: "geocoder".to_string(),
            name: "Berlin Brandenburg Gate".to_string(),
            description: "Forward geocode for 'Brandenburg Gate, Berlin'".to_string(),
        },
        BenchmarkScenario {
            domain: "geocoder".to_string(),
            name: "Reverse 52.5163,13.3777".to_string(),
            description: "Reverse geocode for Brandenburg Gate coordinates".to_string(),
        },
        BenchmarkScenario {
            domain: "routing".to_string(),
            name: "Berlin to Paris".to_string(),
            description: "Route from Berlin (52.5163,13.3777) to Paris (48.8566,2.3522)".to_string(),
        },
        BenchmarkScenario {
            domain: "routing".to_string(),
            name: "NYC to LA".to_string(),
            description: "Route from NYC (40.7128,-74.0060) to LA (34.0522,-118.2437)".to_string(),
        },
        BenchmarkScenario {
            domain: "isoline".to_string(),
            name: "30min drive from Berlin center".to_string(),
            description: "Isoline: 30-minute drive from Berlin center (52.5163,13.3777)".to_string(),
        },
        BenchmarkScenario {
            domain: "matching".to_string(),
            name: "Berlin straight line".to_string(),
            description: "Match GPS trace: 52.5,13.4 -> 52.6,13.5".to_string(),
        },
        BenchmarkScenario {
            domain: "tour".to_string(),
            name: "5 Berlin landmarks".to_string(),
            description: "Optimize tour through 5 Berlin coordinates".to_string(),
        },
        BenchmarkScenario {
            domain: "geocoder".to_string(),
            name: "NYC Empire State Building (Radar)".to_string(),
            description: "Forward geocode for 'Empire State Building, NYC' via Radar".to_string(),
        },
        BenchmarkScenario {
            domain: "routing".to_string(),
            name: "NYC to Boston (Radar)".to_string(),
            description: "Route from NYC (40.7128,-74.0060) to Boston (42.3601,-71.0589) via Radar".to_string(),
        },
        BenchmarkScenario {
            domain: "geofencing".to_string(),
            name: "Geofence search near NYC".to_string(),
            description: "Search geofences near NYC (40.7128,-74.0060)".to_string(),
        },
    ];

    match domain {
        Some(d) => all.into_iter().filter(|s| s.domain == d).collect(),
        None => all,
    }
}