use tui_realm_stdlib::Radio;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::api_tester::app::{Id, Msg};
use crate::api_tester::collection::HttpMethod;

pub struct EditorMethodRadio {
    component: Radio,
}

impl EditorMethodRadio {
    pub fn new(selected: &HttpMethod) -> Self {
        let index = match selected {
            HttpMethod::Get => 0,
            HttpMethod::Post => 1,
            HttpMethod::Put => 2,
            HttpMethod::Patch => 3,
            HttpMethod::Delete => 4,
        };

        let component = Radio::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Cyan),
            )
            .title("Method", Alignment::Left)
            .choices(["GET", "POST", "PUT", "PATCH", "DELETE"])
            .value(index);

        Self { component }
    }
}

impl MockComponent for EditorMethodRadio {
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

impl Component<Msg, NoUserEvent> for EditorMethodRadio {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                Some(Msg::FocusField(Id::EditorUrl))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab, ..
            }) => Some(Msg::FocusField(Id::EditorGroup)),
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => {
                self.perform(Cmd::Move(Direction::Left));
                if let State::One(StateValue::Usize(index)) = self.state() {
                    Some(Msg::MethodChanged(index))
                } else {
                    None
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => {
                self.perform(Cmd::Move(Direction::Right));
                if let State::One(StateValue::Usize(index)) = self.state() {
                    Some(Msg::MethodChanged(index))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selected_index(radio: &EditorMethodRadio) -> usize {
        if let State::One(StateValue::Usize(index)) = radio.state() {
            index
        } else {
            panic!("method radio should always have a selection");
        }
    }

    #[test]
    fn vim_keys_move_method_selection() {
        let mut radio = EditorMethodRadio::new(&HttpMethod::Put);

        assert_eq!(selected_index(&radio), 2);
        assert_eq!(
            radio.on(Event::Keyboard(KeyEvent::new(
                Key::Char('h'),
                KeyModifiers::NONE,
            ))),
            Some(Msg::MethodChanged(1))
        );
        assert_eq!(selected_index(&radio), 1);

        assert_eq!(
            radio.on(Event::Keyboard(KeyEvent::new(
                Key::Char('l'),
                KeyModifiers::NONE,
            ))),
            Some(Msg::MethodChanged(2))
        );
        assert_eq!(selected_index(&radio), 2);
    }

    #[test]
    fn arrow_keys_still_move_method_selection() {
        let mut radio = EditorMethodRadio::new(&HttpMethod::Put);

        assert_eq!(selected_index(&radio), 2);
        assert_eq!(
            radio.on(Event::Keyboard(KeyEvent::new(
                Key::Left,
                KeyModifiers::NONE
            ))),
            Some(Msg::MethodChanged(1))
        );
        assert_eq!(selected_index(&radio), 1);

        assert_eq!(
            radio.on(Event::Keyboard(KeyEvent::new(
                Key::Right,
                KeyModifiers::NONE
            ))),
            Some(Msg::MethodChanged(2))
        );
        assert_eq!(selected_index(&radio), 2);
    }
}
