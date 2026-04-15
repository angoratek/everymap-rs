use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use everymap_core::domains::search::{Geocoder, GeocodeOptions, ReverseGeocodeOptions};
use everymap_core::types::Coordinate;
use everymap_providers_mapbox::MapBoxGeocoder;
use everymap_providers_mapbox::client::MapBoxClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_geocode_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "id": "address.12345",
                "place_type": ["address"],
                "relevance": 0.99,
                "properties": {
                    "feature_type": "address",
                    "address": "10"
                },
                "text": "Unter den Linden",
                "place_name": "10 Unter den Linden, Berlin, Germany",
                "center": [13.3777, 52.5163],
                "geometry": {
                    "type": "Point",
                    "coordinates": [13.3777, 52.5163]
                },
                "bbox": [13.375, 52.514, 13.38, 52.518]
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/search/geocode/v6/forward"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("pk.test123".to_string(), "access_token".to_string()));
    let client = Arc::new(MapBoxClient::new(auth));
    let geocoder = MapBoxGeocoder::with_base_url(client, server.uri());

    let opts = GeocodeOptions::default();
    let res = geocoder.geocode("Unter den Linden, Berlin", &opts).await.unwrap();

    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].coordinate.lat, 52.5163);
    assert_eq!(res.items[0].coordinate.lng, 13.3777);
    assert_eq!(res.items[0].title, Some("10 Unter den Linden, Berlin, Germany".to_string()));
}

#[tokio::test]
async fn test_reverse_geocode_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "id": "place.98765",
                "place_type": ["place"],
                "relevance": 0.85,
                "properties": {},
                "text": "Berlin",
                "place_name": "Berlin, Germany",
                "center": [13.405, 52.52],
                "geometry": {
                    "type": "Point",
                    "coordinates": [13.405, 52.52]
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/search/geocode/v6/reverse"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("pk.test123".to_string(), "access_token".to_string()));
    let client = Arc::new(MapBoxClient::new(auth));
    let geocoder = MapBoxGeocoder::with_base_url(client, server.uri());

    let coord = Coordinate::new(52.52, 13.405).unwrap();
    let opts = ReverseGeocodeOptions::default();
    let res = geocoder.reverse_geocode(&coord, &opts).await.unwrap();

    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].title, Some("Berlin, Germany".to_string()));
}