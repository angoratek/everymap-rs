use everymap_core::auth::{AuthProvider, HeaderAuthProvider};
use everymap_core::domains::search::{GeocodeOptions, Geocoder, ReverseGeocodeOptions};
use everymap_providers_radar::{RadarClient, RadarGeocoder};
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_geocoder_mock() -> (MockServer, RadarGeocoder) {
    let server = MockServer::start().await;
    let auth: Arc<dyn AuthProvider> =
        Arc::new(HeaderAuthProvider::new("prj_test_pk_123".to_string()));
    let client = Arc::new(RadarClient::new(auth));
    let geocoder = RadarGeocoder::with_base_url(
        client,
        format!("{}/v1/geocode/forward", server.uri()),
        format!("{}/v1/geocode/reverse", server.uri()),
    );
    (server, geocoder)
}

#[tokio::test]
async fn test_geocode_contract() {
    let (server, geocoder) = setup_geocoder_mock().await;

    Mock::given(method("GET"))
        .and(path("/v1/geocode/forward"))
        .and(query_param("query", "Brandenburg Gate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "addresses": [{
                "latitude": 52.5163,
                "longitude": 13.3777,
                "country": "Germany",
                "countryCode": "DE",
                "county": "Berlin",
                "confidence": "exact",
                "city": "Berlin",
                "number": "1",
                "postalCode": "10117",
                "stateCode": "BE",
                "state": "Berlin",
                "street": "Pariser Platz",
                "formattedAddress": "Brandenburg Gate, Pariser Platz 1, 10117 Berlin, Germany",
                "layer": "address"
            }]
        })))
        .mount(&server)
        .await;

    let result = geocoder
        .geocode("Brandenburg Gate", &GeocodeOptions::default())
        .await
        .unwrap();
    assert_eq!(result.items.len(), 1);
    let item = &result.items[0];
    assert!((item.coordinate.lat - 52.5163).abs() < 0.001);
    assert!((item.coordinate.lng - 13.3777).abs() < 0.001);
    assert_eq!(item.address.country_code.as_deref(), Some("DE"));
    assert_eq!(item.address.city.as_deref(), Some("Berlin"));
}

#[tokio::test]
async fn test_reverse_geocode_contract() {
    let (server, geocoder) = setup_geocoder_mock().await;

    Mock::given(method("GET"))
        .and(path("/v1/geocode/reverse"))
        .and(query_param("coordinates", "52.5163,13.3777"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "addresses": [{
                "latitude": 52.5163,
                "longitude": 13.3777,
                "country": "Germany",
                "countryCode": "DE",
                "confidence": "exact",
                "city": "Berlin",
                "street": "Pariser Platz",
                "formattedAddress": "Pariser Platz, 10117 Berlin, Germany",
                "distance": 5.2
            }]
        })))
        .mount(&server)
        .await;

    let coordinate = everymap_core::types::Coordinate::new(52.5163, 13.3777).unwrap();
    let result = geocoder
        .reverse_geocode(&coordinate, &ReverseGeocodeOptions::default())
        .await
        .unwrap();
    assert_eq!(result.items.len(), 1);
    assert!(result.items[0].distance.is_some());
}

#[tokio::test]
async fn test_geocode_with_options() {
    let (server, geocoder) = setup_geocoder_mock().await;

    Mock::given(method("GET"))
        .and(path("/v1/geocode/forward"))
        .and(query_param("query", "Berlin"))
        .and(query_param("limit", "5"))
        .and(query_param("country", "DE"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "addresses": [
                { "latitude": 52.52, "longitude": 13.405, "formattedAddress": "Berlin, Germany", "country": "Germany", "countryCode": "DE", "city": "Berlin", "state": "Berlin", "stateCode": "BE", "postalCode": "10115", "street": "", "county": "", "number": "", "confidence": "exact" },
                { "latitude": 52.5, "longitude": 13.4, "formattedAddress": "Berlin Mitte, Germany", "country": "Germany", "countryCode": "DE", "city": "Berlin", "state": "Berlin", "stateCode": "BE", "postalCode": "10178", "street": "", "county": "", "number": "", "confidence": "interpolated" }
            ]
        })))
        .mount(&server)
        .await;

    let options = GeocodeOptions {
        limit: Some(5),
        country_codes: vec!["DE".to_string()],
        ..Default::default()
    };
    let result = geocoder.geocode("Berlin", &options).await.unwrap();
    assert_eq!(result.items.len(), 2);
}
