use serde::{Deserialize, Serialize};

// ============================================================================
// Request types — HERE Positioning API v2
// ============================================================================

/// Options for HERE Positioning API v2.
/// Supports WLAN, cell, and Bluetooth observations for position estimation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HerePositioningOptions {
    /// WLAN (Wi-Fi) access point observations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wlan: Option<Vec<WlanAccessPoint>>,
    /// Cell tower observations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell: Option<Vec<CellTower>>,
    /// Bluetooth beacon observations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bluetooth: Option<Vec<BluetoothBeacon>>,
    /// Fallback behavior when no position can be determined.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Fallback>,
}

/// WLAN access point observation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WlanAccessPoint {
    /// MAC address (BSSID) of the access point.
    pub mac: String,
    /// Received signal strength in dBm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_strength: Option<i32>,
    /// Age of the measurement in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
    /// Channel number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u32>,
    /// SSID of the access point.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssid: Option<String>,
    /// Signal-to-noise ratio in dB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_to_noise_ratio: Option<i32>,
}

/// Cell tower observation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CellTower {
    /// Mobile Country Code.
    pub mcc: u32,
    /// Mobile Network Code.
    pub mnc: u32,
    /// Location Area Code.
    pub lac: u32,
    /// Cell ID.
    pub cid: u32,
    /// Timing advance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ta: Option<u32>,
    /// Received signal strength in dBm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_strength: Option<i32>,
    /// Age of the measurement in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
    /// Radio access technology type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radio_type: Option<RadioType>,
    /// Physical Cell ID (for LTE/NR).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pci: Option<u32>,
    /// Tracking Area Code (for LTE/NR, alternative to LAC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tac: Option<u32>,
    /// NR ARFCN (New Radio Absolute Radio Frequency Channel Number).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrarfcn: Option<u32>,
    /// E-ARFCN (E-UTRA Absolute Radio Frequency Channel Number).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earfcn: Option<u32>,
}

/// Radio access technology type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RadioType {
    #[serde(rename = "gsm")]
    Gsm,
    #[serde(rename = "wcdma")]
    Wcdma,
    #[serde(rename = "lte")]
    Lte,
    #[serde(rename = "nr")]
    Nr,
    #[serde(rename = "cdma")]
    Cdma,
}

/// Bluetooth beacon observation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BluetoothBeacon {
    /// MAC address of the beacon.
    pub mac: String,
    /// Received signal strength in dBm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_strength: Option<i32>,
    /// Age of the measurement in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
    /// UUID (for iBeacon).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Major identifier (for iBeacon).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub major: Option<u32>,
    /// Minor identifier (for iBeacon).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor: Option<u32>,
    /// Measured power at 1 meter in dBm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measured_power: Option<i32>,
}

/// Fallback behavior when no position can be determined.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Fallback {
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "wlan")]
    Wlan,
    #[serde(rename = "cell")]
    Cell,
}

// ============================================================================
// Response types — HERE Positioning API v2
// ============================================================================

/// Full positioning response from the HERE Positioning API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositioningResponse {
    /// The estimated location.
    pub location: PositionLocation,
    /// Altitude information (if available).
    #[serde(default)]
    pub altitude: Option<PositionAltitude>,
}

/// Location estimate from positioning.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositionLocation {
    /// Latitude in degrees.
    pub lat: f64,
    /// Longitude in degrees.
    pub lng: f64,
    /// Accuracy of the position in meters.
    #[serde(default)]
    pub accuracy: Option<f64>,
}

/// Altitude information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositionAltitude {
    /// Altitude value in meters above sea level.
    #[serde(default)]
    pub value: Option<f64>,
    /// Accuracy of the altitude in meters.
    #[serde(default)]
    pub accuracy: Option<f64>,
}