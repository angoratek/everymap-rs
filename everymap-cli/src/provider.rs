use everymap_core::auth::header::HeaderAuthProvider;
use everymap_core::auth::{ApiKeyProvider, AuthProvider};
use everymap_core::domains::attributes::AttributeProvider;
use everymap_core::domains::imaging::MapImageProvider;
use everymap_core::domains::isoline::IsolineProvider;
use everymap_core::domains::matching::RouteMatcher;
use everymap_core::domains::positioning::NetworkPositioner as NetworkPositionerTrait;
use everymap_core::domains::routing::Router;
use everymap_core::domains::search::Geocoder;
use everymap_core::domains::tiling::TileProvider;
use everymap_core::domains::tour::TourPlanner;
use everymap_core::domains::traffic::TrafficProvider;
use std::sync::Arc;

/// Registry that creates trait-object providers for the selected provider.
///
/// This replaces the previous pattern of 5 separate `run_<provider>_commands()`
/// functions with a single unified dispatch. Each accessor method returns a
/// `Box<dyn Trait>` that either delegates to a real implementation or to a
/// stub that returns `EveryMapError::UnsupportedDomain`.
pub struct ProviderRegistry {
    geocoder: Box<dyn Geocoder>,
    router: Box<dyn Router>,
    traffic: Box<dyn TrafficProvider>,
    positioner: Box<dyn NetworkPositionerTrait>,
    isoline: Box<dyn IsolineProvider>,
    route_matcher: Box<dyn RouteMatcher>,
    tour_planner: Box<dyn TourPlanner>,
    tile_provider: Box<dyn TileProvider>,
    attribute_provider: Box<dyn AttributeProvider>,
    image_provider: Box<dyn MapImageProvider>,
}

impl ProviderRegistry {
    pub fn new(provider: &str, api_key: String, key_param: String, verbose: bool) -> Self {
        match provider {
            "here" => Self::new_here(api_key, key_param, verbose),
            "google" => Self::new_google(api_key, key_param, verbose),
            "tomtom" => Self::new_tomtom(api_key, key_param, verbose),
            "mapbox" => Self::new_mapbox(api_key, key_param, verbose),
            "radar" => Self::new_radar(api_key, verbose),
            _ => unreachable!("Provider validation already done in main"),
        }
    }

    fn new_here(api_key: String, key_param: String, verbose: bool) -> Self {
        let auth: Arc<dyn AuthProvider> = Arc::new(ApiKeyProvider::new(api_key, key_param));
        let mut client = everymap_providers_here::client::HereClient::new(auth);
        client.set_verbose(verbose);
        let client = Arc::new(client);

        Self {
            geocoder: Box::new(everymap_providers_here::domain::search::HereGeocoder::new(
                client.clone(),
            )),
            router: Box::new(everymap_providers_here::domain::routing::HereRouter::new(
                client.clone(),
            )),
            traffic: Box::new(everymap_providers_here::domain::traffic::HereTraffic::new(
                client.clone(),
            )),
            positioner: Box::new(
                everymap_providers_here::domain::positioning::HerePositioner::new(client.clone()),
            ),
            isoline: Box::new(everymap_providers_here::domain::isoline::HereIsoline::new(
                client.clone(),
            )),
            route_matcher: Box::new(
                everymap_providers_here::domain::matching::HereRouteMatcher::new(client.clone()),
            ),
            tour_planner: Box::new(everymap_providers_here::domain::tour::HereTourPlanner::new(
                client.clone(),
            )),
            tile_provider: Box::new(
                everymap_providers_here::domain::tiling::HereTileProvider::new(client.clone()),
            ),
            attribute_provider: Box::new(
                everymap_providers_here::domain::attributes::HereAttributeProvider::new(
                    client.clone(),
                ),
            ),
            image_provider: Box::new(
                everymap_providers_here::domain::imaging::HereMapImageProvider::new(client),
            ),
        }
    }

    fn new_google(api_key: String, key_param: String, verbose: bool) -> Self {
        let auth: Arc<dyn AuthProvider> = Arc::new(ApiKeyProvider::new(api_key, key_param));
        let mut client = everymap_providers_google::client::GoogleClient::new(auth);
        client.set_verbose(verbose);
        let client = Arc::new(client);

        Self {
            geocoder: Box::new(everymap_providers_google::GoogleGeocoder::new(
                client.clone(),
            )),
            router: Box::new(everymap_providers_google::GoogleRouter::new(client.clone())),
            traffic: Box::new(everymap_providers_google::GoogleTraffic),
            positioner: Box::new(everymap_providers_google::GooglePositioner::new(
                client.clone(),
            )),
            isoline: Box::new(everymap_providers_google::GoogleIsoline),
            route_matcher: Box::new(everymap_providers_google::GoogleRouteMatcher::new(
                client.clone(),
            )),
            tour_planner: Box::new(everymap_providers_google::GoogleTourPlanner),
            tile_provider: Box::new(everymap_providers_google::GoogleTileProvider),
            attribute_provider: Box::new(everymap_providers_google::GoogleAttributeProvider::new(
                client.clone(),
            )),
            image_provider: Box::new(everymap_providers_google::GoogleMapImageProvider::new(
                client,
            )),
        }
    }

    fn new_tomtom(api_key: String, key_param: String, verbose: bool) -> Self {
        let auth: Arc<dyn AuthProvider> = Arc::new(ApiKeyProvider::new(api_key, key_param));
        let mut client = everymap_providers_tomtom::client::TomTomClient::new(auth);
        client.set_verbose(verbose);
        let client = Arc::new(client);

        Self {
            geocoder: Box::new(everymap_providers_tomtom::TomTomGeocoder::new(
                client.clone(),
            )),
            router: Box::new(everymap_providers_tomtom::TomTomRouter::new(client.clone())),
            traffic: Box::new(everymap_providers_tomtom::TomTomTraffic::new(
                client.clone(),
            )),
            positioner: Box::new(everymap_providers_tomtom::TomTomPositioner),
            isoline: Box::new(everymap_providers_tomtom::TomTomIsoline::new(
                client.clone(),
            )),
            route_matcher: Box::new(everymap_providers_tomtom::TomTomRouteMatcher::new(
                client.clone(),
            )),
            tour_planner: Box::new(everymap_providers_tomtom::TomTomTourPlanner::new(
                client.clone(),
            )),
            tile_provider: Box::new(everymap_providers_tomtom::TomTomTileProvider::new(
                client.clone(),
            )),
            attribute_provider: Box::new(everymap_providers_tomtom::TomTomAttributeProvider),
            image_provider: Box::new(everymap_providers_tomtom::TomTomMapImageProvider::new(
                client,
            )),
        }
    }

    fn new_mapbox(api_key: String, key_param: String, verbose: bool) -> Self {
        let auth: Arc<dyn AuthProvider> = Arc::new(ApiKeyProvider::new(api_key, key_param));
        let mut client = everymap_providers_mapbox::client::MapBoxClient::new(auth);
        client.set_verbose(verbose);
        let client = Arc::new(client);

        Self {
            geocoder: Box::new(everymap_providers_mapbox::MapBoxGeocoder::new(
                client.clone(),
            )),
            router: Box::new(everymap_providers_mapbox::MapBoxRouter::new(client.clone())),
            traffic: Box::new(everymap_providers_mapbox::MapBoxTraffic),
            positioner: Box::new(everymap_providers_mapbox::MapBoxPositioner),
            isoline: Box::new(everymap_providers_mapbox::MapBoxIsoline::new(
                client.clone(),
            )),
            route_matcher: Box::new(everymap_providers_mapbox::MapBoxRouteMatcher::new(
                client.clone(),
            )),
            tour_planner: Box::new(everymap_providers_mapbox::MapBoxTourPlanner::new(
                client.clone(),
            )),
            tile_provider: Box::new(everymap_providers_mapbox::MapBoxTileProvider::new(
                client.clone(),
            )),
            attribute_provider: Box::new(everymap_providers_mapbox::MapBoxAttributeProvider),
            image_provider: Box::new(everymap_providers_mapbox::MapBoxMapImageProvider::new(
                client,
            )),
        }
    }

    fn new_radar(api_key: String, verbose: bool) -> Self {
        let auth: Arc<dyn AuthProvider> = Arc::new(HeaderAuthProvider::new(api_key));
        let mut client = everymap_providers_radar::client::RadarClient::new(auth);
        client.set_verbose(verbose);
        let client = Arc::new(client);

        Self {
            geocoder: Box::new(everymap_providers_radar::RadarGeocoder::new(client.clone())),
            router: Box::new(everymap_providers_radar::RadarRouter::new(client.clone())),
            traffic: Box::new(everymap_providers_radar::RadarTraffic),
            positioner: Box::new(everymap_providers_radar::RadarPositioner),
            isoline: Box::new(everymap_providers_radar::RadarIsoline),
            route_matcher: Box::new(everymap_providers_radar::RadarRouteMatcher::new(
                client.clone(),
            )),
            tour_planner: Box::new(everymap_providers_radar::RadarTourPlanner::new(
                client.clone(),
            )),
            tile_provider: Box::new(everymap_providers_radar::RadarTileProvider),
            attribute_provider: Box::new(everymap_providers_radar::RadarAttributeProvider),
            image_provider: Box::new(everymap_providers_radar::RadarMapImageProvider),
        }
    }

    pub fn geocoder(&self) -> &dyn Geocoder {
        self.geocoder.as_ref()
    }

    pub fn router(&self) -> &dyn Router {
        self.router.as_ref()
    }

    pub fn traffic(&self) -> &dyn TrafficProvider {
        self.traffic.as_ref()
    }

    pub fn positioner(&self) -> &dyn NetworkPositionerTrait {
        self.positioner.as_ref()
    }

    pub fn isoline(&self) -> &dyn IsolineProvider {
        self.isoline.as_ref()
    }

    pub fn route_matcher(&self) -> &dyn RouteMatcher {
        self.route_matcher.as_ref()
    }

    pub fn tour_planner(&self) -> &dyn TourPlanner {
        self.tour_planner.as_ref()
    }

    pub fn tile_provider(&self) -> &dyn TileProvider {
        self.tile_provider.as_ref()
    }

    pub fn attribute_provider(&self) -> &dyn AttributeProvider {
        self.attribute_provider.as_ref()
    }

    pub fn image_provider(&self) -> &dyn MapImageProvider {
        self.image_provider.as_ref()
    }
}
