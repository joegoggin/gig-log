use tui_realm_stdlib::Radio;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, Style, TextModifiers};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::api_tester::app::{Id, InputMode, Msg};
use crate::api_tester::variables::VariableMode;

const MODE_CHOICES: [&str; 2] = ["placeholder", "hidden"];

pub struct VariableModeSelector {
    component: Radio,
    input_mode: InputMode,
}

impl VariableModeSelector {
    pub fn new(mode: VariableMode) -> Self {
        let selected = match mode {
            VariableMode::Placeholder => 0,
            VariableMode::Hidden => 1,
        };

        let component = Radio::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Cyan),
            )
            .title("Mode", Alignment::Left)
            .choices(MODE_CHOICES)
            .value(selected);

        Self {
            component,
            input_mode: InputMode::Normal,
        }
    }

    fn move_selection(&mut self, direction: Direction) -> Option<Msg> {
        self.perform(Cmd::Move(direction));

        let mode = match self.state() {
            State::One(StateValue::Usize(1)) => VariableMode::Hidden,
            _ => VariableMode::Placeholder,
        };

        Some(Msg::VariableModeChanged(mode))
    }
}

impl MockComponent for VariableModeSelector {
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
        let selected = if let State::One(StateValue::Usize(index)) = self.state() {
            index
        } else {
            0
        };

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
            MODE_CHOICES
                .iter()
                .map(|choice| ratatui::text::Line::from(*choice))
                .collect::<Vec<_>>(),
        )
        .block(block)
        .select(selected)
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

impl Component<Msg, NoUserEvent> for VariableModeSelector {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                Some(Msg::FocusField(Id::VariableKeyInput))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab, ..
            }) => Some(Msg::FocusField(Id::VariableValueInput)),
            Event::Keyboard(KeyEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            }) => Some(Msg::SaveVariable),
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) if self.input_mode == InputMode::Insert => self.move_selection(Direction::Left),
            Event::Keyboard(KeyEvent {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) if self.input_mode == InputMode::Insert => self.move_selection(Direction::Right),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selected_index(selector: &VariableModeSelector) -> usize {
        if let State::One(tuirealm::StateValue::Usize(index)) = selector.state() {
            index
        } else {
            0
        }
    }

    #[test]
    fn insert_mode_changes_selected_value() {
        let mut selector = VariableModeSelector::new(VariableMode::Placeholder);
        selector.attr(Attribute::Custom("input_mode"), AttrValue::Flag(true));

        assert_eq!(
            selector.on(Event::Keyboard(KeyEvent::new(
                Key::Right,
                KeyModifiers::NONE,
            ))),
            Some(Msg::VariableModeChanged(VariableMode::Hidden))
        );

        assert_eq!(selected_index(&selector), 1);
    }

    #[test]
    fn ctrl_s_emits_save_variable_message() {
        let mut selector = VariableModeSelector::new(VariableMode::Placeholder);

        assert_eq!(
            selector.on(Event::Keyboard(KeyEvent::new(
                Key::Char('s'),
                KeyModifiers::CONTROL,
            ))),
            Some(Msg::SaveVariable)
        );
    }

    #[test]
    fn enter_does_not_emit_save_variable_message() {
        let mut selector = VariableModeSelector::new(VariableMode::Placeholder);

        assert_eq!(
            selector.on(Event::Keyboard(KeyEvent::new(
                Key::Enter,
                KeyModifiers::NONE,
            ))),
            None
        );
    }
}
