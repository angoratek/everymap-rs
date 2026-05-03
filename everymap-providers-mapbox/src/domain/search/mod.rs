pub mod types;

use crate::client::MapBoxClient;
use async_trait::async_trait;
use everymap_core::domains::search::{
    GeocodeOptions, Geocoder, ReverseGeocodeOptions, SearchResponse, SearchResult, SearchResultType,
};
use everymap_core::error::EveryMapResult;
use everymap_core::types::{Address, BoundingBox, Coordinate};
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

/// Convert MapBox v6 feature_type string to core SearchResultType.
fn classify_feature_type(feature_type: &str) -> SearchResultType {
    match feature_type {
        "address" | "poi" => SearchResultType::ExactMatch,
        "place" | "locality" | "neighborhood" | "region" | "district" | "country" => {
            SearchResultType::Approximate
        }
        _ => SearchResultType::Unknown,
    }
}

impl From<MapBoxFeature> for SearchResult {
    fn from(f: MapBoxFeature) -> Self {
        let props = f.properties.as_ref();

        // Use properties.coordinates (v6), fallback to geometry coordinates
        let coordinate = props
            .and_then(|p| p.coordinates.as_ref())
            .map(|c| Coordinate::new(c.latitude, c.longitude).unwrap_or(Coordinate::ORIGIN))
            .or_else(|| {
                // Fallback: extract from geometry coordinates
                f.geometry.as_ref().and_then(|g| {
                    g.coordinates.as_ref().and_then(|coords| {
                        coords.as_array().and_then(|arr| {
                            if arr.len() >= 2 {
                                Some(
                                    Coordinate::new(
                                        arr[1].as_f64().unwrap_or(0.0),
                                        arr[0].as_f64().unwrap_or(0.0),
                                    )
                                    .unwrap_or(Coordinate::ORIGIN),
                                )
                            } else {
                                None
                            }
                        })
                    })
                })
            })
            .unwrap_or(Coordinate::ORIGIN);

        // Use properties.full_address as title, fallback to name
        let title = props.and_then(|p| p.full_address.clone().or(p.name.clone()));

        // Build address from context
        let mut address = Address::empty();
        if let Some(p) = props {
            address.label = p.full_address.clone();
            address.street = p.name.clone();
            if let Some(ctx) = &p.context {
                if let Some(street) = &ctx.street {
                    address.street = Some(street.name.clone().unwrap_or_default());
                }
                if let Some(region) = &ctx.region {
                    address.state = region.name.clone();
                }
                if let Some(country) = &ctx.country {
                    address.country = country.name.clone();
                    address.country_code = country.country_code.clone();
                }
                if let Some(place) = &ctx.place {
                    address.city = Some(place.name.clone().unwrap_or_default());
                }
                if let Some(district) = &ctx.district {
                    address.district = district.name.clone();
                }
                if let Some(postcode) = &ctx.postcode {
                    address.postal_code = postcode.name.clone();
                }
            }
            if let Some(house) = &p.address {
                address.house_number = Some(house.clone());
            }
        }

        // Use properties.bbox (v6)
        let bounding_box = props.and_then(|p| p.bbox.as_ref()).and_then(|b| {
            if b.len() >= 4 {
                Some(BoundingBox::new(
                    Coordinate::new(b[3], b[2]).ok()?, // north_east
                    Coordinate::new(b[1], b[0]).ok()?, // south_west
                ))
            } else {
                None
            }
        });

        let feature_type_str = props.and_then(|p| p.feature_type.as_deref().map(String::from));
        let result_type = feature_type_str
            .as_deref()
            .map(classify_feature_type)
            .unwrap_or(SearchResultType::Unknown);

        let categories = props
            .and_then(|p| {
                let mut cats = Vec::new();
                if let Some(ft) = &p.feature_type {
                    cats.push(ft.clone());
                }
                if let Some(additional) = &p.additional_feature_types {
                    cats.extend(additional.iter().cloned());
                }
                if cats.is_empty() {
                    None
                } else {
                    Some(cats)
                }
            })
            .unwrap_or_default();

        SearchResult {
            id: f.id.or(props.and_then(|p| p.mapbox_id.clone())),
            coordinate,
            address,
            title,
            result_type,
            distance: None,
            confidence: props.and_then(|p| p.relevance),
            categories,
            bounding_box,
            raw: None,
        }
    }
}

impl From<MapBoxSearchResponse> for SearchResponse {
    fn from(response: MapBoxSearchResponse) -> Self {
        SearchResponse {
            items: response.features.into_iter().map(|f| f.into()).collect(),
        }
    }
}

#[async_trait]
impl Geocoder for MapBoxGeocoder {
    async fn geocode(
        &self,
        query: &str,
        options: &GeocodeOptions,
    ) -> EveryMapResult<SearchResponse> {
        let url = format!("{}/search/geocode/v6/forward", self.base_url);
        let mut params: Vec<(&str, String)> = vec![("q", query.to_string())];

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
            params.push((
                "bbox",
                format!(
                    "{},{},{},{}",
                    bbox.south_west.lng,
                    bbox.south_west.lat,
                    bbox.north_east.lng,
                    bbox.north_east.lat
                ),
            ));
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

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: MapBoxSearchResponse = self.client.request_json(builder).await?;
        Ok(result.into())
    }

    async fn reverse_geocode(
        &self,
        coordinate: &Coordinate,
        options: &ReverseGeocodeOptions,
    ) -> EveryMapResult<SearchResponse> {
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
            eprintln!(
                "WARNING: MapBox Reverse Geocoding v6 does not support radius constraints; \
                 radius ({}) will be ignored",
                radius
            );
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

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: MapBoxSearchResponse = self.client.request_json(builder).await?;
        Ok(result.into())
    }
}
