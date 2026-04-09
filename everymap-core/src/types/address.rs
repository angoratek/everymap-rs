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