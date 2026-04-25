use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::search::{GeocodeOptions, Geocoder, ReverseGeocodeOptions};
use everymap_core::types::Coordinate;
use everymap_providers_mapbox::client::MapBoxClient;
use everymap_providers_mapbox::MapBoxGeocoder;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_geocode_contract() {
    let server = MockServer::start().await;

    // MapBox v6 API response format — data is in properties
    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "id": "dXJuOm1ieHBsYzpBY1E2",
                "geometry": {
                    "type": "Point",
                    "coordinates": [13.395131, 52.517389]
                },
                "properties": {
                    "mapbox_id": "dXJuOm1ieHBsYzpBY1E2",
                    "feature_type": "place",
                    "full_address": "Berlin, Germany",
                    "name": "Berlin",
                    "name_preferred": "Berlin",
                    "coordinates": { "longitude": 13.395131, "latitude": 52.517389 },
                    "place_formatted": "Germany",
                    "bbox": [13.08836, 52.338261, 13.760906, 52.675502],
                    "context": {
                        "region": {
                            "name": "Berlin",
                            "region_code": "BE"
                        },
                        "country": {
                            "name": "Germany",
                            "country_code": "DE"
                        },
                        "place": {
                            "name": "Berlin"
                        }
                    }
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/search/geocode/v6/forward"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "pk.test123".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let geocoder = MapBoxGeocoder::with_base_url(client, server.uri());

    let opts = GeocodeOptions::default();
    let res = geocoder.geocode("Berlin", &opts).await.unwrap();

    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].coordinate.lat, 52.517389);
    assert_eq!(res.items[0].coordinate.lng, 13.395131);
    assert_eq!(res.items[0].title, Some("Berlin, Germany".to_string()));
    assert_eq!(
        res.items[0].result_type,
        everymap_core::domains::search::SearchResultType::Approximate
    );
    assert_eq!(res.items[0].address.country.as_deref(), Some("Germany"));
    assert_eq!(res.items[0].address.country_code.as_deref(), Some("DE"));
}

#[tokio::test]
async fn test_reverse_geocode_contract() {
    let server = MockServer::start().await;

    // MapBox v6 reverse geocode response format
    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "id": "dXJuOm1ieHBsYzpBY1E2",
                "geometry": {
                    "type": "Point",
                    "coordinates": [13.399690, 52.520264]
                },
                "properties": {
                    "mapbox_id": "dXJuOm1ieHBsYzpBY1E2",
                    "feature_type": "address",
                    "full_address": "Bodestraße 1, 10178 Berlin, Germany",
                    "name": "Bodestraße 1",
                    "coordinates": { "longitude": 13.399690, "latitude": 52.520264 },
                    "place_formatted": "Berlin, Germany",
                    "context": {
                        "street": { "name": "Bodestraße" },
                        "region": { "name": "Berlin", "region_code": "BE" },
                        "country": { "name": "Germany", "country_code": "DE" },
                        "place": { "name": "Berlin" },
                        "postcode": { "name": "10178" }
                    },
                    "address": "1"
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/search/geocode/v6/reverse"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "pk.test123".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let geocoder = MapBoxGeocoder::with_base_url(client, server.uri());

    let coord = Coordinate::new(52.52, 13.405).unwrap();
    let opts = ReverseGeocodeOptions::default();
    let res = geocoder.reverse_geocode(&coord, &opts).await.unwrap();

    assert_eq!(res.items.len(), 1);
    assert_eq!(
        res.items[0].title,
        Some("Bodestraße 1, 10178 Berlin, Germany".to_string())
    );
    assert_eq!(
        res.items[0].result_type,
        everymap_core::domains::search::SearchResultType::ExactMatch
    );
}
