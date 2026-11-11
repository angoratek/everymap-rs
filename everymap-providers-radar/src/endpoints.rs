//! Radar API endpoints derived from one configurable service root.
//!
//! Every Radar domain follows one convention: the configurable base value is
//! the service root such as `https://api.radar.io`, and each endpoint is the
//! root followed by its documented path. A root with a trailing slash is
//! accepted. Deriving all endpoints from one root keeps a test server, a
//! proxy or a regional deployment consistent across the search, routing,
//! matching and tour domains.

/// The public Radar API service root.
pub const RADAR_API_ROOT: &str = "https://api.radar.io";

/// Documented path of the forward geocoding endpoint.
pub const FORWARD_GEOCODE_PATH: &str = "/v1/geocode/forward";

/// Documented path of the reverse geocoding endpoint.
pub const REVERSE_GEOCODE_PATH: &str = "/v1/geocode/reverse";

/// Documented path of the IP geocoding endpoint.
pub const IP_GEOCODE_PATH: &str = "/v1/geocode/ip";

/// Documented path of the autocomplete endpoint.
pub const AUTOCOMPLETE_PATH: &str = "/v1/search/autocomplete";

/// Documented path of the address validation endpoint.
pub const ADDRESS_VALIDATION_PATH: &str = "/v1/addresses/validate";

/// Documented path of the place search endpoint.
pub const PLACE_SEARCH_PATH: &str = "/v1/search/places";

/// Documented path of the directions endpoint.
pub const DIRECTIONS_PATH: &str = "/v1/route/directions";

/// Documented path of the distance endpoint.
pub const DISTANCE_PATH: &str = "/v1/route/distance";

/// Documented path of the distance matrix endpoint.
pub const MATRIX_PATH: &str = "/v1/route/matrix";

/// Documented path of the route matching endpoint.
pub const MATCH_ROUTE_PATH: &str = "/v1/route/match";

/// Documented path of the tour optimization endpoint.
pub const OPTIMIZE_TOUR_PATH: &str = "/v1/route/optimize";

/// Builds an endpoint URL from the Radar service root and a documented path.
///
/// # Parameters
/// - `service_root` — the scheme and host serving the API.
/// - `path` — the documented API path, starting with a slash.
///
/// # Returns
/// The joined endpoint URL.
pub(crate) fn endpoint(service_root: &str, path: &str) -> String {
    format!("{}{}", service_root.trim_end_matches('/'), path)
}

#[cfg(test)]
mod tests {
    use super::{endpoint, FORWARD_GEOCODE_PATH, RADAR_API_ROOT};

    #[test]
    fn the_public_root_and_path_join_into_the_documented_endpoint() {
        assert_eq!(
            endpoint(RADAR_API_ROOT, FORWARD_GEOCODE_PATH),
            "https://api.radar.io/v1/geocode/forward"
        );
    }

    #[test]
    fn a_trailing_slash_on_the_root_is_accepted() {
        assert_eq!(
            endpoint("http://127.0.0.1:8080/", FORWARD_GEOCODE_PATH),
            "http://127.0.0.1:8080/v1/geocode/forward"
        );
    }
}
