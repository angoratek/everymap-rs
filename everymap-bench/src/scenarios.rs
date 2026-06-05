use everymap_core::types::Coordinate;

/// Typed parameters for a benchmark scenario.
#[derive(Debug, Clone)]
pub enum ScenarioParams {
    Geocode {
        query: String,
    },
    ReverseGeocode {
        coordinate: Coordinate,
    },
    Route {
        start: Coordinate,
        end: Coordinate,
    },
    Isoline {
        center: Coordinate,
        range: f64,
    },
    Matching {
        points: Vec<Coordinate>,
    },
    Tour {
        stops: Vec<Coordinate>,
    },
    Traffic {
        location: Coordinate,
    },
    Tile {
        z: u32,
        x: u32,
        y: u32,
    },
    Positioning {
        provider_extra: Option<serde_json::Value>,
    },
    Attributes {
        bbox: String,
        provider_extra: Option<serde_json::Value>,
    },
    Image {
        center: Coordinate,
        zoom: u32,
    },
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
        }
    }
}

/// A benchmark scenario defining a test case.
#[derive(Debug, Clone)]
pub struct BenchmarkScenario {
    pub name: String,
    #[allow(dead_code)]
    pub description: String,
    pub params: ScenarioParams,
}

/// Predefined benchmark scenarios for all 10 domains.
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
            name: "NYC Times Square".to_string(),
            description: "Forward geocode for 'Times Square, New York'".to_string(),
            params: ScenarioParams::Geocode {
                query: "Times Square, New York".to_string(),
            },
        },
        BenchmarkScenario {
            name: "Reverse Berlin center".to_string(),
            description: "Reverse geocode at Berlin center (52.52,13.38)".to_string(),
            params: ScenarioParams::ReverseGeocode {
                coordinate: Coordinate::new(52.5163, 13.3777).unwrap(),
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
        BenchmarkScenario {
            name: "1km walk from NYC".to_string(),
            description: "1-kilometer walking distance isoline from NYC".to_string(),
            params: ScenarioParams::Isoline {
                center: Coordinate::new(40.7128, -74.0060).unwrap(),
                range: 1000.0,
            },
        },
        // Matching
        BenchmarkScenario {
            name: "Berlin Unter den Linden".to_string(),
            description: "Match 4 GPS points along Unter den Linden, Berlin".to_string(),
            params: ScenarioParams::Matching {
                points: vec![
                    Coordinate::new(52.5163, 13.3777).unwrap(),
                    Coordinate::new(52.5163, 13.3850).unwrap(),
                    Coordinate::new(52.5163, 13.3920).unwrap(),
                    Coordinate::new(52.5163, 13.4000).unwrap(),
                ],
            },
        },
        BenchmarkScenario {
            name: "NYC along Broadway".to_string(),
            description: "Match 4 GPS points along Broadway, Manhattan".to_string(),
            params: ScenarioParams::Matching {
                points: vec![
                    Coordinate::new(40.7580, -73.9855).unwrap(),
                    Coordinate::new(40.7555, -73.9869).unwrap(),
                    Coordinate::new(40.7530, -73.9883).unwrap(),
                    Coordinate::new(40.7505, -73.9897).unwrap(),
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
        BenchmarkScenario {
            name: "6 Manhattan landmarks".to_string(),
            description: "Optimize tour through 6 Manhattan landmarks".to_string(),
            params: ScenarioParams::Tour {
                stops: vec![
                    Coordinate::new(40.7580, -73.9855).unwrap(), // Times Square
                    Coordinate::new(40.7614, -73.9776).unwrap(), // MoMA
                    Coordinate::new(40.7794, -73.9632).unwrap(), // Met Museum
                    Coordinate::new(40.7484, -73.9857).unwrap(), // Empire State
                    Coordinate::new(40.6892, -74.0445).unwrap(), // Statue of Liberty
                    Coordinate::new(40.7061, -74.0087).unwrap(), // Wall St
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
        BenchmarkScenario {
            name: "NYC traffic".to_string(),
            description: "Traffic flow near NYC center".to_string(),
            params: ScenarioParams::Traffic {
                location: Coordinate::new(40.7128, -74.0060).unwrap(),
            },
        },
        // Tiling
        BenchmarkScenario {
            name: "Berlin tile z10".to_string(),
            description: "Fetch map tile at zoom 10 covering Berlin".to_string(),
            params: ScenarioParams::Tile {
                z: 10,
                x: 550,
                y: 335,
            },
        },
        BenchmarkScenario {
            name: "NYC tile z10".to_string(),
            description: "Fetch map tile at zoom 10 covering Manhattan".to_string(),
            params: ScenarioParams::Tile {
                z: 10,
                x: 301,
                y: 384,
            },
        },
        // Positioning
        BenchmarkScenario {
            name: "WiFi positioning".to_string(),
            description: "Network positioning with WiFi observations".to_string(),
            params: ScenarioParams::Positioning {
                provider_extra: Some(serde_json::json!({
                    "wlan": [
                        {"mac": "a4:56:02:78:9a:bc", "signalStrength": -65, "channel": 6, "age": 1000},
                        {"mac": "b8:27:eb:3c:4d:5e", "signalStrength": -70, "channel": 11, "age": 2000},
                        {"mac": "c0:3f:d5:6e:7a:8b", "signalStrength": -75, "channel": 1, "age": 1500}
                    ]
                })),
            },
        },
        BenchmarkScenario {
            name: "Cell positioning".to_string(),
            description: "Network positioning with cell tower observations".to_string(),
            params: ScenarioParams::Positioning {
                provider_extra: Some(serde_json::json!({
                    "cell": [
                        {"mcc": 262, "mnc": 1, "lac": 4321, "cid": 12345, "signalStrength": -80}
                    ]
                })),
            },
        },
        // Attributes
        BenchmarkScenario {
            name: "Berlin attributes (HERE layers)".to_string(),
            description: "Road geometry attributes near Berlin (HERE layers param)".to_string(),
            params: ScenarioParams::Attributes {
                bbox: "13.3,52.5,13.4,52.52".to_string(),
                provider_extra: Some(serde_json::json!({
                    "layers": ["ROAD_GEOM_FC4"]
                })),
            },
        },
        BenchmarkScenario {
            name: "Berlin attributes (Google path)".to_string(),
            description: "Speed limit attributes along Berlin road (Google path param)".to_string(),
            params: ScenarioParams::Attributes {
                bbox: "13.3,52.5,13.4,52.52".to_string(),
                provider_extra: Some(serde_json::json!({
                    "path": "52.5163,13.3777;52.52,13.4"
                })),
            },
        },
        BenchmarkScenario {
            name: "Berlin attributes (generic)".to_string(),
            description: "Road attributes near Berlin center (no provider_extra)".to_string(),
            params: ScenarioParams::Attributes {
                bbox: "13.3,52.5,13.4,52.52".to_string(),
                provider_extra: None,
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
        BenchmarkScenario {
            name: "NYC map image".to_string(),
            description: "Static map image of Manhattan at zoom 12".to_string(),
            params: ScenarioParams::Image {
                center: Coordinate::new(40.7128, -74.0060).unwrap(),
                zoom: 12,
            },
        },
    ];

    match domain {
        Some(d) => all.into_iter().filter(|s| s.params.domain() == d).collect(),
        None => all,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use everymap_core::types::Coordinate;

    #[test]
    fn test_domain_names() {
        // Verify domain() returns correct strings for every variant
        let geocode = ScenarioParams::Geocode {
            query: "Berlin".to_string(),
        };
        assert_eq!(geocode.domain(), "geocoder");

        let reverse = ScenarioParams::ReverseGeocode {
            coordinate: Coordinate::new(52.52, 13.40).unwrap(),
        };
        assert_eq!(reverse.domain(), "geocoder");

        let route = ScenarioParams::Route {
            start: Coordinate::new(52.52, 13.40).unwrap(),
            end: Coordinate::new(48.86, 2.35).unwrap(),
        };
        assert_eq!(route.domain(), "routing");

        let isoline = ScenarioParams::Isoline {
            center: Coordinate::new(52.52, 13.40).unwrap(),
            range: 1000.0,
        };
        assert_eq!(isoline.domain(), "isoline");

        let matching = ScenarioParams::Matching {
            points: vec![Coordinate::new(52.52, 13.40).unwrap()],
        };
        assert_eq!(matching.domain(), "matching");

        let tour = ScenarioParams::Tour {
            stops: vec![Coordinate::new(52.52, 13.40).unwrap()],
        };
        assert_eq!(tour.domain(), "tour");

        let traffic = ScenarioParams::Traffic {
            location: Coordinate::new(52.52, 13.40).unwrap(),
        };
        assert_eq!(traffic.domain(), "traffic");

        let tile = ScenarioParams::Tile { z: 14, x: 8800, y: 5374 };
        assert_eq!(tile.domain(), "tiling");

        let positioning = ScenarioParams::Positioning {
            provider_extra: None,
        };
        assert_eq!(positioning.domain(), "positioning");

        let attributes = ScenarioParams::Attributes {
            bbox: "13.3,52.5,13.5,52.6".to_string(),
            provider_extra: None,
        };
        assert_eq!(attributes.domain(), "attributes");

        let image = ScenarioParams::Image {
            center: Coordinate::new(52.52, 13.40).unwrap(),
            zoom: 12,
        };
        assert_eq!(image.domain(), "imaging");
    }

    #[test]
    fn test_get_scenarios_all() {
        let all = get_scenarios(None);
        // Should have scenarios across all 10 domains
        assert!(all.len() > 15);
        let domains: std::collections::HashSet<&str> =
            all.iter().map(|s| s.params.domain()).collect();
        assert_eq!(domains.len(), 10);
    }

    #[test]
    fn test_get_scenarios_filtered() {
        let routing = get_scenarios(Some("routing"));
        assert_eq!(routing.len(), 2);
        for s in &routing {
            assert_eq!(s.params.domain(), "routing");
        }

        let geocoder = get_scenarios(Some("geocoder"));
        assert_eq!(geocoder.len(), 3); // Berlin, NYC, reverse Berlin
        for s in &geocoder {
            assert_eq!(s.params.domain(), "geocoder");
        }
    }

    #[test]
    fn test_get_scenarios_unknown_domain() {
        let unknown = get_scenarios(Some("nonexistent"));
        assert!(unknown.is_empty());
    }

    #[test]
    fn test_scenario_name_and_description() {
        let all = get_scenarios(None);
        for scenario in &all {
            assert!(!scenario.name.is_empty());
            assert!(!scenario.description.is_empty());
        }
    }
}
