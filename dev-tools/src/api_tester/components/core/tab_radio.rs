//! Tab-style radio selector wrapper with API tester key bindings.

use tui_realm_stdlib::Radio;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, Style, TextModifiers};
use tuirealm::{AttrValue, Attribute, Event, MockComponent, NoUserEvent, State, StateValue};

use crate::api_tester::app::{InputMode, Msg};

/// Key binding configuration for [`CoreTabRadio`] events.
#[derive(Debug, Clone)]
pub struct RadioBindings {
    /// Message emitted when tab is pressed.
    pub on_tab: Option<Msg>,
    /// Message emitted when reverse-tab is pressed.
    pub on_backtab: Option<Msg>,
    /// Message emitted when control-s is pressed.
    pub on_ctrl_s: Option<Msg>,
}

impl RadioBindings {
    /// Creates tab-cycle bindings for forward and reverse focus movement.
    ///
    /// # Arguments
    ///
    /// * `on_tab` — Message emitted for tab.
    /// * `on_backtab` — Message emitted for reverse-tab.
    ///
    /// # Returns
    ///
    /// A [`RadioBindings`] value with tab handlers configured.
    pub fn tab_cycle(on_tab: Msg, on_backtab: Msg) -> Self {
        Self {
            on_tab: Some(on_tab),
            on_backtab: Some(on_backtab),
            on_ctrl_s: None,
        }
    }
}

/// Reusable tab-style radio component.
pub struct CoreTabRadio {
    /// Underlying standard radio component.
    component: Radio,
    /// Choice labels rendered as tabs.
    choices: Vec<String>,
    /// Current input mode used for key handling.
    input_mode: InputMode,
}

impl CoreTabRadio {
    /// Creates a tab-style radio selector.
    ///
    /// # Arguments
    ///
    /// * `title` — Widget title.
    /// * `choices` — Ordered choice labels.
    /// * `selected` — Initially selected index.
    /// * `border_color` — Border color for the widget.
    ///
    /// # Returns
    ///
    /// A configured [`CoreTabRadio`] instance.
    pub fn new(title: &str, choices: Vec<String>, selected: usize, border_color: Color) -> Self {
        let choice_refs: Vec<&str> = choices.iter().map(String::as_str).collect();
        let component = Radio::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(border_color),
            )
            .title(title, Alignment::Left)
            .choices(choice_refs)
            .value(selected);

        Self {
            component,
            choices,
            input_mode: InputMode::Normal,
        }
    }

    /// Returns the currently selected choice index.
    ///
    /// # Returns
    ///
    /// A [`usize`] selected index.
    pub fn selected_index(&self) -> usize {
        match self.state() {
            State::One(StateValue::Usize(index)) => index,
            _ => 0,
        }
    }

    /// Handles an input event and maps selection changes to messages.
    ///
    /// # Arguments
    ///
    /// * `ev` — Incoming UI event.
    /// * `bindings` — Key binding configuration.
    /// * `on_change` — Callback used to build a message from the new index.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing an emitted application [`Msg`].
    pub fn on_event<F>(
        &mut self,
        ev: Event<NoUserEvent>,
        bindings: &RadioBindings,
        on_change: F,
    ) -> Option<Msg>
    where
        F: Fn(usize) -> Msg,
    {
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
                code: Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) if self.input_mode == InputMode::Insert => {
                self.perform(Cmd::Move(Direction::Left));
                Some(on_change(self.selected_index()))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) if self.input_mode == InputMode::Insert => {
                self.perform(Cmd::Move(Direction::Right));
                Some(on_change(self.selected_index()))
            }
            _ => None,
        }
    }
}

impl MockComponent for CoreTabRadio {
    /// Renders the tab radio widget.
    ///
    /// # Arguments
    ///
    /// * `frame` — Frame to render into.
    /// * `area` — Area allocated to the widget.
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let display = self
            .component
            .query(Attribute::Display)
            .unwrap_or(AttrValue::Flag(true))
            .unwrap_flag();
        if !display {
            return;
        }

        let foreground = self
            .component
            .query(Attribute::Foreground)
            .unwrap_or(AttrValue::Color(Color::Reset))
            .unwrap_color();
        let background = self
            .component
            .query(Attribute::Background)
            .unwrap_or(AttrValue::Color(Color::Reset))
            .unwrap_color();
        let borders = self
            .component
            .query(Attribute::Borders)
            .unwrap_or(AttrValue::Borders(Borders::default()))
            .unwrap_borders();
        let title = self
            .component
            .query(Attribute::Title)
            .map(|value| value.unwrap_title());
        let focus = self
            .component
            .query(Attribute::Focus)
            .unwrap_or(AttrValue::Flag(false))
            .unwrap_flag();
        let inactive_style = self
            .component
            .query(Attribute::FocusStyle)
            .map(|value| value.unwrap_style());

        let mut block = ratatui::widgets::Block::default()
            .borders(borders.sides)
            .border_type(borders.modifiers)
            .border_style(if focus {
                borders.style()
            } else {
                inactive_style.unwrap_or_else(|| Style::default().fg(Color::Reset).bg(Color::Reset))
            });

        if let Some((title, alignment)) = title {
            block = block.title(title).title_alignment(alignment);
        }

        let tabs = ratatui::widgets::Tabs::new(
            self.choices
                .iter()
                .map(|choice| ratatui::text::Line::from(choice.as_str()))
                .collect::<Vec<_>>(),
        )
        .block(block)
        .select(self.selected_index())
        .style(Style::default().fg(foreground).bg(background))
        .highlight_style(
            Style::default()
                .fg(foreground)
                .add_modifier(TextModifiers::REVERSED),
        );

        frame.render_widget(tabs, area);
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
