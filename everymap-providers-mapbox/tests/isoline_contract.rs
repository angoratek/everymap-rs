use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::isoline::{IsolineOptions, IsolineProvider, RangeType};
use everymap_core::types::Coordinate;
use everymap_providers_mapbox::client::MapBoxClient;
use everymap_providers_mapbox::MapBoxIsoline;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_isoline_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": {
                    "contour": 30,
                    "color": "ff0000"
                },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[13.3, 52.4], [13.5, 52.4], [13.5, 52.6], [13.3, 52.6], [13.3, 52.4]]]
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/isochrone/v1/mapbox/driving/13.405,52.52"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "pk.test123".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let isoline = MapBoxIsoline::with_base_url(client, server.uri());

    let center = Coordinate::new(52.52, 13.405).unwrap();
    let options = IsolineOptions {
        range_type: Some(RangeType::Time),
        ..Default::default()
    };
    let response = isoline
        .get_isoline(&center, 1800.0, &options)
        .await
        .unwrap();

    assert_eq!(response.isolines.len(), 1);
    assert_eq!(response.isolines[0].polygon.len(), 5); // 4 corners + closing
    assert_eq!(response.isolines[0].range, Some(1800.0)); // 30 min * 60
}
