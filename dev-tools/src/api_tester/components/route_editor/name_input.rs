//! Name input component for the route editor.

use tuirealm::{Component, Event, MockComponent, NoUserEvent};

use crate::api_tester::app::{Id, Msg, RouteEditorMsg};
use crate::api_tester::components::core::input_field::{CoreInputField, InputBindings};

/// Route name input field.
pub struct EditorNameInput {
    /// Reusable text input implementation.
    field: CoreInputField,
}

impl EditorNameInput {
    /// Creates a route name input field.
    ///
    /// # Arguments
    ///
    /// * `value` — Initial route name value.
    ///
    /// # Returns
    ///
    /// An [`EditorNameInput`] configured with the provided value.
    pub fn new(value: &str) -> Self {
        Self {
            field: CoreInputField::new("Name", value, tuirealm::props::Color::Cyan),
        }
    }
}

impl MockComponent for EditorNameInput {
    /// Renders the name input widget.
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

impl Component<Msg, NoUserEvent> for EditorNameInput {
    /// Handles name input events and emits route editor messages.
    ///
    /// # Arguments
    ///
    /// * `ev` — Incoming component event.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing an emitted application [`Msg`].
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let bindings = InputBindings::tab_cycle(
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::EditorGroup)),
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::EditorHeaders)),
        );

        self.field.on_event(ev, &bindings)
    }
}
