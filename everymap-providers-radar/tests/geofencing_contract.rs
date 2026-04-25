use everymap_core::auth::{AuthProvider, HeaderAuthProvider};
use everymap_core::domains::geofencing::{
    GeofenceCreateOptions, GeofenceOptions, GeofenceProvider, GeofenceType,
};
use everymap_core::types::Coordinate;
use everymap_providers_radar::{RadarClient, RadarGeofenceProvider};
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_geofence_mock() -> (MockServer, RadarGeofenceProvider) {
    let server = MockServer::start().await;
    let auth: Arc<dyn AuthProvider> =
        Arc::new(HeaderAuthProvider::new("prj_test_pk_123".to_string()));
    let client = Arc::new(RadarClient::new(auth));
    let provider = RadarGeofenceProvider::with_base_url(
        client,
        format!("{}/v1/search/geofences", server.uri()),
        format!("{}/v1/geofences", server.uri()),
    );
    (server, provider)
}

#[tokio::test]
async fn test_search_geofences_contract() {
    let (server, provider) = setup_geofence_mock().await;

    Mock::given(method("GET"))
        .and(path("/v1/search/geofences"))
        .and(query_param("near", "40.7128,-74.006"))
        .and(query_param("radius", "1000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "geofences": [{
                "_id": "gf_abc123",
                "tag": "store",
                "externalId": "ext_1",
                "description": "NYC Store Zone",
                "type": "circle",
                "geometryCenter": { "latitude": 40.7128, "longitude": -74.006 },
                "metadata": { "level": "gold" },
                "enabled": true,
                "radius": 500.0
            }]
        })))
        .mount(&server)
        .await;

    let opts = GeofenceOptions {
        near: Some(Coordinate::new(40.7128, -74.006).unwrap()),
        radius: Some(1000.0),
        ..Default::default()
    };
    let result = provider.search_geofences(&opts).await.unwrap();
    assert_eq!(result.geofences.len(), 1);
    assert_eq!(result.geofences[0].id, "gf_abc123");
    assert_eq!(result.geofences[0].tag.as_deref(), Some("store"));
    assert!(matches!(
        result.geofences[0].geofence_type,
        Some(GeofenceType::Circle)
    ));
}

#[tokio::test]
async fn test_search_geofences_with_tags() {
    let (server, provider) = setup_geofence_mock().await;

    Mock::given(method("GET"))
        .and(path("/v1/search/geofences"))
        .and(query_param("near", "40.7128,-74.006"))
        .and(query_param("tags", "store,warehouse"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "geofences": []
        })))
        .mount(&server)
        .await;

    let opts = GeofenceOptions {
        near: Some(Coordinate::new(40.7128, -74.006).unwrap()),
        tags: vec!["store".to_string(), "warehouse".to_string()],
        ..Default::default()
    };
    let result = provider.search_geofences(&opts).await.unwrap();
    assert!(result.geofences.is_empty());
}

#[tokio::test]
async fn test_create_geofence_contract() {
    let (server, provider) = setup_geofence_mock().await;

    Mock::given(method("PUT"))
        .and(path("/v1/geofences"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "geofence": {
                "_id": "gf_new123",
                "tag": "store",
                "description": "New Store Zone",
                "type": "circle",
                "geometryCenter": { "latitude": 40.7128, "longitude": -74.006 },
                "enabled": true,
                "radius": 500.0
            }
        })))
        .mount(&server)
        .await;

    let opts = GeofenceCreateOptions {
        tag: Some("store".to_string()),
        description: Some("New Store Zone".to_string()),
        geofence_type: Some(GeofenceType::Circle),
        center: Some(Coordinate::new(40.7128, -74.006).unwrap()),
        radius: Some(500.0),
        enabled: Some(true),
        ..Default::default()
    };
    let result = provider.create_geofence(&opts).await.unwrap();
    assert_eq!(result.id, "gf_new123");
    assert_eq!(result.tag.as_deref(), Some("store"));
}

#[tokio::test]
async fn test_get_geofence_contract() {
    let (server, provider) = setup_geofence_mock().await;

    Mock::given(method("GET"))
        .and(path("/v1/geofences/gf_abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "geofence": {
                "_id": "gf_abc123",
                "tag": "store",
                "description": "NYC Store Zone",
                "type": "circle",
                "geometryCenter": { "latitude": 40.7128, "longitude": -74.006 },
                "enabled": true
            }
        })))
        .mount(&server)
        .await;

    let result = provider.get_geofence("gf_abc123").await.unwrap();
    assert_eq!(result.id, "gf_abc123");
}

#[tokio::test]
async fn test_delete_geofence_contract() {
    let (server, provider) = setup_geofence_mock().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/geofences/gf_abc123"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let result = provider.delete_geofence("gf_abc123").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_geofences_error_response() {
    let (server, provider) = setup_geofence_mock().await;

    Mock::given(method("GET"))
        .and(path("/v1/search/geofences"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 401 },
            "geofences": []
        })))
        .mount(&server)
        .await;

    let opts = GeofenceOptions {
        near: Some(Coordinate::ORIGIN),
        ..Default::default()
    };
    let result = provider.search_geofences(&opts).await;
    assert!(result.is_err());
}
