use tuirealm::{Component, Event, MockComponent, NoUserEvent};

use crate::api_tester::app::{Id, Msg, RouteEditorMsg, VariableManagerMsg};
use crate::api_tester::components::core::tab_radio::{CoreTabRadio, RadioBindings};
use crate::api_tester::variables::VariableMode;

const MODE_CHOICES: [&str; 2] = ["placeholder", "hidden"];

pub struct VariableModeSelector {
    field: CoreTabRadio,
}

impl VariableModeSelector {
    pub fn new(mode: VariableMode) -> Self {
        let selected = match mode {
            VariableMode::Placeholder => 0,
            VariableMode::Hidden => 1,
        };

        let choices = MODE_CHOICES
            .iter()
            .map(|choice| choice.to_string())
            .collect();

        Self {
            field: CoreTabRadio::new("Mode", choices, selected, tuirealm::props::Color::Cyan),
        }
    }
}

impl MockComponent for VariableModeSelector {
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

impl Component<Msg, NoUserEvent> for VariableModeSelector {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let mut bindings = RadioBindings::tab_cycle(
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::VariableKeyInput)),
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::VariableValueInput)),
        );
        bindings.on_ctrl_s = Some(Msg::VariableManager(VariableManagerMsg::Save));

        self.field.on_event(ev, &bindings, |index| {
            let mode = if index == 1 {
                VariableMode::Hidden
            } else {
                VariableMode::Placeholder
            };

            Msg::VariableManager(VariableManagerMsg::ModeChanged(mode))
        })
    }
}
