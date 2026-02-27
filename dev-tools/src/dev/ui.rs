use ansi_to_tui::IntoText as _;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};
use ratatui::Frame;

use super::log_store::{LogEntry, Service};

#[derive(Debug, Clone, Copy, Default)]
pub struct ViewMetrics {
    pub viewport_height: usize,
    pub max_offset: usize,
}

pub struct AppState {
    pub filter: Option<Service>,
    pub scroll_offset: usize,
    pub follow: bool,
    pub pending_g: bool,
    pub services_running: [bool; 3], // [api, web, docs]
}

impl AppState {
    pub fn new() -> Self {
        Self {
            filter: None,
            scroll_offset: 0,
            follow: true,
            pending_g: false,
            services_running: [false; 3],
        }
    }
}

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

fn service_color(service: Service) -> Color {
    match service {
        Service::Api => Color::Blue,
        Service::Web => Color::Green,
        Service::Docs => Color::Yellow,
    }
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let services = [
        (Service::Api, state.services_running[0]),
        (Service::Web, state.services_running[1]),
        (Service::Docs, state.services_running[2]),
    ];

    let mut spans = vec![Span::styled(
        "  gig-log dev  ",
        Style::default().add_modifier(Modifier::BOLD),
    )];

    for (service, running) in services {
        let color = if running {
            service_color(service)
        } else {
            Color::DarkGray
        };
        spans.push(Span::styled(
            format!(" [{}] ", service.label()),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
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

fn build_log_line(entry: &LogEntry) -> Line<'static> {
    let color = service_color(entry.service);
    let mut spans = vec![
        Span::styled(
            format!(" [{}] ", entry.service.label()),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
    ];

    if let Ok(text) = entry.line.as_bytes().into_text() {
        if let Some(line) = text.lines.into_iter().next() {
            spans.extend(line.spans);
        }
    }

    Line::from(spans)
}

fn render_logs(
    frame: &mut Frame,
    area: Rect,
    entries: &[&LogEntry],
    state: &AppState,
) -> ViewMetrics {
    let visible: Vec<Line> = entries.iter().map(|entry| build_log_line(entry)).collect();

    let block = Block::default().borders(Borders::TOP | Borders::BOTTOM);
    let inner = block.inner(area);
    let viewport_height = inner.height as usize;
    let content_width = inner.width.saturating_sub(1).max(1);

    let wrapped_paragraph = Paragraph::new(visible)
        .block(block)
        .wrap(Wrap { trim: false });

    let total = wrapped_paragraph.line_count(content_width);
    let max_offset = total.saturating_sub(viewport_height);

    let offset = if state.follow {
        max_offset
    } else {
        state.scroll_offset.min(max_offset)
    };

    let paragraph = wrapped_paragraph.scroll((offset.min(u16::MAX as usize) as u16, 0));
    frame.render_widget(paragraph, area);

    let mut scrollbar_state = ScrollbarState::new(total)
        .viewport_content_length(viewport_height)
        .position(offset);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .track_style(Style::default().fg(Color::DarkGray))
        .thumb_style(Style::default().fg(Color::Gray));
    frame.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);

    ViewMetrics {
        viewport_height,
        max_offset,
    }
}

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let filter_label = match state.filter {
        None => "all",
        Some(Service::Api) => "api",
        Some(Service::Web) => "web",
        Some(Service::Docs) => "docs",
    };

    let line = Line::from(vec![
        Span::styled("  Filter: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            filter_label,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  |  c:clear  1:api  2:web  3:docs  a:all  j/k:scroll  gg:top  G:bottom  ^u/^d:page",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}
