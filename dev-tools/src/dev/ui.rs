use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::log_store::{LogEntry, Service};

pub struct AppState {
    pub filter: Option<Service>,
    pub scroll_offset: usize,
    pub follow: bool,
    pub services_running: [bool; 3], // [api, web, docs]
}

impl AppState {
    pub fn new() -> Self {
        Self {
            filter: None,
            scroll_offset: 0,
            follow: true,
            services_running: [false; 3],
        }
    }
}

pub fn render(frame: &mut Frame, entries: &[&LogEntry], state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Min(1),   // log viewport
        Constraint::Length(1), // footer
    ])
    .split(frame.area());

    render_header(frame, chunks[0], state);
    render_logs(frame, chunks[1], entries, state);
    render_footer(frame, chunks[2], state);
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
    spans.push(Span::styled(
        hint,
        Style::default().fg(Color::DarkGray),
    ));

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_logs(frame: &mut Frame, area: Rect, entries: &[&LogEntry], state: &AppState) {
    let viewport_height = area.height.saturating_sub(2) as usize; // account for block borders
    let total = entries.len();

    let offset = if state.follow {
        total.saturating_sub(viewport_height)
    } else {
        state.scroll_offset.min(total.saturating_sub(viewport_height))
    };

    let visible: Vec<Line> = entries
        .iter()
        .skip(offset)
        .take(viewport_height)
        .map(|entry| {
            let color = service_color(entry.service);
            Line::from(vec![
                Span::styled(
                    format!(" [{}] ", entry.service.label()),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " | ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(&entry.line),
            ])
        })
        .collect();

    let block = Block::default().borders(Borders::TOP | Borders::BOTTOM);
    let paragraph = Paragraph::new(visible).block(block).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
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
            "  |  c:clear  1:api  2:web  3:docs  a:all  j/k:scroll  G:bottom",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}
