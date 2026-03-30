//! Variable list component for the variable manager screen.

use tui_realm_stdlib::List;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, TextSpan};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::api_tester::app::{Msg, VariableManagerMsg};
use crate::api_tester::components::core::keymap::{is_jump_to_end, is_plain_g};
use crate::api_tester::variables::VariableMode;

/// Scrollable variable table with Vim-style navigation bindings.
pub struct VariableTable {
    /// Underlying list component.
    component: List,
    /// Row-to-key mapping for selected-row actions.
    keys: Vec<String>,
    /// Tracks pending first `g` for `gg` navigation.
    pending_g: bool,
}

impl VariableTable {
    /// Creates a variable table from display entries.
    ///
    /// # Arguments
    ///
    /// * `entries` — Display entries containing key, value, mode, and source.
    /// * `_secrets_visible` — Reserved visibility flag.
    ///
    /// # Returns
    ///
    /// A [`VariableTable`] initialized with selectable rows.
    pub fn new(entries: &[(String, String, VariableMode, String)], _secrets_visible: bool) -> Self {
        let key_width = entries
            .iter()
            .map(|(key, _, _, _)| key.chars().count())
            .max()
            .unwrap_or(3)
            .max(3)
            .min(24);

        let mut keys = Vec::with_capacity(entries.len());
        let rows: Vec<Vec<TextSpan>> = if entries.is_empty() {
            vec![vec![
                TextSpan::new("No variables. Press 'a' to add one.").fg(Color::DarkGray),
            ]]
        } else {
            entries
                .iter()
                .map(|(key, _, _, _)| {
                    keys.push(key.clone());
                    vec![TextSpan::new(format!("{key:<width$}", width = key_width)).fg(Color::Cyan)]
                })
                .collect()
        };

        let component = List::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Cyan),
            )
            .title("Variables", Alignment::Left)
            .highlighted_color(Color::LightYellow)
            .highlighted_str(">> ")
            .scroll(true)
            .rows(rows)
            .selected_line(0);

        Self {
            component,
            keys,
            pending_g: false,
        }
    }

    /// Returns the key corresponding to the currently selected row.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing the selected variable key.
    fn selected_key(&self) -> Option<String> {
        match self.state() {
            State::One(StateValue::Usize(index)) => self.keys.get(index).cloned(),
            _ => None,
        }
    }

    /// Builds a selection-changed message for the current row.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing a [`Msg::VariableManager`] selection update.
    fn selection_changed_message(&self) -> Option<Msg> {
        self.selected_key()
            .map(|key| Msg::VariableManager(VariableManagerMsg::SelectionChanged(key)))
    }
}

impl MockComponent for VariableTable {
    /// Renders the variable table widget.
    ///
    /// # Arguments
    ///
    /// * `frame` — Frame to render into.
    /// * `area` — Area allocated to the widget.
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        self.component.view(frame, area);
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
    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.component.query(attr)
    }

    /// Updates a widget attribute value.
    ///
    /// # Arguments
    ///
    /// * `attr` — Attribute to update.
    /// * `value` — New attribute value.
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.component.attr(attr, value);
    }

    /// Returns the current widget state.
    ///
    /// # Returns
    ///
    /// A [`State`] snapshot for the widget.
    fn state(&self) -> State {
        self.component.state()
    }

    /// Executes a command against the widget.
    ///
    /// # Arguments
    ///
    /// * `cmd` — Command to execute.
    ///
    /// # Returns
    ///
    /// A [`CmdResult`] produced by the widget.
    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        self.component.perform(cmd)
    }
}

impl Component<Msg, NoUserEvent> for VariableTable {
    /// Handles table keyboard events and emits variable manager messages.
    ///
    /// # Arguments
    ///
    /// * `ev` — Incoming component event.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing an emitted application [`Msg`].
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key) => {
                if self.pending_g {
                    self.pending_g = false;

                    if is_plain_g(&key) {
                        self.perform(Cmd::GoTo(Position::Begin));
                        return self.selection_changed_message();
                    }
                }

                if is_plain_g(&key) {
                    self.pending_g = true;
                    return None;
                }

                if is_jump_to_end(&key) {
                    self.perform(Cmd::GoTo(Position::End));
                    return self.selection_changed_message();
                }

                self.pending_g = false;

                match key {
                    KeyEvent {
                        code: Key::Char('j'),
                        modifiers: KeyModifiers::NONE,
                    }
                    | KeyEvent {
                        code: Key::Down, ..
                    } => {
                        self.perform(Cmd::Move(Direction::Down));
                        self.selection_changed_message()
                    }
                    KeyEvent {
                        code: Key::Char('k'),
                        modifiers: KeyModifiers::NONE,
                    }
                    | KeyEvent { code: Key::Up, .. } => {
                        self.perform(Cmd::Move(Direction::Up));
                        self.selection_changed_message()
                    }
                    KeyEvent {
                        code: Key::Home, ..
                    } => {
                        self.perform(Cmd::GoTo(Position::Begin));
                        self.selection_changed_message()
                    }
                    KeyEvent { code: Key::End, .. } => {
                        self.perform(Cmd::GoTo(Position::End));
                        self.selection_changed_message()
                    }
                    KeyEvent {
                        code: Key::Char('a'),
                        modifiers: KeyModifiers::NONE,
                    } => Some(Msg::VariableManager(VariableManagerMsg::Add)),
                    KeyEvent {
                        code: Key::Char('d'),
                        modifiers: KeyModifiers::NONE,
                    } => self
                        .selected_key()
                        .map(|key| Msg::VariableManager(VariableManagerMsg::Delete(key))),
                    KeyEvent {
                        code: Key::Char('e'),
                        modifiers: KeyModifiers::NONE,
                    } => self
                        .selected_key()
                        .map(|key| Msg::VariableManager(VariableManagerMsg::Edit(key))),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
