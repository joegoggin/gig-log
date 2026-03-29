//! Variable value input component.

use tuirealm::{Component, Event, MockComponent, NoUserEvent};

use crate::api_tester::app::{Id, Msg, RouteEditorMsg, VariableManagerMsg};
use crate::api_tester::components::core::input_field::{CoreInputField, InputBindings};

/// Variable value text input field.
pub struct VariableValueInput {
    /// Reusable text input implementation.
    field: CoreInputField,
}

impl VariableValueInput {
    /// Creates a variable value input field.
    ///
    /// # Arguments
    ///
    /// * `value` — Initial value text.
    ///
    /// # Returns
    ///
    /// A [`VariableValueInput`] configured with the provided value.
    pub fn new(value: &str) -> Self {
        Self {
            field: CoreInputField::new("Value", value, tuirealm::props::Color::Cyan),
        }
    }
}

impl MockComponent for VariableValueInput {
    /// Renders the variable value input widget.
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

impl Component<Msg, NoUserEvent> for VariableValueInput {
    /// Handles value-input events and emits variable manager messages.
    ///
    /// # Arguments
    ///
    /// * `ev` — Incoming component event.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing an emitted application [`Msg`].
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let mut bindings = InputBindings::tab_cycle(
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::VariableMode)),
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::VariableKeyInput)),
        );
        bindings.on_ctrl_s = Some(Msg::VariableManager(VariableManagerMsg::Save));

        self.field.on_event(ev, &bindings)
    }
}
