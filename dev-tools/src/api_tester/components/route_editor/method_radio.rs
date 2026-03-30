//! HTTP method selector component for the route editor.

use tuirealm::{Component, Event, MockComponent, NoUserEvent};

use crate::api_tester::app::{Id, Msg, RouteEditorMsg};
use crate::api_tester::collection::HttpMethod;
use crate::api_tester::components::core::tab_radio::{CoreTabRadio, RadioBindings};

/// Supported HTTP method labels for route editing.
const METHOD_CHOICES: [&str; 5] = ["GET", "POST", "PUT", "PATCH", "DELETE"];

/// Route editor HTTP method selector.
pub struct EditorMethodRadio {
    /// Reusable tab-style radio control.
    field: CoreTabRadio,
}

impl EditorMethodRadio {
    /// Creates a method selector with the current method selected.
    ///
    /// # Arguments
    ///
    /// * `selected` — Current route HTTP method.
    ///
    /// # Returns
    ///
    /// An [`EditorMethodRadio`] initialized to the current method.
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
    /// Renders the method selector widget.
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

impl Component<Msg, NoUserEvent> for EditorMethodRadio {
    /// Handles method selector events and emits route editor messages.
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
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::EditorUrl)),
            Msg::RouteEditor(RouteEditorMsg::FocusField(Id::EditorNewGroup)),
        );

        self.field.on_event(ev, &bindings, |index| {
            Msg::RouteEditor(RouteEditorMsg::MethodChanged(index))
        })
    }
}
