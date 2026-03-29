//! Shared method color helpers for API tester UI surfaces.

use ratatui::style::Color as TuiColor;
use tuirealm::props::Color;

use crate::api_tester::collection::HttpMethod;

/// Maps an HTTP method to a `tuirealm` color.
///
/// # Arguments
///
/// * `method` — HTTP method to style.
///
/// # Returns
///
/// A [`Color`] value for component-level styling.
pub fn method_color(method: &HttpMethod) -> Color {
    match method {
        HttpMethod::Get => Color::Green,
        HttpMethod::Post => Color::Blue,
        HttpMethod::Put => Color::Yellow,
        HttpMethod::Patch => Color::Magenta,
        HttpMethod::Delete => Color::Red,
    }
}

/// Maps an HTTP method to a `ratatui` color.
///
/// # Arguments
///
/// * `method` — HTTP method to style.
///
/// # Returns
///
/// A [`TuiColor`] value for direct ratatui rendering.
pub fn method_tui_color(method: &HttpMethod) -> TuiColor {
    match method {
        HttpMethod::Get => TuiColor::Green,
        HttpMethod::Post => TuiColor::Blue,
        HttpMethod::Put => TuiColor::Yellow,
        HttpMethod::Patch => TuiColor::Magenta,
        HttpMethod::Delete => TuiColor::Red,
    }
}
