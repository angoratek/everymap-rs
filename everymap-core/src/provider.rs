//! Cross-domain description of a provider crate.
//!
//! A [`ProviderDescriptor`] is the one place a host reads what a provider
//! needs to be reached and what it serves: its key, display name, credential
//! scheme, service root, attribution, terms and contracts. Provider crates
//! export one as `PROVIDER_DESCRIPTOR`; a host such as `everymap-cli` builds
//! its provider table from those constants instead of repeating literals.

/// How a provider authenticates a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialScheme {
    /// An API key sent as a query parameter with this name.
    QueryParameter(&'static str),
    /// An API key sent in this HTTP header.
    Header(&'static str),
    /// No API key; the credential slot carries a deployment base URL.
    DeploymentUrl,
    /// No API key; the credential slot carries a source declaration.
    Source,
}

impl CredentialScheme {
    /// The query parameter a host sets the key in, when the scheme has one.
    pub fn query_parameter_name(self) -> Option<&'static str> {
        match self {
            Self::QueryParameter(name) => Some(name),
            Self::Header(_) | Self::DeploymentUrl | Self::Source => None,
        }
    }

    /// Whether a request without a credential is refused by the service.
    pub fn requires_credential(self) -> bool {
        matches!(self, Self::QueryParameter(_) | Self::Header(_))
    }
}

/// A contract from [`crate::domains`] a provider serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServedContract {
    /// Forward and reverse geocoding.
    Geocoding,
    /// Settled place search.
    PlaceSearch,
    /// Type-ahead suggestions.
    Autocomplete,
    /// Routing.
    Routing,
    /// Distance matrix.
    DistanceMatrix,
    /// Traffic flow and incidents.
    Traffic,
    /// Raster tiles.
    RasterTiles,
    /// Vector tiles.
    VectorTiles,
    /// Elevation.
    Elevation,
    /// Isolines.
    Isoline,
    /// Map matching.
    Matching,
    /// Tour planning.
    Tour,
    /// Network positioning.
    Positioning,
    /// Attribute layers.
    Attributes,
    /// Static map images.
    Imaging,
    /// Offline routing.
    OfflineRouting,
}

/// How a provider's requests are addressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceRoot {
    /// One service or deployment root serves every domain.
    Single(&'static str),
    /// Each domain client sets its own base URL.
    PerDomain,
    /// The source declaration supplies the URL template.
    Source,
}

/// What a provider crate serves and how a host reaches it.
///
/// Provider crates export one as `PROVIDER_DESCRIPTOR`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderDescriptor {
    /// The key a host selects the provider by, such as `here`.
    pub key: &'static str,
    /// The provider's name as a person reads it.
    pub display_name: &'static str,
    /// How the provider authenticates a request.
    pub credential: CredentialScheme,
    /// How requests are addressed.
    pub root: ServiceRoot,
    /// Attribution the provider's terms require, when any.
    pub attribution: Option<&'static str>,
    /// Terms text or URL the provider's policy references, when any.
    pub terms: Option<&'static str>,
    /// The contracts the provider serves.
    pub contracts: &'static [ServedContract],
}

impl ProviderDescriptor {
    /// Whether this provider serves `contract`.
    pub fn serves(self, contract: ServedContract) -> bool {
        self.contracts.contains(&contract)
    }
}

#[cfg(test)]
mod tests {
    use super::{CredentialScheme, ProviderDescriptor, ServedContract, ServiceRoot};

    const TEST_DESCRIPTOR: ProviderDescriptor = ProviderDescriptor {
        key: "test-provider",
        display_name: "Test Provider",
        credential: CredentialScheme::QueryParameter("apiKey"),
        root: ServiceRoot::Single("https://api.example.com"),
        attribution: None,
        terms: None,
        contracts: &[ServedContract::Geocoding, ServedContract::Routing],
    };

    #[test]
    fn query_parameter_name_is_reported_only_for_a_query_credential() {
        assert_eq!(
            CredentialScheme::QueryParameter("key").query_parameter_name(),
            Some("key")
        );
        assert_eq!(
            CredentialScheme::Header("Authorization").query_parameter_name(),
            None
        );
        assert_eq!(CredentialScheme::DeploymentUrl.query_parameter_name(), None);
        assert_eq!(CredentialScheme::Source.query_parameter_name(), None);
    }

    #[test]
    fn only_keyed_schemes_require_a_credential() {
        assert!(CredentialScheme::QueryParameter("key").requires_credential());
        assert!(CredentialScheme::Header("Authorization").requires_credential());
        assert!(!CredentialScheme::DeploymentUrl.requires_credential());
        assert!(!CredentialScheme::Source.requires_credential());
    }

    #[test]
    fn serves_reads_the_contract_list() {
        assert!(TEST_DESCRIPTOR.serves(ServedContract::Geocoding));
        assert!(!TEST_DESCRIPTOR.serves(ServedContract::Traffic));
    }
}
