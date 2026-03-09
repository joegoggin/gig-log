use tui_realm_stdlib::Radio;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::api_tester::app::Msg;

pub struct ResponseTabSelector {
    component: Radio,
}

impl ResponseTabSelector {
    pub fn new(selected: usize) -> Self {
        let component = Radio::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Cyan),
            )
            .title("Response", Alignment::Left)
            .choices(["Status", "Headers", "Body"])
            .value(selected);
        Self { component }
    }
}

impl MockComponent for ResponseTabSelector {
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

impl Component<Msg, NoUserEvent> for ResponseTabSelector {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            }) => {
                self.perform(Cmd::Move(Direction::Left));
                if let State::One(StateValue::Usize(i)) = self.state() {
                    Some(Msg::ResponseTabChanged(i))
                } else {
                    None
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            }) => {
                self.perform(Cmd::Move(Direction::Right));
                if let State::One(StateValue::Usize(i)) = self.state() {
                    Some(Msg::ResponseTabChanged(i))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
