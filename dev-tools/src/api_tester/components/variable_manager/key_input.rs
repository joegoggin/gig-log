use tuirealm::{Component, Event, MockComponent, NoUserEvent};

use crate::api_tester::app::{Id, Msg, RouteEditorMsg, VariableManagerMsg};
use crate::api_tester::components::core::input_field::{CoreInputField, InputBindings};

pub struct VariableKeyInput {
    field: CoreInputField,
}

impl VariableKeyInput {
    pub fn new(value: &str) -> Self {
        Self {
            field: CoreInputField::new("Key", value, tuirealm::props::Color::Cyan),
        }
    }
}

impl MockComponent for VariableKeyInput {
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

impl Component<Msg, NoUserEvent> for VariableKeyInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let mut bindings = InputBindings::tab_cycle(
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::VariableValueInput)),
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::VariableMode)),
        );
        bindings.on_ctrl_s = Some(Msg::VariableManager(VariableManagerMsg::Save));

        self.field.on_event(ev, &bindings)
    }
}
