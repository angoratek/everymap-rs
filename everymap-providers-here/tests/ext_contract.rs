use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};
use everymap_core::types::Coordinate;
use everymap_core::auth::ApiKeyProvider;
use everymap_providers_here::client::HereClient;
use everymap_providers_here::ext::{
    HereGeocoderExt, HereTrafficExt, HerePositionerExt, HereTourPlannerExt, HereAttributeExt,
};
use everymap_providers_here::domain::search::{HereGeocoder, DiscoverRequest, HereDiscoverOptions};
use everymap_providers_here::domain::traffic::HereTraffic;
use everymap_providers_here::domain::positioning::HerePositioner;
use everymap_providers_here::domain::tour::HereTourPlanner;
use everymap_providers_here::domain::attributes::HereAttributeProvider;
use everymap_providers_here::{
    HerePositioningOptions, WlanAccessPoint,
    HereRoadAttributesResponse, HereSegmentAttributesResponse,
    HereAdminAreasResponse, HereBuildingsResponse, HereLandmarksResponse,
};
use everymap_providers_here::domain::traffic::{HereFlowOptions, HereIncidentsOptions};
use std::sync::Arc;

// --- HereGeocoderExt tests ---

#[tokio::test]
async fn test_geocoder_ext_discover() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "items": [
            {
                "title": "Brandenburg Gate",
                "id": "here:pds:place-123",
                "resultType": "place",
                "address": {
                    "label": "Brandenburger Tor, Berlin, Germany",
                    "countryCode": "DEU"
                },
                "position": { "lat": 52.5164, "lng": 13.3777 },
                "distance": 450.0
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

    // Call through extension trait
    let req = DiscoverRequest {
        query: "Brandenburg Gate".to_string(),
        options: HereDiscoverOptions {
            at: Some(Coordinate::new(52.52, 13.405).unwrap()),
            ..Default::default()
        },
    };

    let res = HereGeocoderExt::discover(&geocoder, req).await.unwrap();

    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].title.as_ref().unwrap(), "Brandenburg Gate");
    assert_eq!(res.items[0].distance.unwrap(), 450.0);
}

// --- HereTrafficExt tests ---

#[tokio::test]
async fn test_traffic_ext_get_flow() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "results": [
            {
                "location": {
                    "description": "A100",
                    "length": 1200.0
                },
                "currentFlow": {
                    "speed": 55.0,
                    "speedUncapped": 58.0,
                    "freeFlow": 130.0,
                    "jamFactor": 3.2,
                    "confidence": 0.88
                },
                "roadInfo": {
                    "functionalClass": 1,
                    "roadName": "A100 Stadtring",
                    "roadShield": "A100"
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/flow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let traffic = HereTraffic::with_base_url(client, server.uri());

    // Call through extension trait
    let res = HereTrafficExt::get_flow(
        &traffic,
        Coordinate::new(52.52, 13.405).unwrap(),
        &HereFlowOptions::default(),
    ).await.unwrap();

    assert_eq!(res.results.len(), 1);
    assert_eq!(res.results[0].current_flow.speed.unwrap(), 55.0);
    // Verify the serde rename fixes work: roadName and roadShield
    let road_info = res.results[0].road_info.as_ref().unwrap();
    assert_eq!(road_info.road_name.as_deref(), Some("A100 Stadtring"));
    assert_eq!(road_info.road_shield.as_deref(), Some("A100"));
}

#[tokio::test]
async fn test_traffic_ext_get_incidents() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "results": [
            {
                "location": { "description": "A9", "length": 300.0 },
                "incident": {
                    "id": "INC_EXT_1",
                    "type": "construction",
                    "criticality": "minor",
                    "startTime": "2026-04-13T06:00:00Z",
                    "endTime": "2026-04-13T18:00:00Z",
                    "description": { "value": "Roadwork on A9", "language": "en" }
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/incidents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let traffic = HereTraffic::with_base_url(client, server.uri());

    // Call through extension trait
    let res = HereTrafficExt::get_incidents(
        &traffic,
        &HereIncidentsOptions {
            in_filter: Some("bbox:13.0,52.0,14.0,53.0".to_string()),
            ..Default::default()
        },
    ).await.unwrap();

    assert_eq!(res.results.len(), 1);
    assert_eq!(res.results[0].incident.id.as_deref(), Some("INC_EXT_1"));
}

// --- HerePositionerExt tests ---

#[tokio::test]
async fn test_positioner_ext_locate() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "location": {
            "lat": 52.5201,
            "lng": 13.4051,
            "accuracy": 30.0
        },
        "altitude": {
            "value": 42.0,
            "accuracy": 5.0
        }
    });

    Mock::given(method("POST"))
        .and(path("/position"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let positioner = HerePositioner::with_base_url(client, server.uri());

    // Call through extension trait
    let opts = HerePositioningOptions {
        wlan: Some(vec![WlanAccessPoint {
            mac: "AA:BB:CC:DD:EE:FF".to_string(),
            signal_strength: Some(-60),
            age: None,
            channel: None,
            ssid: None,
            signal_to_noise_ratio: None,
        }]),
        ..Default::default()
    };

    let res = HerePositionerExt::locate(&positioner, opts).await.unwrap();

    assert_eq!(res.location.lat, 52.5201);
    assert_eq!(res.location.lng, 13.4051);
    assert_eq!(res.location.accuracy, Some(30.0));
    assert!(res.altitude.is_some());
    let alt = res.altitude.as_ref().unwrap();
    assert_eq!(alt.value, Some(42.0));
}

// --- HereTourPlannerExt tests ---

#[tokio::test]
async fn test_tour_ext_solve() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "statistic": {
            "cost": 75.0,
            "distance": 3000.0,
            "duration": 450.0,
            "stops": 3,
            "tours": 1,
            "unassignedJobs": 0
        },
        "tours": [
            {
                "vehicleId": "v1_0",
                "typeId": "v1",
                "stops": [
                    {
                        "location": { "lat": 52.52, "lng": 13.405 },
                        "time": { "arrival": "2026-04-13T08:00:00Z", "departure": "2026-04-13T08:00:00Z" },
                        "activities": [{ "type": "departure" }],
                        "distance": 0
                    }
                ],
                "statistic": {
                    "cost": 75.0, "distance": 3000.0, "duration": 450.0,
                    "stops": 3, "tours": 1, "unassignedJobs": 0
                }
            }
        ],
        "unassigned": [],
        "notices": []
    });

    Mock::given(method("POST"))
        .and(path("/problems"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let planner = HereTourPlanner::with_base_url(client, server.uri());

    // Call through extension trait
    use everymap_providers_here::TourProblem;
    let res = HereTourPlannerExt::solve(&planner, TourProblem::default()).await.unwrap();

    assert_eq!(res.statistic.cost, 75.0);
    assert_eq!(res.tours.len(), 1);
}

#[tokio::test]
async fn test_tour_ext_get_async_status() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "status": "inProgress",
        "resource": {
            "resourceId": "sol-789",
            "href": "https://tour.hereapi.com/v3/problems/sol-789/solution"
        }
    });

    Mock::given(method("GET"))
        .and(path("/status/status-abc-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let planner = HereTourPlanner::with_base_url(client, server.uri());

    // Call through extension trait
    let res = HereTourPlannerExt::get_async_status(&planner, "status-abc-123").await.unwrap();

    assert!(res.status.is_some());
    assert!(res.resource.is_some());
    let resource = res.resource.as_ref().unwrap();
    assert_eq!(resource.resource_id.as_deref(), Some("sol-789"));
}

#[tokio::test]
async fn test_tour_ext_get_solution() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "statistic": {
            "cost": 200.0,
            "distance": 10000.0,
            "duration": 1200.0,
            "stops": 5,
            "tours": 1,
            "unassignedJobs": 0
        },
        "tours": [],
        "unassigned": [],
        "notices": []
    });

    Mock::given(method("GET"))
        .and(path("/problems/prob-456/solution"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let planner = HereTourPlanner::with_base_url(client, server.uri());

    // Call through extension trait
    let res = HereTourPlannerExt::get_solution(&planner, "prob-456").await.unwrap();

    assert_eq!(res.statistic.cost, 200.0);
    assert_eq!(res.statistic.distance, 10000.0);
}

#[tokio::test]
async fn test_tour_ext_cancel() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "id": "prob-789",
        "status": "canceled",
        "started": "2026-04-13T08:00:00Z",
        "ended": "2026-04-13T08:05:00Z"
    });

    Mock::given(method("PUT"))
        .and(path("/problems/prob-789/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let planner = HereTourPlanner::with_base_url(client, server.uri());

    // Call through extension trait
    let res = HereTourPlannerExt::cancel(&planner, "prob-789").await.unwrap();

    assert_eq!(res.id.as_deref(), Some("prob-789"));
    assert_eq!(res.status.as_deref(), Some("canceled"));
}

#[tokio::test]
async fn test_tour_ext_health() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({ "status": "ok" });

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let planner = HereTourPlanner::with_base_url(client, server.uri());

    // Call through extension trait
    let res = HereTourPlannerExt::health(&planner).await.unwrap();

    assert_eq!(res.status.as_deref(), Some("ok"));
}

#[tokio::test]
async fn test_tour_ext_version() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({ "apiVersion": "3.6.0" });

    Mock::given(method("GET"))
        .and(path("/version"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let planner = HereTourPlanner::with_base_url(client, server.uri());

    // Call through extension trait
    let res = HereTourPlannerExt::version(&planner).await.unwrap();

    assert_eq!(res.api_version.as_deref(), Some("3.6.0"));
}

// --- HereAttributeExt tests ---

#[tokio::test]
async fn test_attribute_ext_get_road_attributes() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": { "type": "LineString", "coordinates": [[13.405, 52.520], [13.406, 52.521]] },
                "properties": {
                    "LINK_ID": "ext_road_1",
                    "FUNCTIONAL_CLASS": 2,
                    "SPEED_LIMIT": 80.0,
                    "ROAD_CLASS": "B",
                    "TRAVEL_DIRECTION": "FORWARD",
                    "LANES": 4
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/attributes/roads"))
        .and(query_param("bbox", "52.5,13.4;52.6,13.5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    // Call through extension trait
    let res: HereRoadAttributesResponse = HereAttributeExt::get_road_attributes(
        &provider, "52.5,13.4;52.6,13.5", None,
    ).await.unwrap();

    assert_eq!(res.features.len(), 1);
    assert_eq!(res.features[0].properties.link_id.as_deref(), Some("ext_road_1"));
    assert_eq!(res.features[0].properties.speed_limit, Some(80.0));
}

#[tokio::test]
async fn test_attribute_ext_get_segment_attributes() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": { "type": "LineString", "coordinates": [[13.4, 52.5], [13.5, 52.6]] },
                "properties": {
                    "LINK_ID": "ext_seg_1",
                    "REF_NODE": "node_x",
                    "NON_REF_NODE": "node_y",
                    "FUNCTIONAL_CLASS": 3,
                    "SPEED_LIMIT": 50.0,
                    "LANE_COUNT": 2
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/attributes/segments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    // Call through extension trait
    let res: HereSegmentAttributesResponse = HereAttributeExt::get_segment_attributes(
        &provider, "52.5,13.4;52.6,13.5", None,
    ).await.unwrap();

    assert_eq!(res.features.len(), 1);
    assert_eq!(res.features[0].properties.link_id.as_deref(), Some("ext_seg_1"));
    assert_eq!(res.features[0].properties.ref_node.as_deref(), Some("node_x"));
}

#[tokio::test]
async fn test_attribute_ext_get_admin_areas() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": { "type": "Polygon", "coordinates": [[[13.3, 52.4], [13.5, 52.6], [13.3, 52.4]]] },
                "properties": {
                    "ADMIN_PLACE_ID": "ext_admin_1",
                    "ADMIN_LEVEL": 3,
                    "COUNTRY_ID": "DEU"
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/attributes/adminAreas"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    // Call through extension trait
    let res: HereAdminAreasResponse = HereAttributeExt::get_admin_areas(
        &provider, "52.4,13.3;52.6,13.5",
    ).await.unwrap();

    assert_eq!(res.features.len(), 1);
    assert_eq!(res.features[0].properties.admin_place_id.as_deref(), Some("ext_admin_1"));
}

#[tokio::test]
async fn test_attribute_ext_get_buildings() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": { "type": "Polygon", "coordinates": [[[13.405, 52.520], [13.406, 52.521], [13.405, 52.520]]] },
                "properties": {
                    "BUILDING_ID": "ext_bldg_1",
                    "BUILDING_HEIGHT": 30.0,
                    "BUILDING_LEVELS": 8
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/attributes/buildings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    // Call through extension trait
    let res: HereBuildingsResponse = HereAttributeExt::get_buildings(
        &provider, "52.5,13.4;52.6,13.5",
    ).await.unwrap();

    assert_eq!(res.features.len(), 1);
    assert_eq!(res.features[0].properties.building_id.as_deref(), Some("ext_bldg_1"));
    assert_eq!(res.features[0].properties.building_height, Some(30.0));
}

#[tokio::test]
async fn test_attribute_ext_get_landmarks() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": { "type": "Point", "coordinates": [13.377, 52.516] },
                "properties": {
                    "LANDMARK_ID": "ext_lm_1",
                    "LANDMARK_TYPE": "MONUMENT",
                    "NAVI_TYPE": "POI"
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/attributes/landmarks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    // Call through extension trait
    let res: HereLandmarksResponse = HereAttributeExt::get_landmarks(
        &provider, "52.5,13.3;52.6,13.5",
    ).await.unwrap();

    assert_eq!(res.features.len(), 1);
    assert_eq!(res.features[0].properties.landmark_id.as_deref(), Some("ext_lm_1"));
}

#[tokio::test]
async fn test_attribute_ext_get_road_attributes_by_ids() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": { "type": "LineString", "coordinates": [[13.4, 52.5], [13.5, 52.6]] },
                "properties": {
                    "LINK_ID": "ext_id_1",
                    "SPEED_LIMIT": 100.0
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/attributes/roads"))
        .and(query_param("ids", "ext_id_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    // Call through extension trait
    let ids = vec!["ext_id_1".to_string()];
    let res: HereRoadAttributesResponse = HereAttributeExt::get_road_attributes_by_ids(
        &provider, &ids,
    ).await.unwrap();

    assert_eq!(res.features.len(), 1);
    assert_eq!(res.features[0].properties.speed_limit, Some(100.0));
}

#[tokio::test]
async fn test_attribute_ext_get_speed_limits() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": { "type": "LineString", "coordinates": [[13.4, 52.5], [13.5, 52.6]] },
                "properties": {
                    "LINK_ID": "ext_speed_1",
                    "SPEED_LIMIT": 50.0,
                    "SPEED_LIMITS_BY_DIRECTION": [
                        { "SPEED_LIMIT": 50.0, "DIRECTION": "FORWARD" },
                        { "SPEED_LIMIT": 30.0, "DIRECTION": "BACKWARD" }
                    ]
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/attributes/roads"))
        .and(query_param("bbox", "52.5,13.4;52.6,13.5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    // Call through extension trait
    let res: HereRoadAttributesResponse = HereAttributeExt::get_speed_limits(
        &provider, "52.5,13.4;52.6,13.5",
    ).await.unwrap();

    assert_eq!(res.features.len(), 1);
    let road = &res.features[0].properties;
    assert_eq!(road.speed_limit, Some(50.0));
    let dir_limits = road.speed_limits_by_direction.as_ref().unwrap();
    assert_eq!(dir_limits.len(), 2);
}