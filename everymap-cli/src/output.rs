use serde_json::Value;
use std::io::{self, Write};

/// Output format for CLI results.
pub enum OutputFormat {
    Json,
    Pretty,
    Summary,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s {
            "pretty" => OutputFormat::Pretty,
            "summary" => OutputFormat::Summary,
            _ => OutputFormat::Json,
        }
    }
}

/// Write formatted output directly to a writer.
///
/// For JSON and Pretty formats, this uses `serde_json::to_writer` to write
/// directly to the writer without allocating an intermediate `String`.
pub fn write_output<W: Write>(writer: &mut W, value: &Value, format: &OutputFormat) -> io::Result<()> {
    match format {
        OutputFormat::Json => {
            serde_json::to_writer(&mut *writer, value)?;
            writeln!(writer)
        }
        OutputFormat::Pretty => {
            serde_json::to_writer_pretty(&mut *writer, value)?;
            writeln!(writer)
        }
        OutputFormat::Summary => {
            let summary = format_summary(value);
            writeln!(writer, "{}", summary)
        }
    }
}

/// Format a JSON value as a condensed human-readable summary.
fn format_summary(value: &Value) -> String {
    // For arrays, show one line per item
    if let Some(items) = value.get("results").and_then(|v| v.as_array()) {
        if items.is_empty() {
            return "No results found.".to_string();
        }
        let mut lines = Vec::new();
        for item in items {
            lines.push(format_item_summary(item));
        }
        lines.join("\n")
    } else if let Some(routes) = value.get("routes").and_then(|v| v.as_array()) {
        if routes.is_empty() {
            return "No routes found.".to_string();
        }
        let mut lines = Vec::new();
        for (i, route) in routes.iter().enumerate() {
            let dist = route.get("distance_m").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let dur = route.get("duration_s").and_then(|v| v.as_f64()).unwrap_or(0.0);
            lines.push(format!("Route {}: {:.1} km, {:.0} min", i + 1, dist / 1000.0, dur / 60.0));
        }
        lines.join("\n")
    } else if value.get("flows").is_some() {
        let count = value.get("flows").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        let incidents = value.get("incidents").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        format!("{} flow measurements, {} incidents", count, incidents)
    } else if value.get("isolines").is_some() {
        let count = value.get("isolines").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        format!("{} isoline(s)", count)
    } else if value.get("coordinate").is_some() {
        let lat = value.get("coordinate").and_then(|v| v.get("lat")).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let lng = value.get("coordinate").and_then(|v| v.get("lng")).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let acc = value.get("accuracy").and_then(|v| v.as_f64());
        let mut s = format!("Position: ({:.6}, {:.6})", lat, lng);
        if let Some(a) = acc {
            s.push_str(&format!(", accuracy: {:.0}m", a));
        }
        s
    } else if value.get("stops").is_some() {
        let count = value.get("stops").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        let dist = value.get("total_distance").and_then(|v| v.as_f64());
        let dur = value.get("total_duration").and_then(|v| v.as_f64());
        let mut s = format!("{} stops", count);
        if let Some(d) = dist {
            s.push_str(&format!(", {:.1} km", d / 1000.0));
        }
        if let Some(d) = dur {
            s.push_str(&format!(", {:.0} min", d / 60.0));
        }
        s
    } else if value.get("matched_points").is_some() {
        let count = value.get("matched_points").and_then(|v| v.as_f64()).unwrap_or(0.0) as usize;
        let dist = value.get("distance_m").and_then(|v| v.as_f64()).unwrap_or(0.0);
        format!("{} matched points, {:.1} m distance", count, dist)
    } else {
        // Fallback: pretty print
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    }
}

fn format_item_summary(item: &Value) -> String {
    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown");
    let lat = item.get("coordinate").and_then(|v| v.get("lat")).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let lng = item.get("coordinate").and_then(|v| v.get("lng")).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let result_type = item.get("result_type").and_then(|v| v.as_str()).unwrap_or("unknown");
    format!("[{}] {} ({:.6}, {:.6})", result_type, title, lat, lng)
}