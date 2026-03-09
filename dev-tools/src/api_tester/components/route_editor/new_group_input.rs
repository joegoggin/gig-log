use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State};

use crate::api_tester::app::{Id, InputMode, Msg};

pub struct EditorNewGroupInput {
    component: Input,
    input_mode: InputMode,
}

impl EditorNewGroupInput {
    pub fn new(value: &str) -> Self {
        let component = Input::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Yellow),
            )
            .title("New Group Name", Alignment::Left)
            .input_type(InputType::Text)
            .value(value);
        Self {
            component,
            input_mode: InputMode::Normal,
        }
    }
}

impl MockComponent for EditorNewGroupInput {
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

impl Component<Msg, NoUserEvent> for EditorNewGroupInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                Some(Msg::FocusField(Id::EditorMethod))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab, ..
            }) => Some(Msg::FocusField(Id::EditorGroup)),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) if self.input_mode == InputMode::Insert => Some(Msg::NewGroupEntered),
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
