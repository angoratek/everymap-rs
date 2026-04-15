/// Safely convert a serializable enum value to its string representation.
///
/// This replaces the unsafe pattern `serde_json::to_value(x).unwrap().as_str().unwrap().to_string()`
/// which panics if serialization fails or the value isn't a string.
///
/// All HERE API enums use `#[serde(rename = "...")]` on variants, guaranteeing
/// string serialization. This helper returns an empty string on the impossible
/// failure path rather than panicking.
pub fn enum_as_str<T: serde::Serialize>(val: &T) -> String {
    serde_json::to_value(val)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default()
}