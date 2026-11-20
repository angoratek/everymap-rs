//! The CLI keeps no provider table of its own.
//!
//! Every provider key, credential name and service URL comes from the provider
//! crate's `PROVIDER_DESCRIPTOR`, so this guard fails when a literal returns to
//! the CLI's source.

use std::fs;
use std::path::Path;

/// The CLI source files that describe providers or resolve credentials.
const CLI_SOURCE_FILES: [&str; 3] = ["src/main.rs", "src/provider.rs", "src/config.rs"];

/// Credential names a descriptor must supply instead of the CLI.
const CREDENTIAL_LITERALS: [&str; 4] = [
    "\"apiKey\"",
    "\"access_token\"",
    "\"Authorization\"",
    "\"key\"",
];

/// URL prefixes a descriptor or the provider crate must supply.
const URL_LITERALS: [&str; 2] = ["\"https://", "\"http://"];

fn read_source(relative_path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
}

fn joined_source() -> String {
    CLI_SOURCE_FILES.map(read_source).join("\n")
}

/// The source with `feature = "name"` conditions flattened, so a cargo
/// feature name is never mistaken for a repeated provider key.
fn source_without_feature_names() -> String {
    joined_source().replace("feature = \"", "feature = ")
}

/// The descriptors of the provider crates this build carries.
// The pushes are cfg-gated, so they cannot collapse into a `vec!`.
#[allow(clippy::vec_init_then_push, unused_mut)]
fn built_provider_descriptors() -> Vec<everymap_core::provider::ProviderDescriptor> {
    let mut descriptors = Vec::new();
    #[cfg(feature = "here")]
    descriptors.push(everymap_providers_here::PROVIDER_DESCRIPTOR);
    #[cfg(feature = "google")]
    descriptors.push(everymap_providers_google::PROVIDER_DESCRIPTOR);
    #[cfg(feature = "tomtom")]
    descriptors.push(everymap_providers_tomtom::PROVIDER_DESCRIPTOR);
    #[cfg(feature = "mapbox")]
    descriptors.push(everymap_providers_mapbox::PROVIDER_DESCRIPTOR);
    #[cfg(feature = "radar")]
    descriptors.push(everymap_providers_radar::PROVIDER_DESCRIPTOR);
    #[cfg(feature = "mapquest")]
    descriptors.push(everymap_providers_mapquest::PROVIDER_DESCRIPTOR);
    #[cfg(feature = "osrm")]
    descriptors.push(everymap_providers_osrm::PROVIDER_DESCRIPTOR);
    #[cfg(feature = "valhalla")]
    descriptors.push(everymap_providers_valhalla::PROVIDER_DESCRIPTOR);
    descriptors
}

#[test]
fn the_cli_source_repeats_no_provider_key() {
    let source = source_without_feature_names();

    for descriptor in built_provider_descriptors() {
        let quoted_key = format!("\"{}\"", descriptor.key);
        assert!(
            !source.contains(&quoted_key),
            "the CLI source repeats the provider key {quoted_key}"
        );
    }
}

#[test]
fn the_cli_source_repeats_no_credential_name() {
    let source = joined_source();
    for literal in CREDENTIAL_LITERALS {
        assert!(
            !source.contains(literal),
            "the CLI source repeats the credential literal {literal}"
        );
    }
}

#[test]
fn the_cli_source_hard_codes_no_service_url() {
    let source = joined_source();
    for literal in URL_LITERALS {
        assert!(
            !source.contains(literal),
            "the CLI source hard-codes a service URL {literal}"
        );
    }
}
