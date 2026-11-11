//! Validation and expansion of the `{z}/{x}/{y}` URL template a tileset
//! publishes.
//!
//! A tile template is accepted only when it is an absolute `http` or `https`
//! URL carrying the three web-mercator placeholders and no placeholder the
//! provider cannot honestly expand. The optional `{s}` placeholder is
//! substituted with one of the source's declared subdomain or scheme values.
//! Validation runs once at construction, so a fetch never builds a broken or
//! partially substituted URL. Raster and vector sources share this validator,
//! so both tile contracts agree on what a valid template is.

use crate::error::{EveryMapError, EveryMapResult};
use url::Url;

/// Zoom-level placeholder a template must carry.
pub const ZOOM_LEVEL_PLACEHOLDER: &str = "{z}";

/// Tile-column placeholder a template must carry.
pub const TILE_COLUMN_PLACEHOLDER: &str = "{x}";

/// Tile-row placeholder a template must carry.
pub const TILE_ROW_PLACEHOLDER: &str = "{y}";

/// Optional subdomain or scheme placeholder a template may carry.
///
/// The value substituted for it is chosen from the source's declared
/// subdomains. Providers use it both for host sharding
/// (`https://{s}.tiles.example.com/…`) and for selecting the URL scheme
/// (`{s}://tiles.example.com/…`).
pub const SUBDOMAIN_PLACEHOLDER: &str = "{s}";

/// Placeholders every template must carry, without the surrounding braces.
const REQUIRED_PLACEHOLDER_NAMES: [&str; 3] = ["z", "x", "y"];

/// Placeholder names this provider expands, without the surrounding braces.
const SUPPORTED_PLACEHOLDER_NAMES: [&str; 4] = ["z", "x", "y", "s"];

/// URL schemes a tile template may use.
const SUPPORTED_URL_SCHEMES: [&str; 2] = ["http", "https"];

/// Zoom level substituted to prove a template parses as an absolute URL.
const SAMPLE_ZOOM_LEVEL: u8 = 0;

/// Tile column substituted to prove a template parses as an absolute URL.
const SAMPLE_TILE_COLUMN: u32 = 0;

/// Tile row substituted to prove a template parses as an absolute URL.
const SAMPLE_TILE_ROW: u32 = 0;

/// A validated tileset URL template together with the values it substitutes
/// for `{s}`.
///
/// Construct one with [`UrlTemplate::new`]; the constructor refuses a
/// template that lacks `{z}`, `{x}` or `{y}`, uses any other placeholder,
/// declares `{s}` without subdomains, or does not parse as an absolute
/// `http` or `https` URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlTemplate {
    template_text: String,
    subdomains: Vec<String>,
}

impl UrlTemplate {
    /// Validates a template and stores it verbatim.
    ///
    /// # Parameters
    /// - `template_text` — the template as the tileset publishes it.
    /// - `subdomains` — values substituted for `{s}`, in the order the source
    ///   declares them. The list may be empty when the template has no `{s}`
    ///   placeholder; it must be non-empty, with no blank entries, otherwise.
    ///
    /// # Returns
    /// A template that can expand every web-mercator tile coordinate.
    ///
    /// # Errors
    /// Returns [`EveryMapError::ValidationError`] when a required placeholder
    /// is missing, an unsupported placeholder is present, a brace is
    /// unbalanced, `{s}` is used without subdomains, a subdomain entry is
    /// blank, or the expanded sample is not an absolute `http`/`https` URL.
    pub fn new(template_text: impl Into<String>, subdomains: &[String]) -> EveryMapResult<Self> {
        let template_text = template_text.into();
        let placeholder_names = placeholder_names_in(&template_text)?;

        for required_placeholder_name in REQUIRED_PLACEHOLDER_NAMES {
            if !placeholder_names
                .iter()
                .any(|name| name == required_placeholder_name)
            {
                return Err(EveryMapError::ValidationError(format!(
                    "tile URL template \"{template_text}\" lacks the \
                     {{{required_placeholder_name}}} placeholder"
                )));
            }
        }
        for placeholder_name in &placeholder_names {
            if !SUPPORTED_PLACEHOLDER_NAMES.contains(&placeholder_name.as_str()) {
                return Err(EveryMapError::ValidationError(format!(
                    "tile URL template \"{template_text}\" uses the unsupported \
                     placeholder {{{placeholder_name}}}; only {{z}}, {{x}}, {{y}} and {{s}} are \
                     expanded"
                )));
            }
        }

        let uses_subdomains = placeholder_names.iter().any(|name| name == "s");
        if uses_subdomains && subdomains.is_empty() {
            return Err(EveryMapError::ValidationError(format!(
                "tile URL template \"{template_text}\" uses {{s}} but declares no subdomains"
            )));
        }
        for subdomain in subdomains {
            if subdomain.trim().is_empty() {
                return Err(EveryMapError::ValidationError(format!(
                    "tile URL template \"{template_text}\" declares a blank subdomain value"
                )));
            }
        }

        let sample_subdomain = subdomains.first().map(String::as_str).unwrap_or_default();
        let sample_url = expand_template(
            &template_text,
            SAMPLE_TILE_COLUMN,
            SAMPLE_TILE_ROW,
            SAMPLE_ZOOM_LEVEL,
            sample_subdomain,
        );
        let parsed_url = Url::parse(&sample_url).map_err(|error| {
            EveryMapError::ValidationError(format!(
                "tile URL template \"{template_text}\" is not a valid absolute URL: {error}"
            ))
        })?;
        if !SUPPORTED_URL_SCHEMES.contains(&parsed_url.scheme()) {
            return Err(EveryMapError::ValidationError(format!(
                "tile URL template \"{template_text}\" uses the {} scheme; only http and \
                 https are supported",
                parsed_url.scheme()
            )));
        }

        Ok(Self {
            template_text,
            subdomains: subdomains.to_vec(),
        })
    }

    /// The template text as supplied, placeholders intact.
    pub fn template_text(&self) -> &str {
        &self.template_text
    }

    /// Expands one tile's URL from the template.
    ///
    /// When the template carries `{s}`, the subdomain is chosen
    /// deterministically from `tile_x + tile_y`, so a tile always maps to the
    /// same host and concurrent fetches spread across the declared values.
    ///
    /// # Parameters
    /// - `tile_x`, `tile_y` — the tile's column and row at `zoom_level`.
    /// - `zoom_level` — the web-mercator zoom level of the tile.
    ///
    /// # Returns
    /// The fully substituted absolute URL.
    pub fn expand(&self, tile_x: u32, tile_y: u32, zoom_level: u8) -> String {
        let subdomain = if self.subdomains.is_empty() {
            ""
        } else {
            let subdomain_index =
                (tile_x as usize).wrapping_add(tile_y as usize) % self.subdomains.len();
            &self.subdomains[subdomain_index]
        };
        expand_template(&self.template_text, tile_x, tile_y, zoom_level, subdomain)
    }
}

/// Substitutes every supported placeholder with the given values.
fn expand_template(
    template_text: &str,
    tile_x: u32,
    tile_y: u32,
    zoom_level: u8,
    subdomain: &str,
) -> String {
    template_text
        .replace(ZOOM_LEVEL_PLACEHOLDER, &zoom_level.to_string())
        .replace(TILE_COLUMN_PLACEHOLDER, &tile_x.to_string())
        .replace(TILE_ROW_PLACEHOLDER, &tile_y.to_string())
        .replace(SUBDOMAIN_PLACEHOLDER, subdomain)
}

/// Reads every `{name}` placeholder out of the template, refusing an
/// unbalanced opening brace because no honest expansion can repair it.
fn placeholder_names_in(template_text: &str) -> EveryMapResult<Vec<String>> {
    let mut placeholder_names = Vec::new();
    let mut remaining_text = template_text;
    while let Some(open_index) = remaining_text.find('{') {
        let after_open = &remaining_text[open_index + 1..];
        let Some(close_index) = after_open.find('}') else {
            return Err(EveryMapError::ValidationError(format!(
                "tile URL template \"{template_text}\" has an opening brace without a \
                 closing brace"
            )));
        };
        placeholder_names.push(after_open[..close_index].to_string());
        remaining_text = &after_open[close_index + 1..];
    }
    Ok(placeholder_names)
}

#[cfg(test)]
mod tests {
    use super::UrlTemplate;
    use crate::error::EveryMapError;

    fn no_subdomains() -> Vec<String> {
        Vec::new()
    }

    #[test]
    fn expands_every_placeholder() {
        let url_template = UrlTemplate::new(
            "https://tiles.example.test/{z}/{x}/{y}.mvt",
            &no_subdomains(),
        )
        .unwrap();
        assert_eq!(
            url_template.expand(12, 34, 5),
            "https://tiles.example.test/5/12/34.mvt"
        );
    }

    #[test]
    fn placeholders_may_repeat_and_sit_in_a_query_string() {
        let url_template = UrlTemplate::new(
            "https://tiles.example.test/{z}/{x}/{y}.mvt?z={z}&style=base",
            &no_subdomains(),
        )
        .unwrap();
        assert_eq!(
            url_template.expand(1, 2, 3),
            "https://tiles.example.test/3/1/2.mvt?z=3&style=base"
        );
    }

    #[test]
    fn subdomains_are_chosen_deterministically_from_the_tile_coordinates() {
        let subdomains = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let url_template =
            UrlTemplate::new("https://{s}.example.test/{z}/{x}/{y}.mvt", &subdomains).unwrap();
        assert_eq!(
            url_template.expand(2, 1, 3),
            "https://a.example.test/3/2/1.mvt"
        );
        assert_eq!(
            url_template.expand(2, 2, 3),
            "https://b.example.test/3/2/2.mvt"
        );
        assert_eq!(
            url_template.expand(2, 3, 3),
            "https://c.example.test/3/2/3.mvt"
        );
        // The same tile always maps to the same host.
        assert_eq!(
            url_template.expand(2, 1, 3),
            "https://a.example.test/3/2/1.mvt"
        );
    }

    #[test]
    fn the_subdomain_value_may_select_the_scheme() {
        let subdomains = vec!["https".to_string()];
        let url_template =
            UrlTemplate::new("{s}://tiles.example.test/{z}/{x}/{y}.mvt", &subdomains).unwrap();
        assert_eq!(
            url_template.expand(1, 2, 3),
            "https://tiles.example.test/3/1/2.mvt"
        );
    }

    #[test]
    fn subdomains_are_ignored_when_the_template_has_no_subdomain_placeholder() {
        let subdomains = vec!["a".to_string()];
        let url_template =
            UrlTemplate::new("https://tiles.example.test/{z}/{x}/{y}.mvt", &subdomains).unwrap();
        assert_eq!(
            url_template.expand(1, 2, 3),
            "https://tiles.example.test/3/1/2.mvt"
        );
    }

    #[test]
    fn missing_required_placeholders_are_rejected() {
        for incomplete_template in [
            "https://tiles.example.test/{x}/{y}.mvt",
            "https://tiles.example.test/{z}/{y}.mvt",
            "https://tiles.example.test/{z}/{x}.mvt",
        ] {
            assert!(matches!(
                UrlTemplate::new(incomplete_template, &no_subdomains()),
                Err(EveryMapError::ValidationError(_))
            ));
        }
    }

    #[test]
    fn unsupported_placeholder_is_rejected() {
        assert!(matches!(
            UrlTemplate::new(
                "https://tiles.example.test/{z}/{x}/{y}.mvt?q={q}",
                &no_subdomains()
            ),
            Err(EveryMapError::ValidationError(_))
        ));
    }

    #[test]
    fn a_subdomain_placeholder_without_values_is_rejected() {
        assert!(matches!(
            UrlTemplate::new("https://{s}.example.test/{z}/{x}/{y}.mvt", &no_subdomains()),
            Err(EveryMapError::ValidationError(_))
        ));
    }

    #[test]
    fn a_blank_subdomain_value_is_rejected() {
        let subdomains = vec!["a".to_string(), "  ".to_string()];
        assert!(matches!(
            UrlTemplate::new("https://{s}.example.test/{z}/{x}/{y}.mvt", &subdomains),
            Err(EveryMapError::ValidationError(_))
        ));
    }

    #[test]
    fn unbalanced_brace_is_rejected() {
        assert!(matches!(
            UrlTemplate::new(
                "https://tiles.example.test/{z}/{x}/{y.mvt",
                &no_subdomains()
            ),
            Err(EveryMapError::ValidationError(_))
        ));
    }

    #[test]
    fn non_http_scheme_is_rejected() {
        assert!(matches!(
            UrlTemplate::new("ftp://tiles.example.test/{z}/{x}/{y}.mvt", &no_subdomains()),
            Err(EveryMapError::ValidationError(_))
        ));
    }

    #[test]
    fn relative_template_is_rejected() {
        assert!(matches!(
            UrlTemplate::new("tiles.example.test/{z}/{x}/{y}.mvt", &no_subdomains()),
            Err(EveryMapError::ValidationError(_))
        ));
    }
}
