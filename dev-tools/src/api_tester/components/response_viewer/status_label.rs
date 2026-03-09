use tui_realm_stdlib::Label;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::props::Color;
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State};

use crate::api_tester::app::Msg;
use crate::api_tester::executor::CurlResponse;

pub struct ResponseStatusLabel {
    component: Label,
}

impl ResponseStatusLabel {
    pub fn new(response: &CurlResponse) -> Self {
        let color = status_color(response.status_code);
        let text = format!("HTTP {}", response.status_code);

        let component = Label::default().text(text).foreground(color);

        Self { component }
    }
}

fn status_color(code: u16) -> Color {
    match code {
        200..=299 => Color::Green,
        300..=399 => Color::Yellow,
        400..=499 => Color::Red,
        500..=599 => Color::LightRed,
        _ => Color::White,
    }
}

impl MockComponent for ResponseStatusLabel {
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        self.component.view(frame, area);
    }
    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.component.query(attr)
    }
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.component.attr(attr, value);
    }
    fn state(&self) -> State {
        self.component.state()
    }
    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        self.component.perform(cmd)
    }
}

impl Component<Msg, NoUserEvent> for ResponseStatusLabel {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}
