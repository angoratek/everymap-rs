//! Decoder for the encoded-polyline shapes Google and Mapbox return.
//!
//! The format stores each point as a pair of signed deltas — latitude first,
//! longitude second — where every delta is a sequence of five-bit chunks, low
//! chunk first, with bit `0x20` marking that another chunk follows. The low
//! bit of the first chunk is the sign, and the remaining bits hold the
//! magnitude. The coordinate scale is a power of ten chosen by the provider:
//! precision 5 is the classic Google/Mapbox encoding, precision 6 is the
//! `polyline6` form Mapbox and Valhalla use.
//!
//! Decoding never touches the network and performs no guessing: a truncated
//! value, a byte outside the encoding alphabet, an overlong value or a
//! coordinate outside the WGS 84 ranges is a decoding failure.

use super::Coordinate;

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

/// Largest shift a chunk may occupy. The accumulator is a 64-bit value, so
/// eleven five-bit chunks (55 bits) already exceed any coordinate delta a
/// provider can produce; a longer sequence is malformed.
const POLYLINE_MAXIMUM_SHIFT: u32 = 55;

/// Base of the coordinate scale (`1e5`, `1e6`).
const POLYLINE_PRECISION_BASE: f64 = 10.0;

/// Decodes an encoded polyline into WGS 84 coordinates.
///
/// # Parameters
/// - `encoded_shape` — the shape exactly as the provider returned it; an
///   empty string encodes an empty shape.
/// - `precision` — decimal places of the encoded deltas: five for the
///   classic encoding, six for `polyline6`.
///
/// # Returns
/// The shape points in traversal order, with latitude and longitude in
/// decimal degrees, or the failure detail for the caller to map onto its own
/// provider error.
pub fn decode_encoded_polyline(
    encoded_shape: &str,
    precision: u8,
) -> Result<Vec<Coordinate>, String> {
    let scale = POLYLINE_PRECISION_BASE.powi(i32::from(precision));
    let mut coordinates = Vec::new();
    let mut latitude_value: i64 = 0;
    let mut longitude_value: i64 = 0;
    let mut encoded_bytes = encoded_shape.bytes();
    while let Some(latitude_delta) = decode_delta(&mut encoded_bytes)? {
        let longitude_delta = decode_delta(&mut encoded_bytes)?
            .ok_or_else(|| "the shape ends after a latitude without its longitude".to_string())?;
        latitude_value = latitude_value.checked_add(latitude_delta).ok_or_else(|| {
            "the shape latitude accumulates outside the representable range".to_string()
        })?;
        longitude_value = longitude_value
            .checked_add(longitude_delta)
            .ok_or_else(|| {
                "the shape longitude accumulates outside the representable range".to_string()
            })?;
        let latitude = latitude_value as f64 / scale;
        let longitude = longitude_value as f64 / scale;
        let coordinate = Coordinate::new(latitude, longitude)
            .map_err(|error| format!("the shape carries an invalid coordinate: {error}"))?;
        coordinates.push(coordinate);
    }
    Ok(coordinates)
}

/// Decodes the next signed delta from the remaining bytes.
///
/// # Returns
/// `Some(delta)` for a complete value, or `None` when the input is exhausted
/// at a value boundary, or the failure detail for a malformed value.
fn decode_delta(encoded_bytes: &mut std::str::Bytes<'_>) -> Result<Option<i64>, String> {
    let mut accumulated_value: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        if shift > POLYLINE_MAXIMUM_SHIFT {
            return Err("the shape carries an overlong encoded value".to_string());
        }
        let Some(encoded_byte) = encoded_bytes.next() else {
            if shift == 0 {
                return Ok(None);
            }
            return Err("the shape ends in the middle of an encoded value".to_string());
        };
        if !(POLYLINE_ALPHABET_MINIMUM..=POLYLINE_ALPHABET_MAXIMUM).contains(&encoded_byte) {
            return Err(format!(
                "the shape contains byte {encoded_byte} outside the polyline alphabet"
            ));
        }
        let chunk = u32::from(encoded_byte - POLYLINE_ALPHABET_MINIMUM);
        accumulated_value |= u64::from(chunk & POLYLINE_CHUNK_DATA_MASK) << shift;
        if chunk & POLYLINE_CONTINUATION_BIT == 0 {
            break;
        }
        shift += POLYLINE_CHUNK_BITS;
    }
    let magnitude = i64::try_from(accumulated_value >> 1)
        .map_err(|_| "the shape carries an overlong encoded value".to_string())?;
    let delta = if accumulated_value & 1 == 1 {
        !magnitude
    } else {
        magnitude
    };
    Ok(Some(delta))
}

#[cfg(test)]
mod tests {
    use super::decode_encoded_polyline;

    /// A polyline6 vector for the Berlin and Paris endpoints.
    const POLYLINE6_TEST_VECTOR: &str = "w{idcBgiooXfzj~Evd}_T";

    /// The Google documentation's polyline5 vector: three points across
    /// California and Nevada.
    const POLYLINE5_TEST_VECTOR: &str = "_p~iF~ps|U_ulLnnqC_mqNvxq`@";

    /// A one-point shape whose latitude is 100 degrees, outside WGS 84.
    const OUT_OF_RANGE_TEST_VECTOR: &str = "_gjaR?";

    #[test]
    fn decodes_a_polyline6_vector() {
        let coordinates =
            decode_encoded_polyline(POLYLINE6_TEST_VECTOR, 6).expect("the vector decodes");
        assert_eq!(coordinates.len(), 2);
        assert!((coordinates[0].lat - 52.5163).abs() < 1e-6);
        assert!((coordinates[0].lng - 13.3777).abs() < 1e-6);
        assert!((coordinates[1].lat - 48.8566).abs() < 1e-6);
        assert!((coordinates[1].lng - 2.3522).abs() < 1e-6);
    }

    #[test]
    fn decodes_a_polyline5_vector() {
        let coordinates =
            decode_encoded_polyline(POLYLINE5_TEST_VECTOR, 5).expect("the vector decodes");
        assert_eq!(coordinates.len(), 3);
        assert!((coordinates[0].lat - 38.5).abs() < 1e-5);
        assert!((coordinates[0].lng + 120.2).abs() < 1e-5);
        assert!((coordinates[2].lat - 43.252).abs() < 1e-5);
        assert!((coordinates[2].lng + 126.453).abs() < 1e-5);
    }

    #[test]
    fn an_empty_shape_decodes_to_no_points() {
        let coordinates = decode_encoded_polyline("", 5).expect("an empty shape is valid");
        assert!(coordinates.is_empty());
    }

    #[test]
    fn a_truncated_value_is_refused() {
        assert!(decode_encoded_polyline("_", 5).is_err());
    }

    #[test]
    fn a_byte_outside_the_alphabet_is_refused() {
        assert!(decode_encoded_polyline("A", 5).is_err());
    }

    #[test]
    fn an_overlong_value_is_refused() {
        let overlong_value = "_".repeat(20);
        assert!(decode_encoded_polyline(&overlong_value, 5).is_err());
    }

    #[test]
    fn a_coordinate_outside_the_wgs84_range_is_refused() {
        assert!(
            decode_encoded_polyline(OUT_OF_RANGE_TEST_VECTOR, 5).is_err(),
            "a 100-degree latitude is not a WGS 84 coordinate"
        );
    }
}
