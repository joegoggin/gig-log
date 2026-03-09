use tui_realm_stdlib::List;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, TextSpan};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State};

use crate::api_tester::app::Msg;
use crate::api_tester::body_preview;

pub struct ResponseBodyView {
    component: List,
}

impl ResponseBodyView {
    pub fn new(body: &str) -> Self {
        let rows = Self::build_rows(body);

        let component = List::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Cyan),
            )
            .title("Body", Alignment::Left)
            .scroll(true)
            .highlighted_str("")
            .rows(rows)
            .selected_line(0);

        Self { component }
    }

    fn build_rows(body: &str) -> Vec<Vec<TextSpan>> {
        let preview = body_preview::build(body);

        // Convert ratatui Lines to TextSpan rows for the List widget
        preview
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| {
                        let mut ts = TextSpan::new(span.content.to_string());
                        if let Some(fg) = span.style.fg {
                            ts = ts.fg(fg);
                        }
                        ts
                    })
                    .collect()
            })
            .collect()
    }
}

impl MockComponent for ResponseBodyView {
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

impl Component<Msg, NoUserEvent> for ResponseBodyView {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => {
                self.perform(Cmd::Move(Direction::Down));
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up));
                None
            }
            _ => None,
        }
    }
}
