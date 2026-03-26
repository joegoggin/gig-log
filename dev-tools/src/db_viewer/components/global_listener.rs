use ratatui::{Frame, layout::Rect};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State,
    command::{Cmd, CmdResult},
};

use crate::db_viewer::app::{AppMsg, Msg};

pub struct GlobalListener;

impl GlobalListener {
    pub fn new() -> Self {
        Self
    }
}

impl MockComponent for GlobalListener {
    fn view(&mut self, _frame: &mut Frame, _area: Rect) {}

    fn query(&self, _attr: Attribute) -> Option<AttrValue> {
        None
    }

    fn attr(&mut self, _attr: Attribute, _value: AttrValue) {}

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
            Event::Keyboard(key) => Some(Msg::App(AppMsg::Key(key))),
            Event::WindowResize(width, height) => {
                Some(Msg::App(AppMsg::TerminalResize(width, height)))
            }
            _ => None,
        }
    }
}
