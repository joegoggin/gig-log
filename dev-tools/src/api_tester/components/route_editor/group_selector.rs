use tuirealm::{Component, Event, MockComponent, NoUserEvent};

use crate::api_tester::app::{Id, Msg, RouteEditorMsg};
use crate::api_tester::components::core::tab_radio::{CoreTabRadio, RadioBindings};

pub struct EditorGroupSelector {
    field: CoreTabRadio,
}

impl EditorGroupSelector {
    pub fn new(group_names: &[String], current_group: &str) -> Self {
        let mut choices: Vec<String> = group_names.to_vec();
        choices.push("New Group...".to_string());

        let index = group_names
            .iter()
            .position(|group| group == current_group)
            .unwrap_or(0);

        Self {
            field: CoreTabRadio::new("Group", choices, index, tuirealm::props::Color::Cyan),
        }
    }
}

impl MockComponent for EditorGroupSelector {
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

impl Component<Msg, NoUserEvent> for EditorGroupSelector {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let bindings = RadioBindings::tab_cycle(
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::EditorNewGroup)),
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::EditorName)),
        );

        self.field.on_event(ev, &bindings, |index| {
            Msg::RouteEditor(RouteEditorMsg::GroupSelected(index))
        })
    }
}
