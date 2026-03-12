use tuirealm::{Component, Event, MockComponent, NoUserEvent};

use crate::api_tester::app::{Id, Msg, RouteEditorMsg};
use crate::api_tester::components::core::input_field::{CoreInputField, InputBindings};

pub struct EditorHeadersInput {
    field: CoreInputField,
}

impl EditorHeadersInput {
    pub fn new(headers: &[String]) -> Self {
        let value = headers.join(", ");
        Self {
            field: CoreInputField::new(
                "Headers (comma-separated)",
                &value,
                tuirealm::props::Color::Cyan,
            ),
        }
    }
}

impl MockComponent for EditorHeadersInput {
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

impl Component<Msg, NoUserEvent> for EditorHeadersInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let bindings = InputBindings::tab_cycle(
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::EditorName)),
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::EditorUrl)),
        );

        self.field.on_event(ev, &bindings)
    }
}
