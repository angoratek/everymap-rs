//! Request encodings shared by the MapQuest providers.
//!
//! MapQuest accepts coordinates either as the documented `lat,lng` query text
//! or as a `latLng` object in a JSON request body, and bounding boxes as four
//! comma-separated values ordered upper-left then lower-right. The helpers
//! here keep every provider on the same spelling.

use everymap_core::types::{BoundingBox, Coordinate};
use serde_json::{json, Value};

/// Separator between the latitude and longitude of one coordinate.
const LATITUDE_LONGITUDE_SEPARATOR: char = ',';

/// Separator between the values of a bounding-box query string.
const BOUNDING_BOX_VALUE_SEPARATOR: char = ',';

/// Encodes a coordinate as a MapQuest `latLng` location object.
///
/// # Parameters
/// - `coordinate` — the coordinate to encode.
///
/// # Returns
/// A JSON object of the form `{"latLng": {"lat": …, "lng": …}}`, the shape the
/// route-matrix and similar JSON endpoints accept in their `locations` array.
pub(crate) fn coordinate_to_request_value(coordinate: &Coordinate) -> Value {
    json!({"latLng": {"lat": coordinate.lat, "lng": coordinate.lng}})
}

/// Formats a coordinate for the documented `lat,lng` query form.
///
/// # Parameters
/// - `coordinate` — the coordinate to format.
///
/// # Returns
/// `"{latitude},{longitude}"`.
pub(crate) fn coordinate_to_location_query_text(coordinate: &Coordinate) -> String {
    format!(
        "{}{LATITUDE_LONGITUDE_SEPARATOR}{}",
        coordinate.lat, coordinate.lng
    )
}

/// Formats a bounding box for the documented `ul.lat,ul.lng,lr.lat,lr.lng`
/// query form.
///
/// The upper-left corner is the north-west corner (the box's maximum latitude
/// and minimum longitude) and the lower-right corner is the south-east corner
/// (the box's minimum latitude and maximum longitude). A bounding box is not a
/// rectangle in a projected space, but on the latitude/longitude grid this is
/// exactly the ground it covers.
///
/// # Parameters
/// - `bounding_box` — the box to format.
///
/// # Returns
/// `"{north},{west},{south},{east}"`, with no spaces.
pub(crate) fn bounding_box_to_query_text(bounding_box: &BoundingBox) -> String {
    format!(
        "{}{BOUNDING_BOX_VALUE_SEPARATOR}{}{BOUNDING_BOX_VALUE_SEPARATOR}{}{BOUNDING_BOX_VALUE_SEPARATOR}{}",
        bounding_box.north_east.lat,
        bounding_box.south_west.lng,
        bounding_box.south_west.lat,
        bounding_box.north_east.lng
    )
}

#[cfg(test)]
mod tests {
    use super::{
        bounding_box_to_query_text, coordinate_to_location_query_text, coordinate_to_request_value,
    };
    use everymap_core::types::{BoundingBox, Coordinate};
    use serde_json::json;

    #[test]
    fn a_coordinate_encodes_as_a_latlng_object() {
        let coordinate = Coordinate::new(39.95, -105.25).unwrap();
        assert_eq!(
            coordinate_to_request_value(&coordinate),
            json!({"latLng": {"lat": 39.95, "lng": -105.25}})
        );
    }

    #[test]
    fn a_coordinate_encodes_as_query_text() {
        let coordinate = Coordinate::new(38.5766, -121.4944).unwrap();
        assert_eq!(
            coordinate_to_location_query_text(&coordinate),
            "38.5766,-121.4944"
        );
    }

    #[test]
    fn a_bounding_box_encodes_upper_left_then_lower_right() {
        let bounding_box = BoundingBox::new(
            Coordinate::new(39.95, -104.71).unwrap(),
            Coordinate::new(39.52, -105.25).unwrap(),
        );
        assert_eq!(
            bounding_box_to_query_text(&bounding_box),
            "39.95,-105.25,39.52,-104.71"
        );
    }
}
