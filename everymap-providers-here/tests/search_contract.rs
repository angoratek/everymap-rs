use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};
use everymap_core::types::Coordinate;
use everymap_core::domains::search::{GeocodeRequest, ReverseGeocodeRequest, Geocoder};
use everymap_providers_here::domain::search::{
    HereGeocoder, HereGeocodeOptions, HereDiscoverOptions, HereAutosuggestOptions,
};
use everymap_providers_here::client::HereClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_geocode_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "items": [
            {
                "position": { "lat": 52.5200, "lng": 13.4050 },
                "address": { "label": "Berlin, Germany" },
                "title": "Berlin",
                "resultType": "locality",
                "id": "here:abc123"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/geocode"))
        .and(query_param("q", "Berlin"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let geocoder = HereGeocoder::with_base_url(client, server.uri());

    let req = GeocodeRequest {
        query: "Berlin".to_string(),
        options: HereGeocodeOptions::default(),
    };
    let res = geocoder.geocode(req).await.unwrap();

    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].coordinate, Coordinate::new(52.5200, 13.4050).unwrap());
    assert_eq!(res.items[0].address.label.as_deref(), Some("Berlin, Germany"));
    assert_eq!(res.items[0].title.as_ref().unwrap(), "Berlin");
}

#[tokio::test]
async fn test_geocode_with_options() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "items": [
            {
                "position": { "lat": 48.8566, "lng": 2.3522 },
                "address": { "label": "Paris, France" },
                "title": "Paris",
                "resultType": "locality",
                "id": "here:paris123"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/geocode"))
        .and(query_param("q", "Paris"))
        .and(query_param("limit", "5"))
        .and(query_param("lang", "en"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let geocoder = HereGeocoder::with_base_url(client, server.uri());

    let req = GeocodeRequest {
        query: "Paris".to_string(),
        options: HereGeocodeOptions {
            limit: Some(5),
            lang: Some("en".to_string()),
            ..Default::default()
        },
    };
    let res = geocoder.geocode(req).await.unwrap();

    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].address.label.as_deref(), Some("Paris, France"));
}

#[tokio::test]
async fn test_reverse_geocode_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "items": [
            {
                "position": { "lat": 52.5200, "lng": 13.4050 },
                "address": { "label": "Berlin, Germany" }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/reverseGeocode"))
        .and(query_param("at", "52.52,13.405"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let geocoder = HereGeocoder::with_base_url(client, server.uri());

    let req = ReverseGeocodeRequest {
        coordinate: Coordinate::new(52.52, 13.405).unwrap(),
        options: HereGeocodeOptions::default(),
    };
    let res = geocoder.reverse_geocode(req).await.unwrap();

    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].coordinate, Coordinate::new(52.5200, 13.4050).unwrap());
    assert_eq!(res.items[0].address.label.as_deref(), Some("Berlin, Germany"));
}

#[tokio::test]
async fn test_discover_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "items": [
            {
                "title": "Brandenburg Gate",
                "id": "here:pds:place-123",
                "resultType": "place",
                "address": {
                    "label": "Brandenburger Tor, Pariser Platz, 10117 Berlin, Germany",
                    "countryCode": "DEU",
                    "countryName": "Germany",
                    "city": "Berlin",
                    "street": "Pariser Platz"
                },
                "position": { "lat": 52.5164, "lng": 13.3777 },
                "distance": 450.0,
                "categories": [
                    { "id": "200-2000-0000", "name": "Landmark-Attraction", "primary": true }
                ]
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/discover"))
        .and(query_param("q", "Brandenburg Gate"))
        .and(query_param("at", "52.52,13.405"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let geocoder = HereGeocoder::with_base_url(client, server.uri());

    let req = everymap_core::domains::search::DiscoverRequest {
        query: "Brandenburg Gate".to_string(),
        options: HereDiscoverOptions {
            at: Some(Coordinate::new(52.52, 13.405).unwrap()),
            ..Default::default()
        },
    };

    let res = geocoder.discover(req).await.unwrap();

    assert_eq!(res.items.len(), 1);
    let item = &res.items[0];
    assert_eq!(item.title.as_ref().unwrap(), "Brandenburg Gate");
    assert_eq!(item.result_type.as_ref().unwrap(), "place");
    assert_eq!(item.distance.unwrap(), 450.0);
    assert_eq!(item.categories.len(), 1);
    assert_eq!(item.categories[0].id.as_ref().unwrap(), "200-2000-0000");
    assert!(item.address.is_some());
    let addr = item.address.as_ref().unwrap();
    assert_eq!(addr.label.as_ref().unwrap(), "Brandenburger Tor, Pariser Platz, 10117 Berlin, Germany");
}

#[tokio::test]
async fn test_autosuggest_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "items": [
            {
                "title": "Berlin",
                "id": "here:cmc:123",
                "resultType": "locality",
                "address": {
                    "label": "Berlin, Germany",
                    "countryCode": "DEU"
                },
                "position": { "lat": 52.52, "lng": 13.405 },
                "highlights": {
                    "title": [{ "start": 0, "end": 6 }]
                }
            }
        ],
        "queryTerms": [
            { "term": "berlin", "displayText": "Berlin" }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/autosuggest"))
        .and(query_param("q", "Berl"))
        .and(query_param("limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let geocoder = HereGeocoder::with_base_url(client, server.uri());

    let req = everymap_core::domains::search::AutosuggestRequest {
        query: "Berl".to_string(),
        options: HereAutosuggestOptions {
            limit: Some(5),
            ..Default::default()
        },
    };

    let res = geocoder.autosuggest(req).await.unwrap();

    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].title.as_ref().unwrap(), "Berlin");
    assert_eq!(res.items[0].result_type.as_ref().unwrap(), "locality");
    assert_eq!(res.query_terms.len(), 1);
    assert_eq!(res.query_terms[0].term.as_ref().unwrap(), "berlin");
}