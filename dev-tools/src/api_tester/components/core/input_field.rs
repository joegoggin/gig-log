use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType};
use tuirealm::{AttrValue, Attribute, Event, MockComponent, NoUserEvent, State};

use crate::api_tester::app::{InputMode, Msg};

#[derive(Debug, Clone)]
pub struct InputBindings {
    pub on_tab: Option<Msg>,
    pub on_backtab: Option<Msg>,
    pub on_ctrl_s: Option<Msg>,
    pub on_insert_enter: Option<Msg>,
}

impl InputBindings {
    pub fn tab_cycle(on_tab: Msg, on_backtab: Msg) -> Self {
        Self {
            on_tab: Some(on_tab),
            on_backtab: Some(on_backtab),
            on_ctrl_s: None,
            on_insert_enter: None,
        }
    }
}

pub struct CoreInputField {
    component: Input,
    input_mode: InputMode,
}

impl CoreInputField {
    pub fn new(title: &str, value: &str, border_color: Color) -> Self {
        let component = Input::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(border_color),
            )
            .title(title, Alignment::Left)
            .input_type(InputType::Text)
            .value(value);

        Self {
            component,
            input_mode: InputMode::Normal,
        }
    }

    pub fn on_event(&mut self, ev: Event<NoUserEvent>, bindings: &InputBindings) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => bindings.on_tab.clone(),
            Event::Keyboard(KeyEvent {
                code: Key::BackTab, ..
            }) => bindings.on_backtab.clone(),
            Event::Keyboard(KeyEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            }) => bindings.on_ctrl_s.clone(),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) if self.input_mode == InputMode::Insert => bindings.on_insert_enter.clone(),
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

impl MockComponent for CoreInputField {
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
