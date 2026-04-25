use serde::{Deserialize, Serialize};

/// Response envelope for all Radar API responses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RadarMeta {
    #[serde(default)]
    pub code: u16,
}

/// A metric with value and formatted text.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RadarMetric {
    #[serde(default)]
    pub value: f64,
    #[serde(default)]
    pub text: String,
}

/// A geographic location.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RadarLocation {
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
}

/// Geometry container (typically a polyline).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RadarGeometry {
    #[serde(default)]
    pub polyline: Option<String>,
}

/// Radar travel mode enum.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RadarTravelMode {
    Car,
    Truck,
    Foot,
    Bike,
}

/// Radar avoid type enum.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RadarAvoid {
    Tolls,
    Highways,
    Ferries,
    BorderCrossings,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radar_meta_default() {
        let meta = RadarMeta::default();
        assert_eq!(meta.code, 0);
    }

    #[test]
    fn test_radar_meta_serde() {
        let json = r#"{"code": 200}"#;
        let meta: RadarMeta = serde_json::from_str(json).unwrap();
        assert_eq!(meta.code, 200);
    }

    #[test]
    fn test_radar_location_default() {
        let loc = RadarLocation::default();
        assert!((loc.latitude - 0.0).abs() < f64::EPSILON);
        assert!((loc.longitude - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_radar_metric_default() {
        let m = RadarMetric::default();
        assert!((m.value - 0.0).abs() < f64::EPSILON);
        assert!(m.text.is_empty());
    }

    #[test]
    fn test_radar_travel_mode_serde() {
        let modes = [
            RadarTravelMode::Car,
            RadarTravelMode::Truck,
            RadarTravelMode::Foot,
            RadarTravelMode::Bike,
        ];
        for mode in &modes {
            let json = serde_json::to_string(mode).unwrap();
            let back: RadarTravelMode = serde_json::from_str(&json).unwrap();
            assert_eq!(*mode, back);
        }
    }

    #[test]
    fn test_radar_avoid_serde() {
        let avoids = [
            RadarAvoid::Tolls,
            RadarAvoid::Highways,
            RadarAvoid::Ferries,
            RadarAvoid::BorderCrossings,
        ];
        for avoid in &avoids {
            let json = serde_json::to_string(avoid).unwrap();
            let back: RadarAvoid = serde_json::from_str(&json).unwrap();
            assert_eq!(*avoid, back);
        }
    }

    #[test]
    fn test_radar_geometry_default() {
        let g = RadarGeometry::default();
        assert!(g.polyline.is_none());
    }

    #[test]
    fn test_radar_location_from_json() {
        let json = r#"{"latitude": 40.7128, "longitude": -74.006}"#;
        let loc: RadarLocation = serde_json::from_str(json).unwrap();
        assert!((loc.latitude - 40.7128).abs() < f64::EPSILON);
        assert!((loc.longitude - (-74.006)).abs() < f64::EPSILON);
    }
}
