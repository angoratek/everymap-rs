pub mod types;

use crate::client::MapBoxClient;
use async_trait::async_trait;
use everymap_core::domains::isoline::{
    IsolineOptions, IsolineProvider, IsolineResponse, IsolineResult, RangeType,
};
use everymap_core::domains::routing::TransportMode;
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use std::sync::Arc;

pub use types::*;

const ISOCHRONE_BASE_URL: &str = "https://api.mapbox.com";

/// Implementation of IsolineProvider for MapBox Isochrone API.
pub struct MapBoxIsoline {
    pub(crate) client: Arc<MapBoxClient>,
    pub(crate) base_url: String,
}

impl MapBoxIsoline {
    pub fn new(client: Arc<MapBoxClient>) -> Self {
        Self {
            client,
            base_url: ISOCHRONE_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<MapBoxClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

/// Convert core TransportMode to MapBox profile.
fn transport_mode_to_profile(mode: &TransportMode) -> &'static str {
    match mode {
        TransportMode::Car => "driving",
        TransportMode::Truck => {
            log::warn!("MapBox Isochrone API does not support truck profile, falling back to driving");
            "driving"
        }
        TransportMode::Pedestrian => "walking",
        TransportMode::Bicycle => "cycling",
        TransportMode::Bus => {
            log::warn!("MapBox Isochrone API does not support bus profile, falling back to driving");
            "driving"
        }
        TransportMode::Taxi => {
            log::warn!("MapBox Isochrone API does not support taxi profile, falling back to driving");
            "driving"
        }
        TransportMode::Scooter => {
            log::warn!("MapBox Isochrone API does not support scooter profile, falling back to driving");
            "driving"
        }
        TransportMode::Unknown => "driving",
    }
}

fn extract_polygon_coordinates(geom: &serde_json::Value) -> Vec<Coordinate> {
    // MapBox isochrone geometry is a Polygon: [[[lng, lat], [lng, lat], ...]]
    geom.as_array()
        .and_then(|rings| rings.first())
        .and_then(|ring| ring.as_array())
        .map(|ring| {
            ring.iter()
                .filter_map(|coordinate| {
                    let arr = coordinate.as_array()?;
                    if arr.len() >= 2 {
                        Some(
                            Coordinate::new(arr[1].as_f64()?, arr[0].as_f64()?)
                                .unwrap_or(Coordinate::ORIGIN),
                        )
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

#[async_trait]
impl IsolineProvider for MapBoxIsoline {
    async fn get_isoline(
        &self,
        center: &Coordinate,
        range: f64,
        options: &IsolineOptions,
    ) -> EveryMapResult<IsolineResponse> {
        let profile = options
            .transport_mode
            .as_ref()
            .map(|m| transport_mode_to_profile(m))
            .unwrap_or("driving");
        let coords = format!("{},{}", center.lng, center.lat);
        let url = format!(
            "{}/isochrone/v1/mapbox/{}/{}",
            self.base_url, profile, coords
        );

        let mut params: Vec<(&str, String)> = Vec::new();

        match options.range_type.as_ref().unwrap_or(&RangeType::Time) {
            RangeType::Time => {
                // MapBox uses minutes for contours_minutes
                let minutes = (range / 60.0).round() as u64;
                params.push(("contours_minutes", minutes.to_string()));
            }
            RangeType::Distance => {
                // MapBox uses meters for contours_meters
                let meters = range.round() as u64;
                params.push(("contours_meters", meters.to_string()));
            }
            RangeType::Consumption => {
                return Err(everymap_core::error::EveryMapError::provider(
                    "mapbox",
                    "UNSUPPORTED_RANGE_TYPE",
                    "MapBox Isochrone API does not support consumption-based ranges",
                ));
            }
        }

        if let Some(departure) = &options.departure_time {
            log::warn!(
                "MapBox Isochrone API does not support departure_time; \
                 departure_time ({}) will be ignored",
                departure
            );
        }
        if !options.avoid.is_empty() {
            log::warn!(
                "MapBox Isochrone API does not support avoid restrictions; \
                 avoid will be ignored"
            );
        }

        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("denoise").and_then(|v| v.as_f64()) {
                    params.push(("denoise", v.to_string()));
                }
                if let Some(v) = obj.get("generalize").and_then(|v| v.as_f64()) {
                    params.push(("generalize", v.to_string()));
                }
            }
        }

        params.push(("polygons", "true".to_string()));

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: MapBoxIsochroneResponse = self.client.request_json(builder).await?;

        let isolines: Vec<IsolineResult> = result
            .features
            .into_iter()
            .map(|f| {
                let polygon = f
                    .geometry
                    .and_then(|g| g.coordinates)
                    .map(|coords| extract_polygon_coordinates(&coords))
                    .unwrap_or_default();

                let range_val = f.properties.and_then(|p| p.contour).map(|c| {
                    match options.range_type.as_ref().unwrap_or(&RangeType::Time) {
                        RangeType::Time => (c as f64) * 60.0, // minutes to seconds
                        RangeType::Distance => c as f64,      // already meters
                        RangeType::Consumption => c as f64,
                    }
                });

                IsolineResult {
                    polygon,
                    range: range_val,
                }
            })
            .collect();

        Ok(IsolineResponse {
            isolines,
            raw: None,
        })
    }
}
