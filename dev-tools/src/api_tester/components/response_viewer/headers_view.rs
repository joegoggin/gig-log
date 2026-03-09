use tui_realm_stdlib::List;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, TextSpan};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State};

use crate::api_tester::app::Msg;

pub struct ResponseHeadersView {
    component: List,
}

impl ResponseHeadersView {
    pub fn new(headers: &[String]) -> Self {
        let rows: Vec<Vec<TextSpan>> = headers
            .iter()
            .map(|h| {
                if let Some((key, value)) = h.split_once(':') {
                    vec![
                        TextSpan::new(format!("{key}:")).fg(Color::Cyan),
                        TextSpan::new(value.to_string()).fg(Color::White),
                    ]
                } else {
                    vec![TextSpan::new(h.clone()).fg(Color::White)]
                }
            })
            .collect();

        let component = List::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Cyan),
            )
            .title("Headers", Alignment::Left)
            .scroll(true)
            .highlighted_str("")
            .rows(rows)
            .selected_line(0);

        Self { component }
    }
}

impl MockComponent for ResponseHeadersView {
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

impl Component<Msg, NoUserEvent> for ResponseHeadersView {
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
