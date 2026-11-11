pub mod types;

use crate::client::MapBoxClient;
use crate::error_mapping::invalid_response;
use async_trait::async_trait;
use everymap_core::domains::search::{
    GeocodeOptions, Geocoder, ReverseGeocodeOptions, SearchQuality, SearchResponse, SearchResult,
    SearchResultType,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
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

/// Maps a MapBox v6 feature type onto the contract's match quality.
fn quality_from_feature_type(feature_type: &str) -> Option<SearchQuality> {
    match feature_type {
        "address" => Some(SearchQuality::StreetAddress),
        "street" => Some(SearchQuality::Street),
        "poi" => Some(SearchQuality::Point),
        "postcode" => Some(SearchQuality::PostalCode),
        "place" | "locality" | "neighborhood" => Some(SearchQuality::City),
        "district" => Some(SearchQuality::County),
        "region" => Some(SearchQuality::State),
        "country" => Some(SearchQuality::Country),
        _ => None,
    }
}

/// Extracts a valid coordinate from a MapBox feature.
///
/// The v6 response carries `properties.coordinates`; older payloads carry a
/// GeoJSON point in `geometry.coordinates`. A feature with neither cannot be
/// placed on a map and is a structurally unusable response.
fn coordinate_from_feature(
    properties: Option<&MapBoxProperties>,
    geometry: Option<&MapBoxGeometry>,
) -> Option<Coordinate> {
    properties
        .and_then(|properties| properties.coordinates.as_ref())
        .and_then(|coordinates| Coordinate::new(coordinates.latitude?, coordinates.longitude?).ok())
        .or_else(|| {
            geometry
                .and_then(|geometry| geometry.coordinates.as_ref())
                .and_then(|coordinates| coordinates.as_array())
                .and_then(|array| {
                    if array.len() >= 2 {
                        Coordinate::new(array[1].as_f64()?, array[0].as_f64()?).ok()
                    } else {
                        None
                    }
                })
        })
}

impl TryFrom<MapBoxFeature> for SearchResult {
    type Error = EveryMapError;

    fn try_from(f: MapBoxFeature) -> Result<Self, Self::Error> {
        let props = f.properties.as_ref();
        let coordinate = coordinate_from_feature(props, f.geometry.as_ref())
            .ok_or_else(|| invalid_response("search result lacks a valid coordinate"))?;

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

        let feature_type_str = props.and_then(|p| p.feature_type.clone());
        let result_type = feature_type_str
            .as_deref()
            .map(classify_feature_type)
            .unwrap_or(SearchResultType::Unknown);
        let quality = feature_type_str
            .as_deref()
            .and_then(quality_from_feature_type);

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

        Ok(Self {
            id: f.id.or(props.and_then(|p| p.mapbox_id.clone())),
            coordinate,
            address,
            title,
            result_type,
            quality,
            distance: None,
            confidence: props.and_then(|p| p.relevance),
            categories,
            bounding_box,
            raw: None,
        })
    }
}

impl TryFrom<MapBoxSearchResponse> for SearchResponse {
    type Error = EveryMapError;

    fn try_from(response: MapBoxSearchResponse) -> Result<Self, Self::Error> {
        Ok(SearchResponse {
            items: response
                .features
                .into_iter()
                .map(SearchResult::try_from)
                .collect::<EveryMapResult<Vec<_>>>()?,
        })
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
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);

        let result: MapBoxSearchResponse = self.client.request_parsed_json(builder).await?;
        SearchResponse::try_from(result)
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
            log::warn!(
                "MapBox Reverse Geocoding v6 does not support radius constraints; \
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
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);

        let result: MapBoxSearchResponse = self.client.request_parsed_json(builder).await?;
        SearchResponse::try_from(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_feature_without_a_coordinate_is_invalid() {
        let feature: MapBoxFeature = serde_json::from_value(json!({
            "type": "Feature",
            "properties": {
                "feature_type": "place",
                "name": "Berlin"
            }
        }))
        .expect("the fixture deserializes");
        match SearchResult::try_from(feature) {
            Err(EveryMapError::ProviderError { code, .. }) => {
                assert_eq!(code, crate::error_mapping::INVALID_RESPONSE_CODE);
            }
            other => panic!("expected an invalid-response error, got {other:?}"),
        }
    }

    #[test]
    fn quality_comes_from_the_feature_type() {
        let feature: MapBoxFeature = serde_json::from_value(json!({
            "type": "Feature",
            "properties": {
                "feature_type": "address",
                "name": "Unter den Linden 1",
                "coordinates": { "longitude": 13.38, "latitude": 52.51 }
            }
        }))
        .expect("the fixture deserializes");
        let mapped = SearchResult::try_from(feature).expect("the fixture maps");
        assert_eq!(mapped.quality, Some(SearchQuality::StreetAddress));
    }
}
