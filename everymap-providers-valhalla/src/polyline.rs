//! Decoder for Valhalla's encoded polyline shapes.
//!
//! Valhalla serializes a leg's geometry as a Google encoded polyline with six
//! decimal places of precision (polyline6). The encoding stores each point as
//! a pair of signed deltas — latitude first, longitude second — where every
//! delta is a sequence of five-bit chunks, low chunk first, with bit `0x20`
//! marking that another chunk follows. The low bit of the first chunk is the
//! sign, and the remaining bits hold the magnitude.
//!
//! Decoding never touches the network and performs no guessing: a truncated
//! value, a byte outside the encoding alphabet, or a coordinate outside the
//! WGS 84 ranges is a decoding failure.

use crate::error_mapping::invalid_polyline;
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;

/// Number of decimal places a Valhalla shape carries.
const POLYLINE6_PRECISION_SCALE: f64 = 1_000_000.0;

/// Number of bits carried by one encoded chunk.
const POLYLINE_CHUNK_BITS: u32 = 5;

/// Mask selecting the data bits of one chunk.
const POLYLINE_CHUNK_DATA_MASK: u32 = 0x1f;

/// Bit marking that another chunk follows.
const POLYLINE_CONTINUATION_BIT: u32 = 0x20;

/// Lowest byte of the encoding alphabet (`'?'` encodes zero).
const POLYLINE_ALPHABET_MINIMUM: u8 = 63;

/// Highest byte of the encoding alphabet (`'~'`).
const POLYLINE_ALPHABET_MAXIMUM: u8 = 126;

/// Largest shift a chunk may occupy before the accumulator overflows a
/// signed 64-bit value; a longer sequence is malformed.
const POLYLINE_MAXIMUM_SHIFT: u32 = 60;

/// Decodes a precision-6 encoded polyline into WGS 84 coordinates.
///
/// # Parameters
/// - `encoded_shape` — the leg shape exactly as Valhalla returned it; an
///   empty string encodes an empty shape.
///
/// # Returns
/// The shape points in traversal order, with latitude and longitude in
/// decimal degrees.
///
/// # Errors
/// Returns [`everymap_core::error::EveryMapError::ProviderError`] with the
/// `INVALID_POLYLINE` code when the text ends in the middle of a value,
/// contains a byte outside the encoding alphabet, or decodes a latitude or
/// longitude outside its WGS 84 range.
pub(crate) fn decode_polyline6(encoded_shape: &str) -> EveryMapResult<Vec<Coordinate>> {
    let mut coordinates = Vec::new();
    let mut latitude_value: i64 = 0;
    let mut longitude_value: i64 = 0;
    let mut encoded_bytes = encoded_shape.bytes();
    while let Some(latitude_delta) = decode_delta(&mut encoded_bytes)? {
        let longitude_delta = decode_delta(&mut encoded_bytes)?.ok_or_else(|| {
            invalid_polyline("Valhalla shape ends after a latitude without its longitude")
        })?;
        latitude_value = latitude_value.checked_add(latitude_delta).ok_or_else(|| {
            invalid_polyline("Valhalla shape latitude accumulates outside the representable range")
        })?;
        longitude_value = longitude_value
            .checked_add(longitude_delta)
            .ok_or_else(|| {
                invalid_polyline(
                    "Valhalla shape longitude accumulates outside the representable range",
                )
            })?;
        let latitude = latitude_value as f64 / POLYLINE6_PRECISION_SCALE;
        let longitude = longitude_value as f64 / POLYLINE6_PRECISION_SCALE;
        let coordinate = Coordinate::new(latitude, longitude).map_err(|error| {
            invalid_polyline(format!(
                "Valhalla shape carries an invalid coordinate: {error}"
            ))
        })?;
        coordinates.push(coordinate);
    }
    Ok(coordinates)
}

/// Decodes the next signed delta from the remaining bytes.
///
/// # Parameters
/// - `encoded_bytes` — the not-yet-consumed bytes of the shape.
///
/// # Returns
/// `Some(delta)` for a complete value, or `None` when the input is exhausted
/// at a value boundary.
///
/// # Errors
/// Returns [`everymap_core::error::EveryMapError::ProviderError`] with the
/// `INVALID_POLYLINE` code when the input ends between chunks, contains a
/// byte outside the encoding alphabet, or keeps a continuation bit set beyond
/// the accumulator's capacity.
fn decode_delta(encoded_bytes: &mut std::str::Bytes<'_>) -> EveryMapResult<Option<i64>> {
    let mut accumulated_value: u32 = 0;
    let mut shift: u32 = 0;
    loop {
        let Some(encoded_byte) = encoded_bytes.next() else {
            if shift == 0 {
                return Ok(None);
            }
            return Err(invalid_polyline(
                "Valhalla shape ends in the middle of an encoded value",
            ));
        };
        if !(POLYLINE_ALPHABET_MINIMUM..=POLYLINE_ALPHABET_MAXIMUM).contains(&encoded_byte) {
            return Err(invalid_polyline(format!(
                "Valhalla shape contains byte {encoded_byte} outside the polyline alphabet"
            )));
        }
        let chunk = u32::from(encoded_byte - POLYLINE_ALPHABET_MINIMUM);
        accumulated_value |= (chunk & POLYLINE_CHUNK_DATA_MASK) << shift;
        if chunk & POLYLINE_CONTINUATION_BIT == 0 {
            break;
        }
        shift += POLYLINE_CHUNK_BITS;
        if shift > POLYLINE_MAXIMUM_SHIFT {
            return Err(invalid_polyline(
                "Valhalla shape carries an overlong encoded value",
            ));
        }
    }
    let magnitude = i64::from(accumulated_value >> 1);
    let delta = if accumulated_value & 1 == 1 {
        !magnitude
    } else {
        magnitude
    };
    Ok(Some(delta))
}

#[cfg(test)]
mod tests {
    use super::decode_polyline6;
    use everymap_core::error::EveryMapError;
    use everymap_core::types::Coordinate;

    /// Tolerance for a decoded coordinate against its authored value; one
    /// encoding unit is 1e-6 degrees.
    const DECODED_COORDINATE_TOLERANCE_DEGREES: f64 = 1e-9;

    fn assert_coordinate(coordinate: &Coordinate, expected_latitude: f64, expected_longitude: f64) {
        assert!(
            (coordinate.lat - expected_latitude).abs() < DECODED_COORDINATE_TOLERANCE_DEGREES,
            "latitude {} differs from {expected_latitude}",
            coordinate.lat
        );
        assert!(
            (coordinate.lng - expected_longitude).abs() < DECODED_COORDINATE_TOLERANCE_DEGREES,
            "longitude {} differs from {expected_longitude}",
            coordinate.lng
        );
    }

    #[test]
    fn decodes_a_known_precision_six_value() {
        // The published three-point example, re-encoded at precision 6.
        let coordinates =
            decode_polyline6(r"_izlhA~rlgdF_{geC~ywl@_kwzCn`{nI").expect("valid shape");
        assert_eq!(coordinates.len(), 3);
        assert_coordinate(&coordinates[0], 38.5, -120.2);
        assert_coordinate(&coordinates[1], 40.7, -120.95);
        assert_coordinate(&coordinates[2], 43.252, -126.453);
    }

    #[test]
    fn decodes_a_single_point_shape() {
        let coordinates = decode_polyline6(r"_izlhA~rlgdF").expect("valid shape");
        assert_eq!(coordinates.len(), 1);
        assert_coordinate(&coordinates[0], 38.5, -120.2);
    }

    #[test]
    fn decodes_an_empty_shape_to_no_coordinates() {
        assert!(decode_polyline6("")
            .expect("an empty shape is valid")
            .is_empty());
    }

    #[test]
    fn decodes_negative_longitude_deltas() {
        let coordinates = decode_polyline6(
            r"szoqhA~dkvfFf@nd@z@fw@gc@vcAwj@vcAg^~iAgEv|AgEv|A?v|AoKnoBgEfiBkCzzA",
        )
        .expect("valid shape");
        assert_eq!(coordinates.len(), 12);
        assert_coordinate(&coordinates[0], 38.57657, -121.4936);
        assert_coordinate(&coordinates[11], 38.57887, -121.50797);
    }

    #[test]
    fn rejects_a_shape_that_ends_mid_value() {
        match decode_polyline6(r"_izlhA") {
            Err(EveryMapError::ProviderError { code, .. }) => {
                assert_eq!(code, "INVALID_POLYLINE");
            }
            other => panic!("expected an invalid-polyline error, got {other:?}"),
        }
    }

    #[test]
    fn rejects_a_latitude_without_its_longitude() {
        assert!(decode_polyline6(r"_").is_err());
    }

    #[test]
    fn rejects_bytes_outside_the_encoding_alphabet() {
        assert!(decode_polyline6(" ").is_err());
    }

    #[test]
    fn rejects_coordinates_outside_wgs_84_ranges() {
        assert!(decode_polyline6(r"_izlhA~rlgdF_ethtB?").is_err());
    }
}
