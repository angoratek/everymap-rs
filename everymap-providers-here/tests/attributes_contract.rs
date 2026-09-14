use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::attributes::{AttributeOptions, AttributeProvider};
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::attributes::{
    HereAdminAreasResponse, HereAttributeProvider, HereBuildingsResponse, HereLandmarksResponse,
    HereRoadAttributesResponse, HereSegmentAttributesResponse,
};
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// --- Roads layer tests ---

#[tokio::test]
async fn test_attributes_roads_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "LineString",
                    "coordinates": [[13.405, 52.520], [13.406, 52.521]]
                },
                "properties": {
                    "LINK_ID": "123456789",
                    "FUNCTIONAL_CLASS": 3,
                    "SPEED_LIMIT": 50,
                    "SPEED_CATEGORY": "5",
                    "ROAD_CLASS": "A",
                    "TRAVEL_DIRECTION": "BOTH",
                    "LANES": 2,
                    "DIVIDER": "YES",
                    "URBAN": "Y",
                    "TUNNEL": "N",
                    "BRIDGE": "N",
                    "RAMP": "N",
                    "ROUNDABOUT": "N"
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/maps/attributes"))
        .and(query_param("in", "bbox:52.5,13.4,52.6,13.5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    let options = AttributeOptions {
        bbox: Some("52.5,13.4;52.6,13.5".to_string()),
        provider_extra: Some(serde_json::json!({"layers": ["roads"]})),
        ..Default::default()
    };

    let response = provider.get_attributes(&options).await.unwrap();
    assert!(response.data.as_object().unwrap().contains_key("features"));
}

#[tokio::test]
async fn test_attributes_roads_typed() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "LineString",
                    "coordinates": [[13.405, 52.520], [13.406, 52.521]]
                },
                "properties": {
                    "LINK_ID": "123456789",
                    "FUNCTIONAL_CLASS": 3,
                    "SPEED_LIMIT": 50.0,
                    "SPEED_CATEGORY": "5",
                    "ROAD_CLASS": "A",
                    "TRAVEL_DIRECTION": "BOTH",
                    "LANES": 2,
                    "DIVIDER": "YES",
                    "URBAN": "Y",
                    "TUNNEL": "N",
                    "BRIDGE": "N",
                    "RAMP": "N",
                    "ROUNDABOUT": "N",
                    "SPEED_LIMITS_BY_DIRECTION": [
                        {"SPEED_LIMIT": 50.0, "DIRECTION": "FORWARD"},
                        {"SPEED_LIMIT": 50.0, "DIRECTION": "BACKWARD"}
                    ],
                    "NAME": [{"VALUE": "Friedrichstraße", "LANGUAGE": "de"}],
                    "ROUTE_DESIGNATION": "B1"
                }
            },
            {
                "type": "Feature",
                "geometry": {
                    "type": "LineString",
                    "coordinates": [[13.410, 52.522], [13.411, 52.523]]
                },
                "properties": {
                    "LINK_ID": "987654321",
                    "FUNCTIONAL_CLASS": 1,
                    "SPEED_LIMIT": 130.0,
                    "SPEED_CATEGORY": "1",
                    "ROAD_CLASS": "M",
                    "TRAVEL_DIRECTION": "FORWARD",
                    "LANES": 3,
                    "RAMP": "N"
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/maps/attributes"))
        .and(query_param("in", "bbox:52.5,13.4,52.6,13.5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    let response: HereRoadAttributesResponse = provider
        .get_road_attributes("52.5,13.4,52.6,13.5", None)
        .await
        .unwrap();

    assert_eq!(response.features.len(), 2);

    let road1 = &response.features[0].properties;
    assert_eq!(road1.link_id.as_deref(), Some("123456789"));
    assert_eq!(road1.functional_class, Some(3));
    assert_eq!(road1.speed_limit, Some(50.0));
    assert_eq!(road1.road_class.as_deref(), Some("A"));
    assert_eq!(road1.travel_direction.as_deref(), Some("BOTH"));
    assert_eq!(road1.lanes, Some(2));
    assert_eq!(road1.divider.as_deref(), Some("YES"));
    assert_eq!(road1.urban.as_deref(), Some("Y"));
    assert_eq!(road1.tunnel.as_deref(), Some("N"));
    assert_eq!(road1.bridge.as_deref(), Some("N"));
    assert!(road1.speed_limits_by_direction.is_some());
    assert!(road1.name.is_some());

    let road2 = &response.features[1].properties;
    assert_eq!(road2.link_id.as_deref(), Some("987654321"));
    assert_eq!(road2.functional_class, Some(1));
    assert_eq!(road2.speed_limit, Some(130.0));
    assert_eq!(road2.lanes, Some(3));
}

#[tokio::test]
async fn test_attributes_roads_with_ids() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "LineString",
                    "coordinates": [[13.405, 52.520], [13.406, 52.521]]
                },
                "properties": {
                    "LINK_ID": "789012",
                    "SPEED_LIMIT": 30.0,
                    "FUNCTIONAL_CLASS": 5
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/maps/attributes"))
        .and(query_param("ids", "789012"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    let options = AttributeOptions {
        provider_extra: Some(serde_json::json!({
            "ids": ["789012"]
        })),
        ..Default::default()
    };

    let response = provider.get_attributes(&options).await.unwrap();
    assert!(response.data.as_object().unwrap().contains_key("features"));
}

// --- Segments layer test ---

#[tokio::test]
async fn test_attributes_segments_typed() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "LineString",
                    "coordinates": [[13.405, 52.520], [13.406, 52.521]]
                },
                "properties": {
                    "LINK_ID": "seg_001",
                    "REF_NODE": "node_a",
                    "NON_REF_NODE": "node_b",
                    "FUNCTIONAL_CLASS": 2,
                    "SPEED_LIMIT": 80.0,
                    "SPEED_CATEGORY": "3",
                    "LANE_COUNT": 2,
                    "ROAD_CLASS": "B"
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/maps/attributes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    let response: HereSegmentAttributesResponse = provider
        .get_segment_attributes("52.5,13.4,52.6,13.5", None)
        .await
        .unwrap();

    assert_eq!(response.features.len(), 1);
    let seg = &response.features[0].properties;
    assert_eq!(seg.link_id.as_deref(), Some("seg_001"));
    assert_eq!(seg.ref_node.as_deref(), Some("node_a"));
    assert_eq!(seg.non_ref_node.as_deref(), Some("node_b"));
    assert_eq!(seg.functional_class, Some(2));
    assert_eq!(seg.speed_limit, Some(80.0));
    assert_eq!(seg.lane_count, Some(2));
}

// --- AdminAreas layer test ---

#[tokio::test]
async fn test_attributes_admin_areas_typed() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[13.3, 52.4], [13.5, 52.4], [13.5, 52.6], [13.3, 52.6], [13.3, 52.4]]]
                },
                "properties": {
                    "ADMIN_PLACE_ID": "admin_001",
                    "ADMIN_LEVEL": 2,
                    "ADMIN_NAME": [{"VALUE": "Mitte", "LANGUAGE": "de"}],
                    "COUNTRY_ID": "DEU",
                    "STATE_ID": "DE-BE",
                    "CITY_ID": "BER"
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/maps/attributes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    let response: HereAdminAreasResponse = provider
        .get_admin_areas("52.4,13.3,52.6,13.5")
        .await
        .unwrap();

    assert_eq!(response.features.len(), 1);
    let admin = &response.features[0].properties;
    assert_eq!(admin.admin_place_id.as_deref(), Some("admin_001"));
    assert_eq!(admin.admin_level, Some(2));
    assert_eq!(admin.country_id.as_deref(), Some("DEU"));
    assert_eq!(admin.state_id.as_deref(), Some("DE-BE"));
}

// --- Buildings layer test ---

#[tokio::test]
async fn test_attributes_buildings_typed() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[13.405, 52.520], [13.406, 52.520], [13.406, 52.521], [13.405, 52.521], [13.405, 52.520]]]
                },
                "properties": {
                    "BUILDING_ID": "bldg_001",
                    "BUILDING_HEIGHT": 45.0,
                    "BUILDING_LEVELS": 12,
                    "ROOF_COLOR": "gray",
                    "ROOF_SHAPE": "flat"
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/maps/attributes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    let response: HereBuildingsResponse =
        provider.get_buildings("52.5,13.4,52.6,13.5").await.unwrap();

    assert_eq!(response.features.len(), 1);
    let bldg = &response.features[0].properties;
    assert_eq!(bldg.building_id.as_deref(), Some("bldg_001"));
    assert_eq!(bldg.building_height, Some(45.0));
    assert_eq!(bldg.building_levels, Some(12));
    assert_eq!(bldg.roof_color.as_deref(), Some("gray"));
    assert_eq!(bldg.roof_shape.as_deref(), Some("flat"));
}

// --- Landmarks layer test ---

#[tokio::test]
async fn test_attributes_landmarks_typed() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [13.377, 52.516]
                },
                "properties": {
                    "LANDMARK_ID": "lm_001",
                    "LANDMARK_NAME": [{"VALUE": "Brandenburg Gate", "LANGUAGE": "en"}],
                    "LANDMARK_TYPE": "MONUMENT",
                    "NAVI_TYPE": "POI"
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/maps/attributes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    let response: HereLandmarksResponse =
        provider.get_landmarks("52.5,13.3,52.6,13.5").await.unwrap();

    assert_eq!(response.features.len(), 1);
    let lm = &response.features[0].properties;
    assert_eq!(lm.landmark_id.as_deref(), Some("lm_001"));
    assert_eq!(lm.landmark_type.as_deref(), Some("MONUMENT"));
    assert_eq!(lm.navi_type.as_deref(), Some("POI"));
}

// --- Speed limits convenience test ---

#[tokio::test]
async fn test_attributes_speed_limits() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {"type": "LineString", "coordinates": [[13.405, 52.520], [13.406, 52.521]]},
                "properties": {
                    "LINK_ID": "speed_001",
                    "SPEED_LIMIT": 50.0,
                    "SPEED_LIMITS_BY_DIRECTION": [
                        {"SPEED_LIMIT": 50.0, "DIRECTION": "FORWARD"},
                        {"SPEED_LIMIT": 30.0, "DIRECTION": "BACKWARD"}
                    ],
                    "FUNCTIONAL_CLASS": 3,
                    "TRAVEL_DIRECTION": "BOTH",
                    "NAME": [{"VALUE": "Unter den Linden", "LANGUAGE": "de"}]
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/maps/attributes"))
        .and(query_param("layers", "SPEED_LIMITS_FCn"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    let response: HereRoadAttributesResponse = provider
        .get_speed_limits("52.5,13.4,52.6,13.5")
        .await
        .unwrap();

    assert_eq!(response.features.len(), 1);
    let road = &response.features[0].properties;
    assert_eq!(road.speed_limit, Some(50.0));
    let dir_limits = road.speed_limits_by_direction.as_ref().unwrap();
    assert_eq!(dir_limits.len(), 2);
    assert_eq!(dir_limits[0].speed_limit, Some(50.0));
    assert_eq!(dir_limits[1].speed_limit, Some(30.0));
}

// --- Road attributes by IDs test ---

#[tokio::test]
async fn test_attributes_roads_by_ids() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {"type": "LineString", "coordinates": [[13.405, 52.520], [13.406, 52.521]]},
                "properties": {
                    "LINK_ID": "link_1",
                    "SPEED_LIMIT": 60.0
                }
            },
            {
                "type": "Feature",
                "geometry": {"type": "LineString", "coordinates": [[13.410, 52.522], [13.411, 52.523]]},
                "properties": {
                    "LINK_ID": "link_2",
                    "SPEED_LIMIT": 100.0
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/maps/attributes"))
        .and(query_param("ids", "link_1,link_2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    let ids = vec!["link_1".to_string(), "link_2".to_string()];
    let response: HereRoadAttributesResponse =
        provider.get_road_attributes_by_ids(&ids).await.unwrap();

    assert_eq!(response.features.len(), 2);
    assert_eq!(response.features[0].properties.speed_limit, Some(60.0));
    assert_eq!(response.features[1].properties.speed_limit, Some(100.0));
}

// --- Partial fields / missing optional fields test ---

#[tokio::test]
async fn test_attributes_roads_minimal_fields() {
    let server = MockServer::start().await;

    // Simulate a response with only LINK_ID — all other fields missing
    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {"type": "LineString", "coordinates": [[13.405, 52.520], [13.406, 52.521]]},
                "properties": {
                    "LINK_ID": "minimal_001"
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/maps/attributes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    let response: HereRoadAttributesResponse = provider
        .get_road_attributes("52.5,13.4,52.6,13.5", None)
        .await
        .unwrap();

    assert_eq!(response.features.len(), 1);
    let road = &response.features[0].properties;
    assert_eq!(road.link_id.as_deref(), Some("minimal_001"));
    assert_eq!(road.functional_class, None);
    assert_eq!(road.speed_limit, None);
    assert_eq!(road.lanes, None);
    assert!(road.speed_limits_by_direction.is_none());
}
