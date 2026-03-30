//! Ratatui rendering for the development orchestrator terminal interface.
//!
//! This module renders a three-row layout containing a service status header,
//! scrollable logs viewport, and keyboard shortcut footer.

use ansi_to_tui::IntoText as _;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::log_store::{LogEntry, Service};

const SERVICE_LABEL_ALIGN_WIDTH: usize = "DEV-TOOLS".len();

/// Describes viewport sizing derived from the latest rendered frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct ViewMetrics {
    /// Reports visible log rows in the viewport.
    pub viewport_height: usize,
    /// Reports maximum scroll offset for the rendered content.
    pub max_offset: usize,
}

/// Holds interactive UI state for filtering, scrolling, and service badges.
pub struct AppState {
    /// Stores the active service filter, or `None` for all services.
    pub filter: Option<Service>,
    /// Stores current vertical scroll offset.
    pub scroll_offset: usize,
    /// Indicates whether scroll should follow the latest log lines.
    pub follow: bool,
    /// Tracks pending first `g` key for Vim-style `gg` behavior.
    pub pending_g: bool,
    /// Tracks running status badges for each service channel.
    pub services_running: [bool; 6], // [api, web, common, dev-tools, docs, system]
    /// Controls whether orchestrator batch events clear logs automatically.
    pub auto_clear: bool,
}

impl AppState {
    /// Creates default UI state for a fresh TUI session.
    ///
    /// # Returns
    ///
    /// An [`AppState`] configured for follow mode and all-service filtering.
    pub fn new() -> Self {
        Self {
            filter: None,
            scroll_offset: 0,
            follow: true,
            pending_g: false,
            services_running: [false; 6],
            auto_clear: true,
        }
    }
}

/// Renders the complete frame and returns viewport metrics.
///
/// # Arguments
///
/// * `frame` — Frame surface to draw into.
/// * `entries` — Filtered log entries to display.
/// * `state` — Current application state for layout and labels.
///
/// # Returns
///
/// A [`ViewMetrics`] snapshot describing the rendered log viewport.
pub fn render(frame: &mut Frame, entries: &[&LogEntry], state: &AppState) -> ViewMetrics {
    let chunks = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Min(1),    // log viewport
        Constraint::Length(1), // footer
    ])
    .split(frame.area());

    render_header(frame, chunks[0], state);
    let metrics = render_logs(frame, chunks[1], entries, state);
    render_footer(frame, chunks[2], state);

    metrics
}

/// Returns the UI color used for a service label.
///
/// # Arguments
///
/// * `service` — Service to map to a color.
///
/// # Returns
///
/// A [`Color`] used in headers and line prefixes.
fn service_color(service: Service) -> Color {
    match service {
        Service::Api => Color::Blue,
        Service::Web => Color::Green,
        Service::Common => Color::Cyan,
        Service::DevTools => Color::Magenta,
        Service::Docs => Color::Yellow,
        Service::System => Color::Red,
    }
}

/// Renders the top header row with service badges and quit hint.
///
/// # Arguments
///
/// * `frame` — Frame surface to draw into.
/// * `area` — Header rectangle for rendering.
/// * `state` — Current service running state and view flags.
fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let services = [
        (Service::Api, state.services_running[0]),
        (Service::Web, state.services_running[1]),
        (Service::Common, state.services_running[2]),
        (Service::DevTools, state.services_running[3]),
        (Service::Docs, state.services_running[4]),
        (Service::System, state.services_running[5]),
    ];

    let mut spans = vec![Span::styled(
        "  gig-log dev  ",
        Style::default().add_modifier(Modifier::BOLD),
    )];

    for (service, running) in services {
        let color = service_color(service);
        let mut style = Style::default().fg(color).add_modifier(Modifier::BOLD);
        if !running {
            style = style.add_modifier(Modifier::DIM);
        }
        spans.push(Span::styled(format!(" [{}] ", service.label()), style));
    }

    // Right-align quit hint
    let used: usize = spans.iter().map(|s| s.width()).sum();
    let hint = " q:quit ";
    if area.width as usize > used + hint.len() {
        let padding = area.width as usize - used - hint.len();
        spans.push(Span::raw(" ".repeat(padding)));
    }
    spans.push(Span::styled(hint, Style::default().fg(Color::DarkGray)));

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Builds one styled log line with aligned service prefix.
///
/// # Arguments
///
/// * `entry` — Log entry to render.
///
/// # Returns
///
/// A styled [`Line`] representing one rendered log row.
fn build_log_line(entry: &LogEntry) -> Line<'static> {
    let color = service_color(entry.service);
    let label = entry.service.label();
    let label_padding = SERVICE_LABEL_ALIGN_WIDTH.saturating_sub(label.len());
    let mut spans = vec![
        Span::styled(
            format!(" [{label}]"),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(label_padding)),
        Span::styled("| ", Style::default().fg(Color::DarkGray)),
    ];

    if let Ok(text) = entry.line.as_bytes().into_text() {
        if let Some(line) = text.lines.into_iter().next() {
            spans.extend(line.spans);
        }
    }

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::build_log_line;
    use crate::dev::log_store::{LogEntry, Service};

    fn rendered_text(entry: LogEntry) -> String {
        build_log_line(&entry)
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>()
    }

    #[test]
    fn aligns_service_pipe_column_for_all_labels() {
        let samples = [
            LogEntry {
                service: Service::Api,
                line: "api log".to_string(),
            },
            LogEntry {
                service: Service::Web,
                line: "web log".to_string(),
            },
            LogEntry {
                service: Service::Docs,
                line: "docs log".to_string(),
            },
            LogEntry {
                service: Service::System,
                line: "system log".to_string(),
            },
            LogEntry {
                service: Service::DevTools,
                line: "dev-tools log".to_string(),
            },
        ];

        let pipe_columns: Vec<usize> = samples
            .into_iter()
            .map(rendered_text)
            .map(|line| line.find('|').expect("line should include pipe separator"))
            .collect();

        let expected_column = pipe_columns[0];
        assert!(
            pipe_columns.iter().all(|column| *column == expected_column),
            "pipe columns should match: {pipe_columns:?}"
        );
    }
}

/// Renders the log viewport with wrapping, scroll offset, and scrollbar.
///
/// # Arguments
///
/// * `frame` — Frame surface to draw into.
/// * `area` — Rectangle allocated for the log viewport.
/// * `entries` — Filtered entries to render.
/// * `state` — Current scroll and follow state.
///
/// # Returns
///
/// A [`ViewMetrics`] value describing viewport height and max offset.
fn render_logs(
    frame: &mut Frame,
    area: Rect,
    entries: &[&LogEntry],
    state: &AppState,
) -> ViewMetrics {
    let visible: Vec<Line> = entries.iter().map(|entry| build_log_line(entry)).collect();

    let block = Block::default().borders(Borders::TOP | Borders::BOTTOM);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let log_width = inner.width.saturating_sub(1).max(1);
    let log_area = Rect {
        x: inner.x,
        y: inner.y,
        width: log_width,
        height: inner.height,
    };
    let scrollbar_area = Rect {
        x: inner.x.saturating_add(log_width),
        y: inner.y,
        width: inner.width.saturating_sub(log_width),
        height: inner.height,
    };

    let viewport_height = log_area.height as usize;
    let content_width = log_area.width.max(1);

    let wrapped_paragraph = Paragraph::new(visible).wrap(Wrap { trim: false });

    let total = wrapped_paragraph.line_count(content_width);
    let max_offset = total.saturating_sub(viewport_height);

    let offset = if state.follow {
        max_offset
    } else {
        state.scroll_offset.min(max_offset)
    };

    let paragraph = wrapped_paragraph.scroll((offset.min(u16::MAX as usize) as u16, 0));
    frame.render_widget(paragraph, log_area);

    if scrollbar_area.width > 0 && viewport_height > 0 {
        let thumb_row = if max_offset == 0 {
            0
        } else {
            offset.saturating_mul(viewport_height.saturating_sub(1)) / max_offset
        };
        let bar_width = scrollbar_area.width as usize;
        let scrollbar_lines: Vec<Line> = (0..viewport_height)
            .map(|row| {
                let (symbol, style) = if row == thumb_row {
                    ("", Style::default().fg(Color::DarkGray))
                } else if row < thumb_row {
                    ("", Style::default().fg(Color::DarkGray))
                } else {
                    ("", Style::default().fg(Color::DarkGray))
                };
                Line::from(Span::styled(symbol.repeat(bar_width), style))
            })
            .collect();
        frame.render_widget(Paragraph::new(scrollbar_lines), scrollbar_area);
    }

    ViewMetrics {
        viewport_height,
        max_offset,
    }
}

/// Renders the footer row with filters and key bindings.
///
/// # Arguments
///
/// * `frame` — Frame surface to draw into.
/// * `area` — Footer rectangle for rendering.
/// * `state` — Current app state used for labels.
fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let filter_label = match state.filter {
        None => "all",
        Some(Service::Api) => "api",
        Some(Service::Web) => "web",
        Some(Service::Common) => "common",
        Some(Service::DevTools) => "dev-tools",
        Some(Service::Docs) => "docs",
        Some(Service::System) => "system",
    };

    let auto_clear_label = if state.auto_clear { "on" } else { "off" };

    let line = Line::from(vec![
        Span::styled("  Filter: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            filter_label,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  Auto-clear: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            auto_clear_label,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  |  c:clear  x:auto-clear  1:api  2:web  3:common  4:dev-tools  5:docs  6:system  a:all  j/k:scroll  gg:top  G:bottom  ^u/^d:page",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}
