//! Photon provider for EveryMap: geocoding, place search and type-ahead.
//!
//! Photon is the open-source geocoder behind `photon.komoot.io`. It answers
//! forward queries from `/api` and reverse queries from `/reverse`, both as a
//! GeoJSON `FeatureCollection` whose features carry a class in `type` and the
//! address components in `properties`.
//!
//! The crate implements the vendor-neutral EveryMap contracts
//! [`Geocoder`],
//! [`PlaceSearchProvider`]
//! and
//! [`AutocompleteProvider`]
//! over one deployment. It is keyless: the public instance needs no
//! credential, and a self-hosted deployment is addressed by its base URL.
//!
//! # Configuration
//!
//! [`PhotonProviderConfiguration`] carries the deployment root and the
//! `User-Agent` the service sees. The public Photon instance asks clients to
//! identify themselves, so the default is a provider-versioned agent.
//!
//! ```no_run
//! use std::sync::Arc;
//! use everymap_providers_photon::{PhotonClient, PhotonGeocoder, PhotonProviderConfiguration};
//!
//! let client = Arc::new(PhotonClient::new(PhotonProviderConfiguration::default()));
//! let geocoder = PhotonGeocoder::new(client);
//! # let _ = geocoder;
//! ```
//!
//! # Targets
//!
//! The crate builds for `wasm32-unknown-unknown`; every request goes through
//! [`everymap_core::client::ProviderClient`], whose transport is `reqwest`
//! natively and `window.fetch` in a browser. Tests inject a transport through
//! a wiremock server and never open an external socket.

use async_trait::async_trait;
use everymap_core::auth::NoAuthProvider;
use everymap_core::client::ProviderClient;
use everymap_core::domains::autocomplete::{
    AutocompleteOptions, AutocompleteProvider, AutocompleteResponse, AutocompleteResult,
};
use everymap_core::domains::search::{
    GeocodeOptions, Geocoder, PlaceSearchOptions, PlaceSearchProvider, ReverseGeocodeOptions,
    SearchQuality, SearchResponse, SearchResult, SearchResultType,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::http::{HttpTransport, Method};
use everymap_core::types::{Address, BoundingBox, Coordinate};
use serde_json::Value;
use std::sync::Arc;

/// Base URL of the public Photon instance.
pub const DEFAULT_PHOTON_BASE_URL: &str = "https://photon.komoot.io";

/// `User-Agent` value sent when the configuration carries none.
pub const DEFAULT_PHOTON_USER_AGENT: &str =
    concat!("everymap-photon-provider/", env!("CARGO_PKG_VERSION"));

/// Name this provider reports in errors and request observations.
pub const PHOTON_PROVIDER_NAME: &str = "photon";

/// The forward-query endpoint path.
const SEARCH_ENDPOINT_PATH: &str = "api";

/// The reverse-query endpoint path.
const REVERSE_ENDPOINT_PATH: &str = "reverse";

/// Candidates a type-ahead asks for: the reference applications show five.
pub const AUTOCOMPLETE_RESULT_LIMIT: u32 = 5;

/// Candidates a settled search asks for: a longer list than the type-ahead.
pub const SEARCH_RESULT_LIMIT: u32 = 10;

/// Header carrying the configured user agent.
const USER_AGENT_HEADER_NAME: &str = "user-agent";

/// Separator between a display line's address components.
const DISPLAY_COMPONENT_SEPARATOR: &str = ", ";

/// Connection settings for one Photon deployment.
///
/// # Fields
/// - `base_url` — deployment root without the API path, for example
///   `https://photon.komoot.io`. Tests point it at an in-process server.
/// - `user_agent` — `User-Agent` header sent with every request. An empty
///   value suppresses the header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotonProviderConfiguration {
    /// Deployment root URL without the API path.
    pub base_url: String,
    /// `User-Agent` header value sent with every request.
    pub user_agent: String,
}

impl Default for PhotonProviderConfiguration {
    /// Settings for the public Photon instance.
    fn default() -> Self {
        Self {
            base_url: DEFAULT_PHOTON_BASE_URL.to_string(),
            user_agent: DEFAULT_PHOTON_USER_AGENT.to_string(),
        }
    }
}

/// The keyless client every Photon provider shares.
///
/// The client wraps [`ProviderClient`], so pacing, retries, timeouts and
/// request observation match every other provider's pipeline.
pub struct PhotonClient {
    inner: ProviderClient,
    base_url: String,
    user_agent: String,
}

impl PhotonClient {
    /// Builds a client on the target's default transport.
    pub fn new(configuration: PhotonProviderConfiguration) -> Self {
        Self {
            inner: ProviderClient::new(Arc::new(NoAuthProvider::new()), PHOTON_PROVIDER_NAME),
            base_url: configuration.base_url,
            user_agent: configuration.user_agent,
        }
    }

    /// Builds a client that executes requests through an explicit transport.
    ///
    /// # Parameters
    /// - `configuration` — deployment root and user agent.
    /// - `transport` — the transport that sends each request.
    pub fn with_transport(
        configuration: PhotonProviderConfiguration,
        transport: Arc<dyn HttpTransport>,
    ) -> Self {
        Self {
            inner: ProviderClient::with_transport(
                transport,
                Arc::new(NoAuthProvider::new()),
                PHOTON_PROVIDER_NAME,
            ),
            base_url: configuration.base_url,
            user_agent: configuration.user_agent,
        }
    }

    /// Sends one GET query and decodes its JSON body.
    async fn get_json(
        &self,
        path: &str,
        query_parameters: Vec<(String, String)>,
    ) -> EveryMapResult<Value> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path);
        let mut request = self.inner.build_request(Method::Get, &url);
        for (parameter_name, parameter_value) in query_parameters {
            request = request.query_param(parameter_name, parameter_value);
        }
        if !self.user_agent.is_empty() {
            request = request.header(USER_AGENT_HEADER_NAME, &self.user_agent);
        }
        let response = self.inner.request(request).await?;
        // A body that is not JSON is the provider's answer, not a transport
        // failure; the shared error keeps the parser's own message.
        serde_json::from_str(&response.text())
            .map_err(|source| EveryMapError::SerializationError { source })
    }
}

/// Forward and reverse geocoding over Photon.
pub struct PhotonGeocoder {
    client: Arc<PhotonClient>,
}

impl PhotonGeocoder {
    /// Builds a geocoder around a shared client.
    pub fn new(client: Arc<PhotonClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Geocoder for PhotonGeocoder {
    async fn geocode(
        &self,
        query: &str,
        options: &GeocodeOptions,
    ) -> EveryMapResult<SearchResponse> {
        let limit = options.limit.unwrap_or(SEARCH_RESULT_LIMIT);
        run_search(
            &self.client,
            query,
            limit,
            options.bounding_box,
            options.language.as_deref(),
        )
        .await
    }

    async fn reverse_geocode(
        &self,
        coordinate: &Coordinate,
        _options: &ReverseGeocodeOptions,
    ) -> EveryMapResult<SearchResponse> {
        let query_parameters = vec![
            ("lon".to_string(), coordinate.lng.to_string()),
            ("lat".to_string(), coordinate.lat.to_string()),
            ("limit".to_string(), 1.to_string()),
        ];
        let payload = self
            .client
            .get_json(REVERSE_ENDPOINT_PATH, query_parameters)
            .await?;
        Ok(SearchResponse {
            items: places_from_feature_collection(&payload)?,
        })
    }
}

/// Settled place search over Photon.
pub struct PhotonPlaceSearchProvider {
    client: Arc<PhotonClient>,
}

impl PhotonPlaceSearchProvider {
    /// Builds a search provider around a shared client.
    pub fn new(client: Arc<PhotonClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl PlaceSearchProvider for PhotonPlaceSearchProvider {
    async fn search_places(
        &self,
        query: &str,
        options: &PlaceSearchOptions,
    ) -> EveryMapResult<SearchResponse> {
        let limit = options.limit.unwrap_or(SEARCH_RESULT_LIMIT);
        run_search(
            &self.client,
            query,
            limit,
            options.bounding_box,
            options.language.as_deref(),
        )
        .await
    }
}

/// Type-ahead completion over Photon.
///
/// Photon has one forward endpoint, so a completion is the same query with
/// the type-ahead's smaller result cap; each feature's own position becomes
/// the candidate's coordinate when it has one.
pub struct PhotonAutocompleteProvider {
    client: Arc<PhotonClient>,
}

impl PhotonAutocompleteProvider {
    /// Builds a completion provider around a shared client.
    pub fn new(client: Arc<PhotonClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl AutocompleteProvider for PhotonAutocompleteProvider {
    async fn autocomplete(
        &self,
        query: &str,
        options: &AutocompleteOptions,
    ) -> EveryMapResult<AutocompleteResponse> {
        let limit = options.limit.unwrap_or(AUTOCOMPLETE_RESULT_LIMIT);
        let response = run_search(
            &self.client,
            query,
            limit,
            options.bounding_box,
            options.language.as_deref(),
        )
        .await?;
        Ok(AutocompleteResponse {
            items: response
                .items
                .into_iter()
                .map(autocomplete_result)
                .collect(),
        })
    }
}

/// Runs one forward query and maps its features onto search results.
async fn run_search(
    client: &PhotonClient,
    query: &str,
    limit: u32,
    bounding_box: Option<BoundingBox>,
    language: Option<&str>,
) -> EveryMapResult<SearchResponse> {
    if query.trim().is_empty() {
        return Err(EveryMapError::ValidationError(
            "the search text must not be empty".to_string(),
        ));
    }
    let mut query_parameters = vec![
        ("q".to_string(), query.to_string()),
        ("limit".to_string(), limit.to_string()),
    ];
    if let Some(language) = language {
        query_parameters.push(("lang".to_string(), language.to_string()));
    }
    if let Some(bounding_box) = bounding_box {
        let center = bounding_box_center(&bounding_box);
        query_parameters.push(("lat".to_string(), center.lat.to_string()));
        query_parameters.push(("lon".to_string(), center.lng.to_string()));
    }
    let payload = client
        .get_json(SEARCH_ENDPOINT_PATH, query_parameters)
        .await?;
    Ok(SearchResponse {
        items: places_from_feature_collection(&payload)?,
    })
}

/// The centre of a bounding box, the point Photon ranks around.
fn bounding_box_center(bounding_box: &BoundingBox) -> Coordinate {
    Coordinate {
        lat: (bounding_box.south_west.lat + bounding_box.north_east.lat) / 2.0,
        lng: (bounding_box.south_west.lng + bounding_box.north_east.lng) / 2.0,
    }
}

/// Projects a GeoJSON `FeatureCollection` onto search results.
fn places_from_feature_collection(payload: &Value) -> EveryMapResult<Vec<SearchResult>> {
    let features = payload
        .get("features")
        .and_then(Value::as_array)
        .ok_or_else(|| EveryMapError::ProviderError {
            provider: PHOTON_PROVIDER_NAME.to_string(),
            code: "MISSING_FEATURES".to_string(),
            message: "photon returned no features collection".to_string(),
        })?;
    features
        .iter()
        .filter_map(place_from_feature)
        .collect::<EveryMapResult<Vec<SearchResult>>>()
}

/// Projects one GeoJSON feature onto a search result, or `None` when the
/// feature carries no usable position.
fn place_from_feature(feature: &Value) -> Option<EveryMapResult<SearchResult>> {
    let coordinates = feature
        .pointer("/geometry/coordinates")
        .and_then(Value::as_array)?;
    let longitude = coordinates.first()?.as_f64()?;
    let latitude = coordinates.get(1)?.as_f64()?;
    Some((|| {
        let coordinate =
            Coordinate::new(latitude, longitude).map_err(|error| EveryMapError::ProviderError {
                provider: PHOTON_PROVIDER_NAME.to_string(),
                code: "INVALID_COORDINATE".to_string(),
                message: error.to_string(),
            })?;
        let properties = feature.get("properties").cloned().unwrap_or(Value::Null);
        let property = |name: &str| {
            properties
                .get(name)
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string)
        };
        let feature_type = property("type").unwrap_or_default();
        let quality = quality_for_feature_type(&feature_type);
        let display_text = property("name").unwrap_or_else(|| {
            address_display_text(
                &property("street"),
                &property("housenumber"),
                &property("postcode"),
                &property("city"),
                &property("state"),
                &property("country"),
            )
        });
        let address = Address {
            label: Some(display_text.clone()),
            street: property("street"),
            house_number: property("housenumber"),
            city: property("city"),
            district: property("district"),
            state: property("state"),
            postal_code: property("postcode"),
            country: property("country"),
            country_code: property("countrycode"),
            county: property("county"),
            ..Address::empty()
        };
        Ok(SearchResult {
            id: property("osm_id")
                .map(|osm_id| format!("{}{osm_id}", property("osm_type").unwrap_or_default())),
            coordinate,
            address,
            title: Some(display_text),
            result_type: SearchResultType::ExactMatch,
            quality: Some(quality),
            distance: None,
            confidence: None,
            categories: vec![feature_type],
            bounding_box: extent_bounding_box(&properties),
            raw: Some(feature.clone()),
        })
    })())
}

/// The display line Photon's features compose from their address parts.
fn address_display_text(
    street: &Option<String>,
    house_number: &Option<String>,
    postcode: &Option<String>,
    city: &Option<String>,
    state: &Option<String>,
    country: &Option<String>,
) -> String {
    let street_line = match (house_number, street) {
        (Some(house_number), Some(street)) => Some(format!("{house_number} {street}")),
        (None, Some(street)) => Some(street.clone()),
        _ => None,
    };
    [
        street_line,
        postcode.clone(),
        city.clone(),
        state.clone(),
        country.clone(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<String>>()
    .join(DISPLAY_COMPONENT_SEPARATOR)
}

/// Maps Photon's `type` vocabulary onto the core's match granularity.
fn quality_for_feature_type(feature_type: &str) -> SearchQuality {
    match feature_type {
        "house" | "building" => SearchQuality::StreetAddress,
        "street" => SearchQuality::Street,
        "locality" | "district" | "city" | "town" | "village" => SearchQuality::City,
        "county" => SearchQuality::County,
        "state" => SearchQuality::State,
        "country" => SearchQuality::Country,
        "postcode" => SearchQuality::PostalCode,
        _ => SearchQuality::Point,
    }
}

/// Reads a feature's `extent` (`[minLon, minLat, maxLon, maxLat]`) as bounds.
fn extent_bounding_box(properties: &Value) -> Option<BoundingBox> {
    let extent = properties.get("extent").and_then(Value::as_array)?;
    let coordinate_at = |index: usize| -> Option<Coordinate> {
        Some(Coordinate {
            lng: extent.get(index)?.as_f64()?,
            lat: extent.get(index + 1)?.as_f64()?,
        })
    };
    Some(BoundingBox {
        south_west: coordinate_at(0)?,
        north_east: coordinate_at(2)?,
    })
}

/// Projects one search result onto a completion candidate.
fn autocomplete_result(result: SearchResult) -> AutocompleteResult {
    AutocompleteResult {
        id: result.id,
        title: result
            .title
            .unwrap_or_else(|| result.address.label.clone().unwrap_or_default()),
        subtitle: result.address.city.or(result.address.country),
        coordinate: Some(result.coordinate),
        bounding_box: result.bounding_box,
        quality: result.quality,
        categories: result.categories,
        raw: result.raw,
    }
}

#[cfg(test)]
mod tests {
    use super::{address_display_text, quality_for_feature_type};
    use everymap_core::domains::search::SearchQuality;

    #[test]
    fn the_feature_class_maps_onto_the_quality_scale() {
        assert_eq!(
            quality_for_feature_type("house"),
            SearchQuality::StreetAddress
        );
        assert_eq!(quality_for_feature_type("street"), SearchQuality::Street);
        assert_eq!(quality_for_feature_type("locality"), SearchQuality::City);
        assert_eq!(quality_for_feature_type("county"), SearchQuality::County);
        assert_eq!(quality_for_feature_type("state"), SearchQuality::State);
        assert_eq!(quality_for_feature_type("country"), SearchQuality::Country);
        assert_eq!(
            quality_for_feature_type("postcode"),
            SearchQuality::PostalCode
        );
        assert_eq!(quality_for_feature_type("unknown"), SearchQuality::Point);
    }

    #[test]
    fn the_display_line_joins_the_address_parts_it_has() {
        let text = address_display_text(
            &Some("Main Street".to_string()),
            &Some("1".to_string()),
            &Some("95814".to_string()),
            &Some("Sacramento".to_string()),
            &Some("California".to_string()),
            &Some("United States".to_string()),
        );
        assert_eq!(
            text,
            "1 Main Street, 95814, Sacramento, California, United States"
        );
    }
}
