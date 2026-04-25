//! Integration tests for live API verification.
//!
//! These tests only run when the `integration` feature is enabled AND the
//! relevant API key environment variables are set. To run:
//!
//! ```sh
//! cargo test -p everymap-core --features integration -- --ignored
//! ```
//!
//! Or with specific provider keys:
//! ```sh
//! EVERYMAP_HERE_API_KEY=pk.xxx cargo test -p everymap-core --features integration -- --ignored
//! ```

#[cfg(feature = "integration")]
mod tests {
    use everymap_core::auth::ApiKeyProvider;
    use everymap_core::domains::routing::{RouteOptions, Router, TransportMode};
    use everymap_core::domains::search::{GeocodeOptions, Geocoder};
    use everymap_core::types::Coordinate;
    use std::sync::Arc;

    fn get_here_api_key() -> Option<String> {
        std::env::var("EVERYMAP_HERE_API_KEY")
            .ok()
            .or_else(|| std::env::var("EVERYMAP_API_KEY").ok())
    }

    fn get_google_api_key() -> Option<String> {
        std::env::var("EVERYMAP_GOOGLE_API_KEY")
            .ok()
            .or_else(|| std::env::var("EVERYMAP_API_KEY").ok())
    }

    fn get_tomtom_api_key() -> Option<String> {
        std::env::var("EVERYMAP_TOMTOM_API_KEY")
            .ok()
            .or_else(|| std::env::var("EVERYMAP_API_KEY").ok())
    }

    fn get_mapbox_api_key() -> Option<String> {
        std::env::var("EVERYMAP_MAPBOX_API_KEY")
            .ok()
            .or_else(|| std::env::var("EVERYMAP_API_KEY").ok())
    }

    // --- HERE Integration Tests ---

    #[tokio::test]
    #[ignore]
    async fn test_here_geocode_live() {
        let api_key = get_here_api_key().expect("Set EVERYMAP_HERE_API_KEY");
        let auth = Arc::new(ApiKeyProvider::new(api_key, "apiKey".to_string()));
        let client = Arc::new(everymap_providers_here::client::HereClient::new(auth));
        let geocoder = everymap_providers_here::domain::search::HereGeocoder::new(client);

        let opts = GeocodeOptions::default();
        let res = geocoder
            .geocode("Brandenburg Gate, Berlin", &opts)
            .await
            .expect("HERE geocode request failed");

        assert!(!res.items.is_empty(), "Expected at least one result");
        let first = &res.items[0];
        assert!(
            first.coordinate.lat > 52.0 && first.coordinate.lat < 53.0,
            "Expected Berlin latitude"
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_here_reverse_geocode_live() {
        let api_key = get_here_api_key().expect("Set EVERYMAP_HERE_API_KEY");
        let auth = Arc::new(ApiKeyProvider::new(api_key, "apiKey".to_string()));
        let client = Arc::new(everymap_providers_here::client::HereClient::new(auth));
        let geocoder = everymap_providers_here::domain::search::HereGeocoder::new(client);

        let coord = Coordinate::new(52.5163, 13.3777).unwrap();
        let opts = everymap_core::domains::search::ReverseGeocodeOptions::default();
        let res = geocoder
            .reverse_geocode(&coord, &opts)
            .await
            .expect("HERE reverse geocode failed");

        assert!(!res.items.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_here_route_live() {
        let api_key = get_here_api_key().expect("Set EVERYMAP_HERE_API_KEY");
        let auth = Arc::new(ApiKeyProvider::new(api_key, "apiKey".to_string()));
        let client = Arc::new(everymap_providers_here::client::HereClient::new(auth));
        let router = everymap_providers_here::domain::routing::HereRouter::new(client);

        let start = Coordinate::new(52.5163, 13.3777).unwrap();
        let end = Coordinate::new(48.8566, 2.3522).unwrap();
        let opts = RouteOptions {
            transport_mode: Some(TransportMode::Car),
            ..Default::default()
        };
        let res = router
            .calculate_route(&start, &end, &opts)
            .await
            .expect("HERE routing failed");

        assert!(!res.routes.is_empty());
        assert!(
            res.routes[0].distance > 100_000.0,
            "Berlin-Paris should be >100km"
        );
    }

    // --- Google Integration Tests ---

    #[tokio::test]
    #[ignore]
    async fn test_google_geocode_live() {
        let api_key = get_google_api_key().expect("Set EVERYMAP_GOOGLE_API_KEY");
        let auth = Arc::new(ApiKeyProvider::new(api_key, "key".to_string()));
        let client = Arc::new(everymap_providers_google::client::GoogleClient::new(auth));
        let geocoder = everymap_providers_google::GoogleGeocoder::new(client);

        let opts = GeocodeOptions::default();
        let res = geocoder
            .geocode("Brandenburg Gate, Berlin", &opts)
            .await
            .expect("Google geocode request failed");

        assert!(!res.items.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_google_route_live() {
        let api_key = get_google_api_key().expect("Set EVERYMAP_GOOGLE_API_KEY");
        let auth = Arc::new(ApiKeyProvider::new(api_key, "key".to_string()));
        let client = Arc::new(everymap_providers_google::client::GoogleClient::new(auth));
        let router = everymap_providers_google::GoogleRouter::new(client);

        let start = Coordinate::new(52.5163, 13.3777).unwrap();
        let end = Coordinate::new(48.8566, 2.3522).unwrap();
        let opts = RouteOptions {
            transport_mode: Some(TransportMode::Car),
            ..Default::default()
        };
        let res = router
            .calculate_route(&start, &end, &opts)
            .await
            .expect("Google routing failed");

        assert!(!res.routes.is_empty());
    }

    // --- TomTom Integration Tests ---

    #[tokio::test]
    #[ignore]
    async fn test_tomtom_geocode_live() {
        let api_key = get_tomtom_api_key().expect("Set EVERYMAP_TOMTOM_API_KEY");
        let auth = Arc::new(ApiKeyProvider::new(api_key, "key".to_string()));
        let client = Arc::new(everymap_providers_tomtom::client::TomTomClient::new(auth));
        let geocoder = everymap_providers_tomtom::TomTomGeocoder::new(client);

        let opts = GeocodeOptions::default();
        let res = geocoder
            .geocode("Brandenburg Gate, Berlin", &opts)
            .await
            .expect("TomTom geocode request failed");

        assert!(!res.items.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_tomtom_route_live() {
        let api_key = get_tomtom_api_key().expect("Set EVERYMAP_TOMTOM_API_KEY");
        let auth = Arc::new(ApiKeyProvider::new(api_key, "key".to_string()));
        let client = Arc::new(everymap_providers_tomtom::client::TomTomClient::new(auth));
        let router = everymap_providers_tomtom::TomTomRouter::new(client);

        let start = Coordinate::new(52.5163, 13.3777).unwrap();
        let end = Coordinate::new(48.8566, 2.3522).unwrap();
        let opts = RouteOptions {
            transport_mode: Some(TransportMode::Car),
            ..Default::default()
        };
        let res = router
            .calculate_route(&start, &end, &opts)
            .await
            .expect("TomTom routing failed");

        assert!(!res.routes.is_empty());
    }

    // --- MapBox Integration Tests ---

    #[tokio::test]
    #[ignore]
    async fn test_mapbox_geocode_live() {
        let api_key = get_mapbox_api_key().expect("Set EVERYMAP_MAPBOX_API_KEY");
        let auth = Arc::new(ApiKeyProvider::new(api_key, "access_token".to_string()));
        let client = Arc::new(everymap_providers_mapbox::client::MapBoxClient::new(auth));
        let geocoder = everymap_providers_mapbox::MapBoxGeocoder::new(client);

        let opts = GeocodeOptions::default();
        let res = geocoder
            .geocode("Brandenburg Gate, Berlin", &opts)
            .await
            .expect("MapBox geocode request failed");

        assert!(!res.items.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_mapbox_route_live() {
        let api_key = get_mapbox_api_key().expect("Set EVERYMAP_MAPBOX_API_KEY");
        let auth = Arc::new(ApiKeyProvider::new(api_key, "access_token".to_string()));
        let client = Arc::new(everymap_providers_mapbox::client::MapBoxClient::new(auth));
        let router = everymap_providers_mapbox::MapBoxRouter::new(client);

        let start = Coordinate::new(52.5163, 13.3777).unwrap();
        let end = Coordinate::new(48.8566, 2.3522).unwrap();
        let opts = RouteOptions {
            transport_mode: Some(TransportMode::Car),
            ..Default::default()
        };
        let res = router
            .calculate_route(&start, &end, &opts)
            .await
            .expect("MapBox routing failed");

        assert!(!res.routes.is_empty());
    }
}
