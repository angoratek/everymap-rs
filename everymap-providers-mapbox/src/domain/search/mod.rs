pub mod types;

use async_trait::async_trait;
use everymap_core::domains::search::{Geocoder, GeocodeOptions, ReverseGeocodeOptions, SearchResponse, SearchResult, SearchResultType};
use everymap_core::error::EveryMapResult;
use everymap_core::types::{Coordinate, Address, BoundingBox};
use crate::client::MapBoxClient;
use std::sync::Arc;

pub use types::*;

const SEARCH_BASE_URL: &str = "https://api.mapbox.com";

/// Implementation of Geocoder for MapBox Geocoding API v6.
pub struct MapBoxGeocoder {
    pub(crate) client: Arc<MapBoxClient>,
    pub(crate) base_url: String,
}

impl MapBoxGeocoder {
    pub fn new(client: Arc<MapBoxClient>) -> Self {
        Self {
            client,
            base_url: SEARCH_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<MapBoxClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

/// Convert MapBox place_type list to core SearchResultType.
fn classify_place_type(place_types: &[String]) -> SearchResultType {
    if place_types.iter().any(|t| t == "address" || t == "poi") {
        SearchResultType::ExactMatch
    } else if place_types.iter().any(|t| t == "place" || t == "locality" || t == "neighborhood") {
        SearchResultType::Approximate
    } else {
        SearchResultType::Unknown
    }
}

impl From<MapBoxFeature> for SearchResult {
    fn from(f: MapBoxFeature) -> Self {
        let coordinate = f.center
            .as_ref()
            .and_then(|c| {
                if c.len() >= 2 {
                    Some(Coordinate::new(c[1], c[0]).unwrap_or(Coordinate::ORIGIN))
                } else {
                    None
                }
            })
            .or_else(|| {
                // Fallback: extract from geometry coordinates
                f.geometry.as_ref().and_then(|g| {
                    g.coordinates.as_ref().and_then(|coords| {
                        coords.as_array().and_then(|arr| {
                            if arr.len() >= 2 {
                                Some(Coordinate::new(
                                    arr[1].as_f64().unwrap_or(0.0),
                                    arr[0].as_f64().unwrap_or(0.0),
                                ).unwrap_or(Coordinate::ORIGIN))
                            } else {
                                None
                            }
                        })
                    })
                })
            })
            .unwrap_or(Coordinate::ORIGIN);

        let title = f.place_name.clone().or(f.text.clone());

        let address = if let Some(addr) = f.properties.as_ref().and_then(|p| p.address.clone()) {
            let mut a = Address::empty();
            a.label = f.place_name.clone();
            a.street = f.text.clone();
            a.house_number = Some(addr);
            a
        } else {
            let mut a = Address::empty();
            a.label = f.place_name.clone();
            a.street = f.text.clone();
            a
        };

        let bounding_box = f.bbox.and_then(|b| {
            if b.len() >= 4 {
                Some(BoundingBox::new(
                    Coordinate::new(b[3], b[2]).ok()?,  // north_east
                    Coordinate::new(b[1], b[0]).ok()?,  // south_west
                ))
            } else {
                None
            }
        });

        let result_type = classify_place_type(&f.place_type);

        SearchResult {
            id: f.id,
            coordinate,
            address,
            title,
            result_type,
            distance: None,
            confidence: f.relevance,
            categories: f.place_type,
            bounding_box,
            raw: None,
        }
    }
}

impl From<MapBoxSearchResponse> for SearchResponse {
    fn from(res: MapBoxSearchResponse) -> Self {
        SearchResponse {
            items: res.features.into_iter().map(|f| f.into()).collect(),
        }
    }
}

#[async_trait]
impl Geocoder for MapBoxGeocoder {
    async fn geocode(&self, query: &str, options: &GeocodeOptions) -> EveryMapResult<SearchResponse> {
        let url = format!("{}/search/geocode/v6/forward", self.base_url);
        let mut params: Vec<(&str, String)> = vec![
            ("q", query.to_string()),
        ];

        if let Some(limit) = options.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(lang) = &options.language {
            params.push(("language", lang.clone()));
        }
        if !options.country_codes.is_empty() {
            let codes: String = options.country_codes.join(",");
            params.push(("country", codes));
        }
        if let Some(bbox) = &options.bounding_box {
            params.push(("bbox", format!("{},{},{},{}",
                bbox.south_west.lng, bbox.south_west.lat,
                bbox.north_east.lng, bbox.north_east.lat)));
        }
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("proximity").and_then(|v| v.as_str()) {
                    params.push(("proximity", v.to_string()));
                }
                if let Some(v) = obj.get("types").and_then(|v| v.as_str()) {
                    params.push(("types", v.to_string()));
                }
                if let Some(v) = obj.get("worldview").and_then(|v| v.as_str()) {
                    params.push(("worldview", v.to_string()));
                }
            }
        }

        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: MapBoxSearchResponse = self.client.request_json(builder).await?;
        Ok(result.into())
    }

    async fn reverse_geocode(&self, coordinate: &Coordinate, options: &ReverseGeocodeOptions) -> EveryMapResult<SearchResponse> {
        let url = format!("{}/search/geocode/v6/reverse", self.base_url);
        let mut params: Vec<(&str, String)> = vec![
            ("longitude", coordinate.lng.to_string()),
            ("latitude", coordinate.lat.to_string()),
        ];

        if let Some(limit) = options.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(lang) = &options.language {
            params.push(("language", lang.clone()));
        }
        if let Some(radius) = options.radius {
            // MapBox doesn't have a radius param for reverse geocode, skip
            let _ = radius;
        }
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("types").and_then(|v| v.as_str()) {
                    params.push(("types", v.to_string()));
                }
                if let Some(v) = obj.get("worldview").and_then(|v| v.as_str()) {
                    params.push(("worldview", v.to_string()));
                }
            }
        }

        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: MapBoxSearchResponse = self.client.request_json(builder).await?;
        Ok(result.into())
    }
}