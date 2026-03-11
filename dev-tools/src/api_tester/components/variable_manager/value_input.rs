use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State};

use crate::api_tester::app::{Id, InputMode, Msg};

pub struct VariableValueInput {
    component: Input,
    input_mode: InputMode,
}

impl VariableValueInput {
    pub fn new(value: &str) -> Self {
        let component = Input::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Cyan),
            )
            .title("Value", Alignment::Left)
            .input_type(InputType::Text)
            .value(value);
        Self {
            component,
            input_mode: InputMode::Normal,
        }
    }
}

impl MockComponent for VariableValueInput {
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

impl Component<Msg, NoUserEvent> for VariableValueInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                Some(Msg::FocusField(Id::VariableMode))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab, ..
            }) => Some(Msg::FocusField(Id::VariableKeyInput)),
            Event::Keyboard(KeyEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            }) => Some(Msg::SaveVariable),
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) if self.input_mode == InputMode::Normal => {
                self.perform(Cmd::Move(Direction::Left));
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
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

    fn value(input: &VariableValueInput) -> String {
        if let State::One(tuirealm::StateValue::String(current)) = input.state() {
            current
        } else {
            String::new()
        }
    }

    #[test]
    fn normal_mode_ignores_character_input() {
        let mut input = VariableValueInput::new("");

        input.on(Event::Keyboard(KeyEvent::new(
            Key::Char('x'),
            KeyModifiers::NONE,
        )));

        assert_eq!(value(&input), "");
    }

    #[test]
    fn insert_mode_accepts_character_input() {
        let mut input = VariableValueInput::new("");
        input.attr(Attribute::Custom("input_mode"), AttrValue::Flag(true));

        input.on(Event::Keyboard(KeyEvent::new(
            Key::Char('x'),
            KeyModifiers::NONE,
        )));

        assert_eq!(value(&input), "x");
    }

    #[test]
    fn ctrl_s_emits_save_variable_message() {
        let mut input = VariableValueInput::new("");

        assert_eq!(
            input.on(Event::Keyboard(KeyEvent::new(
                Key::Char('s'),
                KeyModifiers::CONTROL,
            ))),
            Some(Msg::SaveVariable)
        );
    }

    #[test]
    fn enter_does_not_emit_save_variable_message() {
        let mut input = VariableValueInput::new("");

        assert_eq!(
            input.on(Event::Keyboard(KeyEvent::new(
                Key::Enter,
                KeyModifiers::NONE,
            ))),
            None
        );
    }
}
