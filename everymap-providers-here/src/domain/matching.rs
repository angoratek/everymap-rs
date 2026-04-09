pub mod types;

use async_trait::async_trait;
use everymap_core::domains::matching::{RouteMatcher, TraceRequest, TraceResponse, MatchedPoint};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use crate::client::HereClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use types::*;

const MATCHING_BASE_URL: &str = "https://routematching.hereapi.com/v8";

/// Exhaustive options for HERE Route Matching API v8.
/// Organized into groups matching the API parameter structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereMatchingOptions {
    // --- Match mode ---
    pub route_match: Option<u8>,
    pub mode: Option<MatchMode>,
    pub legal: Option<LegalConstraint>,
    pub traverse_gates: Option<bool>,
    pub oneway: Option<bool>,

    // --- Matching parameters ---
    pub map_match_radius: Option<u32>,
    pub align_to_gps_time: Option<bool>,
    pub ignore_zero_speed_points: Option<bool>,
    pub wp_dist: Option<u32>,
    pub speed_fc_cat: Option<String>,
    pub map_match_tolerance: Option<u32>,
    pub heading: Option<f64>,

    // --- Vehicle dimensions & weight ---
    pub limited_weight: Option<u32>,
    pub height: Option<u32>,
    pub length: Option<u32>,
    pub width: Option<u32>,
    pub vehicle_number_axles: Option<u32>,
    pub trailer_number_axles: Option<u32>,
    pub trailer_type: Option<TrailerType>,
    pub vehicle_weight: Option<u32>,
    pub trailer_weight: Option<u32>,
    pub weight_per_axle: Option<u32>,
    pub height_above_1st_axle: Option<u32>,
    pub trailers_count: Option<u32>,

    // --- Emission & fuel ---
    pub emission_type: Option<EmissionType>,
    pub co2_emission_class: Option<u32>,
    pub fuel_type: Option<MatchFuelType>,
    pub hybrid: Option<bool>,

    // --- Restrictions ---
    pub avoid_links: Option<Vec<String>>,
    pub avoid_areas: Option<Vec<String>>,
    pub avoid_turns: Option<Vec<String>>,
    pub avoid_features: Option<Vec<AvoidFeature>>,
    pub avoid_private: Option<bool>,
    pub avoid_country_change: Option<bool>,
    pub shipped_hazardous_goods: Option<Vec<HazardousGoodsType>>,
    pub tunnel_category: Option<TunnelCategory>,

    // --- Commercial ---
    pub commercial: Option<bool>,
    pub passengers_count: Option<u32>,
    pub tires_count: Option<u32>,
    pub disabled_equipped: Option<bool>,
    pub license_plate: Option<String>,

    // --- Time ---
    pub departure: Option<String>,
    pub arrival: Option<String>,

    // --- Response attributes ---
    pub leg_attributes: Option<String>,
    pub link_attributes: Option<String>,
    pub response_attributes: Option<String>,
    pub route_attributes: Option<String>,
    pub meta_attributes: Option<String>,
    pub maneuver_attributes: Option<String>,
    pub instruction_format: Option<InstructionFormat>,
    pub language: Option<String>,

    // --- Toll ---
    pub toll_vehicle_type: Option<u32>,
    pub toll_pass: Option<String>,
    pub currency: Option<String>,
    pub driver_cost: Option<String>,
    pub vehicle_cost: Option<String>,
    pub vehicle_cost_on_ferry: Option<String>,
    pub cost_per_consumption_unit: Option<String>,

    // --- Advanced ---
    pub max_speed: Option<u32>,
    pub alternatives: Option<u32>,
    pub truck_verified: Option<bool>,
    pub ignore_waypoint_vehicle_restriction: Option<bool>,
    pub admin_truck_restrictions: Option<bool>,
    pub ignore_preferred_routes: Option<bool>,
    pub exclude_zone_types: Option<String>,
    pub overlays: Option<String>,
    pub custom_restrict_limit: Option<String>,
    pub custom_attributes: Option<String>,
    pub custom_consumption_details: Option<String>,
    pub timeout: Option<u32>,
    pub driving_report: Option<bool>,
    pub ehorizon_limits: Option<String>,
}

/// Implementation of RouteMatcher for HERE Technologies.
pub struct HereRouteMatcher {
    client: Arc<HereClient>,
    base_url: String,
}

impl HereRouteMatcher {
    pub fn new(client: Arc<HereClient>) -> Self {
        Self {
            client,
            base_url: MATCHING_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<HereClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

// Internal deserialization for backward-compatible core trait
#[derive(Debug, Deserialize)]
struct HereLegacyMatchResponse {
    trace: Vec<HereLegacyPoint>,
    summary: HereLegacySummary,
}

#[derive(Debug, Deserialize)]
struct HereLegacyPoint {
    lat: f64,
    lng: f64,
}

#[derive(Debug, Deserialize)]
struct HereLegacySummary {
    length: f64,
}

fn add_option(params: &mut Vec<(String, String)>, key: &str, val: Option<impl std::fmt::Display>) {
    if let Some(v) = val {
        params.push((key.to_string(), v.to_string()));
    }
}

fn add_option_ref(params: &mut Vec<(String, String)>, key: &str, val: Option<&String>) {
    if let Some(v) = val {
        params.push((key.to_string(), v.clone()));
    }
}

#[async_trait]
impl RouteMatcher for HereRouteMatcher {
    type Options = HereMatchingOptions;
    type Response = TraceResponse;

    async fn match_route(&self, req: TraceRequest<Self::Options>) -> EveryMapResult<Self::Response> {
        let points_str = req.points.iter()
            .map(|p| format!("{},{}", p.lat, p.lng))
            .collect::<Vec<_>>()
            .join(";");

        let mut params: Vec<(String, String)> = vec![("trace".to_string(), points_str)];

        let opts = &req.options;

        // Match mode
        add_option(&mut params, "routeMatch", opts.route_match);
        if let Some(mode) = &opts.mode {
            params.push(("mode".to_string(), serde_json::to_value(mode).unwrap().as_str().unwrap().to_string()));
        }
        if let Some(legal) = &opts.legal {
            params.push(("legal".to_string(), serde_json::to_value(legal).unwrap().as_str().unwrap().to_string()));
        }
        add_option(&mut params, "traverseGates", opts.traverse_gates);
        add_option(&mut params, "oneway", opts.oneway);

        // Matching parameters
        add_option(&mut params, "mapMatchRadius", opts.map_match_radius);
        add_option(&mut params, "alignToGpsTime", opts.align_to_gps_time);
        add_option(&mut params, "ignoreZeroSpeedPoints", opts.ignore_zero_speed_points);
        add_option(&mut params, "wpDist", opts.wp_dist);
        add_option_ref(&mut params, "speedFcCat", opts.speed_fc_cat.as_ref());
        add_option(&mut params, "mapMatchTolerance", opts.map_match_tolerance);
        add_option(&mut params, "heading", opts.heading);

        // Vehicle dimensions
        add_option(&mut params, "limitedWeight", opts.limited_weight);
        add_option(&mut params, "height", opts.height);
        add_option(&mut params, "length", opts.length);
        add_option(&mut params, "width", opts.width);
        add_option(&mut params, "vehicleNumberAxles", opts.vehicle_number_axles);
        add_option(&mut params, "trailerNumberAxles", opts.trailer_number_axles);
        if let Some(tt) = &opts.trailer_type {
            params.push(("trailerType".to_string(), serde_json::to_value(tt).unwrap().as_str().unwrap().to_string()));
        }
        add_option(&mut params, "vehicleWeight", opts.vehicle_weight);
        add_option(&mut params, "trailerWeight", opts.trailer_weight);
        add_option(&mut params, "weightPerAxle", opts.weight_per_axle);
        add_option(&mut params, "heightAbove1stAxle", opts.height_above_1st_axle);
        add_option(&mut params, "trailersCount", opts.trailers_count);

        // Emission & fuel
        if let Some(et) = &opts.emission_type {
            params.push(("emissionType".to_string(), serde_json::to_value(et).unwrap().as_str().unwrap().to_string()));
        }
        add_option(&mut params, "co2EmissionClass", opts.co2_emission_class);
        if let Some(ft) = &opts.fuel_type {
            params.push(("fuelType".to_string(), serde_json::to_value(ft).unwrap().as_str().unwrap().to_string()));
        }
        add_option(&mut params, "hybrid", opts.hybrid);

        // Restrictions
        if let Some(links) = &opts.avoid_links { params.push(("avoidLinks".to_string(), links.join(","))); }
        if let Some(areas) = &opts.avoid_areas { params.push(("avoidAreas".to_string(), areas.join(","))); }
        if let Some(turns) = &opts.avoid_turns { params.push(("avoidTurns".to_string(), turns.join(","))); }
        if let Some(af) = &opts.avoid_features {
            params.push(("avoidFeatures".to_string(), af.iter().map(|f| serde_json::to_value(f).unwrap().as_str().unwrap().to_string()).collect::<Vec<_>>().join(",")));
        }
        add_option(&mut params, "avoidPrivate", opts.avoid_private);
        add_option(&mut params, "avoidCountryChange", opts.avoid_country_change);
        if let Some(hg) = &opts.shipped_hazardous_goods {
            params.push(("shippedHazardousGoods".to_string(), hg.iter().map(|g| serde_json::to_value(g).unwrap().as_str().unwrap().to_string()).collect::<Vec<_>>().join(",")));
        }
        if let Some(tc) = &opts.tunnel_category {
            params.push(("tunnelCategory".to_string(), serde_json::to_value(tc).unwrap().as_str().unwrap().to_string()));
        }

        // Commercial
        add_option(&mut params, "commercial", opts.commercial);
        add_option(&mut params, "passengersCount", opts.passengers_count);
        add_option(&mut params, "tiresCount", opts.tires_count);
        add_option(&mut params, "disabledEquipped", opts.disabled_equipped);
        add_option_ref(&mut params, "licensePlate", opts.license_plate.as_ref());

        // Time
        add_option_ref(&mut params, "departure", opts.departure.as_ref());
        add_option_ref(&mut params, "arrival", opts.arrival.as_ref());

        // Response attributes
        add_option_ref(&mut params, "legAttributes", opts.leg_attributes.as_ref());
        add_option_ref(&mut params, "linkAttributes", opts.link_attributes.as_ref());
        add_option_ref(&mut params, "responseAttributes", opts.response_attributes.as_ref());
        add_option_ref(&mut params, "routeAttributes", opts.route_attributes.as_ref());
        add_option_ref(&mut params, "metaAttributes", opts.meta_attributes.as_ref());
        add_option_ref(&mut params, "maneuverAttributes", opts.maneuver_attributes.as_ref());
        if let Some(fmt) = &opts.instruction_format {
            params.push(("instructionFormat".to_string(), serde_json::to_value(fmt).unwrap().as_str().unwrap().to_string()));
        }
        add_option_ref(&mut params, "language", opts.language.as_ref());

        // Toll
        add_option(&mut params, "tollVehicleType", opts.toll_vehicle_type);
        add_option_ref(&mut params, "tollPass", opts.toll_pass.as_ref());
        add_option_ref(&mut params, "currency", opts.currency.as_ref());
        add_option_ref(&mut params, "driverCost", opts.driver_cost.as_ref());
        add_option_ref(&mut params, "vehicleCost", opts.vehicle_cost.as_ref());
        add_option_ref(&mut params, "vehicleCostOnFerry", opts.vehicle_cost_on_ferry.as_ref());
        add_option_ref(&mut params, "costPerConsumptionUnit", opts.cost_per_consumption_unit.as_ref());

        // Advanced
        add_option(&mut params, "maxSpeed", opts.max_speed);
        add_option(&mut params, "alternatives", opts.alternatives);
        add_option(&mut params, "truckVerified", opts.truck_verified);
        add_option(&mut params, "ignoreWaypointVehicleRestriction", opts.ignore_waypoint_vehicle_restriction);
        add_option(&mut params, "adminTruckRestrictions", opts.admin_truck_restrictions);
        add_option(&mut params, "ignorePreferredRoutes", opts.ignore_preferred_routes);
        add_option_ref(&mut params, "excludeZoneTypes", opts.exclude_zone_types.as_ref());
        add_option_ref(&mut params, "overlays", opts.overlays.as_ref());
        add_option_ref(&mut params, "customRestrLimit", opts.custom_restrict_limit.as_ref());
        add_option_ref(&mut params, "customAttributes", opts.custom_attributes.as_ref());
        add_option_ref(&mut params, "customConsumptionDetails", opts.custom_consumption_details.as_ref());
        add_option(&mut params, "timeout", opts.timeout);
        add_option(&mut params, "drivingReport", opts.driving_report);
        add_option_ref(&mut params, "ehorizonLimits", opts.ehorizon_limits.as_ref());

        let url = format!("{}/match/routelinks", self.base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let response = self.client.request(builder).await?;
        let here_res: HereLegacyMatchResponse = response.json().await?;

        let matched_points = here_res.trace.into_iter()
            .map(|p| MatchedPoint {
                coordinate: Coordinate::new(p.lat, p.lng).unwrap(),
                confidence: None,
                road_name: None,
            })
            .collect();

        Ok(TraceResponse {
            matched_points,
            distance: here_res.summary.length,
            duration: None,
            raw: None,
        })
    }
}