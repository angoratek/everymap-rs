use serde::{Deserialize, Serialize};

/// Options for Google Geolocation API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GooglePositioningOptions {
    /// Whether to return the IP-based location as a fallback.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consider_ip: Option<bool>,
    /// WiFi access point observations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wifi_access_points: Option<Vec<GoogleWifiAccessPoint>>,
    /// Cell tower observations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_towers: Option<Vec<GoogleCellTower>>,
}

/// WiFi access point observation for Google Geolocation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleWifiAccessPoint {
    /// MAC address (BSSID) of the access point.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,
    /// Received signal strength in dBm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_strength: Option<i32>,
    /// Age of the measurement in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
    /// Channel number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u32>,
    /// Signal-to-noise ratio in dB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_to_noise_ratio: Option<i32>,
}

/// Cell tower observation for Google Geolocation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleCellTower {
    /// Mobile Country Code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_id: Option<u64>,
    /// Location Area Code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_area_code: Option<u32>,
    /// Mobile Country Code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile_country_code: Option<u32>,
    /// Mobile Network Code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile_network_code: Option<u32>,
    /// Age of the measurement in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
    /// Received signal strength in dBm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_strength: Option<i32>,
    /// Timing advance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing_advance: Option<u32>,
}

/// Response from Google Geolocation API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoogleGeolocationResponse {
    /// The estimated latitude in degrees.
    #[serde(default)]
    pub location: GoogleGeolocationLocation,
    /// Accuracy of the position estimate in meters.
    #[serde(default)]
    pub accuracy: f64,
}

/// Location from Google Geolocation API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoogleGeolocationLocation {
    /// Latitude in degrees.
    #[serde(default)]
    pub lat: f64,
    /// Longitude in degrees.
    #[serde(default)]
    pub lng: f64,
}
