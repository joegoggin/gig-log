use tui_realm_stdlib::Radio;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, Style, TextModifiers};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::api_tester::app::{Id, InputMode, Msg};
use crate::api_tester::collection::HttpMethod;

const METHOD_CHOICES: [&str; 5] = ["GET", "POST", "PUT", "PATCH", "DELETE"];

pub struct EditorMethodRadio {
    component: Radio,
    input_mode: InputMode,
}

impl EditorMethodRadio {
    pub fn new(selected: &HttpMethod) -> Self {
        let index = match selected {
            HttpMethod::Get => 0,
            HttpMethod::Post => 1,
            HttpMethod::Put => 2,
            HttpMethod::Patch => 3,
            HttpMethod::Delete => 4,
        };

        let component = Radio::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Cyan),
            )
            .title("Method", Alignment::Left)
            .choices(METHOD_CHOICES)
            .value(index);

        Self {
            component,
            input_mode: InputMode::Normal,
        }
    }

    fn move_selection(&mut self, direction: Direction) -> Option<Msg> {
        self.perform(Cmd::Move(direction));

        if let State::One(StateValue::Usize(index)) = self.state() {
            Some(Msg::MethodChanged(index))
        } else {
            None
        }
    }
}

impl MockComponent for EditorMethodRadio {
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
            METHOD_CHOICES
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

impl Component<Msg, NoUserEvent> for EditorMethodRadio {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                Some(Msg::FocusField(Id::EditorUrl))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab, ..
            }) => Some(Msg::FocusField(Id::EditorGroup)),
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

    fn selected_index(radio: &EditorMethodRadio) -> usize {
        if let State::One(StateValue::Usize(index)) = radio.state() {
            index
        } else {
            panic!("method radio should always have a selection");
        }
    }

    #[test]
    fn normal_mode_ignores_method_selection_keys() {
        let mut radio = EditorMethodRadio::new(&HttpMethod::Put);

        assert_eq!(selected_index(&radio), 2);
        assert_eq!(
            radio.on(Event::Keyboard(KeyEvent::new(
                Key::Char('h'),
                KeyModifiers::NONE,
            ))),
            None
        );
        assert_eq!(selected_index(&radio), 2);

        assert_eq!(
            radio.on(Event::Keyboard(KeyEvent::new(
                Key::Left,
                KeyModifiers::NONE
            ))),
            None
        );
        assert_eq!(selected_index(&radio), 2);
    }

    #[test]
    fn vim_keys_move_method_selection_in_insert_mode() {
        let mut radio = EditorMethodRadio::new(&HttpMethod::Put);
        radio.attr(Attribute::Custom("input_mode"), AttrValue::Flag(true));

        assert_eq!(selected_index(&radio), 2);
        assert_eq!(
            radio.on(Event::Keyboard(KeyEvent::new(
                Key::Char('h'),
                KeyModifiers::NONE,
            ))),
            Some(Msg::MethodChanged(1))
        );
        assert_eq!(selected_index(&radio), 1);

        assert_eq!(
            radio.on(Event::Keyboard(KeyEvent::new(
                Key::Char('l'),
                KeyModifiers::NONE,
            ))),
            Some(Msg::MethodChanged(2))
        );
        assert_eq!(selected_index(&radio), 2);
    }

    #[test]
    fn arrow_keys_move_method_selection_in_insert_mode() {
        let mut radio = EditorMethodRadio::new(&HttpMethod::Put);
        radio.attr(Attribute::Custom("input_mode"), AttrValue::Flag(true));

        assert_eq!(selected_index(&radio), 2);
        assert_eq!(
            radio.on(Event::Keyboard(KeyEvent::new(
                Key::Left,
                KeyModifiers::NONE
            ))),
            Some(Msg::MethodChanged(1))
        );
        assert_eq!(selected_index(&radio), 1);

        assert_eq!(
            radio.on(Event::Keyboard(KeyEvent::new(
                Key::Right,
                KeyModifiers::NONE
            ))),
            Some(Msg::MethodChanged(2))
        );
        assert_eq!(selected_index(&radio), 2);
    }
}
