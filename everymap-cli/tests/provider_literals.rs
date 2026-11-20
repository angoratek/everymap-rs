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

#[test]
fn the_cli_source_repeats_no_provider_key() {
    let source = joined_source();
    let descriptors = [
        everymap_providers_here::PROVIDER_DESCRIPTOR,
        everymap_providers_google::PROVIDER_DESCRIPTOR,
        everymap_providers_tomtom::PROVIDER_DESCRIPTOR,
        everymap_providers_mapbox::PROVIDER_DESCRIPTOR,
        everymap_providers_radar::PROVIDER_DESCRIPTOR,
        everymap_providers_mapquest::PROVIDER_DESCRIPTOR,
        everymap_providers_osrm::PROVIDER_DESCRIPTOR,
        everymap_providers_valhalla::PROVIDER_DESCRIPTOR,
    ];

    for descriptor in descriptors {
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
