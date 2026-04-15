use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};
use everymap_core::types::Coordinate;
use everymap_core::auth::ApiKeyProvider;
use everymap_providers_mapbox::client::MapBoxClient;
use everymap_providers_mapbox::MapBoxGeocoder;
use everymap_providers_mapbox::MapBoxRouter;
use everymap_providers_mapbox::ext::{MapBoxGeocoderExt, MapBoxRouterExt};
use std::sync::Arc;

#[tokio::test]
async fn test_permanent_geocode_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "id": "mapbox_permanent_1",
                "place_type": ["poi"],
                "relevance": 0.99,
                "properties": { "feature_type": "poi", "category": "restaurant" },
                "text": "Brandenburg Gate",
                "place_name": "Brandenburg Gate, Berlin, Germany",
                "center": [13.3777, 52.5163],
                "geometry": { "type": "Point", "coordinates": [13.3777, 52.5163] }
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

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "access_token".to_string()));
    let client = Arc::new(MapBoxClient::new(auth));
    let geocoder = MapBoxGeocoder::with_base_url(client, server.uri());

    let res = geocoder.permanent_geocode("Brandenburg Gate", Some(1), None).await.unwrap();

    assert_eq!(res.features.len(), 1);
    assert_eq!(res.features[0].id, Some("mapbox_permanent_1".to_string()));
    assert_eq!(res.features[0].place_name, Some("Brandenburg Gate, Berlin, Germany".to_string()));
    assert!(res.features[0].center.is_some());
    let center = res.features[0].center.as_ref().unwrap();
    assert_eq!(center[0], 13.3777); // lng
    assert_eq!(center[1], 52.5163); // lat
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
                "place_type": ["place"],
                "text": "Berlin",
                "place_name": "Berlin, Germany",
                "center": [13.405, 52.52],
                "geometry": { "type": "Point", "coordinates": [13.405, 52.52] }
            }
        ]
    });

    let mock_response_2 = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "id": "batch_2",
                "place_type": ["place"],
                "text": "Paris",
                "place_name": "Paris, France",
                "center": [2.3522, 48.8566],
                "geometry": { "type": "Point", "coordinates": [2.3522, 48.8566] }
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

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "access_token".to_string()));
    let client = Arc::new(MapBoxClient::new(auth));
    let geocoder = MapBoxGeocoder::with_base_url(client, server.uri());

    let queries = vec!["Berlin".to_string(), "Paris".to_string()];
    let res = geocoder.batch_geocode(&queries, None).await.unwrap();

    assert_eq!(res.len(), 2);
    assert_eq!(res[0].features[0].text, Some("Berlin".to_string()));
    assert_eq!(res[1].features[0].text, Some("Paris".to_string()));
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
        .and(path("/directions/v5/mapbox/driving/13.3777,52.5163;2.3522,48.8566"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "access_token".to_string()));
    let client = Arc::new(MapBoxClient::new(auth));
    let router = MapBoxRouter::with_base_url(client, server.uri());

    let coords = vec![
        Coordinate::new(52.5163, 13.3777).unwrap(),
        Coordinate::new(48.8566, 2.3522).unwrap(),
    ];
    let res = router.route_with_profile(&coords, "driving", None).await.unwrap();

    assert_eq!(res.code, Some("Ok".to_string()));
    assert_eq!(res.routes.len(), 1);
    assert_eq!(res.routes[0].distance, 1052000.0);
    assert_eq!(res.routes[0].duration, 36720.0);
    assert_eq!(res.routes[0].legs.len(), 1);
    assert_eq!(res.routes[0].legs[0].steps.len(), 1);
}

#[tokio::test]
async fn test_route_with_profile_insufficient_coords() {
    let server = MockServer::start().await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "access_token".to_string()));
    let client = Arc::new(MapBoxClient::new(auth));
    let router = MapBoxRouter::with_base_url(client, server.uri());

    let coords = vec![Coordinate::new(52.5163, 13.3777).unwrap()];
    let res = router.route_with_profile(&coords, "driving", None).await;

    assert!(res.is_err());
}