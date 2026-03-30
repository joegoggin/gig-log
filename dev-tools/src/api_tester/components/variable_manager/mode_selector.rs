//! Variable mode selector component.

use tuirealm::{Component, Event, MockComponent, NoUserEvent};

use crate::api_tester::app::{Id, Msg, RouteEditorMsg, VariableManagerMsg};
use crate::api_tester::components::core::tab_radio::{CoreTabRadio, RadioBindings};
use crate::api_tester::variables::VariableMode;

/// Variable mode labels displayed in the selector.
const MODE_CHOICES: [&str; 2] = ["placeholder", "hidden"];

/// Variable mode selector control.
pub struct VariableModeSelector {
    /// Reusable tab-style radio control.
    field: CoreTabRadio,
}

impl VariableModeSelector {
    /// Creates a variable mode selector initialized to the current mode.
    ///
    /// # Arguments
    ///
    /// * `mode` — Initially selected variable mode.
    ///
    /// # Returns
    ///
    /// A [`VariableModeSelector`] configured with the provided mode.
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
    /// Renders the mode selector widget.
    ///
    /// # Arguments
    ///
    /// * `frame` — Frame to render into.
    /// * `area` — Area allocated to the widget.
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        self.field.view(frame, area);
    }

    /// Queries a widget attribute value.
    ///
    /// # Arguments
    ///
    /// * `attr` — Attribute to query.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing the queried attribute value.
    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        self.field.query(attr)
    }

    /// Updates a widget attribute value.
    ///
    /// # Arguments
    ///
    /// * `attr` — Attribute to update.
    /// * `value` — New attribute value.
    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        self.field.attr(attr, value);
    }

    /// Returns the current widget state.
    ///
    /// # Returns
    ///
    /// A [`tuirealm::State`] snapshot for the widget.
    fn state(&self) -> tuirealm::State {
        self.field.state()
    }

    /// Executes a command against the widget.
    ///
    /// # Arguments
    ///
    /// * `cmd` — Command to execute.
    ///
    /// # Returns
    ///
    /// A [`tuirealm::command::CmdResult`] produced by the widget.
    fn perform(&mut self, cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        self.field.perform(cmd)
    }
}

impl Component<Msg, NoUserEvent> for VariableModeSelector {
    /// Handles mode selector events and emits variable manager messages.
    ///
    /// # Arguments
    ///
    /// * `ev` — Incoming component event.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing an emitted application [`Msg`].
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
