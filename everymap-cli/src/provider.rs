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
use everymap_core::provider::ProviderDescriptor;
use everymap_providers_mapquest::{
    MapQuestClient, MapQuestGeocoder, MapQuestProviderConfiguration, MapQuestRasterTileProvider,
    MapQuestRouter, MapQuestTrafficProvider,
};
use everymap_providers_osrm::{OsrmProviderConfiguration, OsrmRouter};
use everymap_providers_photon::{PhotonClient, PhotonGeocoder, PhotonProviderConfiguration};
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

/// The providers this binary can construct, in help order.
///
/// Each entry is the provider crate's own descriptor, so the keys, credential
/// names and service roots are stated once per crate rather than in the CLI.
pub static SUPPORTED_PROVIDER_DESCRIPTORS: [ProviderDescriptor; 11] = [
    everymap_providers_here::PROVIDER_DESCRIPTOR,
    everymap_providers_google::PROVIDER_DESCRIPTOR,
    everymap_providers_tomtom::PROVIDER_DESCRIPTOR,
    everymap_providers_mapbox::PROVIDER_DESCRIPTOR,
    everymap_providers_radar::PROVIDER_DESCRIPTOR,
    everymap_providers_mapquest::PROVIDER_DESCRIPTOR,
    everymap_providers_photon::PROVIDER_DESCRIPTOR,
    everymap_providers_osrm::PROVIDER_DESCRIPTOR,
    everymap_providers_valhalla::PROVIDER_DESCRIPTOR,
    everymap_providers_url_template_raster::PROVIDER_DESCRIPTOR,
    everymap_providers_url_template_vector::PROVIDER_DESCRIPTOR,
];

/// The descriptor for `provider`, when this binary can construct it.
pub fn supported_provider_descriptor(provider: &str) -> Option<ProviderDescriptor> {
    SUPPORTED_PROVIDER_DESCRIPTORS
        .iter()
        .copied()
        .find(|descriptor| descriptor.key == provider)
}

/// The keys of the providers this binary can construct, in help order.
pub fn supported_provider_keys() -> Vec<&'static str> {
    SUPPORTED_PROVIDER_DESCRIPTORS
        .iter()
        .map(|descriptor| descriptor.key)
        .collect()
}

/// The constructor to run for a supported provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderKind {
    Here,
    Google,
    TomTom,
    MapBox,
    Radar,
    MapQuest,
    Photon,
    Osrm,
    Valhalla,
    UrlTemplateRaster,
    UrlTemplateVector,
}

/// Resolve a provider key to its constructor without repeating the key.
fn provider_kind(provider: &str) -> Option<ProviderKind> {
    if provider == everymap_providers_here::PROVIDER_DESCRIPTOR.key {
        Some(ProviderKind::Here)
    } else if provider == everymap_providers_google::PROVIDER_DESCRIPTOR.key {
        Some(ProviderKind::Google)
    } else if provider == everymap_providers_tomtom::PROVIDER_DESCRIPTOR.key {
        Some(ProviderKind::TomTom)
    } else if provider == everymap_providers_mapbox::PROVIDER_DESCRIPTOR.key {
        Some(ProviderKind::MapBox)
    } else if provider == everymap_providers_radar::PROVIDER_DESCRIPTOR.key {
        Some(ProviderKind::Radar)
    } else if provider == everymap_providers_mapquest::PROVIDER_DESCRIPTOR.key {
        Some(ProviderKind::MapQuest)
    } else if provider == everymap_providers_photon::PROVIDER_DESCRIPTOR.key {
        Some(ProviderKind::Photon)
    } else if provider == everymap_providers_osrm::PROVIDER_DESCRIPTOR.key {
        Some(ProviderKind::Osrm)
    } else if provider == everymap_providers_valhalla::PROVIDER_DESCRIPTOR.key {
        Some(ProviderKind::Valhalla)
    } else if provider == everymap_providers_url_template_raster::PROVIDER_DESCRIPTOR.key {
        Some(ProviderKind::UrlTemplateRaster)
    } else if provider == everymap_providers_url_template_vector::PROVIDER_DESCRIPTOR.key {
        Some(ProviderKind::UrlTemplateVector)
    } else {
        None
    }
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
everymap_core::unsupported_isoline!(
    MapQuestUnsupportedIsoline,
    everymap_providers_mapquest::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_matcher!(
    MapQuestUnsupportedRouteMatcher,
    everymap_providers_mapquest::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_tour!(
    MapQuestUnsupportedTourPlanner,
    everymap_providers_mapquest::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_positioner!(
    MapQuestUnsupportedPositioner,
    everymap_providers_mapquest::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_attributes!(
    MapQuestUnsupportedAttributeProvider,
    everymap_providers_mapquest::PROVIDER_DESCRIPTOR.key
);

everymap_core::unsupported_geocoder!(
    OsrmUnsupportedGeocoder,
    everymap_providers_osrm::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_traffic!(
    OsrmUnsupportedTraffic,
    everymap_providers_osrm::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_isoline!(
    OsrmUnsupportedIsoline,
    everymap_providers_osrm::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_matcher!(
    OsrmUnsupportedRouteMatcher,
    everymap_providers_osrm::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_tour!(
    OsrmUnsupportedTour,
    everymap_providers_osrm::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_positioner!(
    OsrmUnsupportedPositioner,
    everymap_providers_osrm::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_attributes!(
    OsrmUnsupportedAttributes,
    everymap_providers_osrm::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_image!(
    OsrmUnsupportedImage,
    everymap_providers_osrm::PROVIDER_DESCRIPTOR.key
);

everymap_core::unsupported_geocoder!(
    ValhallaUnsupportedGeocoder,
    everymap_providers_valhalla::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_traffic!(
    ValhallaUnsupportedTraffic,
    everymap_providers_valhalla::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_isoline!(
    ValhallaUnsupportedIsoline,
    everymap_providers_valhalla::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_matcher!(
    ValhallaUnsupportedRouteMatcher,
    everymap_providers_valhalla::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_tour!(
    ValhallaUnsupportedTour,
    everymap_providers_valhalla::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_positioner!(
    ValhallaUnsupportedPositioner,
    everymap_providers_valhalla::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_attributes!(
    ValhallaUnsupportedAttributes,
    everymap_providers_valhalla::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_image!(
    ValhallaUnsupportedImage,
    everymap_providers_valhalla::PROVIDER_DESCRIPTOR.key
);

// Photon serves geocoding, place search and type-ahead; the CLI's registry
// exposes only the geocoder, so every other contract answers unsupported.
everymap_core::unsupported_router!(
    PhotonUnsupportedRouter,
    everymap_providers_photon::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_traffic!(
    PhotonUnsupportedTraffic,
    everymap_providers_photon::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_isoline!(
    PhotonUnsupportedIsoline,
    everymap_providers_photon::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_matcher!(
    PhotonUnsupportedRouteMatcher,
    everymap_providers_photon::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_tour!(
    PhotonUnsupportedTour,
    everymap_providers_photon::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_positioner!(
    PhotonUnsupportedPositioner,
    everymap_providers_photon::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_attributes!(
    PhotonUnsupportedAttributes,
    everymap_providers_photon::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_image!(
    PhotonUnsupportedImage,
    everymap_providers_photon::PROVIDER_DESCRIPTOR.key
);

// The URL-template crates serve one tile contract each. The tile command
// builds their provider from the source declaration, so the registry carries
// only the unsupported stubs for every other contract.
everymap_core::unsupported_geocoder!(
    UrlTemplateRasterUnsupportedGeocoder,
    everymap_providers_url_template_raster::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_router!(
    UrlTemplateRasterUnsupportedRouter,
    everymap_providers_url_template_raster::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_traffic!(
    UrlTemplateRasterUnsupportedTraffic,
    everymap_providers_url_template_raster::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_isoline!(
    UrlTemplateRasterUnsupportedIsoline,
    everymap_providers_url_template_raster::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_matcher!(
    UrlTemplateRasterUnsupportedRouteMatcher,
    everymap_providers_url_template_raster::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_tour!(
    UrlTemplateRasterUnsupportedTour,
    everymap_providers_url_template_raster::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_positioner!(
    UrlTemplateRasterUnsupportedPositioner,
    everymap_providers_url_template_raster::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_attributes!(
    UrlTemplateRasterUnsupportedAttributes,
    everymap_providers_url_template_raster::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_image!(
    UrlTemplateRasterUnsupportedImage,
    everymap_providers_url_template_raster::PROVIDER_DESCRIPTOR.key
);

everymap_core::unsupported_geocoder!(
    UrlTemplateVectorUnsupportedGeocoder,
    everymap_providers_url_template_vector::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_router!(
    UrlTemplateVectorUnsupportedRouter,
    everymap_providers_url_template_vector::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_traffic!(
    UrlTemplateVectorUnsupportedTraffic,
    everymap_providers_url_template_vector::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_isoline!(
    UrlTemplateVectorUnsupportedIsoline,
    everymap_providers_url_template_vector::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_matcher!(
    UrlTemplateVectorUnsupportedRouteMatcher,
    everymap_providers_url_template_vector::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_tour!(
    UrlTemplateVectorUnsupportedTour,
    everymap_providers_url_template_vector::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_positioner!(
    UrlTemplateVectorUnsupportedPositioner,
    everymap_providers_url_template_vector::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_attributes!(
    UrlTemplateVectorUnsupportedAttributes,
    everymap_providers_url_template_vector::PROVIDER_DESCRIPTOR.key
);
everymap_core::unsupported_image!(
    UrlTemplateVectorUnsupportedImage,
    everymap_providers_url_template_vector::PROVIDER_DESCRIPTOR.key
);

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
        match provider_kind(provider) {
            Some(ProviderKind::Here) => Ok(Self::new_here(api_key, key_param, verbose)),
            Some(ProviderKind::Google) => Ok(Self::new_google(api_key, key_param, verbose)),
            Some(ProviderKind::TomTom) => Ok(Self::new_tomtom(api_key, key_param, verbose)),
            Some(ProviderKind::MapBox) => Ok(Self::new_mapbox(api_key, key_param, verbose)),
            Some(ProviderKind::Radar) => Ok(Self::new_radar(api_key, verbose)),
            Some(ProviderKind::MapQuest) => Ok(Self::new_mapquest(api_key, verbose)),
            Some(ProviderKind::Photon) => Ok(Self::new_photon(api_key)),
            Some(ProviderKind::Osrm) => Ok(Self::new_osrm(api_key)),
            Some(ProviderKind::Valhalla) => Ok(Self::new_valhalla(api_key)),
            Some(ProviderKind::UrlTemplateRaster) => Ok(Self::new_url_template_raster()),
            Some(ProviderKind::UrlTemplateVector) => Ok(Self::new_url_template_vector()),
            None => Err(everymap_core::error::EveryMapError::ValidationError(
                format!("unsupported provider {provider:?}"),
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
            image_provider: Box::new(everymap_providers_mapquest::MapQuestMapImageProvider::new(
                client,
            )),
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

    /// Photon: geocoding and reverse geocoding over a deployment root; an
    /// empty credential reaches the public instance.
    fn new_photon(base_url: String) -> Self {
        let client = Arc::new(PhotonClient::new(PhotonProviderConfiguration {
            base_url: deployment_base_url_or_default(
                base_url,
                everymap_providers_photon::DEFAULT_PHOTON_BASE_URL,
            ),
            ..Default::default()
        }));
        Self {
            geocoder: Box::new(PhotonGeocoder::new(client)),
            router: Box::new(PhotonUnsupportedRouter),
            traffic: Box::new(PhotonUnsupportedTraffic),
            positioner: Box::new(PhotonUnsupportedPositioner),
            isoline: Box::new(PhotonUnsupportedIsoline),
            route_matcher: Box::new(PhotonUnsupportedRouteMatcher),
            tour_planner: Box::new(PhotonUnsupportedTour),
            raster_tile_provider: None,
            vector_tile_provider: None,
            attribute_provider: Box::new(PhotonUnsupportedAttributes),
            image_provider: Box::new(PhotonUnsupportedImage),
        }
    }

    /// A URL-template raster registry: only the tile command serves, and it
    /// builds the provider from the source declaration it is given.
    fn new_url_template_raster() -> Self {
        Self {
            geocoder: Box::new(UrlTemplateRasterUnsupportedGeocoder),
            router: Box::new(UrlTemplateRasterUnsupportedRouter),
            traffic: Box::new(UrlTemplateRasterUnsupportedTraffic),
            positioner: Box::new(UrlTemplateRasterUnsupportedPositioner),
            isoline: Box::new(UrlTemplateRasterUnsupportedIsoline),
            route_matcher: Box::new(UrlTemplateRasterUnsupportedRouteMatcher),
            tour_planner: Box::new(UrlTemplateRasterUnsupportedTour),
            raster_tile_provider: None,
            vector_tile_provider: None,
            attribute_provider: Box::new(UrlTemplateRasterUnsupportedAttributes),
            image_provider: Box::new(UrlTemplateRasterUnsupportedImage),
        }
    }

    /// A URL-template vector registry, addressed like the raster one.
    fn new_url_template_vector() -> Self {
        Self {
            geocoder: Box::new(UrlTemplateVectorUnsupportedGeocoder),
            router: Box::new(UrlTemplateVectorUnsupportedRouter),
            traffic: Box::new(UrlTemplateVectorUnsupportedTraffic),
            positioner: Box::new(UrlTemplateVectorUnsupportedPositioner),
            isoline: Box::new(UrlTemplateVectorUnsupportedIsoline),
            route_matcher: Box::new(UrlTemplateVectorUnsupportedRouteMatcher),
            tour_planner: Box::new(UrlTemplateVectorUnsupportedTour),
            raster_tile_provider: None,
            vector_tile_provider: None,
            attribute_provider: Box::new(UrlTemplateVectorUnsupportedAttributes),
            image_provider: Box::new(UrlTemplateVectorUnsupportedImage),
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

#[cfg(test)]
mod tests {
    use super::{
        provider_kind, supported_provider_descriptor, supported_provider_keys, ProviderRegistry,
        SUPPORTED_PROVIDER_DESCRIPTORS,
    };

    #[test]
    fn every_supported_descriptor_key_constructs_a_registry() {
        for descriptor in SUPPORTED_PROVIDER_DESCRIPTORS {
            let credential = if descriptor.credential.requires_credential() {
                "test-key".to_string()
            } else {
                String::new()
            };
            let key_param = descriptor
                .credential
                .query_parameter_name()
                .unwrap_or_default()
                .to_string();
            assert!(
                ProviderRegistry::try_new(descriptor.key, credential, key_param, false).is_ok(),
                "{} did not construct a registry",
                descriptor.key
            );
            assert_eq!(
                supported_provider_descriptor(descriptor.key).map(|found| found.key),
                Some(descriptor.key)
            );
            assert!(provider_kind(descriptor.key).is_some());
        }
    }

    #[test]
    fn descriptor_keys_are_unique_and_non_empty() {
        let mut keys = supported_provider_keys();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), SUPPORTED_PROVIDER_DESCRIPTORS.len());
        assert!(keys.iter().all(|key| !key.is_empty()));
    }

    #[test]
    fn an_unknown_provider_has_no_descriptor_or_kind() {
        assert!(supported_provider_descriptor("not-a-provider").is_none());
        assert!(provider_kind("not-a-provider").is_none());
        assert!(
            ProviderRegistry::try_new("not-a-provider", String::new(), String::new(), false)
                .is_err()
        );
    }
}
