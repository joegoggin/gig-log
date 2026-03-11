use tui_realm_stdlib::Radio;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, Style, TextModifiers};
use tuirealm::{AttrValue, Attribute, Event, MockComponent, NoUserEvent, State, StateValue};

use crate::api_tester::app::{InputMode, Msg};

#[derive(Debug, Clone)]
pub struct RadioBindings {
    pub on_tab: Option<Msg>,
    pub on_backtab: Option<Msg>,
    pub on_ctrl_s: Option<Msg>,
}

impl RadioBindings {
    pub fn tab_cycle(on_tab: Msg, on_backtab: Msg) -> Self {
        Self {
            on_tab: Some(on_tab),
            on_backtab: Some(on_backtab),
            on_ctrl_s: None,
        }
    }
}

pub struct CoreTabRadio {
    component: Radio,
    choices: Vec<String>,
    input_mode: InputMode,
}

impl CoreTabRadio {
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

    pub fn selected_index(&self) -> usize {
        match self.state() {
            State::One(StateValue::Usize(index)) => index,
            _ => 0,
        }
    }

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

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.component.query(attr)
    }

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

    fn state(&self) -> State {
        self.component.state()
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        self.component.perform(cmd)
    }
}
