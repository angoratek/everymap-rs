use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::search::{GeocodeOptions, Geocoder, ReverseGeocodeOptions};
use everymap_core::types::Coordinate;
use everymap_providers_tomtom::client::TomTomClient;
use everymap_providers_tomtom::TomTomGeocoder;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_geocode_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "results": [
            {
                "position": { "lat": 52.5200, "lon": 13.4050 },
                "address": {
                    "freeformAddress": "Brandenburg Gate, Berlin",
                    "municipality": "Berlin",
                    "country": "Germany",
                    "countryCode": "DEU"
                },
                "resultType": "Point Address",
                "score": 0.95,
                "id": "tomtom_place_1"
            }
        ],
        "summary": { "numResults": 1, "query": "Brandenburg Gate" }
    });

    Mock::given(method("GET"))
        .and(path("/search/2/geocode/Brandenburg+Gate.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let geocoder = TomTomGeocoder::with_base_url(client, server.uri());

    let options = GeocodeOptions::default();
    let response = geocoder
        .geocode("Brandenburg Gate", &options)
        .await
        .unwrap();

    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].coordinate.lat, 52.5200);
    assert_eq!(response.items[0].coordinate.lng, 13.4050);
}

#[tokio::test]
async fn test_reverse_geocode_contract() {
    let server = MockServer::start().await;

    // TomTom reverse geocode API returns results under "addresses" (not "results")
    // and position is a "lat,lon" string (not an object)
    let mock_response = serde_json::json!({
        "addresses": [
            {
                "address": {
                    "streetName": "Bodestraße",
                    "countryCode": "DE",
                    "countrySubdivision": "Berlin",
                    "municipality": "Berlin",
                    "postalCode": "10178",
                    "neighbourhood": "Museumsinsel",
                    "country": "Deutschland",
                    "freeformAddress": "Bodestraße, 10178 Berlin",
                    "localName": "Berlin"
                },
                "position": "52.520264,13.399690",
                "id": "N2etwGOTOIfZi68KYi2JQQ"
            }
        ],
        "summary": { "numResults": 1, "queryType": "ReverseGeometry" }
    });

    Mock::given(method("GET"))
        .and(path("/search/2/reverseGeocode/52.52,13.405.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let geocoder = TomTomGeocoder::with_base_url(client, server.uri());

    let coordinate = Coordinate::new(52.52, 13.405).unwrap();
    let options = ReverseGeocodeOptions::default();
    let response = geocoder
        .reverse_geocode(&coordinate, &options)
        .await
        .unwrap();

    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].coordinate.lat, 52.520264);
    assert_eq!(response.items[0].coordinate.lng, 13.399690);
    assert_eq!(
        response.items[0].address.street.as_deref(),
        Some("Bodestraße")
    );
    assert_eq!(response.items[0].address.city.as_deref(), Some("Berlin"));
}
