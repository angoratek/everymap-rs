use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use everymap_core::domains::isoline::{IsolineProvider, IsolineOptions, RangeType};
use everymap_core::types::Coordinate;
use everymap_providers_tomtom::TomTomIsoline;
use everymap_providers_tomtom::client::TomTomClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_isoline_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "reachableRange": {
            "center": { "latitude": 52.52, "longitude": 13.405 },
            "boundary": [
                { "latitude": 52.55, "longitude": 13.405 },
                { "latitude": 52.54, "longitude": 13.45 },
                { "latitude": 52.50, "longitude": 13.45 },
                { "latitude": 52.49, "longitude": 13.405 },
                { "latitude": 52.50, "longitude": 13.36 },
                { "latitude": 52.54, "longitude": 13.36 }
            ]
        },
        "report": { "effectiveSettings": {} }
    });

    Mock::given(method("GET"))
        .and(path("/routing/1/calculateReachableRange/52.52,13.405/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(TomTomClient::new(auth));
    let isoline = TomTomIsoline::with_base_url(client, server.uri());

    let center = Coordinate::new(52.52, 13.405).unwrap();
    let opts = IsolineOptions {
        range_type: Some(RangeType::Distance),
        ..Default::default()
    };

    let res = isoline.get_isoline(&center, 5000.0, &opts).await.unwrap();

    assert_eq!(res.isolines.len(), 1);
    assert_eq!(res.isolines[0].polygon.len(), 6);
    assert_eq!(res.isolines[0].range, Some(5000.0));
}