use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::search::{GeocodeOptions, Geocoder, ReverseGeocodeOptions};
use everymap_core::types::Coordinate;
use everymap_providers_google::client::GoogleClient;
use everymap_providers_google::GoogleGeocoder;
use std::sync::Arc;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_geocode_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "results": [
            {
                "formatted_address": "Brandenburger Tor, 10117 Berlin, Germany",
                "geometry": {
                    "location": { "lat": 52.5163, "lng": 13.3777 },
                    "location_type": "ROOFTOP",
                    "viewport": {
                        "northeast": { "lat": 52.5176, "lng": 13.3790 },
                        "southwest": { "lat": 52.5151, "lng": 13.3764 }
                    }
                },
                "place_id": "ChIJiQ7xR0ROqEcRkR7QsRVEs",
                "types": ["street_address", "political"],
                "address_components": [
                    { "long_name": "Brandenburger Tor", "short_name": "Brandenburger Tor", "types": ["point_of_interest", "tourist_attraction"] },
                    { "long_name": "10117", "short_name": "10117", "types": ["postal_code"] },
                    { "long_name": "Berlin", "short_name": "Berlin", "types": ["locality", "political"] },
                    { "long_name": "Germany", "short_name": "DE", "types": ["country", "political"] }
                ]
            }
        ],
        "status": "OK"
    });

    // GoogleGeocoder uses base_url directly as the full URL, so with_base_url
    // replaces the entire URL with the mock server URI (path becomes "/")
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let geocoder = GoogleGeocoder::with_base_url(client, server.uri());

    let result = geocoder
        .geocode("Brandenburg Gate, Berlin", &GeocodeOptions::default())
        .await
        .unwrap();

    assert_eq!(result.items.len(), 1);
    let item = &result.items[0];
    assert_eq!(
        item.title,
        Some("Brandenburger Tor, 10117 Berlin, Germany".to_string())
    );
    assert_eq!(item.id, Some("ChIJiQ7xR0ROqEcRkR7QsRVEs".to_string()));
    assert!(item.coordinate.lat > 52.5 && item.coordinate.lat < 52.6);
    assert!(item.coordinate.lng > 13.3 && item.coordinate.lng < 13.4);
    assert!(item.bounding_box.is_some());
    assert!(item.raw.is_some());
}

#[tokio::test]
async fn test_geocode_with_options() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "results": [
            {
                "formatted_address": "Paris, France",
                "geometry": {
                    "location": { "lat": 48.8566, "lng": 2.3522 },
                    "location_type": "APPROXIMATE"
                },
                "place_id": "ChIJD7fiBh9u5kcRYJSMa1j2v",
                "types": ["locality", "political"],
                "address_components": [
                    { "long_name": "Paris", "short_name": "Paris", "types": ["locality", "political"] },
                    { "long_name": "France", "short_name": "FR", "types": ["country", "political"] }
                ]
            }
        ],
        "status": "OK"
    });

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let geocoder = GoogleGeocoder::with_base_url(client, server.uri());

    let opts = GeocodeOptions {
        language: Some("fr".to_string()),
        ..Default::default()
    };

    let result = geocoder.geocode("Paris", &opts).await.unwrap();
    assert_eq!(result.items.len(), 1);
    assert!(result.items[0].address.city.is_some());
}

#[tokio::test]
async fn test_reverse_geocode_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "results": [
            {
                "formatted_address": "Pariser Platz, 10117 Berlin, Germany",
                "geometry": {
                    "location": { "lat": 52.5163, "lng": 13.3777 },
                    "location_type": "ROOFTOP"
                },
                "place_id": "ChIJw0kFYBROqEcQtnV3RDRRGk",
                "types": ["route"],
                "address_components": [
                    { "long_name": "Pariser Platz", "short_name": "Pariser Platz", "types": ["route"] },
                    { "long_name": "Mitte", "short_name": "Mitte", "types": ["sublocality", "political"] },
                    { "long_name": "Berlin", "short_name": "Berlin", "types": ["locality", "political"] },
                    { "long_name": "DE", "short_name": "DE", "types": ["country", "political"] }
                ]
            }
        ],
        "status": "OK"
    });

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let geocoder = GoogleGeocoder::with_base_url(client, server.uri());

    let coord = Coordinate::new(52.5163, 13.3777).unwrap();
    let result = geocoder
        .reverse_geocode(&coord, &ReverseGeocodeOptions::default())
        .await
        .unwrap();

    assert_eq!(result.items.len(), 1);
    assert!(result.items[0]
        .title
        .as_ref()
        .is_some_and(|t| t.contains("Pariser Platz")));
}

#[tokio::test]
async fn test_geocode_zero_results() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "results": [],
        "status": "ZERO_RESULTS"
    });

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let geocoder = GoogleGeocoder::with_base_url(client, server.uri());

    let result = geocoder
        .geocode("nonexistentxyz123", &GeocodeOptions::default())
        .await
        .unwrap();
    assert_eq!(result.items.len(), 0);
}

#[tokio::test]
async fn test_geocode_error_response() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "results": [],
        "status": "REQUEST_DENIED",
        "error_message": "The provided API key is invalid."
    });

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let geocoder = GoogleGeocoder::with_base_url(client, server.uri());

    let result = geocoder.geocode("Berlin", &GeocodeOptions::default()).await;
    assert!(result.is_err());
}
