use ratatui::{Frame, layout::Rect};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State,
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent, KeyModifiers},
};

use crate::api_tester::app::{ActiveView, InputMode, Msg};

pub struct GlobalListener {
    input_mode: InputMode,
}

impl GlobalListener {
    pub fn new() -> Self {
        Self {
            input_mode: InputMode::Normal,
        }
    }

    fn map_normal_key(key: KeyEvent) -> Option<Msg> {
        match key.code {
            Key::Char('q') => Some(Msg::AppClose),
            Key::Char('v') => Some(Msg::SwitchView(ActiveView::VariableManager)),
            Key::Char('i') => Some(Msg::EnterInsertMode),
            Key::Esc => Some(Msg::CancelEdit),
            Key::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Msg::SaveRoute),
            Key::Char('b') => Some(Msg::OpenBodyEditor),
            _ => None,
        }
    }

    fn map_insert_key(key: KeyEvent) -> Option<Msg> {
        match key.code {
            Key::Esc => Some(Msg::EnterNormalMode),
            _ => None, // All other keys pass through to focused component
        }
    }
}

impl MockComponent for GlobalListener {
    fn view(&mut self, _frame: &mut Frame, _area: Rect) {}

    fn query(&self, _attr: Attribute) -> Option<AttrValue> {
        None
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
        }
    }

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }
}

impl Component<Msg, NoUserEvent> for GlobalListener {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key) => match self.input_mode {
                InputMode::Normal => Self::map_normal_key(key),
                InputMode::Insert => Self::map_insert_key(key),
            },
            Event::WindowResize(width, height) => Some(Msg::TerminalResize(width, height)),
            _ => None,
        }
    }
}
