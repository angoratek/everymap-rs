use everymap_core::auth::header::HeaderAuthProvider;
use everymap_core::auth::{ApiKeyProvider, AuthProvider};
use everymap_core::domains::attributes::AttributeProvider;
use everymap_core::domains::imaging::MapImageProvider;
use everymap_core::domains::isoline::IsolineProvider;
use everymap_core::domains::matching::RouteMatcher;
use everymap_core::domains::positioning::NetworkPositioner as NetworkPositionerTrait;
use everymap_core::domains::routing::Router;
use everymap_core::domains::search::Geocoder;
use everymap_core::domains::tiling::{RasterTileProvider, VectorTileProvider};
use everymap_core::domains::tour::TourPlanner;
use everymap_core::domains::traffic::TrafficProvider;
use everymap_providers_mapquest::{
    MapQuestClient, MapQuestGeocoder, MapQuestProviderConfiguration, MapQuestRasterTileProvider,
    MapQuestRouter, MapQuestTrafficProvider,
};
use everymap_providers_osrm::{OsrmProviderConfiguration, OsrmRouter};
use everymap_providers_valhalla::{ValhallaProviderConfiguration, ValhallaRouter};
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
    raster_tile_provider: Option<Box<dyn RasterTileProvider>>,
    vector_tile_provider: Option<Box<dyn VectorTileProvider>>,
    attribute_provider: Box<dyn AttributeProvider>,
    image_provider: Box<dyn MapImageProvider>,
}

/// The deployment base URL a keyless provider reads as its credential.
fn deployment_base_url_or_default(credential: String, default_base_url: &str) -> String {
    let trimmed_credential = credential.trim();
    if trimmed_credential.is_empty() {
        default_base_url.to_string()
    } else {
        trimmed_credential.to_string()
    }
}

// The domains MapQuest's service does not serve, and every domain OSRM and
// Valhalla do not serve: each answers with the documented unsupported error
// instead of pretending to work. MapQuest serves traffic through its own
// provider, so no traffic stub exists for it.
everymap_core::unsupported_isoline!(MapQuestUnsupportedIsoline, "mapquest");
everymap_core::unsupported_matcher!(MapQuestUnsupportedRouteMatcher, "mapquest");
everymap_core::unsupported_tour!(MapQuestUnsupportedTourPlanner, "mapquest");
everymap_core::unsupported_positioner!(MapQuestUnsupportedPositioner, "mapquest");
everymap_core::unsupported_attributes!(MapQuestUnsupportedAttributeProvider, "mapquest");
everymap_core::unsupported_image!(MapQuestUnsupportedMapImageProvider, "mapquest");

everymap_core::unsupported_geocoder!(OsrmUnsupportedGeocoder, "osrm");
everymap_core::unsupported_traffic!(OsrmUnsupportedTraffic, "osrm");
everymap_core::unsupported_isoline!(OsrmUnsupportedIsoline, "osrm");
everymap_core::unsupported_matcher!(OsrmUnsupportedRouteMatcher, "osrm");
everymap_core::unsupported_tour!(OsrmUnsupportedTour, "osrm");
everymap_core::unsupported_positioner!(OsrmUnsupportedPositioner, "osrm");
everymap_core::unsupported_attributes!(OsrmUnsupportedAttributes, "osrm");
everymap_core::unsupported_image!(OsrmUnsupportedImage, "osrm");

everymap_core::unsupported_geocoder!(ValhallaUnsupportedGeocoder, "valhalla");
everymap_core::unsupported_traffic!(ValhallaUnsupportedTraffic, "valhalla");
everymap_core::unsupported_isoline!(ValhallaUnsupportedIsoline, "valhalla");
everymap_core::unsupported_matcher!(ValhallaUnsupportedRouteMatcher, "valhalla");
everymap_core::unsupported_tour!(ValhallaUnsupportedTour, "valhalla");
everymap_core::unsupported_positioner!(ValhallaUnsupportedPositioner, "valhalla");
everymap_core::unsupported_attributes!(ValhallaUnsupportedAttributes, "valhalla");
everymap_core::unsupported_image!(ValhallaUnsupportedImage, "valhalla");

impl ProviderRegistry {
    /// Builds the provider set for one provider key. An unknown key answers a
    /// validation error naming it, so a caller that skips the CLI's own
    /// provider check still gets a reported failure rather than a panic.
    pub fn try_new(
        provider: &str,
        api_key: String,
        key_param: String,
        verbose: bool,
    ) -> Result<Self, everymap_core::error::EveryMapError> {
        match provider {
            "here" => Ok(Self::new_here(api_key, key_param, verbose)),
            "google" => Ok(Self::new_google(api_key, key_param, verbose)),
            "tomtom" => Ok(Self::new_tomtom(api_key, key_param, verbose)),
            "mapbox" => Ok(Self::new_mapbox(api_key, key_param, verbose)),
            "radar" => Ok(Self::new_radar(api_key, verbose)),
            "mapquest" => Ok(Self::new_mapquest(api_key, verbose)),
            "osrm" => Ok(Self::new_osrm(api_key)),
            "valhalla" => Ok(Self::new_valhalla(api_key)),
            unknown => Err(everymap_core::error::EveryMapError::ValidationError(
                format!("unsupported provider {unknown:?}"),
            )),
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
            raster_tile_provider: None,
            vector_tile_provider: Some(Box::new(
                everymap_providers_here::domain::tiling::HereTileProvider::new(client.clone()),
            )),
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
            raster_tile_provider: None,
            vector_tile_provider: None,
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
            raster_tile_provider: Some(Box::new(
                everymap_providers_tomtom::TomTomTileProvider::new(client.clone()),
            )),
            vector_tile_provider: None,
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
            raster_tile_provider: None,
            vector_tile_provider: Some(Box::new(
                everymap_providers_mapbox::MapBoxTileProvider::new(client.clone()),
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
            raster_tile_provider: None,
            vector_tile_provider: None,
            attribute_provider: Box::new(everymap_providers_radar::RadarAttributeProvider),
            image_provider: Box::new(everymap_providers_radar::RadarMapImageProvider),
        }
    }

    /// MapQuest: geocoding, routing and traffic; the remaining domains have
    /// no implementation on this service and answer with the unsupported
    /// error.
    fn new_mapquest(api_key: String, verbose: bool) -> Self {
        let mut client = MapQuestClient::new(MapQuestProviderConfiguration {
            api_key,
            ..MapQuestProviderConfiguration::default()
        });
        client.set_verbose(verbose);
        let client = Arc::new(client);
        Self {
            geocoder: Box::new(MapQuestGeocoder::new(client.clone())),
            router: Box::new(MapQuestRouter::new(client.clone())),
            traffic: Box::new(MapQuestTrafficProvider::new(client.clone())),
            positioner: Box::new(MapQuestUnsupportedPositioner),
            isoline: Box::new(MapQuestUnsupportedIsoline),
            route_matcher: Box::new(MapQuestUnsupportedRouteMatcher),
            tour_planner: Box::new(MapQuestUnsupportedTourPlanner),
            raster_tile_provider: Some(Box::new(MapQuestRasterTileProvider::new(client.clone()))),
            vector_tile_provider: None,
            attribute_provider: Box::new(MapQuestUnsupportedAttributeProvider),
            image_provider: Box::new(MapQuestUnsupportedMapImageProvider),
        }
    }

    /// OSRM: routing only. The credential, when given, is the deployment's
    /// base URL; an empty one targets the crate's documented local default.
    fn new_osrm(base_url: String) -> Self {
        let router = OsrmRouter::new(OsrmProviderConfiguration {
            base_url: deployment_base_url_or_default(
                base_url,
                everymap_providers_osrm::DEFAULT_OSRM_BASE_URL,
            ),
            ..Default::default()
        });
        Self {
            geocoder: Box::new(OsrmUnsupportedGeocoder),
            router: Box::new(router),
            traffic: Box::new(OsrmUnsupportedTraffic),
            positioner: Box::new(OsrmUnsupportedPositioner),
            isoline: Box::new(OsrmUnsupportedIsoline),
            route_matcher: Box::new(OsrmUnsupportedRouteMatcher),
            tour_planner: Box::new(OsrmUnsupportedTour),
            raster_tile_provider: None,
            vector_tile_provider: None,
            attribute_provider: Box::new(OsrmUnsupportedAttributes),
            image_provider: Box::new(OsrmUnsupportedImage),
        }
    }

    /// Valhalla: routing only, addressed like OSRM.
    fn new_valhalla(base_url: String) -> Self {
        let router = ValhallaRouter::new(ValhallaProviderConfiguration {
            base_url: deployment_base_url_or_default(
                base_url,
                everymap_providers_valhalla::DEFAULT_VALHALLA_BASE_URL,
            ),
            ..Default::default()
        });
        Self {
            geocoder: Box::new(ValhallaUnsupportedGeocoder),
            router: Box::new(router),
            traffic: Box::new(ValhallaUnsupportedTraffic),
            positioner: Box::new(ValhallaUnsupportedPositioner),
            isoline: Box::new(ValhallaUnsupportedIsoline),
            route_matcher: Box::new(ValhallaUnsupportedRouteMatcher),
            tour_planner: Box::new(ValhallaUnsupportedTour),
            raster_tile_provider: None,
            vector_tile_provider: None,
            attribute_provider: Box::new(ValhallaUnsupportedAttributes),
            image_provider: Box::new(ValhallaUnsupportedImage),
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

    pub fn raster_tile_provider(&self) -> Option<&dyn RasterTileProvider> {
        self.raster_tile_provider.as_deref()
    }

    pub fn vector_tile_provider(&self) -> Option<&dyn VectorTileProvider> {
        self.vector_tile_provider.as_deref()
    }

    pub fn attribute_provider(&self) -> &dyn AttributeProvider {
        self.attribute_provider.as_ref()
    }

    pub fn image_provider(&self) -> &dyn MapImageProvider {
        self.image_provider.as_ref()
    }
}
