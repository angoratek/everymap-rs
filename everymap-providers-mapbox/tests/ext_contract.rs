use everymap_core::auth::ApiKeyProvider;
use everymap_core::types::Coordinate;
use everymap_providers_mapbox::client::MapBoxClient;
use everymap_providers_mapbox::ext::{MapBoxGeocoderExt, MapBoxRouterExt};
use everymap_providers_mapbox::MapBoxGeocoder;
use everymap_providers_mapbox::MapBoxRouter;
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_permanent_geocode_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "id": "dXJuOm1ieHBsYzpBY1E2",
                "geometry": { "type": "Point", "coordinates": [13.3777, 52.5163] },
                "properties": {
                    "mapbox_id": "dXJuOm1ieHBsYzpBY1E2",
                    "feature_type": "poi",
                    "full_address": "Brandenburg Gate, Berlin, Germany",
                    "name": "Brandenburg Gate",
                    "name_preferred": "Brandenburg Gate",
                    "coordinates": { "longitude": 13.3777, "latitude": 52.5163 },
                    "place_formatted": "Berlin, Germany",
                    "category": "restaurant"
                }
            }
        ],
        "attribution": "MapBox"
    });

    Mock::given(method("GET"))
        .and(path("/search/geocode/v6/forward"))
        .and(query_param("q", "Brandenburg Gate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let geocoder = MapBoxGeocoder::with_base_url(client, server.uri());

    let response = geocoder
        .permanent_geocode("Brandenburg Gate", Some(1), None)
        .await
        .unwrap();

    assert_eq!(response.features.len(), 1);
    assert_eq!(
        response.features[0].id,
        Some("dXJuOm1ieHBsYzpBY1E2".to_string())
    );
    let props = response.features[0].properties.as_ref().unwrap();
    assert_eq!(
        props.full_address,
        Some("Brandenburg Gate, Berlin, Germany".to_string())
    );
    let coords = props.coordinates.as_ref().unwrap();
    assert_eq!(coords.longitude, 13.3777);
    assert_eq!(coords.latitude, 52.5163);
}

#[tokio::test]
async fn test_batch_geocode_contract() {
    let server = MockServer::start().await;

    let mock_response_1 = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "id": "batch_1",
                "geometry": { "type": "Point", "coordinates": [13.405, 52.52] },
                "properties": {
                    "mapbox_id": "batch_1",
                    "feature_type": "place",
                    "full_address": "Berlin, Germany",
                    "name": "Berlin",
                    "coordinates": { "longitude": 13.405, "latitude": 52.52 }
                }
            }
        ]
    });

    let mock_response_2 = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "id": "batch_2",
                "geometry": { "type": "Point", "coordinates": [2.3522, 48.8566] },
                "properties": {
                    "mapbox_id": "batch_2",
                    "feature_type": "place",
                    "full_address": "Paris, France",
                    "name": "Paris",
                    "coordinates": { "longitude": 2.3522, "latitude": 48.8566 }
                }
            }
        ]
    });

    // Since batch_geocode does sequential lookups, mount responses for each query
    Mock::given(method("GET"))
        .and(path("/search/geocode/v6/forward"))
        .and(query_param("q", "Berlin"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response_1))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/search/geocode/v6/forward"))
        .and(query_param("q", "Paris"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response_2))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let geocoder = MapBoxGeocoder::with_base_url(client, server.uri());

    let queries = vec!["Berlin".to_string(), "Paris".to_string()];
    let response = geocoder.batch_geocode(&queries, None).await.unwrap();

    assert_eq!(response.len(), 2);
    assert_eq!(
        response[0].features[0].properties.as_ref().unwrap().name,
        Some("Berlin".to_string())
    );
    assert_eq!(
        response[1].features[0].properties.as_ref().unwrap().name,
        Some("Paris".to_string())
    );
}

#[tokio::test]
async fn test_route_with_profile_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "code": "Ok",
        "routes": [
            {
                "distance": 1052000.0,
                "duration": 36720.0,
                "geometry": "_m_I??c~_I??~o~@",
                "legs": [
                    {
                        "distance": 1052000.0,
                        "duration": 36720.0,
                        "summary": "Berlin to Paris",
                        "steps": [
                            {
                                "distance": 500.0,
                                "duration": 60.0,
                                "instruction": "Head north on A2",
                                "name": "A2",
                                "maneuver": { "type": "depart", "modifier": "left", "location": [13.3777, 52.5163] }
                            }
                        ]
                    }
                ]
            }
        ],
        "waypoints": [
            { "name": "Berlin", "location": [13.3777, 52.5163] },
            { "name": "Paris", "location": [2.3522, 48.8566] }
        ]
    });

    Mock::given(method("GET"))
        .and(path(
            "/directions/v5/mapbox/driving/13.3777,52.5163;2.3522,48.8566",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let router = MapBoxRouter::with_base_url(client, server.uri());

    let coords = vec![
        Coordinate::new(52.5163, 13.3777).unwrap(),
        Coordinate::new(48.8566, 2.3522).unwrap(),
    ];
    let response = router
        .route_with_profile(&coords, "driving", None)
        .await
        .unwrap();

    assert_eq!(response.code, Some("Ok".to_string()));
    assert_eq!(response.routes.len(), 1);
    assert_eq!(response.routes[0].distance, 1052000.0);
    assert_eq!(response.routes[0].duration, 36720.0);
    assert_eq!(response.routes[0].legs.len(), 1);
    assert_eq!(response.routes[0].legs[0].steps.len(), 1);
}

#[tokio::test]
async fn test_route_with_profile_insufficient_coords() {
    let server = MockServer::start().await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let router = MapBoxRouter::with_base_url(client, server.uri());

    let coords = vec![Coordinate::new(52.5163, 13.3777).unwrap()];
    let response = router.route_with_profile(&coords, "driving", None).await;

    assert!(response.is_err());
}
