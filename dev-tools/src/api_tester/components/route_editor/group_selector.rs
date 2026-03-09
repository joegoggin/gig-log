use tui_realm_stdlib::Radio;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::api_tester::app::{Id, Msg};

pub struct EditorGroupSelector {
    component: Radio,
    group_names: Vec<String>,
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
        }
    }

    pub fn is_new_group_selected(&self) -> bool {
        if let State::One(StateValue::Usize(i)) = self.state() {
            i >= self.group_names.len()
        } else {
            false
        }
    }
}

impl MockComponent for EditorGroupSelector {
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        self.component.view(frame, area);
    }
    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.component.query(attr)
    }
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
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
            }) => {
                self.perform(Cmd::Move(Direction::Left));
                if let State::One(StateValue::Usize(index)) = self.state() {
                    Some(Msg::GroupSelected(index))
                } else {
                    None
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => {
                self.perform(Cmd::Move(Direction::Right));
                if let State::One(StateValue::Usize(index)) = self.state() {
                    Some(Msg::GroupSelected(index))
                } else {
                    None
                }
            }
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
    fn vim_keys_move_group_selection() {
        let groups = sample_groups();
        let mut selector = EditorGroupSelector::new(&groups, "group-b");

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
    fn arrow_keys_still_move_group_selection() {
        let groups = sample_groups();
        let mut selector = EditorGroupSelector::new(&groups, "group-b");

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
