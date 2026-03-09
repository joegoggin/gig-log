use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State};

use crate::api_tester::app::{Id, InputMode, Msg};

pub struct EditorUrlInput {
    component: Input,
    input_mode: InputMode,
}

impl EditorUrlInput {
    pub fn new(value: &str) -> Self {
        let component = Input::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Cyan),
            )
            .title("URL", Alignment::Left)
            .input_type(InputType::Text)
            .value(value);
        Self {
            component,
            input_mode: InputMode::Normal,
        }
    }
}

impl MockComponent for EditorUrlInput {
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        self.component.view(frame, area);
    }
    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.component.query(attr)
    }
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        if attr == Attribute::Custom("input_mode") {
            if let AttrValue::Flag(is_insert) = value {
                self.input_mode = if is_insert {
                    InputMode::Insert
                } else {
                    InputMode::Normal
                };
            }
            return;
        }

        self.component.attr(attr, value);
    }
    fn state(&self) -> State {
        self.component.state()
    }
    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        self.component.perform(cmd)
    }
}

impl Component<Msg, NoUserEvent> for EditorUrlInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                Some(Msg::FocusField(Id::EditorHeaders))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab, ..
            }) => Some(Msg::FocusField(Id::EditorMethod)),
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            }) if self.input_mode == InputMode::Normal => {
                self.perform(Cmd::Move(Direction::Left));
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            }) if self.input_mode == InputMode::Normal => {
                self.perform(Cmd::Move(Direction::Right));
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) if self.input_mode == InputMode::Normal => {
                self.perform(Cmd::Move(Direction::Left));
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) if self.input_mode == InputMode::Normal => {
                self.perform(Cmd::Move(Direction::Right));
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::SHIFT,
            }) if self.input_mode == InputMode::Insert => {
                self.perform(Cmd::Type(ch));
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) if self.input_mode == InputMode::Insert => {
                self.perform(Cmd::Delete);
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) if self.input_mode == InputMode::Insert => {
                self.perform(Cmd::Cancel);
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) if self.input_mode == InputMode::Insert => {
                self.perform(Cmd::Move(Direction::Left));
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) if self.input_mode == InputMode::Insert => {
                self.perform(Cmd::Move(Direction::Right));
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(input: &EditorUrlInput) -> String {
        if let State::One(tuirealm::StateValue::String(current)) = input.state() {
            current
        } else {
            String::new()
        }
    }

    #[test]
    fn normal_mode_moves_cursor_with_vim_keys() {
        let mut input = EditorUrlInput::new("ab");

        input.on(Event::Keyboard(KeyEvent::new(
            Key::Char('h'),
            KeyModifiers::NONE,
        )));
        input.attr(Attribute::Custom("input_mode"), AttrValue::Flag(true));
        input.on(Event::Keyboard(KeyEvent::new(
            Key::Char('x'),
            KeyModifiers::NONE,
        )));

        assert_eq!(value(&input), "axb");
    }

    #[test]
    fn normal_mode_moves_cursor_with_arrow_keys() {
        let mut input = EditorUrlInput::new("ab");

        input.on(Event::Keyboard(KeyEvent::new(
            Key::Left,
            KeyModifiers::NONE,
        )));
        input.attr(Attribute::Custom("input_mode"), AttrValue::Flag(true));
        input.on(Event::Keyboard(KeyEvent::new(
            Key::Char('x'),
            KeyModifiers::NONE,
        )));

        assert_eq!(value(&input), "axb");
    }
}
