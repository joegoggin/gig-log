use tuirealm::{Component, Event, MockComponent, NoUserEvent};

use crate::api_tester::app::{Id, Msg, RouteEditorMsg};
use crate::api_tester::components::core::input_field::{CoreInputField, InputBindings};

pub struct EditorNewGroupInput {
    field: CoreInputField,
}

impl EditorNewGroupInput {
    pub fn new(value: &str) -> Self {
        Self {
            field: CoreInputField::new("New Group Name", value, tuirealm::props::Color::Yellow),
        }
    }
}

impl MockComponent for EditorNewGroupInput {
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        self.field.view(frame, area);
    }

    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        self.field.query(attr)
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        self.field.attr(attr, value);
    }

    fn state(&self) -> tuirealm::State {
        self.field.state()
    }

    fn perform(&mut self, cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        self.field.perform(cmd)
    }
}

impl Component<Msg, NoUserEvent> for EditorNewGroupInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let mut bindings = InputBindings::tab_cycle(
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::EditorMethod)),
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::EditorGroup)),
        );
        bindings.on_insert_enter = Some(Msg::RouteEditor(RouteEditorMsg::NewGroupEntered));

        self.field.on_event(ev, &bindings)
    }
}
