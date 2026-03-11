use tuirealm::{Component, Event, MockComponent, NoUserEvent};

use crate::api_tester::app::{Id, Msg, RouteEditorMsg};
use crate::api_tester::collection::HttpMethod;
use crate::api_tester::components::core::tab_radio::{CoreTabRadio, RadioBindings};

const METHOD_CHOICES: [&str; 5] = ["GET", "POST", "PUT", "PATCH", "DELETE"];

pub struct EditorMethodRadio {
    field: CoreTabRadio,
}

impl EditorMethodRadio {
    pub fn new(selected: &HttpMethod) -> Self {
        let index = match selected {
            HttpMethod::Get => 0,
            HttpMethod::Post => 1,
            HttpMethod::Put => 2,
            HttpMethod::Patch => 3,
            HttpMethod::Delete => 4,
        };

        let choices = METHOD_CHOICES
            .iter()
            .map(|choice| choice.to_string())
            .collect();

        Self {
            field: CoreTabRadio::new("Method", choices, index, tuirealm::props::Color::Cyan),
        }
    }
}

impl MockComponent for EditorMethodRadio {
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

impl Component<Msg, NoUserEvent> for EditorMethodRadio {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let bindings = RadioBindings::tab_cycle(
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::EditorUrl)),
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::EditorGroup)),
        );

        self.field.on_event(ev, &bindings, |index| {
            Msg::RouteEditor(RouteEditorMsg::MethodChanged(index))
        })
    }
}
