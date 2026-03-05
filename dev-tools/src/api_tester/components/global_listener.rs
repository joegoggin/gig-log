use ratatui::{Frame, layout::Rect};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State,
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent},
};

use crate::api_tester::app::{ActiveView, Msg};

#[derive(Default)]
pub struct GlobalListener;

fn map_global_key(key: KeyEvent) -> Option<Msg> {
    match key.code {
        Key::Char('q') => Some(Msg::AppClose),
        Key::Char('v') => Some(Msg::SwitchView(ActiveView::VariableManager)),
        Key::Esc => Some(Msg::SwitchView(ActiveView::RouteList)),
        _ => None,
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
            Event::Keyboard(key) => map_global_key(key),
            _ => None,
        }
    }
}
