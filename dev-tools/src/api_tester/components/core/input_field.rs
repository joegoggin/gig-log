//! Text input wrapper with API tester key bindings.

use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType};
use tuirealm::{AttrValue, Attribute, Event, MockComponent, NoUserEvent, State};

use crate::api_tester::app::{InputMode, Msg};

/// Key binding configuration for [`CoreInputField`] events.
#[derive(Debug, Clone)]
pub struct InputBindings {
    /// Message emitted when tab is pressed.
    pub on_tab: Option<Msg>,
    /// Message emitted when reverse-tab is pressed.
    pub on_backtab: Option<Msg>,
    /// Message emitted when control-s is pressed.
    pub on_ctrl_s: Option<Msg>,
    /// Message emitted when enter is pressed in insert mode.
    pub on_insert_enter: Option<Msg>,
}

impl InputBindings {
    /// Creates tab-cycle bindings for forward and reverse focus movement.
    ///
    /// # Arguments
    ///
    /// * `on_tab` — Message emitted for tab.
    /// * `on_backtab` — Message emitted for reverse-tab.
    ///
    /// # Returns
    ///
    /// An [`InputBindings`] value with tab handlers configured.
    pub fn tab_cycle(on_tab: Msg, on_backtab: Msg) -> Self {
        Self {
            on_tab: Some(on_tab),
            on_backtab: Some(on_backtab),
            on_ctrl_s: None,
            on_insert_enter: None,
        }
    }
}

/// Reusable single-line text input component with insert/normal behavior.
pub struct CoreInputField {
    /// Underlying standard input component.
    component: Input,
    /// Current input mode used for key handling.
    input_mode: InputMode,
}

impl CoreInputField {
    /// Creates a core input field with title, value, and border styling.
    ///
    /// # Arguments
    ///
    /// * `title` — Field title text.
    /// * `value` — Initial input value.
    /// * `border_color` — Border color for the field widget.
    ///
    /// # Returns
    ///
    /// A configured [`CoreInputField`] instance.
    pub fn new(title: &str, value: &str, border_color: Color) -> Self {
        let component = Input::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(border_color),
            )
            .title(title, Alignment::Left)
            .input_type(InputType::Text)
            .value(value);

        Self {
            component,
            input_mode: InputMode::Normal,
        }
    }

    /// Handles an input event using configured key bindings.
    ///
    /// # Arguments
    ///
    /// * `ev` — Incoming UI event.
    /// * `bindings` — Key binding configuration.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing an emitted application [`Msg`].
    pub fn on_event(&mut self, ev: Event<NoUserEvent>, bindings: &InputBindings) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => bindings.on_tab.clone(),
            Event::Keyboard(KeyEvent {
                code: Key::BackTab, ..
            }) => bindings.on_backtab.clone(),
            Event::Keyboard(KeyEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            }) => bindings.on_ctrl_s.clone(),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) if self.input_mode == InputMode::Insert => bindings.on_insert_enter.clone(),
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) if self.input_mode == InputMode::Normal => {
                self.perform(Cmd::Move(Direction::Left));
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) if self.input_mode == InputMode::Normal => {
                self.perform(Cmd::Move(Direction::Right));
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::SHIFT,
            }) if self.input_mode == InputMode::Insert => {
                self.perform(Cmd::Type(ch));
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) if self.input_mode == InputMode::Insert => {
                self.perform(Cmd::Delete);
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) if self.input_mode == InputMode::Insert => {
                self.perform(Cmd::Cancel);
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) if self.input_mode == InputMode::Insert => {
                self.perform(Cmd::Move(Direction::Left));
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) if self.input_mode == InputMode::Insert => {
                self.perform(Cmd::Move(Direction::Right));
                None
            }
            _ => None,
        }
    }
}

impl MockComponent for CoreInputField {
    /// Renders the input field widget.
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
        if attr == Attribute::Custom("input_mode") {
            if let AttrValue::Flag(is_insert) = value {
                self.input_mode = if is_insert {
                    InputMode::Insert
                } else {
                    InputMode::Normal
                };
            }
            return;
        }

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
