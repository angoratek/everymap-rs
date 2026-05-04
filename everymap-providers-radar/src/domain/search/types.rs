use crate::domain::types::RadarMeta;
use serde::{Deserialize, Serialize};

/// Response from Radar forward/reverse geocoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarGeocodeResponse {
    pub meta: RadarMeta,
    #[serde(default)]
    pub addresses: Vec<RadarAddress>,
}

/// A Radar address result with rich structured fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarAddress {
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
    #[serde(default)]
    pub geometry: Option<serde_json::Value>,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub country_code: String,
    #[serde(default)]
    pub county: String,
    #[serde(default)]
    pub confidence: Option<String>,
    #[serde(default)]
    pub borough: Option<String>,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub number: String,
    #[serde(default)]
    pub neighborhood: Option<String>,
    #[serde(default)]
    pub postal_code: String,
    #[serde(default)]
    pub state_code: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub street: String,
    #[serde(default)]
    pub layer: Option<String>,
    #[serde(default)]
    pub formatted_address: String,
    #[serde(default)]
    pub address_label: Option<String>,
    #[serde(default)]
    pub time_zone: Option<RadarTimeZone>,
    #[serde(default)]
    pub distance: Option<f64>,
    #[serde(default)]
    pub place_label: Option<String>,
}

/// Time zone information in a Radar address.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarTimeZone {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub current_time: String,
    #[serde(default)]
    pub utc_offset: f64,
    #[serde(default)]
    pub dst_offset: f64,
}

/// Response from Radar IP geocoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarIpGeocodeResponse {
    pub meta: RadarMeta,
    pub address: Option<RadarAddress>,
    #[serde(default)]
    pub proxy: bool,
    #[serde(default)]
    pub ip: String,
}

/// Response from Radar autocomplete.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarAutocompleteResponse {
    pub meta: RadarMeta,
    #[serde(default)]
    pub addresses: Vec<RadarAddress>,
}

/// Options for Radar autocomplete.
#[derive(Debug, Clone, Default)]
pub struct RadarAutocompleteOptions {
    pub layers: Option<String>,
    pub limit: Option<u32>,
    pub country_code: Option<String>,
}

/// Response from Radar address validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarAddressValidationResponse {
    pub meta: RadarMeta,
    pub address: Option<RadarValidatedAddress>,
    pub result: Option<RadarValidationResult>,
}

/// Validated address from Radar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarValidatedAddress {
    #[serde(default)]
    pub address_label: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub number: String,
    #[serde(default)]
    pub street: String,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub state_code: String,
    #[serde(default)]
    pub postal_code: String,
    #[serde(default)]
    pub plus4: Option<String>,
    #[serde(default)]
    pub county: String,
    #[serde(default)]
    pub country_code: String,
    #[serde(default)]
    pub formatted_address: String,
    pub metadata: Option<RadarAddressMetadata>,
}

/// Address metadata from validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarAddressMetadata {
    #[serde(default)]
    pub record_type: Option<String>,
    #[serde(default)]
    pub property_type: Option<String>,
}

/// Result of address validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarValidationResult {
    #[serde(default)]
    pub verification_status: String,
}

/// Options for Radar address validation.
#[derive(Debug, Clone, Default)]
pub struct RadarAddressValidationOptions {
    pub city: String,
    pub state_code: String,
    pub postal_code: String,
    pub country_code: String,
    pub number: Option<String>,
    pub street: Option<String>,
    pub unit: Option<String>,
    pub address_label: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radar_geocode_response_deserialize() {
        let json = r#"{
            "meta": {"code": 200},
            "addresses": [{
                "latitude": 40.7128,
                "longitude": -74.006,
                "country": "US",
                "countryCode": "US",
                "county": "New York",
                "confidence": "exact",
                "city": "New York",
                "number": "350",
                "neighborhood": "Midtown",
                "postalCode": "10018",
                "stateCode": "NY",
                "state": "New York",
                "street": "5th Avenue",
                "formattedAddress": "350 5th Avenue, New York, NY 10018",
                "distance": 0.5
            }]
        }"#;
        let response: RadarGeocodeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.meta.code, 200);
        assert_eq!(response.addresses.len(), 1);
        assert_eq!(response.addresses[0].city, "New York");
        assert_eq!(response.addresses[0].confidence.as_deref(), Some("exact"));
    }

    #[test]
    fn test_radar_geocode_response_empty_addresses() {
        let json = r#"{"meta": {"code": 200}}"#;
        let response: RadarGeocodeResponse = serde_json::from_str(json).unwrap();
        assert!(response.addresses.is_empty());
    }

    #[test]
    fn test_radar_ip_geocode_response_deserialize() {
        let json = r#"{
            "meta": {"code": 200},
            "address": {"latitude": 40.7128, "longitude": -74.006, "country": "US", "countryCode": "US", "county": "New York", "city": "New York", "number": "", "postalCode": "10018", "stateCode": "NY", "state": "New York", "street": "", "formattedAddress": "New York, NY"},
            "proxy": false,
            "ip": "1.2.3.4"
        }"#;
        let response: RadarIpGeocodeResponse = serde_json::from_str(json).unwrap();
        assert!(response.address.is_some());
        assert!(!response.proxy);
        assert_eq!(response.ip, "1.2.3.4");
    }

    #[test]
    fn test_radar_autocomplete_options_default() {
        let options = RadarAutocompleteOptions::default();
        assert!(options.layers.is_none());
        assert!(options.limit.is_none());
        assert!(options.country_code.is_none());
    }

    #[test]
    fn test_radar_address_validation_options() {
        let options = RadarAddressValidationOptions {
            city: "New York".to_string(),
            state_code: "NY".to_string(),
            postal_code: "10018".to_string(),
            country_code: "US".to_string(),
            number: Some("350".to_string()),
            street: Some("5th Avenue".to_string()),
            unit: None,
            address_label: None,
        };
        assert_eq!(options.city, "New York");
        assert_eq!(options.number.as_deref(), Some("350"));
    }

    #[test]
    fn test_radar_time_zone_deserialize() {
        let json = r#"{
            "id": "America/New_York",
            "name": "Eastern Time",
            "code": "EST",
            "currentTime": "2024-06-01T12:00:00",
            "utcOffset": -5.0,
            "dstOffset": 1.0
        }"#;
        let tz: RadarTimeZone = serde_json::from_str(json).unwrap();
        assert_eq!(tz.id, "America/New_York");
        assert!((tz.utc_offset - (-5.0)).abs() < f64::EPSILON);
    }
}
