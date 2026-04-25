use serde::{Deserialize, Serialize};

/// Available attribute layers in the HERE Map Attributes API v8.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum AttributeLayer {
    #[default]
    #[serde(rename = "roads")]
    Roads,
    #[serde(rename = "adminAreas")]
    AdminAreas,
    #[serde(rename = "buildings")]
    Buildings,
    #[serde(rename = "landmarks")]
    Landmarks,
    #[serde(rename = "segments")]
    Segments,
}

/// Response format for attribute queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum AttributeFormat {
    #[default]
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "geojson")]
    GeoJson,
    #[serde(rename = "protobuf")]
    Protobuf,
}

// --- Structured response types per layer ---
// These types handle the HERE Map Attributes API v8 response format,
// which follows GeoJSON FeatureCollection structure with properties
// nested inside each Feature object.

/// A feature from the roads attribute layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereRoadFeature {
    /// Unique link identifier.
    #[serde(default, rename = "LINK_ID")]
    pub link_id: Option<String>,
    /// Functional road class (1=most important, 5=least).
    #[serde(default, rename = "FUNCTIONAL_CLASS")]
    pub functional_class: Option<u32>,
    /// Speed limit in km/h or mph depending on region.
    #[serde(default, rename = "SPEED_LIMIT")]
    pub speed_limit: Option<f64>,
    /// Speed category (e.g., "1", "2", ... "8").
    #[serde(default, rename = "SPEED_CATEGORY")]
    pub speed_category: Option<String>,
    /// Road class (e.g., "A", "B", "C").
    #[serde(default, rename = "ROAD_CLASS")]
    pub road_class: Option<String>,
    /// Travel direction: "BOTH", "FORWARD", "BACKWARD".
    #[serde(default, rename = "TRAVEL_DIRECTION")]
    pub travel_direction: Option<String>,
    /// Number of lanes.
    #[serde(default, rename = "LANES")]
    pub lanes: Option<u32>,
    /// Whether the road has a physical divider.
    #[serde(default, rename = "DIVIDER")]
    pub divider: Option<String>,
    /// Whether the road is in an urban area.
    #[serde(default, rename = "URBAN")]
    pub urban: Option<String>,
    /// Whether the road segment is a tunnel.
    #[serde(default, rename = "TUNNEL")]
    pub tunnel: Option<String>,
    /// Whether the road segment is a bridge.
    #[serde(default, rename = "BRIDGE")]
    pub bridge: Option<String>,
    /// Whether the road segment is a ramp.
    #[serde(default, rename = "RAMP")]
    pub ramp: Option<String>,
    /// Whether the road segment is a roundabout.
    #[serde(default, rename = "ROUNDABOUT")]
    pub roundabout: Option<String>,
    /// Speed limits by direction of travel.
    #[serde(default, rename = "SPEED_LIMITS_BY_DIRECTION")]
    pub speed_limits_by_direction: Option<Vec<DirectionalSpeedLimit>>,
    /// Road name(s).
    #[serde(default, rename = "NAME")]
    pub name: Option<Vec<LocalizedText>>,
    /// Road shield info.
    #[serde(default, rename = "ROAD_SHIELD")]
    pub road_shield: Option<String>,
    /// Route designation (e.g., "INTERSTATE", "US_HIGHWAY").
    #[serde(default, rename = "ROUTE_DESIGNATION")]
    pub route_designation: Option<String>,
    /// Pavement type.
    #[serde(default, rename = "PAVEMENT_TYPE")]
    pub pavement_type: Option<String>,
    /// Number of permanent lanes.
    #[serde(default, rename = "PERMANENT_LANE_COUNT")]
    pub permanent_lane_count: Option<u32>,
}

/// Speed limit for a specific direction of travel.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DirectionalSpeedLimit {
    /// Speed limit value.
    #[serde(default, rename = "SPEED_LIMIT")]
    pub speed_limit: Option<f64>,
    /// Direction: "FORWARD" or "BACKWARD".
    #[serde(default, rename = "DIRECTION")]
    pub direction: Option<String>,
}

/// Localized text value.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalizedText {
    /// The text value.
    #[serde(default, rename = "VALUE")]
    pub value: Option<String>,
    /// Language code.
    #[serde(default, rename = "LANGUAGE")]
    pub language: Option<String>,
}

/// A feature from the segments (topology) attribute layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereSegmentFeature {
    /// Unique link identifier.
    #[serde(default, rename = "LINK_ID")]
    pub link_id: Option<String>,
    /// Reference node ID.
    #[serde(default, rename = "REF_NODE")]
    pub ref_node: Option<String>,
    /// Non-reference node ID.
    #[serde(default, rename = "NON_REF_NODE")]
    pub non_ref_node: Option<String>,
    /// Functional road class.
    #[serde(default, rename = "FUNCTIONAL_CLASS")]
    pub functional_class: Option<u32>,
    /// Speed limit in km/h.
    #[serde(default, rename = "SPEED_LIMIT")]
    pub speed_limit: Option<f64>,
    /// Speed category.
    #[serde(default, rename = "SPEED_CATEGORY")]
    pub speed_category: Option<String>,
    /// Number of lanes.
    #[serde(default, rename = "LANE_COUNT")]
    pub lane_count: Option<u32>,
    /// Road class.
    #[serde(default, rename = "ROAD_CLASS")]
    pub road_class: Option<String>,
}

/// A feature from the adminAreas attribute layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereAdminAreaFeature {
    /// Admin place identifier.
    #[serde(default, rename = "ADMIN_PLACE_ID")]
    pub admin_place_id: Option<String>,
    /// Admin level (0=country, 1=state, 2=county, etc.).
    #[serde(default, rename = "ADMIN_LEVEL")]
    pub admin_level: Option<u32>,
    /// Admin name.
    #[serde(default, rename = "ADMIN_NAME")]
    pub admin_name: Option<Vec<LocalizedText>>,
    /// Country ID.
    #[serde(default, rename = "COUNTRY_ID")]
    pub country_id: Option<String>,
    /// State ID.
    #[serde(default, rename = "STATE_ID")]
    pub state_id: Option<String>,
    /// City ID.
    #[serde(default, rename = "CITY_ID")]
    pub city_id: Option<String>,
}

/// A feature from the buildings attribute layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereBuildingFeature {
    /// Building identifier.
    #[serde(default, rename = "BUILDING_ID")]
    pub building_id: Option<String>,
    /// Building height in meters.
    #[serde(default, rename = "BUILDING_HEIGHT")]
    pub building_height: Option<f64>,
    /// Number of building levels.
    #[serde(default, rename = "BUILDING_LEVELS")]
    pub building_levels: Option<u32>,
    /// Roof color.
    #[serde(default, rename = "ROOF_COLOR")]
    pub roof_color: Option<String>,
    /// Roof shape.
    #[serde(default, rename = "ROOF_SHAPE")]
    pub roof_shape: Option<String>,
}

/// A feature from the landmarks attribute layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereLandmarkFeature {
    /// Landmark identifier.
    #[serde(default, rename = "LANDMARK_ID")]
    pub landmark_id: Option<String>,
    /// Landmark name.
    #[serde(default, rename = "LANDMARK_NAME")]
    pub landmark_name: Option<Vec<LocalizedText>>,
    /// Landmark type.
    #[serde(default, rename = "LANDMARK_TYPE")]
    pub landmark_type: Option<String>,
    /// Navigation type.
    #[serde(default, rename = "NAVI_TYPE")]
    pub navi_type: Option<String>,
}

// --- GeoJSON Feature wrappers ---
// The HERE API returns GeoJSON FeatureCollection with properties
// nested inside each Feature's "properties" object.

/// A GeoJSON Feature with typed road properties.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereRoadAttributeFeature {
    #[serde(default, rename = "type")]
    pub feature_type: Option<String>,
    #[serde(default)]
    pub geometry: Option<serde_json::Value>,
    #[serde(default)]
    pub properties: HereRoadFeature,
}

/// A GeoJSON Feature with typed segment properties.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereSegmentAttributeFeature {
    #[serde(default, rename = "type")]
    pub feature_type: Option<String>,
    #[serde(default)]
    pub geometry: Option<serde_json::Value>,
    #[serde(default)]
    pub properties: HereSegmentFeature,
}

/// A GeoJSON Feature with typed admin area properties.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereAdminAreaAttributeFeature {
    #[serde(default, rename = "type")]
    pub feature_type: Option<String>,
    #[serde(default)]
    pub geometry: Option<serde_json::Value>,
    #[serde(default)]
    pub properties: HereAdminAreaFeature,
}

/// A GeoJSON Feature with typed building properties.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereBuildingAttributeFeature {
    #[serde(default, rename = "type")]
    pub feature_type: Option<String>,
    #[serde(default)]
    pub geometry: Option<serde_json::Value>,
    #[serde(default)]
    pub properties: HereBuildingFeature,
}

/// A GeoJSON Feature with typed landmark properties.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereLandmarkAttributeFeature {
    #[serde(default, rename = "type")]
    pub feature_type: Option<String>,
    #[serde(default)]
    pub geometry: Option<serde_json::Value>,
    #[serde(default)]
    pub properties: HereLandmarkFeature,
}

// --- Typed response types ---

/// Typed response from the roads attribute layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereRoadAttributesResponse {
    #[serde(default, rename = "type")]
    pub response_type: Option<String>,
    #[serde(default)]
    pub features: Vec<HereRoadAttributeFeature>,
}

/// Typed response from the segments attribute layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereSegmentAttributesResponse {
    #[serde(default, rename = "type")]
    pub response_type: Option<String>,
    #[serde(default)]
    pub features: Vec<HereSegmentAttributeFeature>,
}

/// Typed response from the adminAreas attribute layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereAdminAreasResponse {
    #[serde(default, rename = "type")]
    pub response_type: Option<String>,
    #[serde(default)]
    pub features: Vec<HereAdminAreaAttributeFeature>,
}

/// Typed response from the buildings attribute layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereBuildingsResponse {
    #[serde(default, rename = "type")]
    pub response_type: Option<String>,
    #[serde(default)]
    pub features: Vec<HereBuildingAttributeFeature>,
}

/// Typed response from the landmarks attribute layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereLandmarksResponse {
    #[serde(default, rename = "type")]
    pub response_type: Option<String>,
    #[serde(default)]
    pub features: Vec<HereLandmarkAttributeFeature>,
}
