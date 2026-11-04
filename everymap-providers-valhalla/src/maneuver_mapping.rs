//! Mapping of Valhalla maneuver `type` integers onto the unified
//! [`everymap_core::domains::routing::ManeuverType`].
//!
//! Valhalla numbers each maneuver action; the table below covers every type
//! the routing engine emits. The table is deliberately total: an unknown
//! integer maps to [`ManeuverType::Continue`] rather than failing a route
//! that is otherwise usable. Transit and vertical-transport maneuvers
//! (elevators, stairs, escalators) also map to `Continue`, because the
//! contract has no dedicated classifications for them and this provider does
//! not request transit costing.

use everymap_core::domains::routing::{ManeuverType, TurnDirection};

/// Valhalla's `kNone` action.
const MANEUVER_NONE: i32 = 0;
/// Route start.
const MANEUVER_START: i32 = 1;
/// Route start with a right-side departure.
const MANEUVER_START_RIGHT: i32 = 2;
/// Route start with a left-side departure.
const MANEUVER_START_LEFT: i32 = 3;
/// Route destination.
const MANEUVER_DESTINATION: i32 = 4;
/// Route destination on the right side of the street.
const MANEUVER_DESTINATION_RIGHT: i32 = 5;
/// Route destination on the left side of the street.
const MANEUVER_DESTINATION_LEFT: i32 = 6;
/// The road changes name or class without a turn.
const MANEUVER_BECOMES: i32 = 7;
/// Continue along the current road.
const MANEUVER_CONTINUE: i32 = 8;
/// Slight right turn.
const MANEUVER_SLIGHT_RIGHT: i32 = 9;
/// Right turn.
const MANEUVER_RIGHT: i32 = 10;
/// Sharp right turn.
const MANEUVER_SHARP_RIGHT: i32 = 11;
/// Right-hand U-turn.
const MANEUVER_U_TURN_RIGHT: i32 = 12;
/// Left-hand U-turn.
const MANEUVER_U_TURN_LEFT: i32 = 13;
/// Sharp left turn.
const MANEUVER_SHARP_LEFT: i32 = 14;
/// Left turn.
const MANEUVER_LEFT: i32 = 15;
/// Slight left turn.
const MANEUVER_SLIGHT_LEFT: i32 = 16;
/// Straight on-ramp.
const MANEUVER_RAMP_STRAIGHT: i32 = 17;
/// Right on-ramp.
const MANEUVER_RAMP_RIGHT: i32 = 18;
/// Left on-ramp.
const MANEUVER_RAMP_LEFT: i32 = 19;
/// Right off-ramp.
const MANEUVER_EXIT_RIGHT: i32 = 20;
/// Left off-ramp.
const MANEUVER_EXIT_LEFT: i32 = 21;
/// Keep straight at a fork.
const MANEUVER_STAY_STRAIGHT: i32 = 22;
/// Keep right at a fork.
const MANEUVER_STAY_RIGHT: i32 = 23;
/// Keep left at a fork.
const MANEUVER_STAY_LEFT: i32 = 24;
/// Merge onto another road.
const MANEUVER_MERGE: i32 = 25;
/// Enter a roundabout; carries the exit count.
const MANEUVER_ROUNDABOUT_ENTER: i32 = 26;
/// Leave a roundabout.
const MANEUVER_ROUNDABOUT_EXIT: i32 = 27;
/// Board a ferry.
const MANEUVER_FERRY_ENTER: i32 = 28;
/// Leave a ferry.
const MANEUVER_FERRY_EXIT: i32 = 29;
/// Merge right onto another road.
const MANEUVER_MERGE_RIGHT: i32 = 38;
/// Merge left onto another road.
const MANEUVER_MERGE_LEFT: i32 = 39;

/// Exit index used when Valhalla reports a roundabout without a count. The
/// contract numbers exits from one, so the first exit is the honest default.
const DEFAULT_ROUNDABOUT_EXIT_INDEX: u8 = 1;

/// Maps one Valhalla maneuver integer to the contract's classification.
///
/// # Parameters
/// - `valhalla_maneuver_type` — the `type` field of a Valhalla maneuver.
/// - `roundabout_exit_index` — the `exit_number` or `roundabout_exit_count`
///   field, used only by the roundabout types.
///
/// # Returns
/// The matching [`ManeuverType`]; unknown integers map to
/// [`ManeuverType::Continue`].
pub(crate) fn map_valhalla_maneuver_type(
    valhalla_maneuver_type: i32,
    roundabout_exit_index: Option<u8>,
) -> ManeuverType {
    match valhalla_maneuver_type {
        MANEUVER_START | MANEUVER_START_RIGHT | MANEUVER_START_LEFT => ManeuverType::Departure,
        MANEUVER_DESTINATION | MANEUVER_DESTINATION_RIGHT | MANEUVER_DESTINATION_LEFT => {
            ManeuverType::Arrival
        }
        MANEUVER_SLIGHT_RIGHT => ManeuverType::Turn(TurnDirection::SlightRight),
        MANEUVER_RIGHT => ManeuverType::Turn(TurnDirection::Right),
        MANEUVER_SHARP_RIGHT => ManeuverType::Turn(TurnDirection::SharpRight),
        MANEUVER_U_TURN_RIGHT | MANEUVER_U_TURN_LEFT => ManeuverType::Turn(TurnDirection::UTurn),
        MANEUVER_SHARP_LEFT => ManeuverType::Turn(TurnDirection::SharpLeft),
        MANEUVER_LEFT => ManeuverType::Turn(TurnDirection::Left),
        MANEUVER_SLIGHT_LEFT => ManeuverType::Turn(TurnDirection::SlightLeft),
        MANEUVER_RAMP_STRAIGHT | MANEUVER_RAMP_RIGHT | MANEUVER_RAMP_LEFT => ManeuverType::OnRamp,
        MANEUVER_EXIT_RIGHT | MANEUVER_EXIT_LEFT => ManeuverType::OffRamp,
        MANEUVER_STAY_STRAIGHT | MANEUVER_STAY_RIGHT | MANEUVER_STAY_LEFT => ManeuverType::Fork,
        MANEUVER_MERGE | MANEUVER_MERGE_RIGHT | MANEUVER_MERGE_LEFT => ManeuverType::Merge,
        MANEUVER_ROUNDABOUT_ENTER => {
            ManeuverType::Roundabout(roundabout_exit_index.unwrap_or(DEFAULT_ROUNDABOUT_EXIT_INDEX))
        }
        MANEUVER_FERRY_ENTER => ManeuverType::Ferry,
        // `kNone`, road-name changes, continuing, leaving a roundabout or a
        // ferry, transit, and vertical transport have no distinct contract
        // classification; an unknown integer degrades the same way rather
        // than dropping the step.
        MANEUVER_NONE
        | MANEUVER_BECOMES
        | MANEUVER_CONTINUE
        | MANEUVER_ROUNDABOUT_EXIT
        | MANEUVER_FERRY_EXIT => ManeuverType::Continue,
        _ => ManeuverType::Continue,
    }
}

#[cfg(test)]
mod tests {
    use super::map_valhalla_maneuver_type;
    use everymap_core::domains::routing::{ManeuverType, TurnDirection};

    #[test]
    fn start_and_destination_types_fence_the_route() {
        assert_eq!(map_valhalla_maneuver_type(1, None), ManeuverType::Departure);
        assert_eq!(map_valhalla_maneuver_type(2, None), ManeuverType::Departure);
        assert_eq!(map_valhalla_maneuver_type(3, None), ManeuverType::Departure);
        assert_eq!(map_valhalla_maneuver_type(4, None), ManeuverType::Arrival);
        assert_eq!(map_valhalla_maneuver_type(5, None), ManeuverType::Arrival);
        assert_eq!(map_valhalla_maneuver_type(6, None), ManeuverType::Arrival);
    }

    #[test]
    fn turn_types_carry_their_direction() {
        assert_eq!(
            map_valhalla_maneuver_type(9, None),
            ManeuverType::Turn(TurnDirection::SlightRight)
        );
        assert_eq!(
            map_valhalla_maneuver_type(10, None),
            ManeuverType::Turn(TurnDirection::Right)
        );
        assert_eq!(
            map_valhalla_maneuver_type(11, None),
            ManeuverType::Turn(TurnDirection::SharpRight)
        );
        assert_eq!(
            map_valhalla_maneuver_type(12, None),
            ManeuverType::Turn(TurnDirection::UTurn)
        );
        assert_eq!(
            map_valhalla_maneuver_type(13, None),
            ManeuverType::Turn(TurnDirection::UTurn)
        );
        assert_eq!(
            map_valhalla_maneuver_type(14, None),
            ManeuverType::Turn(TurnDirection::SharpLeft)
        );
        assert_eq!(
            map_valhalla_maneuver_type(15, None),
            ManeuverType::Turn(TurnDirection::Left)
        );
        assert_eq!(
            map_valhalla_maneuver_type(16, None),
            ManeuverType::Turn(TurnDirection::SlightLeft)
        );
    }

    #[test]
    fn ramps_forks_and_merges_classify_by_role() {
        assert_eq!(map_valhalla_maneuver_type(17, None), ManeuverType::OnRamp);
        assert_eq!(map_valhalla_maneuver_type(18, None), ManeuverType::OnRamp);
        assert_eq!(map_valhalla_maneuver_type(19, None), ManeuverType::OnRamp);
        assert_eq!(map_valhalla_maneuver_type(20, None), ManeuverType::OffRamp);
        assert_eq!(map_valhalla_maneuver_type(21, None), ManeuverType::OffRamp);
        assert_eq!(map_valhalla_maneuver_type(22, None), ManeuverType::Fork);
        assert_eq!(map_valhalla_maneuver_type(23, None), ManeuverType::Fork);
        assert_eq!(map_valhalla_maneuver_type(24, None), ManeuverType::Fork);
        assert_eq!(map_valhalla_maneuver_type(25, None), ManeuverType::Merge);
        assert_eq!(map_valhalla_maneuver_type(38, None), ManeuverType::Merge);
        assert_eq!(map_valhalla_maneuver_type(39, None), ManeuverType::Merge);
    }

    #[test]
    fn roundabouts_carry_the_reported_exit_index() {
        assert_eq!(
            map_valhalla_maneuver_type(26, Some(3)),
            ManeuverType::Roundabout(3)
        );
        assert_eq!(
            map_valhalla_maneuver_type(26, None),
            ManeuverType::Roundabout(1)
        );
        assert_eq!(
            map_valhalla_maneuver_type(27, Some(3)),
            ManeuverType::Continue
        );
    }

    #[test]
    fn ferry_entry_is_a_ferry_and_unknown_types_continue() {
        assert_eq!(map_valhalla_maneuver_type(28, None), ManeuverType::Ferry);
        assert_eq!(map_valhalla_maneuver_type(29, None), ManeuverType::Continue);
        assert_eq!(map_valhalla_maneuver_type(0, None), ManeuverType::Continue);
        assert_eq!(map_valhalla_maneuver_type(7, None), ManeuverType::Continue);
        assert_eq!(map_valhalla_maneuver_type(8, None), ManeuverType::Continue);
        assert_eq!(
            map_valhalla_maneuver_type(999, None),
            ManeuverType::Continue
        );
    }
}
