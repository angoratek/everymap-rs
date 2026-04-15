use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Configuration loaded from `~/.everymap/config.toml`.
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub providers: Providers,
}

/// Provider-specific configuration sections.
#[derive(Debug, Default, Deserialize)]
pub struct Providers {
    #[serde(default)]
    pub here: ProviderConfig,
    #[serde(default)]
    pub google: ProviderConfig,
    #[serde(default)]
    pub tomtom: ProviderConfig,
    #[serde(default)]
    pub mapbox: ProviderConfig,
    #[serde(default)]
    pub radar: ProviderConfig,
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
            Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to parse config file {:?}: {}", path, e);
                Self::default()
            }),
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
    pub fn resolve_api_key(&self, cli_key: &Option<String>, provider: &str, env_var: &str) -> Option<String> {
        // 1. CLI flag takes precedence
        if cli_key.is_some() {
            return cli_key.clone();
        }
        // 2. Config file
        let config_key = match provider {
            "here" => self.providers.here.api_key.clone(),
            "google" => self.providers.google.api_key.clone(),
            "tomtom" => self.providers.tomtom.api_key.clone(),
            "mapbox" => self.providers.mapbox.api_key.clone(),
            "radar" => self.providers.radar.api_key.clone(),
            _ => None,
        };
        if config_key.is_some() {
            return config_key;
        }
        // 3. Environment variable
        std::env::var(env_var).ok()
    }
}