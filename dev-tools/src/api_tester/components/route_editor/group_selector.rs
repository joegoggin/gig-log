use tui_realm_stdlib::Radio;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, Style, TextModifiers};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::api_tester::app::{Id, InputMode, Msg};

pub struct EditorGroupSelector {
    component: Radio,
    group_names: Vec<String>,
    input_mode: InputMode,
}

impl EditorGroupSelector {
    pub fn new(group_names: &[String], current_group: &str) -> Self {
        let mut choices: Vec<String> = group_names.to_vec();
        choices.push("New Group...".to_string());

        let index = group_names
            .iter()
            .position(|g| g == current_group)
            .unwrap_or(0);

        let choice_refs: Vec<&str> = choices.iter().map(|s| s.as_str()).collect();
        let component = Radio::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Cyan),
            )
            .title("Group", Alignment::Left)
            .choices(choice_refs)
            .value(index);

        Self {
            component,
            group_names: group_names.to_vec(),
            input_mode: InputMode::Normal,
        }
    }

    pub fn is_new_group_selected(&self) -> bool {
        if let State::One(StateValue::Usize(i)) = self.state() {
            i >= self.group_names.len()
        } else {
            false
        }
    }

    fn move_selection(&mut self, direction: Direction) -> Option<Msg> {
        self.perform(Cmd::Move(direction));

        if let State::One(StateValue::Usize(index)) = self.state() {
            Some(Msg::GroupSelected(index))
        } else {
            None
        }
    }
}

impl MockComponent for EditorGroupSelector {
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

        let mut choices = self.group_names.clone();
        choices.push("New Group...".to_string());

        let tabs = ratatui::widgets::Tabs::new(
            choices
                .iter()
                .map(|choice| ratatui::text::Line::from(choice.as_str()))
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

impl Component<Msg, NoUserEvent> for EditorGroupSelector {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                Some(Msg::FocusField(Id::EditorMethod))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab, ..
            }) => Some(Msg::FocusField(Id::EditorName)),
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

    fn selected_index(selector: &EditorGroupSelector) -> usize {
        if let State::One(StateValue::Usize(index)) = selector.state() {
            index
        } else {
            panic!("group selector should always have a selection");
        }
    }

    fn sample_groups() -> Vec<String> {
        vec![
            "group-a".to_string(),
            "group-b".to_string(),
            "group-c".to_string(),
        ]
    }

    #[test]
    fn normal_mode_ignores_group_selection_keys() {
        let groups = sample_groups();
        let mut selector = EditorGroupSelector::new(&groups, "group-b");

        assert_eq!(selected_index(&selector), 1);
        assert_eq!(
            selector.on(Event::Keyboard(KeyEvent::new(
                Key::Char('h'),
                KeyModifiers::NONE,
            ))),
            None
        );
        assert_eq!(selected_index(&selector), 1);

        assert_eq!(
            selector.on(Event::Keyboard(KeyEvent::new(
                Key::Left,
                KeyModifiers::NONE
            ))),
            None
        );
        assert_eq!(selected_index(&selector), 1);
    }

    #[test]
    fn vim_keys_move_group_selection_in_insert_mode() {
        let groups = sample_groups();
        let mut selector = EditorGroupSelector::new(&groups, "group-b");
        selector.attr(Attribute::Custom("input_mode"), AttrValue::Flag(true));

        assert_eq!(selected_index(&selector), 1);
        assert_eq!(
            selector.on(Event::Keyboard(KeyEvent::new(
                Key::Char('h'),
                KeyModifiers::NONE,
            ))),
            Some(Msg::GroupSelected(0))
        );
        assert_eq!(selected_index(&selector), 0);

        assert_eq!(
            selector.on(Event::Keyboard(KeyEvent::new(
                Key::Char('l'),
                KeyModifiers::NONE,
            ))),
            Some(Msg::GroupSelected(1))
        );
        assert_eq!(selected_index(&selector), 1);
    }

    #[test]
    fn arrow_keys_move_group_selection_in_insert_mode() {
        let groups = sample_groups();
        let mut selector = EditorGroupSelector::new(&groups, "group-b");
        selector.attr(Attribute::Custom("input_mode"), AttrValue::Flag(true));

        assert_eq!(selected_index(&selector), 1);
        assert_eq!(
            selector.on(Event::Keyboard(KeyEvent::new(
                Key::Left,
                KeyModifiers::NONE
            ))),
            Some(Msg::GroupSelected(0))
        );
        assert_eq!(selected_index(&selector), 0);

        assert_eq!(
            selector.on(Event::Keyboard(KeyEvent::new(
                Key::Right,
                KeyModifiers::NONE
            ))),
            Some(Msg::GroupSelected(1))
        );
        assert_eq!(selected_index(&selector), 1);
    }
}
