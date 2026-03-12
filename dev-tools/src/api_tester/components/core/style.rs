use ratatui::style::Color as TuiColor;
use tuirealm::props::Color;

use crate::api_tester::collection::HttpMethod;

pub fn method_color(method: &HttpMethod) -> Color {
    match method {
        HttpMethod::Get => Color::Green,
        HttpMethod::Post => Color::Blue,
        HttpMethod::Put => Color::Yellow,
        HttpMethod::Patch => Color::Magenta,
        HttpMethod::Delete => Color::Red,
    }
}

pub fn method_tui_color(method: &HttpMethod) -> TuiColor {
    match method {
        HttpMethod::Get => TuiColor::Green,
        HttpMethod::Post => TuiColor::Blue,
        HttpMethod::Put => TuiColor::Yellow,
        HttpMethod::Patch => TuiColor::Magenta,
        HttpMethod::Delete => TuiColor::Red,
    }
}
