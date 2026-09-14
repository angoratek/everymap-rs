use everymap_core::auth::HeaderAuthProvider;
use everymap_core::types::Coordinate;
use everymap_providers_radar::client::RadarClient;
use everymap_providers_radar::{
    RadarAutocompleteOptions, RadarGeocoderExt, RadarMatchingExt, RadarRouterExt, RadarSearchExt,
    RadarTravelMode,
};
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_coordinate() -> Coordinate {
    Coordinate::new(52.5163, 13.3777).unwrap()
}

fn auth() -> Arc<HeaderAuthProvider> {
    Arc::new(HeaderAuthProvider::new("prj_test_pk_123".to_string()))
}

// --- RadarGeocoderExt tests ---

#[tokio::test]
async fn test_ip_geocode_contract() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/geocode/ip"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "address": {
                "latitude": 52.5163,
                "longitude": 13.3777,
                "country": "Germany",
                "countryCode": "DE",
                "city": "Berlin",
                "state": "Berlin",
                "stateCode": "BE",
                "postalCode": "10117"
            }
        })))
        .mount(&server)
        .await;

    let client = Arc::new(RadarClient::new(auth()));
    let geocoder = everymap_providers_radar::RadarGeocoder::with_ext_urls(
        client,
        format!("{}/v1/geocode/forward", server.uri()),
        format!("{}/v1/geocode/reverse", server.uri()),
        format!("{}/v1/geocode/ip", server.uri()),
        format!("{}/v1/search/autocomplete", server.uri()),
        format!("{}/v1/addresses/validate", server.uri()),
        format!("{}/v1/search/places", server.uri()),
    );

    let result = RadarGeocoderExt::ip_geocode(&geocoder).await.unwrap();
    let addr = result.address.as_ref().unwrap();
    assert_eq!(addr.country_code, "DE");
    assert_eq!(addr.city, "Berlin");
}

#[tokio::test]
async fn test_autocomplete_contract() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/search/autocomplete"))
        .and(query_param("query", "Branden"))
        .and(query_param("near", "52.5163,13.3777"))
        .and(query_param("layers", "address"))
        .and(query_param("limit", "3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "addresses": [
                {
                    "latitude": 52.5163,
                    "longitude": 13.3777,
                    "formattedAddress": "Brandenburg Gate, Pariser Platz, Berlin, Germany",
                    "country": "Germany",
                    "countryCode": "DE",
                    "city": "Berlin",
                    "confidence": "exact"
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = Arc::new(RadarClient::new(auth()));
    let geocoder = everymap_providers_radar::RadarGeocoder::with_ext_urls(
        client,
        format!("{}/v1/geocode/forward", server.uri()),
        format!("{}/v1/geocode/reverse", server.uri()),
        format!("{}/v1/geocode/ip", server.uri()),
        format!("{}/v1/search/autocomplete", server.uri()),
        format!("{}/v1/addresses/validate", server.uri()),
        format!("{}/v1/search/places", server.uri()),
    );

    let options = RadarAutocompleteOptions {
        layers: Some("address".to_string()),
        limit: Some(3),
        country_code: None,
    };
    let result =
        RadarGeocoderExt::autocomplete(&geocoder, "Branden", Some(&test_coordinate()), &options)
            .await
            .unwrap();
    assert_eq!(result.addresses.len(), 1);
    assert_eq!(result.addresses[0].country_code, "DE");
}

// --- RadarRouterExt tests ---

#[tokio::test]
async fn test_distance_contract() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/route/distance"))
        .and(query_param("origin", "52.5163,13.3777"))
        .and(query_param("destination", "52.52,13.405"))
        .and(query_param("modes", "car,bike"))
        .and(query_param("units", "metric"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "routes": {
                "car": {
                    "distance": { "value": 3200.0, "text": "3.2 km" },
                    "duration": { "value": 480.0, "text": "8 min" }
                },
                "bike": {
                    "distance": { "value": 2900.0, "text": "2.9 km" },
                    "duration": { "value": 600.0, "text": "10 min" }
                }
            }
        })))
        .mount(&server)
        .await;

    let client = Arc::new(RadarClient::new(auth()));
    let router = everymap_providers_radar::RadarRouter::with_ext_urls(
        client,
        format!("{}/v1/route/directions", server.uri()),
        format!("{}/v1/route/distance", server.uri()),
        format!("{}/v1/route/matrix", server.uri()),
    );

    let origin = test_coordinate();
    let destination = Coordinate::new(52.52, 13.405).unwrap();
    let result = RadarRouterExt::distance(
        &router,
        &origin,
        &destination,
        &[RadarTravelMode::Car, RadarTravelMode::Bike],
        Some("metric"),
    )
    .await
    .unwrap();
    assert!(result.routes.car.is_some());
    assert!(result.routes.bike.is_some());
    assert_eq!(result.routes.car.as_ref().unwrap().distance.value, 3200.0);
}

#[tokio::test]
async fn test_matrix_contract() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/route/matrix"))
        .and(query_param("origins", "52.5163,13.3777|52.52,13.405"))
        .and(query_param("destinations", "52.51,13.4"))
        .and(query_param("mode", "car"))
        .and(query_param("units", "metric"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "matrix": [
                [{
                    "distance": { "value": 1500.0, "text": "1.5 km" },
                    "duration": { "value": 240.0, "text": "4 min" }
                }],
                [{
                    "distance": { "value": 2200.0, "text": "2.2 km" },
                    "duration": { "value": 360.0, "text": "6 min" }
                }]
            ]
        })))
        .mount(&server)
        .await;

    let client = Arc::new(RadarClient::new(auth()));
    let router = everymap_providers_radar::RadarRouter::with_ext_urls(
        client,
        format!("{}/v1/route/directions", server.uri()),
        format!("{}/v1/route/distance", server.uri()),
        format!("{}/v1/route/matrix", server.uri()),
    );

    let origins = vec![test_coordinate(), Coordinate::new(52.52, 13.405).unwrap()];
    let destinations = vec![Coordinate::new(52.51, 13.40).unwrap()];
    let result = RadarRouterExt::matrix(
        &router,
        &origins,
        &destinations,
        RadarTravelMode::Car,
        Some("metric"),
    )
    .await
    .unwrap();
    assert_eq!(result.matrix.len(), 2);
}

// --- RadarSearchExt tests ---

#[tokio::test]
async fn test_search_places_contract() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/search/places"))
        .and(query_param("near", "52.5163,13.3777"))
        .and(query_param("chains", "starbucks"))
        .and(query_param("categories", "coffee-shop"))
        .and(query_param("radius", "500"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "places": [
                {
                    "id": "place_1",
                    "name": "Starbucks",
                    "categories": ["coffee-shop"],
                    "location": { "type": "Point", "coordinates": [13.3777, 52.5165] },
                    "chain": { "name": "Starbucks", "slug": "starbucks" }
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = Arc::new(RadarClient::new(auth()));
    let geocoder = everymap_providers_radar::RadarGeocoder::with_ext_urls(
        client,
        format!("{}/v1/geocode/forward", server.uri()),
        format!("{}/v1/geocode/reverse", server.uri()),
        format!("{}/v1/geocode/ip", server.uri()),
        format!("{}/v1/search/autocomplete", server.uri()),
        format!("{}/v1/addresses/validate", server.uri()),
        format!("{}/v1/search/places", server.uri()),
    );

    let result = RadarSearchExt::search_places(
        &geocoder,
        &test_coordinate(),
        Some(&["starbucks".to_string()]),
        Some(&["coffee-shop".to_string()]),
        Some(500.0),
        Some(10),
    )
    .await
    .unwrap();
    assert_eq!(result.places.len(), 1);
    assert_eq!(result.places[0].name, "Starbucks");
}

// --- RadarMatchingExt tests ---

#[tokio::test]
async fn test_match_route_with_attributes_contract() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/route/match"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "matchedPath": [
                {
                    "latitude": 52.5163,
                    "longitude": 13.3777,
                    "confidence": 0.95
                },
                {
                    "latitude": 52.5170,
                    "longitude": 13.3785,
                    "confidence": 0.92
                }
            ],
            "roadAttributes": [
                {
                    "speedLimit": { "value": 50.0, "unit": "km/h" },
                    "names": ["Pariser Platz"],
                    "roadClass": "local"
                },
                {
                    "speedLimit": { "value": 30.0, "unit": "km/h" },
                    "names": ["Unter den Linden"],
                    "roadClass": "main"
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = Arc::new(RadarClient::new(auth()));
    let matcher = everymap_providers_radar::RadarRouteMatcher::with_base_url(
        client,
        format!("{}/v1/route/match", server.uri()),
    );

    let points = vec![test_coordinate(), Coordinate::new(52.517, 13.3785).unwrap()];
    let result =
        RadarMatchingExt::match_route_with_attributes(&matcher, &points, RadarTravelMode::Car)
            .await
            .unwrap();
    assert_eq!(result.matched_path.len(), 2);
    assert!((result.matched_path[0].latitude - 52.5163).abs() < 0.001);
    let attrs = result.road_attributes.as_ref().unwrap();
    assert_eq!(attrs.len(), 2);
    assert!((attrs[0].speed_limit.as_ref().unwrap().value - 50.0).abs() < 0.01);
}
