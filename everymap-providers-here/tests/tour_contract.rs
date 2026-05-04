use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::tour::{TourOptions, TourPlanner};
use everymap_core::types::Coordinate;
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::tour::{
    Fleet, FleetTraffic, HereTourPlanner, Job, JobPlace, JobTask, JobTasks, Objective, Plan,
    Profile, ShiftStart, TourLocation, TourProblem, VehicleCosts, VehicleShift, VehicleType,
};
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_tour_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "statistic": {
            "cost": 100.0,
            "distance": 5000.0,
            "duration": 600.0,
            "stops": 4,
            "tours": 1,
            "unassignedJobs": 0
        },
        "tours": [
            {
                "vehicleId": "vehicle_1_0",
                "typeId": "vehicle_1",
                "stops": [
                    {
                        "location": { "lat": 52.52, "lng": 13.405 },
                        "time": { "arrival": "2024-01-01T08:00:00Z", "departure": "2024-01-01T08:00:00Z" },
                        "activities": [{ "type": "departure" }],
                        "distance": 0
                    },
                    {
                        "location": { "lat": 52.53, "lng": 13.41 },
                        "time": { "arrival": "2024-01-01T08:10:00Z", "departure": "2024-01-01T08:11:00Z" },
                        "activities": [{ "jobId": "stop_0", "type": "delivery" }],
                        "distance": 1000
                    },
                    {
                        "location": { "lat": 52.54, "lng": 13.42 },
                        "time": { "arrival": "2024-01-01T08:20:00Z", "departure": "2024-01-01T08:21:00Z" },
                        "activities": [{ "jobId": "stop_1", "type": "delivery" }],
                        "distance": 1000
                    },
                    {
                        "location": { "lat": 52.52, "lng": 13.405 },
                        "time": { "arrival": "2024-01-01T08:30:00Z", "departure": "2024-01-01T08:30:00Z" },
                        "activities": [{ "type": "arrival" }],
                        "distance": 3000
                    }
                ],
                "statistic": {
                    "cost": 100.0,
                    "distance": 5000.0,
                    "duration": 600.0,
                    "stops": 4,
                    "tours": 1,
                    "unassignedJobs": 0
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

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let planner = HereTourPlanner::with_base_url(client, server.uri());

    let stops = vec![
        Coordinate::new(52.53, 13.41).unwrap(),
        Coordinate::new(52.54, 13.42).unwrap(),
    ];
    let options = TourOptions::default();

    let response = planner.optimize_tour(&stops, &options).await.unwrap();

    // Core trait returns the tour stops
    assert!(!response.stops.is_empty());
    assert_eq!(response.total_distance, Some(5000.0));
    assert_eq!(response.total_duration, Some(600.0));
}

#[tokio::test]
async fn test_tour_solve_rich() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "statistic": {
            "cost": 150.0,
            "distance": 8000.0,
            "duration": 900.0,
            "times": { "driving": 500, "serving": 300, "waiting": 50, "break": 50 },
            "stops": 4,
            "tours": 1,
            "unassignedJobs": 0
        },
        "tours": [
            {
                "vehicleId": "truck_1_0",
                "typeId": "truck_1",
                "stops": [
                    {
                        "location": { "lat": 52.52, "lng": 13.405 },
                        "time": { "arrival": "2024-01-01T08:00:00Z", "departure": "2024-01-01T08:00:00Z" },
                        "activities": [{ "type": "departure" }],
                        "distance": 0
                    },
                    {
                        "location": { "lat": 52.53, "lng": 13.41 },
                        "time": { "arrival": "2024-01-01T08:10:00Z", "departure": "2024-01-01T08:11:00Z" },
                        "activities": [{ "jobId": "job1", "type": "delivery", "demand": [1] }],
                        "distance": 4000
                    }
                ],
                "statistic": {
                    "cost": 150.0,
                    "distance": 8000.0,
                    "duration": 900.0,
                    "stops": 2,
                    "tours": 1,
                    "unassignedJobs": 0
                }
            }
        ],
        "unassigned": [
            {
                "jobId": "job2",
                "reasons": [{ "code": "CAPACITY_CONSTRAINT", "description": "Vehicle capacity exceeded" }]
            }
        ],
        "notices": [{ "code": "unusedVehicleProfile", "title": "Profile not used" }]
    });

    Mock::given(method("POST"))
        .and(path("/problems"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let planner = HereTourPlanner::with_base_url(client, server.uri());

    let problem = TourProblem {
        fleet: Fleet {
            types: vec![VehicleType {
                id: "truck_1".to_string(),
                profile: "truck_profile".to_string(),
                costs: VehicleCosts {
                    fixed: Some(50.0),
                    distance: Some(0.01),
                    time: None,
                    job: None,
                },
                shifts: vec![VehicleShift {
                    start: ShiftStart {
                        time: Some("2024-01-01T08:00:00Z".to_string()),
                        earliest: None,
                        location: Some(TourLocation {
                            lat: 52.52,
                            lng: 13.405,
                        }),
                    },
                    ..Default::default()
                }],
                capacity: Some(vec![10]),
                amount: Some(1),
                ..Default::default()
            }],
            profiles: vec![Profile::Truck {
                name: "truck_profile".to_string(),
                departure_time: None,
                traffic: None,
            }],
            traffic: Some(FleetTraffic::Automatic),
        },
        plan: Plan {
            jobs: vec![Job {
                id: "job1".to_string(),
                tasks: JobTasks {
                    deliveries: Some(vec![JobTask {
                        places: vec![JobPlace {
                            location: TourLocation {
                                lat: 52.53,
                                lng: 13.41,
                            },
                            duration: 60,
                            ..Default::default()
                        }],
                        demand: vec![1],
                        ..Default::default()
                    }]),
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        },
        objectives: Some(vec![Objective::MinimizeCost]),
        ..Default::default()
    };

    let solution = planner.solve(problem).await.unwrap();

    // Verify rich response
    assert_eq!(solution.tours.len(), 1);
    assert_eq!(solution.statistic.cost, 150.0);
    assert_eq!(solution.unassigned.len(), 1);
    assert_eq!(solution.unassigned[0].job_id.as_deref(), Some("job2"));
    assert_eq!(solution.notices.len(), 1);
    assert!(solution.statistic.times.is_some());
    let times = solution.statistic.times.as_ref().unwrap();
    assert_eq!(times.driving, 500);
}

#[tokio::test]
async fn test_tour_async_submit() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "statusId": "abc-123-def",
        "href": "https://tourplanning.hereapi.com/v3/status/abc-123-def"
    });

    Mock::given(method("POST"))
        .and(path("/problems/async"))
        .respond_with(ResponseTemplate::new(202).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let planner = HereTourPlanner::with_base_url(client, server.uri());

    let result = planner.solve_async(TourProblem::default()).await.unwrap();

    assert_eq!(result.status_id.as_deref(), Some("abc-123-def"));
}

#[tokio::test]
async fn test_tour_version() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({ "apiVersion": "3.5.0" });

    Mock::given(method("GET"))
        .and(path("/version"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let planner = HereTourPlanner::with_base_url(client, server.uri());

    let version = planner.version().await.unwrap();
    assert_eq!(version.api_version.as_deref(), Some("3.5.0"));
}
