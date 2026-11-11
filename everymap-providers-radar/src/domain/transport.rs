//! The one mapping from the SDK's transport modes to the Radar service's
//! `mode` values.
//!
//! Routing, matching and tour planning all send the same parameter, and each
//! used to carry its own table: routing and matching mapped a scooter to
//! `foot` while tour planning printed the enum's variant name (`"pedestrian"`,
//! `"scooter"`), which Radar does not accept. The tables live here once so a
//! new mode cannot diverge between the three endpoints.

use everymap_core::domains::routing::TransportMode;

/// The Radar `mode` value for a contract transport mode.
///
/// Radar documents `car`, `truck`, `bike` and `foot`. A scooter shares the
/// foot profile (Radar has no scooter mode), and a mode Radar does not serve
/// falls back to `car` with a warning rather than sending a value the service
/// rejects.
pub fn radar_transport_mode(transport_mode: &TransportMode) -> &'static str {
    match transport_mode {
        TransportMode::Car => "car",
        TransportMode::Truck => "truck",
        TransportMode::Pedestrian | TransportMode::Scooter => "foot",
        TransportMode::Bicycle => "bike",
        unsupported => {
            log::warn!(
                "Radar does not support transport mode {:?}, falling back to car",
                unsupported
            );
            "car"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::radar_transport_mode;
    use everymap_core::domains::routing::TransportMode;

    #[test]
    fn every_supported_mode_maps_to_a_documented_radar_value() {
        assert_eq!(radar_transport_mode(&TransportMode::Car), "car");
        assert_eq!(radar_transport_mode(&TransportMode::Truck), "truck");
        assert_eq!(radar_transport_mode(&TransportMode::Pedestrian), "foot");
        assert_eq!(radar_transport_mode(&TransportMode::Scooter), "foot");
        assert_eq!(radar_transport_mode(&TransportMode::Bicycle), "bike");
    }

    #[test]
    fn an_unserved_mode_falls_back_to_car() {
        assert_eq!(radar_transport_mode(&TransportMode::Bus), "car");
        assert_eq!(radar_transport_mode(&TransportMode::Taxi), "car");
    }
}
