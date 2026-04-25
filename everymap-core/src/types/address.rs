use serde::{Deserialize, Serialize};

/// Structured address with optional fields for cross-provider compatibility.
///
/// Each field is optional because not all providers return all address components.
/// The `raw` field carries provider-specific data for power users who need more detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    /// Formatted address string (e.g., "1 Main St, Springfield, IL 62701")
    pub label: Option<String>,
    /// Street name
    pub street: Option<String>,
    /// House/building number
    pub house_number: Option<String>,
    /// City/town name
    pub city: Option<String>,
    /// District/neighborhood name
    pub district: Option<String>,
    /// Sub-district name
    pub sub_district: Option<String>,
    /// State/province/region name
    pub state: Option<String>,
    /// State/province code (e.g., "IL")
    pub state_code: Option<String>,
    /// Postal/ZIP code
    pub postal_code: Option<String>,
    /// Country name
    pub country: Option<String>,
    /// ISO 3166-1 alpha-2 country code (e.g., "US")
    pub country_code: Option<String>,
    /// County name
    pub county: Option<String>,
    /// Building name (for places like shopping centers)
    pub building: Option<String>,
    /// Block number (used in some Asian addressing systems)
    pub block: Option<String>,
    /// Unit/apartment number
    pub unit: Option<String>,
}

impl Address {
    /// Create an empty address with no fields set.
    pub fn empty() -> Self {
        Self {
            label: None,
            street: None,
            house_number: None,
            city: None,
            district: None,
            sub_district: None,
            state: None,
            state_code: None,
            postal_code: None,
            country: None,
            country_code: None,
            county: None,
            building: None,
            block: None,
            unit: None,
        }
    }

    /// Create an address from a label string only.
    pub fn from_label(label: String) -> Self {
        Self {
            label: Some(label),
            ..Self::empty()
        }
    }
}

impl Default for Address {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_all_fields_none() {
        let addr = Address::empty();
        assert!(addr.label.is_none());
        assert!(addr.street.is_none());
        assert!(addr.house_number.is_none());
        assert!(addr.city.is_none());
        assert!(addr.district.is_none());
        assert!(addr.sub_district.is_none());
        assert!(addr.state.is_none());
        assert!(addr.state_code.is_none());
        assert!(addr.postal_code.is_none());
        assert!(addr.country.is_none());
        assert!(addr.country_code.is_none());
        assert!(addr.county.is_none());
        assert!(addr.building.is_none());
        assert!(addr.block.is_none());
        assert!(addr.unit.is_none());
    }

    #[test]
    fn test_from_label_only_label_set() {
        let addr = Address::from_label("1 Main St, Springfield, IL 62701".to_string());
        assert_eq!(
            addr.label.as_deref(),
            Some("1 Main St, Springfield, IL 62701")
        );
        // All other fields should be None
        assert!(addr.street.is_none());
        assert!(addr.house_number.is_none());
        assert!(addr.city.is_none());
        assert!(addr.district.is_none());
        assert!(addr.sub_district.is_none());
        assert!(addr.state.is_none());
        assert!(addr.state_code.is_none());
        assert!(addr.postal_code.is_none());
        assert!(addr.country.is_none());
        assert!(addr.country_code.is_none());
        assert!(addr.county.is_none());
        assert!(addr.building.is_none());
        assert!(addr.block.is_none());
        assert!(addr.unit.is_none());
    }

    #[test]
    fn test_default_trait_equals_empty() {
        let default_addr = Address::default();
        let empty_addr = Address::empty();
        assert!(default_addr.label.is_none());
        assert!(empty_addr.label.is_none());
        // Both should have all fields as None
        assert!(default_addr.street.is_none());
        assert!(empty_addr.street.is_none());
    }

    #[test]
    fn test_full_construction_all_fields() {
        let addr = Address {
            label: Some("123 Main St, Springfield, IL 62701".to_string()),
            street: Some("Main St".to_string()),
            house_number: Some("123".to_string()),
            city: Some("Springfield".to_string()),
            district: Some("Downtown".to_string()),
            sub_district: Some("Central".to_string()),
            state: Some("Illinois".to_string()),
            state_code: Some("IL".to_string()),
            postal_code: Some("62701".to_string()),
            country: Some("United States".to_string()),
            country_code: Some("US".to_string()),
            county: Some("Sangamon".to_string()),
            building: Some("City Hall".to_string()),
            block: Some("5".to_string()),
            unit: Some("2A".to_string()),
        };
        assert_eq!(
            addr.label.as_deref(),
            Some("123 Main St, Springfield, IL 62701")
        );
        assert_eq!(addr.street.as_deref(), Some("Main St"));
        assert_eq!(addr.house_number.as_deref(), Some("123"));
        assert_eq!(addr.city.as_deref(), Some("Springfield"));
        assert_eq!(addr.district.as_deref(), Some("Downtown"));
        assert_eq!(addr.sub_district.as_deref(), Some("Central"));
        assert_eq!(addr.state.as_deref(), Some("Illinois"));
        assert_eq!(addr.state_code.as_deref(), Some("IL"));
        assert_eq!(addr.postal_code.as_deref(), Some("62701"));
        assert_eq!(addr.country.as_deref(), Some("United States"));
        assert_eq!(addr.country_code.as_deref(), Some("US"));
        assert_eq!(addr.county.as_deref(), Some("Sangamon"));
        assert_eq!(addr.building.as_deref(), Some("City Hall"));
        assert_eq!(addr.block.as_deref(), Some("5"));
        assert_eq!(addr.unit.as_deref(), Some("2A"));
    }

    #[test]
    fn test_serde_roundtrip_all_fields() {
        let addr = Address {
            label: Some("1 Park Ave".to_string()),
            street: Some("Park Ave".to_string()),
            house_number: Some("1".to_string()),
            city: Some("New York".to_string()),
            district: Some("Manhattan".to_string()),
            sub_district: Some("Midtown".to_string()),
            state: Some("New York".to_string()),
            state_code: Some("NY".to_string()),
            postal_code: Some("10016".to_string()),
            country: Some("United States".to_string()),
            country_code: Some("US".to_string()),
            county: Some("New York County".to_string()),
            building: None,
            block: None,
            unit: None,
        };
        let json = serde_json::to_string(&addr).unwrap();
        let back: Address = serde_json::from_str(&json).unwrap();
        assert_eq!(back.label, addr.label);
        assert_eq!(back.street, addr.street);
        assert_eq!(back.house_number, addr.house_number);
        assert_eq!(back.city, addr.city);
        assert_eq!(back.district, addr.district);
        assert_eq!(back.sub_district, addr.sub_district);
        assert_eq!(back.state, addr.state);
        assert_eq!(back.state_code, addr.state_code);
        assert_eq!(back.postal_code, addr.postal_code);
        assert_eq!(back.country, addr.country);
        assert_eq!(back.country_code, addr.country_code);
        assert_eq!(back.county, addr.county);
        assert_eq!(back.building, addr.building);
        assert_eq!(back.block, addr.block);
        assert_eq!(back.unit, addr.unit);
    }

    #[test]
    fn test_serde_roundtrip_empty_address() {
        let addr = Address::empty();
        let json = serde_json::to_string(&addr).unwrap();
        let back: Address = serde_json::from_str(&json).unwrap();
        assert!(back.label.is_none());
        assert!(back.street.is_none());
        assert!(back.city.is_none());
        assert!(back.country.is_none());
        assert!(back.postal_code.is_none());
    }

    #[test]
    fn test_serde_roundtrip_from_label() {
        let addr = Address::from_label("Somewhere".to_string());
        let json = serde_json::to_string(&addr).unwrap();
        let back: Address = serde_json::from_str(&json).unwrap();
        assert_eq!(back.label.as_deref(), Some("Somewhere"));
        assert!(back.street.is_none());
    }

    #[test]
    fn test_clone_address() {
        let addr = Address::from_label("Test".to_string());
        let cloned = addr.clone();
        assert_eq!(addr.label, cloned.label);
    }

    #[test]
    fn test_debug_address() {
        let addr = Address::from_label("Test".to_string());
        let debug_str = format!("{:?}", addr);
        assert!(debug_str.contains("Test"));
    }
}
