//! Route group selector component for the route editor.

use tuirealm::{Component, Event, MockComponent, NoUserEvent};

use crate::api_tester::app::{Id, Msg, RouteEditorMsg};
use crate::api_tester::components::core::tab_radio::{CoreTabRadio, RadioBindings};

/// Route editor group selector with a trailing "New Group..." option.
pub struct EditorGroupSelector {
    /// Reusable radio control implementation.
    field: CoreTabRadio,
}

impl EditorGroupSelector {
    /// Creates a group selector initialized from known groups.
    ///
    /// # Arguments
    ///
    /// * `group_names` — Existing route group names.
    /// * `current_group` — Group currently assigned to the edited route.
    ///
    /// # Returns
    ///
    /// An [`EditorGroupSelector`] ready for mounting in the route editor.
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
    /// Renders the group selector widget.
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

impl Component<Msg, NoUserEvent> for EditorGroupSelector {
    /// Handles group selector events and emits route editor messages.
    ///
    /// # Arguments
    ///
    /// * `ev` — Incoming component event.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing an emitted application [`Msg`].
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
