use crate::domain::matching::types::*;
use crate::domain::routing::types::*;
use crate::domain::search::types::*;
use crate::RadarTravelMode;
use async_trait::async_trait;
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;

/// Extension trait for Radar-specific geocoding methods.
#[async_trait]
pub trait RadarGeocoderExt {
    /// IP-based geolocation — resolves requester's IP to city/state/country.
    async fn ip_geocode(&self) -> EveryMapResult<RadarIpGeocodeResponse>;

    /// Autocomplete partial addresses and place names.
    async fn autocomplete(
        &self,
        query: &str,
        near: Option<&Coordinate>,
        options: &RadarAutocompleteOptions,
    ) -> EveryMapResult<RadarAutocompleteResponse>;

    /// Validate a structured address (US/Canada).
    async fn validate_address(
        &self,
        options: &RadarAddressValidationOptions,
    ) -> EveryMapResult<RadarAddressValidationResponse>;
}

/// Extension trait for Radar-specific routing methods.
#[async_trait]
pub trait RadarRouterExt {
    /// Distance and duration between origin and destination (supports multi-mode).
    async fn distance(
        &self,
        origin: &Coordinate,
        destination: &Coordinate,
        modes: &[RadarTravelMode],
        units: Option<&str>,
    ) -> EveryMapResult<RadarDistanceResponse>;

    /// Travel distance/duration matrix (up to 625 routes).
    async fn matrix(
        &self,
        origins: &[Coordinate],
        destinations: &[Coordinate],
        mode: RadarTravelMode,
        units: Option<&str>,
    ) -> EveryMapResult<RadarMatrixResponse>;
}

/// Extension trait for Radar-specific search methods.
#[async_trait]
pub trait RadarSearchExt {
    /// Search places near a location, filtered by chain/category.
    async fn search_places(
        &self,
        near: &Coordinate,
        chains: Option<&[String]>,
        categories: Option<&[String]>,
        radius: Option<f64>,
        limit: Option<u32>,
    ) -> EveryMapResult<RadarPlaceSearchResponse>;
}

/// Extension trait for Radar-specific route matching with road attributes.
#[async_trait]
pub trait RadarMatchingExt {
    /// Route match with road attributes (speed limits, names, road class).
    async fn match_route_with_attributes(
        &self,
        points: &[Coordinate],
        mode: RadarTravelMode,
    ) -> EveryMapResult<RadarRouteMatchResponse>;
}

// --- Search types for extension traits ---

use crate::domain::types::RadarMeta;
use serde::{Deserialize, Serialize};

/// Response from Radar Search Places API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarPlaceSearchResponse {
    pub meta: RadarMeta,
    #[serde(default)]
    pub places: Vec<RadarPlace>,
}

/// A Radar place (POI).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarPlace {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub categories: Vec<String>,
    pub location: Option<crate::domain::types::RadarLocation>,
    pub chain: Option<RadarChain>,
}

/// A Radar chain (brand/brand group).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarChain {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub external_id: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

// --- Extension trait implementations ---

use crate::domain::matching::RadarRouteMatcher;
use crate::domain::routing::RadarRouter;
use crate::domain::search::RadarGeocoder;

#[async_trait]
impl RadarGeocoderExt for RadarGeocoder {
    async fn ip_geocode(&self) -> EveryMapResult<RadarIpGeocodeResponse> {
        let builder = self
            .client
            .build_request(reqwest::Method::GET, &self.ip_geocode_url);
        self.client.request_json(builder).await
    }

    async fn autocomplete(
        &self,
        query: &str,
        near: Option<&Coordinate>,
        options: &RadarAutocompleteOptions,
    ) -> EveryMapResult<RadarAutocompleteResponse> {
        let mut params: Vec<(&str, String)> = vec![("query", query.to_string())];

        if let Some(n) = near {
            params.push(("near", format!("{},{}", n.lat, n.lng)));
        }
        if let Some(layers) = &options.layers {
            params.push(("layers", layers.clone()));
        }
        if let Some(limit) = options.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(cc) = &options.country_code {
            params.push(("countryCode", cc.clone()));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &self.autocomplete_url)
            .query(&params);

        self.client.request_json(builder).await
    }

    async fn validate_address(
        &self,
        options: &RadarAddressValidationOptions,
    ) -> EveryMapResult<RadarAddressValidationResponse> {
        let mut params: Vec<(&str, String)> = vec![
            ("city", options.city.clone()),
            ("stateCode", options.state_code.clone()),
            ("postalCode", options.postal_code.clone()),
            ("countryCode", options.country_code.clone()),
        ];

        if let Some(num) = &options.number {
            params.push(("number", num.clone()));
        }
        if let Some(street) = &options.street {
            params.push(("street", street.clone()));
        }
        if let Some(unit) = &options.unit {
            params.push(("unit", unit.clone()));
        }
        if let Some(label) = &options.address_label {
            params.push(("addressLabel", label.clone()));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &self.validate_url)
            .query(&params);

        self.client.request_json(builder).await
    }
}

#[async_trait]
impl RadarRouterExt for RadarRouter {
    async fn distance(
        &self,
        origin: &Coordinate,
        destination: &Coordinate,
        modes: &[RadarTravelMode],
        units: Option<&str>,
    ) -> EveryMapResult<RadarDistanceResponse> {
        let mode_str: Vec<String> = modes
            .iter()
            .map(|m| match m {
                RadarTravelMode::Car => "car".to_string(),
                RadarTravelMode::Truck => "truck".to_string(),
                RadarTravelMode::Foot => "foot".to_string(),
                RadarTravelMode::Bike => "bike".to_string(),
            })
            .collect();

        let mut params: Vec<(&str, String)> = vec![
            ("origin", format!("{},{}", origin.lat, origin.lng)),
            (
                "destination",
                format!("{},{}", destination.lat, destination.lng),
            ),
            ("modes", mode_str.join(",")),
        ];

        if let Some(u) = units {
            params.push(("units", u.to_string()));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &self.distance_url)
            .query(&params);

        self.client.request_json(builder).await
    }

    async fn matrix(
        &self,
        origins: &[Coordinate],
        destinations: &[Coordinate],
        mode: RadarTravelMode,
        units: Option<&str>,
    ) -> EveryMapResult<RadarMatrixResponse> {
        let mode_str = match mode {
            RadarTravelMode::Car => "car",
            RadarTravelMode::Truck => "truck",
            RadarTravelMode::Foot => "foot",
            RadarTravelMode::Bike => "bike",
        };

        let origins_str: Vec<String> = origins
            .iter()
            .map(|o| format!("{},{}", o.lat, o.lng))
            .collect();
        let dests_str: Vec<String> = destinations
            .iter()
            .map(|destination| format!("{},{}", destination.lat, destination.lng))
            .collect();

        let mut params: Vec<(&str, String)> = vec![
            ("origins", origins_str.join("|")),
            ("destinations", dests_str.join("|")),
            ("mode", mode_str.to_string()),
        ];

        if let Some(u) = units {
            params.push(("units", u.to_string()));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &self.matrix_url)
            .query(&params);

        self.client.request_json(builder).await
    }
}

#[async_trait]
impl RadarSearchExt for RadarGeocoder {
    async fn search_places(
        &self,
        near: &Coordinate,
        chains: Option<&[String]>,
        categories: Option<&[String]>,
        radius: Option<f64>,
        limit: Option<u32>,
    ) -> EveryMapResult<RadarPlaceSearchResponse> {
        let mut params: Vec<(&str, String)> = vec![("near", format!("{},{}", near.lat, near.lng))];

        if let Some(c) = chains {
            params.push(("chains", c.join(",")));
        }
        if let Some(cats) = categories {
            params.push(("categories", cats.join(",")));
        }
        if let Some(r) = radius {
            params.push(("radius", r.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &self.places_url)
            .query(&params);

        self.client.request_json(builder).await
    }
}

#[async_trait]
impl RadarMatchingExt for RadarRouteMatcher {
    async fn match_route_with_attributes(
        &self,
        points: &[Coordinate],
        mode: RadarTravelMode,
    ) -> EveryMapResult<RadarRouteMatchResponse> {
        let mode_str = match mode {
            RadarTravelMode::Car => "car",
            RadarTravelMode::Truck => "truck",
            RadarTravelMode::Foot => "foot",
            RadarTravelMode::Bike => "bike",
        };

        let path: Vec<serde_json::Value> = points
            .iter()
            .map(|p| {
                serde_json::json!({
                    "coordinates": format!("{},{}", p.lat, p.lng)
                })
            })
            .collect();

        let body = serde_json::json!({
            "path": path,
            "mode": mode_str,
            "roadAttributes": "speedLimit,names,roadClass",
            "geometry": "polyline6"
        });

        self.client.post_json(&self.base_url, &body).await
    }
}
