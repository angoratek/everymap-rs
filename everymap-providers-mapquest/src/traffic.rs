//! Traffic flow and incidents through the MapQuest Traffic API v2.
//!
//! One call to
//! [`TrafficProvider::get_traffic`]
//! reads the JSON flow endpoint and, when the caller asks for them, the
//! incident endpoint.
//!
//! # Flow
//!
//! The flow endpoint serves an image by default; with `outFormat=json` it
//! returns a GeoJSON `FeatureCollection` of congestion polylines. Each feature
//! carries `properties.traffic` with one of the documented overlay buckets
//! (`green`, `yellow`, `red`, `black`) and a `MultiLineString` of road
//! segments. No per-segment speed is reported, so each projected
//! [`TrafficFlow`] carries its
//! geometry and congestion level and leaves the speed fields empty.
//!
//! The service bounds one flow request to roughly one square degree, so the
//! provider selects the most detailed documented Web-Mercator zoom level whose
//! square coverage still encloses the requested ground and stays inside that
//! limit. A box that cannot be served is rejected before any request.
//!
//! # Ground
//!
//! [`TrafficOptions::bounding_box`](everymap_core::domains::traffic::TrafficOptions)
//! is honoured exactly: the flow view and the incident query cover that box.
//! When no box is supplied, the provider derives one around the requested
//! location from the requested radius, or a documented default radius.
//!
//! # Incidents
//!
//! The incident endpoint returns `incidents[]`. Congestion entries
//! (`type == 3`) are flow conditions rather than disruptions and are skipped;
//! construction (`1`), event (`2`) and accident (`4`) entries are projected.
//! The service's severity scale (0 through 4) maps onto the core levels from
//! [`IncidentSeverity::Low`] through [`IncidentSeverity::Major`].

use crate::client::MapQuestClient;
use crate::error_mapping::invalid_response;
use crate::json_coordinates::bounding_box_to_query_text;
use async_trait::async_trait;
use everymap_core::domains::traffic::{
    CongestionLevel, IncidentSeverity, TrafficFlow, TrafficIncident, TrafficOptions,
    TrafficProvider, TrafficResponse,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::{BoundingBox, Coordinate};
use serde_json::Value;
use std::sync::Arc;

/// Service path of the JSON flow endpoint.
const FLOW_PATH: &str = "traffic/v2/flow";

/// Service path of the incident endpoint.
const INCIDENTS_PATH: &str = "traffic/v2/incidents";

/// Query parameter naming the incident bounding box.
const BOUNDING_BOX_PARAMETER: &str = "boundingBox";

/// Query parameter selecting the map projection.
const PROJECTION_PARAMETER: &str = "projection";

/// Projection value requesting Web-Mercator.
const MERCATOR_PROJECTION: &str = "merc";

/// Query parameter naming the view centre latitude.
const MAP_LATITUDE_PARAMETER: &str = "mapLat";

/// Query parameter naming the view centre longitude.
const MAP_LONGITUDE_PARAMETER: &str = "mapLng";

/// Query parameter naming the view width in pixels.
const MAP_WIDTH_PARAMETER: &str = "mapWidth";

/// Query parameter naming the view height in pixels.
const MAP_HEIGHT_PARAMETER: &str = "mapHeight";

/// Query parameter naming the documented map scale.
const MAP_SCALE_PARAMETER: &str = "mapScale";

/// Query parameter selecting the lowest severity bucket to include.
const MINIMUM_SEVERITY_PARAMETER: &str = "minSeverity";

/// Lowest severity bucket; the request asks for every overlay colour.
const MINIMUM_SEVERITY_FILTER: &str = "green";

/// Query parameter capping the number of drawn line segments.
const MAX_LINES_PARAMETER: &str = "maxLines";

/// Maximum line count the flow request asks for.
const MAXIMUM_FLOW_LINE_COUNT: u32 = 1000;

/// Query parameter selecting the response format.
const OUTPUT_FORMAT_PARAMETER: &str = "outFormat";

/// Response format value requesting GeoJSON.
const JSON_OUTPUT_FORMAT: &str = "json";

/// Square map side, in pixels, the flow request never exceeds.
const MAXIMUM_FLOW_PIXEL_SIDE: u32 = 1600;

/// Degrees of Web-Mercator latitude covered per pixel at scale 1.0.
///
/// The service derives covered degrees as
/// `pixels * mapScale / 72 / 5_700_000`.
const DEGREES_PER_PIXEL_PER_SCALE: f64 = 1.0 / (72.0 * 5_700_000.0);

/// Web-Mercator zoom levels the service accepts, largest scale (most zoomed
/// out) to smallest, with their documented `mapScale` values.
const MAP_SCALE_BY_ZOOM_LEVEL: [(u32, f64); 8] = [
    (11, 216_671.0),
    (12, 108_335.0),
    (13, 54_167.0),
    (14, 27_083.0),
    (15, 13_541.0),
    (16, 6_770.0),
    (17, 3_385.0),
    (18, 1_692.0),
];

/// Safety margin subtracted from the one-degree area limit to absorb the
/// Mercator approximations the service applies to map width and height.
const AREA_LIMIT_SAFETY_MARGIN_DEGREES: f64 = 0.02;

/// Meters per degree of latitude at the equator.
const METERS_PER_DEGREE_LATITUDE: f64 = 111_320.0;

/// Radius used to derive a box when the caller supplies neither a bounding
/// box nor a radius.
const DEFAULT_SEARCH_RADIUS_METERS: f64 = 1_000.0;

/// Smallest cosine divisor used when deriving a box near a pole, so the
/// longitude span stays finite.
const MINIMUM_LATITUDE_COSINE: f64 = 0.01;

/// Traffic flow and incident data from the MapQuest Traffic API v2.
///
/// Construct one with [`Self::new`] around a shared [`MapQuestClient`].
pub struct MapQuestTrafficProvider {
    client: Arc<MapQuestClient>,
}

impl MapQuestTrafficProvider {
    /// Builds a traffic provider around a shared MapQuest client.
    ///
    /// # Parameters
    /// - `client` — the client every request goes through.
    ///
    /// # Returns
    /// A provider ready to read flow and incident data.
    pub fn new(client: Arc<MapQuestClient>) -> Self {
        Self { client }
    }
}

/// Request geometry for one flow call: the selected Web-Mercator zoom level,
/// its documented `mapScale`, and the square pixel side that covers the box.
struct FlowViewParameters {
    /// Documented `mapScale` matching the selected zoom level.
    map_scale: f64,
    /// Square map width and height in pixels.
    side_pixels: u32,
}

#[async_trait]
impl TrafficProvider for MapQuestTrafficProvider {
    /// Returns traffic flow and, when requested, incidents for an area.
    ///
    /// # Parameters
    /// - `location` — the area centre when the options carry no bounding box.
    /// - `options` — the bounding box, radius, incident switch and any
    ///   provider extras. A bounding box is used exactly; without one the
    ///   provider derives a box from `radius` or a default radius.
    ///
    /// # Returns
    /// Flow observations, one per reported road polyline, and the requested
    /// incident list.
    ///
    /// # Errors
    /// - [`EveryMapError::ValidationError`] when the box exceeds the service's
    ///   area limit or cannot be served at the box's latitude.
    /// - The shared client's HTTP, rate-limit or transport error for an
    ///   unsuccessful response.
    /// - [`EveryMapError::SerializationError`] when a body is not JSON.
    /// - [`EveryMapError::ProviderError`] when a payload lacks a required
    ///   field or reports an unknown bucket or incident type.
    async fn get_traffic(
        &self,
        location: &Coordinate,
        options: &TrafficOptions,
    ) -> EveryMapResult<TrafficResponse> {
        if options.language.is_some() {
            log::warn!("MapQuest traffic does not support a response language; ignoring");
        }
        if options.provider_extra.is_some() {
            log::warn!("MapQuest traffic does not consume provider_extra; ignoring");
        }
        let bounding_box = match options.bounding_box {
            Some(bounding_box) => bounding_box,
            None => bounding_box_around(
                location,
                options.radius.unwrap_or(DEFAULT_SEARCH_RADIUS_METERS),
            )?,
        };
        let view = select_flow_view(&bounding_box)?;
        let center_latitude = (bounding_box.south_west.lat + bounding_box.north_east.lat) / 2.0;
        let center_longitude = (bounding_box.south_west.lng + bounding_box.north_east.lng) / 2.0;
        let flow_query = vec![
            (
                PROJECTION_PARAMETER.to_string(),
                MERCATOR_PROJECTION.to_string(),
            ),
            (
                MAP_LATITUDE_PARAMETER.to_string(),
                center_latitude.to_string(),
            ),
            (
                MAP_LONGITUDE_PARAMETER.to_string(),
                center_longitude.to_string(),
            ),
            (
                MAP_WIDTH_PARAMETER.to_string(),
                view.side_pixels.to_string(),
            ),
            (
                MAP_HEIGHT_PARAMETER.to_string(),
                view.side_pixels.to_string(),
            ),
            (MAP_SCALE_PARAMETER.to_string(), view.map_scale.to_string()),
            (
                MINIMUM_SEVERITY_PARAMETER.to_string(),
                MINIMUM_SEVERITY_FILTER.to_string(),
            ),
            (
                MAX_LINES_PARAMETER.to_string(),
                MAXIMUM_FLOW_LINE_COUNT.to_string(),
            ),
            (
                OUTPUT_FORMAT_PARAMETER.to_string(),
                JSON_OUTPUT_FORMAT.to_string(),
            ),
        ];
        let flow_payload = self.client.get_json(FLOW_PATH, &flow_query).await?;
        let flows = parse_flow_payload(&flow_payload)?;

        let incidents = if options.include_incidents.unwrap_or(false) {
            let incident_query = vec![(
                BOUNDING_BOX_PARAMETER.to_string(),
                bounding_box_to_query_text(&bounding_box),
            )];
            let incident_payload = self
                .client
                .get_json(INCIDENTS_PATH, &incident_query)
                .await?;
            parse_incidents_payload(&incident_payload)?
        } else {
            Vec::new()
        };

        Ok(TrafficResponse {
            flows,
            incidents,
            raw: None,
        })
    }
}

/// Derives a latitude/longitude box around a point.
///
/// # Parameters
/// - `location` — the box centre.
/// - `radius_meters` — half the box's side in meters.
///
/// # Returns
/// A box whose north-east and south-west corners are `radius_meters` from the
/// centre; the longitude span is widened by the centre's latitude so the box
/// covers the same ground at any latitude.
///
/// # Errors
/// Returns [`EveryMapError::ValidationError`] when the derived corner falls
/// outside the coordinate range.
fn bounding_box_around(location: &Coordinate, radius_meters: f64) -> EveryMapResult<BoundingBox> {
    let latitude_delta = radius_meters / METERS_PER_DEGREE_LATITUDE;
    let center_cosine = location
        .lat
        .to_radians()
        .cos()
        .abs()
        .max(MINIMUM_LATITUDE_COSINE);
    let longitude_delta = latitude_delta / center_cosine;
    let invalid_coordinate = |error: everymap_core::types::coordinate::CoordinateError| {
        EveryMapError::ValidationError(error.to_string())
    };
    let north_east = Coordinate::new(
        location.lat + latitude_delta,
        location.lng + longitude_delta,
    )
    .map_err(invalid_coordinate)?;
    let south_west = Coordinate::new(
        location.lat - latitude_delta,
        location.lng - longitude_delta,
    )
    .map_err(invalid_coordinate)?;
    Ok(BoundingBox::new(north_east, south_west))
}

/// Selects the most detailed flow view that covers a box within the service's
/// area limit.
///
/// # Parameters
/// - `bounding_box` — the ground the flow request must cover.
///
/// # Returns
/// The documented `mapScale` and square pixel side for the request.
///
/// # Errors
/// Returns [`EveryMapError::ValidationError`] when the box spans more than the
/// service's one-degree area limit, or when no documented zoom level can cover
/// it at the box's latitude.
fn select_flow_view(bounding_box: &BoundingBox) -> EveryMapResult<FlowViewParameters> {
    let latitude_span_degrees = bounding_box.north_east.lat - bounding_box.south_west.lat;
    let longitude_span_degrees = bounding_box.north_east.lng - bounding_box.south_west.lng;
    let center_latitude = (bounding_box.south_west.lat + bounding_box.north_east.lat) / 2.0;
    let center_cosine = center_latitude.to_radians().cos();
    let area_limit_degrees = 1.0 - AREA_LIMIT_SAFETY_MARGIN_DEGREES;
    if latitude_span_degrees > area_limit_degrees || longitude_span_degrees > area_limit_degrees {
        return Err(EveryMapError::ValidationError(
            "traffic flow bounding box exceeds the MapQuest one-degree area limit".to_string(),
        ));
    }
    for (zoom_level, map_scale) in MAP_SCALE_BY_ZOOM_LEVEL.iter().rev() {
        let latitude_pixels =
            (latitude_span_degrees / (map_scale * DEGREES_PER_PIXEL_PER_SCALE)).ceil();
        let longitude_pixels = (longitude_span_degrees * center_cosine
            / (map_scale * DEGREES_PER_PIXEL_PER_SCALE))
            .ceil();
        let side_pixels = latitude_pixels.max(longitude_pixels);
        if side_pixels > f64::from(MAXIMUM_FLOW_PIXEL_SIDE) {
            continue;
        }
        let covered_degrees = side_pixels * map_scale * DEGREES_PER_PIXEL_PER_SCALE;
        let fits_area_limit = covered_degrees <= area_limit_degrees
            && covered_degrees / center_cosine <= area_limit_degrees;
        if fits_area_limit {
            return Ok(FlowViewParameters {
                map_scale: *map_scale,
                side_pixels: side_pixels as u32,
            });
        }
        log::trace!(
            "flow zoom level {zoom_level} covers {covered_degrees} degrees and misses the area limit"
        );
    }
    Err(EveryMapError::ValidationError(
        "traffic flow bounding box cannot be served within the MapQuest area limit at this latitude"
            .to_string(),
    ))
}

/// Projects a GeoJSON flow collection into per-polyline flow observations.
///
/// # Parameters
/// - `payload` — a decoded flow response body carrying a `features` array.
///
/// # Returns
/// One [`TrafficFlow`] per line string, with its geometry and congestion
/// level; the speed fields stay empty because the service reports none.
///
/// # Errors
/// Returns [`EveryMapError::ProviderError`] when `features` is missing, a
/// feature lacks its traffic bucket or geometry, a point is not a
/// `[longitude, latitude]` pair, or a line string carries no points.
pub fn parse_flow_payload(payload: &Value) -> EveryMapResult<Vec<TrafficFlow>> {
    let features = payload
        .get("features")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("flow payload lacks the features array"))?;
    let mut flows = Vec::new();
    for feature in features {
        let congestion_level = map_traffic_bucket_to_congestion_level(
            feature
                .pointer("/properties/traffic")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid_response("flow feature lacks properties.traffic"))?,
        )?;
        let line_strings = feature
            .pointer("/geometry/coordinates")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid_response("flow feature lacks geometry coordinates"))?;
        for line_string in line_strings {
            let point_pairs = line_string
                .as_array()
                .ok_or_else(|| invalid_response("flow line string is not an array"))?;
            let mut geometry = Vec::with_capacity(point_pairs.len());
            for point_pair in point_pairs {
                let values = point_pair
                    .as_array()
                    .ok_or_else(|| invalid_response("flow point is not an [lng, lat] array"))?;
                if values.len() != 2 {
                    return Err(invalid_response("flow point lacks exactly lng and lat"));
                }
                let longitude = values[0]
                    .as_f64()
                    .ok_or_else(|| invalid_response("flow point lacks lng"))?;
                let latitude = values[1]
                    .as_f64()
                    .ok_or_else(|| invalid_response("flow point lacks lat"))?;
                geometry.push(
                    Coordinate::new(latitude, longitude)
                        .map_err(|error| invalid_response(error.to_string()))?,
                );
            }
            if geometry.is_empty() {
                return Err(invalid_response("flow line string carries no points"));
            }
            flows.push(TrafficFlow {
                speed: None,
                free_flow_speed: None,
                jam_factor: None,
                confidence: None,
                road_name: None,
                geometry,
                congestion_level: Some(congestion_level),
            });
        }
    }
    Ok(flows)
}

/// Maps the documented overlay buckets onto the core congestion levels.
///
/// # Parameters
/// - `traffic_bucket_text` — the feature's `properties.traffic` value.
///
/// # Returns
/// `green` maps to [`CongestionLevel::FreeFlow`], `yellow` to
/// [`CongestionLevel::Slow`], `red` to [`CongestionLevel::Congested`] and
/// `black` to [`CongestionLevel::Blocked`].
///
/// # Errors
/// Returns [`EveryMapError::ProviderError`] for any other bucket.
pub fn map_traffic_bucket_to_congestion_level(
    traffic_bucket_text: &str,
) -> EveryMapResult<CongestionLevel> {
    match traffic_bucket_text {
        "green" => Ok(CongestionLevel::FreeFlow),
        "yellow" => Ok(CongestionLevel::Slow),
        "red" => Ok(CongestionLevel::Congested),
        "black" => Ok(CongestionLevel::Blocked),
        other => Err(invalid_response(format!(
            "unknown flow traffic bucket {other:?}"
        ))),
    }
}

/// Projects an incident payload into vendor-neutral incidents.
///
/// # Parameters
/// - `payload` — a decoded incident response body carrying an `incidents`
///   array.
///
/// # Returns
/// One [`TrafficIncident`] per disruption. Congestion entries (`type == 3`)
/// are skipped because they describe flow rather than a disruption.
///
/// # Errors
/// Returns [`EveryMapError::ProviderError`] when `incidents` is missing, an
/// incident lacks its type or position, or its type is not documented.
pub fn parse_incidents_payload(payload: &Value) -> EveryMapResult<Vec<TrafficIncident>> {
    let incidents = payload
        .get("incidents")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("incidents payload lacks incidents"))?;
    let mut parsed_incidents = Vec::new();
    for incident in incidents {
        if let Some(parsed_incident) = project_incident(incident)? {
            parsed_incidents.push(parsed_incident);
        }
    }
    Ok(parsed_incidents)
}

/// Projects one `incidents[]` entry; `None` for a congestion entry.
fn project_incident(incident: &Value) -> EveryMapResult<Option<TrafficIncident>> {
    let incident_type = match incident.get("type").and_then(Value::as_i64) {
        Some(1) => "construction",
        Some(2) => "event",
        Some(3) => return Ok(None),
        Some(4) => "accident",
        Some(other) => {
            return Err(invalid_response(format!("unknown incident type {other}")));
        }
        None => {
            return Err(invalid_response("incident lacks the type field"));
        }
    };
    let latitude = incident
        .get("lat")
        .and_then(Value::as_f64)
        .ok_or_else(|| invalid_response("incident lacks lat"))?;
    let longitude = incident
        .get("lng")
        .and_then(Value::as_f64)
        .ok_or_else(|| invalid_response("incident lacks lng"))?;
    // The core incident model carries no coordinate; validate the position so
    // a malformed location is not silently accepted.
    Coordinate::new(latitude, longitude).map_err(|error| invalid_response(error.to_string()))?;
    let severity = map_incident_severity(
        incident
            .get("severity")
            .and_then(Value::as_i64)
            .filter(|severity| (0..=4).contains(severity)),
    );
    Ok(Some(TrafficIncident {
        id: incident.get("id").and_then(value_to_text),
        incident_type: Some(incident_type.to_string()),
        severity,
        description: incident
            .get("shortDesc")
            .and_then(Value::as_str)
            .or_else(|| incident.get("fullDesc").and_then(Value::as_str))
            .map(str::to_string),
        road_name: None,
        start_time: incident
            .get("startTime")
            .and_then(Value::as_str)
            .map(str::to_string),
        end_time: incident
            .get("endTime")
            .and_then(Value::as_str)
            .map(str::to_string),
    }))
}

/// Maps the service's 0-through-4 severity scale onto the core levels.
///
/// # Parameters
/// - `severity_value` — the incident's numeric severity, or `None` when the
///   service reports none.
///
/// # Returns
/// `0` and `1` map to [`IncidentSeverity::Low`], `2` to
/// [`IncidentSeverity::Minor`], `3` to [`IncidentSeverity::Moderate`] and `4`
/// to [`IncidentSeverity::Major`]. An absent or out-of-range value maps to
/// `None`.
pub fn map_incident_severity(severity_value: Option<i64>) -> Option<IncidentSeverity> {
    match severity_value? {
        0 | 1 => Some(IncidentSeverity::Low),
        2 => Some(IncidentSeverity::Minor),
        3 => Some(IncidentSeverity::Moderate),
        4 => Some(IncidentSeverity::Major),
        _ => None,
    }
}

/// Reads a JSON scalar as text.
fn value_to_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        map_incident_severity, map_traffic_bucket_to_congestion_level, parse_flow_payload,
        parse_incidents_payload, select_flow_view,
    };
    use everymap_core::domains::traffic::{CongestionLevel, IncidentSeverity};
    use everymap_core::error::EveryMapError;
    use everymap_core::types::{BoundingBox, Coordinate};
    use serde_json::{json, Value};

    fn fixture(name: &str) -> Value {
        let fixture_path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(
            &std::fs::read_to_string(fixture_path).expect("the fixture must be readable"),
        )
        .expect("the fixture must be valid JSON")
    }

    fn denver_bounding_box() -> BoundingBox {
        BoundingBox::new(
            Coordinate::new(39.95, -104.71).unwrap(),
            Coordinate::new(39.52, -105.25).unwrap(),
        )
    }

    #[test]
    fn a_recorded_flow_projection_keeps_geometry_and_congestion() {
        let flows = parse_flow_payload(&fixture("traffic_flow_new_york.json"))
            .expect("the recorded flow must project");
        assert_eq!(flows.len(), 4);
        let levels = flows
            .iter()
            .map(|flow| flow.congestion_level)
            .collect::<Vec<Option<CongestionLevel>>>();
        assert_eq!(
            levels,
            vec![
                Some(CongestionLevel::FreeFlow),
                Some(CongestionLevel::Slow),
                Some(CongestionLevel::Congested),
                Some(CongestionLevel::Blocked),
            ]
        );
        let first = &flows[0];
        assert_eq!(first.geometry.len(), 23);
        assert!((first.geometry[0].lat - 40.63701).abs() < 1e-9);
        assert!((first.geometry[0].lng - (-74.01926)).abs() < 1e-9);
        assert!(first.speed.is_none());
        assert!(first.free_flow_speed.is_none());
    }

    #[test]
    fn an_unknown_bucket_rejects_the_payload() {
        assert!(map_traffic_bucket_to_congestion_level("purple").is_err());
        let payload = json!({
            "features": [{
                "properties": {"traffic": "purple"},
                "geometry": {"coordinates": [[[-73.99, 40.73]]]}
            }]
        });
        assert!(parse_flow_payload(&payload).is_err());
    }

    #[test]
    fn a_recorded_incident_projection_keeps_type_severity_and_times() {
        let incidents = parse_incidents_payload(&fixture("traffic_incidents_denver.json"))
            .expect("the recorded incidents must project");
        assert_eq!(incidents.len(), 5);
        let first = &incidents[0];
        assert_eq!(first.incident_type.as_deref(), Some("accident"));
        assert_eq!(first.severity, Some(IncidentSeverity::Moderate));
        assert_eq!(first.description.as_deref(), Some("Closed"));
        assert!(first
            .id
            .as_deref()
            .is_some_and(|id| id.contains("1396961518")));
        assert_eq!(first.start_time.as_deref(), Some("2026-10-26T12:03:28"));
        assert_eq!(first.end_time.as_deref(), Some("2026-10-28T18:03:28"));
    }

    #[test]
    fn congestion_entries_are_skipped() {
        let payload = json!({
            "incidents": [
                {"type": 3, "lat": 40.0, "lng": -73.0, "severity": 1},
                {"type": 1, "lat": 40.0, "lng": -73.0, "severity": 0}
            ]
        });
        let incidents = parse_incidents_payload(&payload).unwrap();
        assert_eq!(incidents.len(), 1);
        assert_eq!(incidents[0].incident_type.as_deref(), Some("construction"));
    }

    #[test]
    fn the_severity_scale_maps_onto_the_core_levels() {
        assert_eq!(map_incident_severity(Some(0)), Some(IncidentSeverity::Low));
        assert_eq!(map_incident_severity(Some(1)), Some(IncidentSeverity::Low));
        assert_eq!(
            map_incident_severity(Some(2)),
            Some(IncidentSeverity::Minor)
        );
        assert_eq!(
            map_incident_severity(Some(3)),
            Some(IncidentSeverity::Moderate)
        );
        assert_eq!(
            map_incident_severity(Some(4)),
            Some(IncidentSeverity::Major)
        );
        assert_eq!(map_incident_severity(None), None);
    }

    #[test]
    fn the_view_selection_covers_the_box_without_exceeding_the_area_limit() {
        let denver_view =
            select_flow_view(&denver_bounding_box()).expect("Denver fits the documented limit");
        assert_eq!(denver_view.side_pixels, 815);
        assert!((denver_view.map_scale - 216_671.0).abs() < 1.0);

        let small_box = BoundingBox::new(
            Coordinate::new(40.735, -73.985).unwrap(),
            Coordinate::new(40.73, -73.99).unwrap(),
        );
        assert!(select_flow_view(&small_box).is_ok());

        let oversized_box = BoundingBox::new(
            Coordinate::new(39.97, -104.0).unwrap(),
            Coordinate::new(39.0, -105.0).unwrap(),
        );
        assert!(matches!(
            select_flow_view(&oversized_box),
            Err(EveryMapError::ValidationError(_))
        ));
    }
}
