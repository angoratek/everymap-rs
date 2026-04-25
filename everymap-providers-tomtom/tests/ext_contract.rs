use everymap_core::auth::ApiKeyProvider;
use everymap_core::types::Coordinate;
use everymap_providers_tomtom::client::TomTomClient;
use everymap_providers_tomtom::ext::{TomTomGeocoderExt, TomTomTrafficExt};
use everymap_providers_tomtom::TomTomGeocoder;
use everymap_providers_tomtom::TomTomTraffic;
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_nearby_search_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "results": [
            {
                "position": { "lat": 52.5200, "lon": 13.4050 },
                "address": {
                    "freeformAddress": "Brandenburg Gate, Berlin",
                    "municipality": "Berlin",
                    "country": "Germany"
                },
                "type": "POI",
                "score": 0.9,
                "id": "tomtom_poi_1",
                "dist": 150.0
            }
        ],
        "summary": { "numResults": 1, "query": "restaurant" }
    });

    Mock::given(method("GET"))
        .and(path("/search/2/nearbySearch/.json"))
        .and(query_param("query", "restaurant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let geocoder = TomTomGeocoder::with_base_url(client, server.uri());

    let location = Coordinate::new(52.52, 13.405).unwrap();
    let res = geocoder
        .nearby_search(&location, 5000, "restaurant", None, None)
        .await
        .unwrap();

    assert_eq!(res.results.len(), 1);
    assert_eq!(res.results[0].id, Some("tomtom_poi_1".to_string()));
    assert_eq!(res.results[0].dist, Some(150.0));
}

#[tokio::test]
async fn test_category_search_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "results": [
            {
                "position": { "lat": 52.5210, "lon": 13.4060 },
                "address": {
                    "freeformAddress": "Berlin Hauptbahnhof",
                    "municipality": "Berlin",
                    "country": "Germany"
                },
                "type": "POI",
                "score": 0.85,
                "id": "tomtom_poi_2"
            }
        ],
        "summary": { "numResults": 1, "query": "RESTAURANT" }
    });

    Mock::given(method("GET"))
        .and(path("/search/2/categorySearch/RESTAURANT.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let geocoder = TomTomGeocoder::with_base_url(client, server.uri());

    let location = Coordinate::new(52.52, 13.405).unwrap();
    let res = geocoder
        .category_search("RESTAURANT", &location, Some(3000), Some(5), None)
        .await
        .unwrap();

    assert_eq!(res.results.len(), 1);
    assert_eq!(res.results[0].id, Some("tomtom_poi_2".to_string()));
}

#[tokio::test]
async fn test_traffic_ext_get_flow() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "flowSegmentData": {
            "frc": "FRC0",
            "currentSpeed": 45,
            "freeFlowSpeed": 80,
            "currentTravelTime": 180,
            "freeFlowTravelTime": 95,
            "confidence": 0.85,
            "roadName": "A100",
            "coordinates": {
                "coordinate": [
                    { "latitude": 52.52, "longitude": 13.405 },
                    { "latitude": 52.521, "longitude": 13.406 }
                ]
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/traffic/services/4/flowSegmentData/absolute/10/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let traffic = TomTomTraffic::with_base_url(client, server.uri());

    let location = Coordinate::new(52.52, 13.405).unwrap();
    let res = traffic.get_flow(&location, None).await.unwrap();

    assert!(res.flow_segment_data.is_some());
    let flow = res.flow_segment_data.unwrap();
    assert_eq!(flow.current_speed, 45.0);
    assert_eq!(flow.free_flow_speed, 80.0);
    assert_eq!(flow.road_name, Some("A100".to_string()));
    assert!(flow.coordinates.is_some());
    let coords = flow.coordinates.unwrap();
    assert_eq!(coords.coordinate.len(), 2);
}

#[tokio::test]
async fn test_traffic_ext_get_incidents() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "incidents": [
            {
                "id": "inc456",
                "type": "Accident",
                "severity": "minor",
                "description": "Fender bender on A100",
                "from": "A100 Exit 5",
                "to": "A100 Exit 6",
                "delay": 300,
                "length": 1.5,
                "startTime": "2026-04-12T10:00:00Z",
                "endTime": "2026-04-12T11:00:00Z"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/traffic/services/5/incidentDetails"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let traffic = TomTomTraffic::with_base_url(client, server.uri());

    let res = traffic
        .get_incidents("52.4,13.3,52.6,13.5", None)
        .await
        .unwrap();

    assert_eq!(res.incidents.len(), 1);
    assert_eq!(res.incidents[0].id, Some("inc456".to_string()));
    assert_eq!(
        res.incidents[0].start_time,
        Some("2026-04-12T10:00:00Z".to_string())
    );
    assert_eq!(
        res.incidents[0].end_time,
        Some("2026-04-12T11:00:00Z".to_string())
    );
}
