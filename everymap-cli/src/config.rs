use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// Configuration loaded from `~/.everymap/config.toml`.
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    /// Provider sections keyed by the provider's descriptor key, as in
    /// `[providers.here]`.
    #[serde(default)]
    pub providers: BTreeMap<String, ProviderConfig>,
}

/// Configuration for a single provider.
#[derive(Debug, Default, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
}

impl Config {
    /// Load config from the default location (`~/.everymap/config.toml`).
    /// Returns a default config if the file doesn't exist.
    pub fn load() -> Self {
        let path = Self::config_path();
        Self::load_from(&path)
    }

    /// Load config from a specific path. Returns default if file doesn't exist.
    pub fn load_from(path: &PathBuf) -> Self {
        match fs::read_to_string(path) {
            Ok(contents) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = fs::metadata(path) {
                        let mode = metadata.permissions().mode();
                        if mode & 0o006 != 0 {
                            eprintln!(
                                "Warning: config file {:?} is group/world-readable (mode={:o}). \
                                 Consider running: chmod 600 {:?}",
                                path,
                                mode & 0o777,
                                path
                            );
                        }
                    }
                }
                toml::from_str(&contents).unwrap_or_else(|e| {
                    eprintln!("Warning: Failed to parse config file {:?}: {}", path, e);
                    Self::default()
                })
            }
            Err(_) => Self::default(),
        }
    }

    /// Get the config file path: `~/.everymap/config.toml`
    fn config_path() -> PathBuf {
        directories::BaseDirs::new()
            .map(|bd| bd.home_dir().join(".everymap").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from(".everymap/config.toml"))
    }

    /// Get the API key for a provider, checking CLI flag, then config, then env var.
    pub fn resolve_api_key(
        &self,
        cli_key: &Option<String>,
        provider: &str,
        env_var: &str,
    ) -> Option<String> {
        // 1. CLI flag takes precedence
        if cli_key.is_some() {
            return cli_key.clone();
        }
        // 2. Config file. A keyless engine stores its deployment base URL or
        // source declaration in the same `api_key` slot.
        let config_key = self
            .providers
            .get(provider)
            .and_then(|section| section.api_key.clone());
        if config_key.is_some() {
            return config_key;
        }
        // 3. Environment variable
        std::env::var(env_var).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    const UNSET_ENVIRONMENT_VARIABLE: &str = "EVERYMAP_TEST_UNSET_CONFIG_VARIABLE";

    fn config_with_one_section(key: &str) -> Config {
        toml::from_str(&format!("[providers.{key}]\napi_key = \"configured\"\n"))
            .expect("the test configuration parses")
    }

    const CONFIGURED_PROVIDER: &str = "configured-provider";

    #[test]
    fn a_provider_section_is_read_by_its_key() {
        let config = config_with_one_section(CONFIGURED_PROVIDER);
        assert_eq!(
            config.resolve_api_key(&None, CONFIGURED_PROVIDER, UNSET_ENVIRONMENT_VARIABLE),
            Some("configured".to_string())
        );
    }

    #[test]
    fn an_unknown_provider_has_no_section() {
        let config = config_with_one_section(CONFIGURED_PROVIDER);
        assert_eq!(
            config.resolve_api_key(&None, "another-provider", UNSET_ENVIRONMENT_VARIABLE),
            None
        );
    }
}
