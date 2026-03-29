//! Scrollable response details component.
//!
//! This module renders successful and failed request executions, including
//! status, headers, and formatted body content with wrapping and scrollbar
//! support.

use http::StatusCode;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use tui_realm_stdlib::List;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, TextSpan};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::api_tester::app::Msg;
use crate::api_tester::body_preview;
use crate::api_tester::executor::CurlResponse;

/// Number of lines moved for page-up/page-down shortcuts.
const PAGE_SCROLL_STEP: usize = 8;
/// Reserved width for the optional vertical scrollbar.
const SCROLLBAR_WIDTH: u16 = 1;

/// Intermediate styled text chunk before conversion to `TextSpan`.
#[derive(Clone)]
struct StyledChunk {
    /// Chunk text.
    text: String,
    /// Optional foreground color.
    color: Option<Color>,
}

/// Scrollable response details viewer component.
pub struct ResponseDetailsView {
    /// Underlying list component.
    component: List,
    /// Source response content currently displayed.
    response: ResponseContent,
    /// Current wrapping width used for row generation.
    wrap_width: usize,
    /// Total rendered line count after wrapping.
    line_count: usize,
    /// Tracks pending first `g` for `gg` navigation.
    pending_g: bool,
}

/// Response content variants rendered by [`ResponseDetailsView`].
#[derive(Clone)]
enum ResponseContent {
    /// Successful HTTP response payload.
    Success(CurlResponse),
    /// Execution error text payload.
    Error(String),
}

impl ResponseDetailsView {
    /// Creates a response details view from a successful response payload.
    ///
    /// # Arguments
    ///
    /// * `response` — Executed HTTP response.
    ///
    /// # Returns
    ///
    /// A [`ResponseDetailsView`] initialized with success content.
    pub fn new(response: &CurlResponse) -> Self {
        let response = ResponseContent::Success(response.clone());
        let rows = Self::build_rows(&response, usize::MAX);
        let line_count = rows.len();
        let component = Self::build_component(rows, 0);

        Self {
            component,
            response,
            wrap_width: usize::MAX,
            line_count,
            pending_g: false,
        }
    }

    /// Creates a response details view from an execution error.
    ///
    /// # Arguments
    ///
    /// * `error` — Execution error message.
    ///
    /// # Returns
    ///
    /// A [`ResponseDetailsView`] initialized with error content.
    pub fn new_error(error: impl Into<String>) -> Self {
        let response = ResponseContent::Error(error.into());
        let rows = Self::build_rows(&response, usize::MAX);
        let line_count = rows.len();
        let component = Self::build_component(rows, 0);

        Self {
            component,
            response,
            wrap_width: usize::MAX,
            line_count,
            pending_g: false,
        }
    }

    /// Returns whether a key event matches plain lowercase `g`.
    ///
    /// # Arguments
    ///
    /// * `key` — Keyboard event to inspect.
    ///
    /// # Returns
    ///
    /// A [`bool`] indicating whether the event is plain `g`.
    fn is_plain_g(key: &KeyEvent) -> bool {
        key.code == Key::Char('g') && key.modifiers == KeyModifiers::NONE
    }

    /// Returns whether a key event matches jump-to-end bindings.
    ///
    /// # Arguments
    ///
    /// * `key` — Keyboard event to inspect.
    ///
    /// # Returns
    ///
    /// A [`bool`] indicating whether the event means "jump to end".
    fn is_jump_to_end(key: &KeyEvent) -> bool {
        (key.code == Key::Char('G')
            && (key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT))
            || (key.code == Key::Char('g') && key.modifiers == KeyModifiers::SHIFT)
    }

    /// Builds a status line label from an HTTP status code.
    ///
    /// # Arguments
    ///
    /// * `code` — Numeric HTTP status code.
    ///
    /// # Returns
    ///
    /// A human-readable [`String`] status label.
    fn status_text(code: u16) -> String {
        if let Ok(status_code) = StatusCode::from_u16(code) {
            if let Some(reason) = status_code.canonical_reason() {
                return format!("HTTP {code} {reason}");
            }
        }

        format!("HTTP {code} Unknown Status")
    }

    /// Returns the display color for an HTTP status code.
    ///
    /// # Arguments
    ///
    /// * `code` — Numeric HTTP status code.
    ///
    /// # Returns
    ///
    /// A [`Color`] mapped from the status class.
    fn status_color(code: u16) -> Color {
        match code {
            200..=299 => Color::Green,
            300..=399 => Color::Yellow,
            400..=499 => Color::Red,
            500..=599 => Color::LightRed,
            _ => Color::White,
        }
    }

    /// Builds the backing list component for response rows.
    ///
    /// # Arguments
    ///
    /// * `rows` — Render rows as text spans.
    /// * `selected_line` — Initial selected row index.
    ///
    /// # Returns
    ///
    /// A configured [`List`] instance.
    fn build_component(rows: Vec<Vec<TextSpan>>, selected_line: usize) -> List {
        List::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Cyan),
            )
            .title("Response", Alignment::Left)
            .scroll(true)
            .highlighted_str("")
            .rows(rows)
            .selected_line(selected_line)
    }

    /// Creates a styled chunk helper value.
    ///
    /// # Arguments
    ///
    /// * `text` — Chunk text.
    /// * `color` — Optional foreground color.
    ///
    /// # Returns
    ///
    /// A [`StyledChunk`] value.
    fn styled_chunk(text: impl Into<String>, color: Option<Color>) -> StyledChunk {
        StyledChunk {
            text: text.into(),
            color,
        }
    }

    /// Builds styled rows for a successful response payload.
    ///
    /// # Arguments
    ///
    /// * `response` — Successful response payload.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of styled rows.
    fn build_styled_rows(response: &CurlResponse) -> Vec<Vec<StyledChunk>> {
        let mut rows = vec![vec![Self::styled_chunk(
            Self::status_text(response.status_code),
            Some(Self::status_color(response.status_code)),
        )]];
        rows.push(vec![Self::styled_chunk(String::new(), None)]);
        rows.push(vec![Self::styled_chunk("Headers", Some(Color::Cyan))]);

        if response.headers.is_empty() {
            rows.push(vec![Self::styled_chunk("  (none)", Some(Color::DarkGray))]);
        } else {
            rows.extend(response.headers.iter().map(|header| {
                if let Some((key, value)) = header.split_once(':') {
                    vec![
                        Self::styled_chunk(format!("  {key}:"), Some(Color::Cyan)),
                        Self::styled_chunk(value.to_string(), Some(Color::White)),
                    ]
                } else {
                    vec![Self::styled_chunk(
                        format!("  {header}"),
                        Some(Color::White),
                    )]
                }
            }));
        }

        rows.push(vec![Self::styled_chunk(String::new(), None)]);
        rows.push(vec![Self::styled_chunk("Body", Some(Color::Cyan))]);

        let body_preview = body_preview::build(&response.body);
        if body_preview.lines.is_empty() {
            rows.push(vec![Self::styled_chunk("  (empty)", Some(Color::DarkGray))]);
        } else {
            rows.extend(body_preview.lines.into_iter().map(|line| {
                line.spans
                    .into_iter()
                    .map(|span| Self::styled_chunk(span.content.to_string(), span.style.fg))
                    .collect()
            }));
        }

        rows
    }

    /// Builds styled rows for an execution error payload.
    ///
    /// # Arguments
    ///
    /// * `error` — Execution error message text.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of styled rows.
    fn build_error_styled_rows(error: &str) -> Vec<Vec<StyledChunk>> {
        let mut rows = vec![vec![Self::styled_chunk(
            "Execution Error",
            Some(Color::Red),
        )]];
        rows.push(vec![Self::styled_chunk(String::new(), None)]);

        if error.is_empty() {
            rows.push(vec![Self::styled_chunk("(empty)", Some(Color::DarkGray))]);
            return rows;
        }

        rows.extend(
            error
                .lines()
                .map(|line| vec![Self::styled_chunk(line.to_string(), Some(Color::Red))]),
        );

        rows
    }

    /// Finds the nearest UTF-8 character boundary for wrap width.
    ///
    /// # Arguments
    ///
    /// * `text` — Source text.
    /// * `width` — Maximum character width.
    ///
    /// # Returns
    ///
    /// A [`usize`] byte index for safe splitting.
    fn wrap_char_boundary(text: &str, width: usize) -> usize {
        if width == 0 {
            return 0;
        }

        let mut count = 0;
        for (index, _) in text.char_indices() {
            if count == width {
                return index;
            }
            count += 1;
        }

        text.len()
    }

    /// Wraps a styled row into multiple lines at a fixed width.
    ///
    /// # Arguments
    ///
    /// * `row` — Styled row to wrap.
    /// * `width` — Maximum row width in characters.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of wrapped styled rows.
    fn wrap_row(row: Vec<StyledChunk>, width: usize) -> Vec<Vec<StyledChunk>> {
        if width == 0 {
            return vec![row];
        }

        let row_len: usize = row.iter().map(|chunk| chunk.text.chars().count()).sum();
        if row_len == 0 {
            return vec![vec![Self::styled_chunk(String::new(), None)]];
        }

        let mut wrapped_rows = Vec::new();
        let mut current_row = Vec::new();
        let mut remaining_width = width;

        for chunk in row {
            if chunk.text.is_empty() {
                continue;
            }

            let mut rest = chunk.text.as_str();
            while !rest.is_empty() {
                if remaining_width == 0 {
                    wrapped_rows.push(current_row);
                    current_row = Vec::new();
                    remaining_width = width;
                }

                let split_at = Self::wrap_char_boundary(rest, remaining_width);
                let piece = &rest[..split_at];

                if !piece.is_empty() {
                    current_row.push(Self::styled_chunk(piece.to_string(), chunk.color));
                    remaining_width = remaining_width.saturating_sub(piece.chars().count());
                }

                rest = &rest[split_at..];
                if !rest.is_empty() && remaining_width == 0 {
                    wrapped_rows.push(current_row);
                    current_row = Vec::new();
                    remaining_width = width;
                }
            }
        }

        if !current_row.is_empty() {
            wrapped_rows.push(current_row);
        }

        if wrapped_rows.is_empty() {
            wrapped_rows.push(vec![Self::styled_chunk(String::new(), None)]);
        }

        wrapped_rows
    }

    /// Wraps all styled rows at a fixed width.
    ///
    /// # Arguments
    ///
    /// * `rows` — Styled rows to wrap.
    /// * `width` — Maximum row width in characters.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of wrapped styled rows.
    fn wrap_styled_rows(rows: Vec<Vec<StyledChunk>>, width: usize) -> Vec<Vec<StyledChunk>> {
        rows.into_iter()
            .flat_map(|row| Self::wrap_row(row, width))
            .collect()
    }

    /// Builds render rows from response content and wrap width.
    ///
    /// # Arguments
    ///
    /// * `response` — Response content variant.
    /// * `wrap_width` — Maximum wrapping width.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of list rows represented as [`TextSpan`] values.
    fn build_rows(response: &ResponseContent, wrap_width: usize) -> Vec<Vec<TextSpan>> {
        let wrap_width = wrap_width.max(1);
        let styled_rows = match response {
            ResponseContent::Success(response) => Self::build_styled_rows(response),
            ResponseContent::Error(error) => Self::build_error_styled_rows(error),
        };

        Self::wrap_styled_rows(styled_rows, wrap_width)
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|chunk| {
                        let mut span = TextSpan::new(chunk.text);
                        if let Some(color) = chunk.color {
                            span = span.fg(color);
                        }
                        span
                    })
                    .collect()
            })
            .collect()
    }

    /// Returns the selected line index from component state.
    ///
    /// # Returns
    ///
    /// A [`usize`] selected row index.
    fn selected_line(&self) -> usize {
        if let State::One(StateValue::Usize(index)) = self.component.state() {
            index
        } else {
            0
        }
    }

    /// Moves selection repeatedly in one direction.
    ///
    /// # Arguments
    ///
    /// * `direction` — Direction to move selection.
    /// * `amount` — Number of move operations.
    fn move_many(&mut self, direction: Direction, amount: usize) {
        for _ in 0..amount {
            self.perform(Cmd::Move(direction));
        }
    }

    /// Calculates body wrap width for a render area.
    ///
    /// # Arguments
    ///
    /// * `area` — Render area allocated to the response list.
    ///
    /// # Returns
    ///
    /// A [`usize`] wrapping width.
    fn wrap_width_for_area(area: Rect) -> usize {
        area.width.saturating_sub(2).max(1) as usize
    }

    /// Calculates visible viewport height for a render area.
    ///
    /// # Arguments
    ///
    /// * `area` — Render area allocated to the response list.
    ///
    /// # Returns
    ///
    /// A [`usize`] viewport line count.
    fn viewport_height_for_area(area: Rect) -> usize {
        area.height.saturating_sub(2).max(1) as usize
    }

    /// Rebuilds wrapped rows when the wrapping width changes.
    ///
    /// # Arguments
    ///
    /// * `wrap_width` — Newly computed wrap width.
    fn ensure_wrapped_for_width(&mut self, wrap_width: usize) {
        if wrap_width == self.wrap_width {
            return;
        }

        let rows = Self::build_rows(&self.response, wrap_width);
        self.line_count = rows.len();
        let selected_line = self.selected_line().min(rows.len().saturating_sub(1));
        self.component = Self::build_component(rows, selected_line);
        self.wrap_width = wrap_width;
    }

    /// Renders the scrollbar for long response content.
    ///
    /// # Arguments
    ///
    /// * `frame` — Frame to render into.
    /// * `area` — Scrollbar render area.
    /// * `viewport_height` — Visible line count in the content viewport.
    fn render_scrollbar(&self, frame: &mut ratatui::Frame<'_>, area: Rect, viewport_height: usize) {
        if area.width == 0 || area.height == 0 || viewport_height == 0 {
            return;
        }

        if self.line_count <= viewport_height {
            return;
        }

        let position = self.selected_line().min(self.line_count.saturating_sub(1));
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_style(Style::default().fg(ratatui::style::Color::DarkGray))
            .thumb_style(Style::default().fg(ratatui::style::Color::Cyan));
        let mut state = ScrollbarState::new(self.line_count)
            .position(position)
            .viewport_content_length(viewport_height);

        frame.render_stateful_widget(scrollbar, area, &mut state);
    }

    /// Builds unwrapped response lines for test assertions.
    ///
    /// # Arguments
    ///
    /// * `response` — Response payload used to build lines.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of rendered line strings.
    #[cfg(test)]
    fn build_text_lines(response: &CurlResponse) -> Vec<String> {
        Self::build_wrapped_text_lines(response, usize::MAX)
    }

    /// Builds wrapped response lines for test assertions.
    ///
    /// # Arguments
    ///
    /// * `response` — Response payload used to build lines.
    /// * `wrap_width` — Maximum wrapping width in characters.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of rendered line strings.
    #[cfg(test)]
    fn build_wrapped_text_lines(response: &CurlResponse, wrap_width: usize) -> Vec<String> {
        Self::wrap_styled_rows(Self::build_styled_rows(response), wrap_width.max(1))
            .into_iter()
            .map(|row| row.into_iter().map(|chunk| chunk.text).collect::<String>())
            .collect()
    }
}

impl MockComponent for ResponseDetailsView {
    /// Renders response details content and optional scrollbar.
    ///
    /// # Arguments
    ///
    /// * `frame` — Frame to render into.
    /// * `area` — Area allocated to the widget.
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let can_render_scrollbar = area.width > SCROLLBAR_WIDTH;
        let (content_area, scrollbar_area) = if can_render_scrollbar {
            let chunks =
                Layout::horizontal([Constraint::Min(0), Constraint::Length(SCROLLBAR_WIDTH)])
                    .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            (area, None)
        };

        self.ensure_wrapped_for_width(Self::wrap_width_for_area(content_area));
        self.component.view(frame, content_area);

        if let Some(scrollbar_area) = scrollbar_area {
            self.render_scrollbar(
                frame,
                scrollbar_area,
                Self::viewport_height_for_area(content_area),
            );
        }
    }

    /// Queries a widget attribute value.
    ///
    /// # Arguments
    ///
    /// * `attr` — Attribute to query.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing the queried attribute value.
    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.component.query(attr)
    }

    /// Updates a widget attribute value.
    ///
    /// # Arguments
    ///
    /// * `attr` — Attribute to update.
    /// * `value` — New attribute value.
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.component.attr(attr, value);
    }

    /// Returns the current widget state.
    ///
    /// # Returns
    ///
    /// A [`State`] snapshot for the widget.
    fn state(&self) -> State {
        self.component.state()
    }

    /// Executes a command against the widget.
    ///
    /// # Arguments
    ///
    /// * `cmd` — Command to execute.
    ///
    /// # Returns
    ///
    /// A [`CmdResult`] produced by the widget.
    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        self.component.perform(cmd)
    }
}

impl Component<Msg, NoUserEvent> for ResponseDetailsView {
    /// Handles keyboard events for response navigation.
    ///
    /// # Arguments
    ///
    /// * `ev` — Incoming component event.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing an emitted application [`Msg`].
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key) => {
                if self.pending_g {
                    self.pending_g = false;

                    if Self::is_plain_g(&key) {
                        self.perform(Cmd::GoTo(Position::Begin));
                        return None;
                    }
                }

                if Self::is_plain_g(&key) {
                    self.pending_g = true;
                    return None;
                }

                if Self::is_jump_to_end(&key) {
                    self.pending_g = false;
                    self.perform(Cmd::GoTo(Position::End));
                    return None;
                }

                self.pending_g = false;

                match key {
                    KeyEvent {
                        code: Key::Char('j'),
                        modifiers: KeyModifiers::NONE,
                    }
                    | KeyEvent {
                        code: Key::Down, ..
                    } => {
                        self.perform(Cmd::Move(Direction::Down));
                        None
                    }
                    KeyEvent {
                        code: Key::Char('k'),
                        modifiers: KeyModifiers::NONE,
                    }
                    | KeyEvent { code: Key::Up, .. } => {
                        self.perform(Cmd::Move(Direction::Up));
                        None
                    }
                    KeyEvent {
                        code: Key::PageDown,
                        ..
                    } => {
                        self.move_many(Direction::Down, PAGE_SCROLL_STEP);
                        None
                    }
                    KeyEvent {
                        code: Key::PageUp, ..
                    } => {
                        self.move_many(Direction::Up, PAGE_SCROLL_STEP);
                        None
                    }
                    KeyEvent {
                        code: Key::Home, ..
                    } => {
                        self.perform(Cmd::GoTo(Position::Begin));
                        None
                    }
                    KeyEvent { code: Key::End, .. } => {
                        self.perform(Cmd::GoTo(Position::End));
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_event(code: Key, modifiers: KeyModifiers) -> Event<NoUserEvent> {
        Event::Keyboard(KeyEvent::new(code, modifiers))
    }

    fn selected_line(view: &ResponseDetailsView) -> usize {
        if let State::One(StateValue::Usize(index)) = view.state() {
            index
        } else {
            0
        }
    }

    fn sample_response(status_code: u16) -> CurlResponse {
        CurlResponse {
            status_code,
            headers: vec![
                "content-type: application/json".to_string(),
                "x-request-id: abc-123".to_string(),
            ],
            body: "{\"message\":\"not found\"}".to_string(),
        }
    }

    #[test]
    fn status_text_includes_reason_phrase() {
        assert_eq!(ResponseDetailsView::status_text(404), "HTTP 404 Not Found");
    }

    #[test]
    fn status_text_falls_back_for_unknown_codes() {
        assert_eq!(
            ResponseDetailsView::status_text(799),
            "HTTP 799 Unknown Status"
        );
    }

    #[test]
    fn text_lines_are_ordered_status_headers_body() {
        let response = sample_response(404);
        let lines = ResponseDetailsView::build_text_lines(&response);

        assert_eq!(
            lines.first().map(|line| line.as_str()),
            Some("HTTP 404 Not Found")
        );
        assert_eq!(lines.get(2).map(|line| line.as_str()), Some("Headers"));
        assert_eq!(lines.get(5).map(|line| line.as_str()), Some(""));
        assert_eq!(lines.get(6).map(|line| line.as_str()), Some("Body"));
        assert!(
            lines
                .iter()
                .any(|line| line == "  content-type: application/json")
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("\"message\": \"not found\""))
        );
    }

    #[test]
    fn wrapped_lines_respect_width() {
        let response = CurlResponse {
            status_code: 200,
            headers: vec!["x-very-long-header-name: abcdefghijklmnop".to_string()],
            body: "{\"veryLongKey\":\"veryLongValue\"}".to_string(),
        };

        let wrapped = ResponseDetailsView::build_wrapped_text_lines(&response, 12);
        assert!(wrapped.iter().all(|line| line.chars().count() <= 12));

        let wrapped_concat: String = wrapped.concat();
        let unwrapped_concat: String = ResponseDetailsView::build_text_lines(&response).concat();
        assert_eq!(wrapped_concat, unwrapped_concat);
    }

    #[test]
    fn gg_jumps_to_top() {
        let response = sample_response(200);
        let mut view = ResponseDetailsView::new(&response);

        for _ in 0..6 {
            view.on(key_event(Key::Down, KeyModifiers::NONE));
        }
        assert!(selected_line(&view) > 0);

        view.on(key_event(Key::Char('g'), KeyModifiers::NONE));
        view.on(key_event(Key::Char('g'), KeyModifiers::NONE));

        assert_eq!(selected_line(&view), 0);
    }

    #[test]
    fn uppercase_g_jumps_to_bottom() {
        let response = sample_response(200);
        let mut view = ResponseDetailsView::new(&response);

        view.on(key_event(Key::Home, KeyModifiers::NONE));
        view.on(key_event(Key::Char('G'), KeyModifiers::SHIFT));
        let jump_end_line = selected_line(&view);

        view.on(key_event(Key::Home, KeyModifiers::NONE));
        view.on(key_event(Key::End, KeyModifiers::NONE));
        let end_key_line = selected_line(&view);

        assert_eq!(jump_end_line, end_key_line);
        assert!(jump_end_line > 0);
    }
}
