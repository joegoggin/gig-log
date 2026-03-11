use tui_realm_stdlib::List;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, TextSpan};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::api_tester::app::Msg;
use crate::api_tester::variables::VariableMode;

pub struct VariableTable {
    component: List,
    keys: Vec<String>,
    pending_g: bool,
}

impl VariableTable {
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
                TextSpan::new("No variables. Press 'a' to add one.").fg(Color::DarkGray)
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

    fn selected_key(&self) -> Option<String> {
        if let State::One(StateValue::Usize(index)) = self.state() {
            self.keys.get(index).cloned()
        } else {
            None
        }
    }

    fn is_plain_g(key: &KeyEvent) -> bool {
        key.code == Key::Char('g') && key.modifiers == KeyModifiers::NONE
    }

    fn is_jump_to_end(key: &KeyEvent) -> bool {
        (key.code == Key::Char('G')
            && (key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT))
            || (key.code == Key::Char('g') && key.modifiers == KeyModifiers::SHIFT)
    }

    fn selection_changed_message(&self) -> Option<Msg> {
        self.selected_key().map(Msg::VariableSelectionChanged)
    }
}

impl MockComponent for VariableTable {
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

impl Component<Msg, NoUserEvent> for VariableTable {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key) => {
                if self.pending_g {
                    self.pending_g = false;

                    if Self::is_plain_g(&key) {
                        self.perform(Cmd::GoTo(Position::Begin));
                        return self.selection_changed_message();
                    }
                }

                if Self::is_plain_g(&key) {
                    self.pending_g = true;
                    return None;
                }

                if Self::is_jump_to_end(&key) {
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
                    } => Some(Msg::AddVariable),
                    KeyEvent {
                        code: Key::Char('d'),
                        modifiers: KeyModifiers::NONE,
                    } => self.selected_key().map(Msg::DeleteVariable),
                    KeyEvent {
                        code: Key::Char('e'),
                        modifiers: KeyModifiers::NONE,
                    } => self.selected_key().map(Msg::EditVariable),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_delete_uses_selected_key() {
        let entries = vec![
            (
                "API_TOKEN".to_string(),
                "secret".to_string(),
                VariableMode::Hidden,
                "scoped".to_string(),
            ),
            (
                "API_HOST".to_string(),
                "https://example.com".to_string(),
                VariableMode::Placeholder,
                "global".to_string(),
            ),
        ];
        let mut table = VariableTable::new(&entries, false);

        table.on(Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));

        assert_eq!(
            table.on(Event::Keyboard(KeyEvent::new(
                Key::Char('d'),
                KeyModifiers::NONE,
            ))),
            Some(Msg::DeleteVariable("API_HOST".to_string()))
        );
    }

    #[test]
    fn empty_table_does_not_emit_edit_or_delete_messages() {
        let mut table = VariableTable::new(&[], false);

        assert_eq!(
            table.on(Event::Keyboard(KeyEvent::new(
                Key::Char('d'),
                KeyModifiers::NONE,
            ))),
            None
        );
        assert_eq!(
            table.on(Event::Keyboard(KeyEvent::new(
                Key::Char('e'),
                KeyModifiers::NONE,
            ))),
            None
        );
    }

    #[test]
    fn moving_selection_emits_variable_selection_changed_message() {
        let entries = vec![
            (
                "API_TOKEN".to_string(),
                "secret".to_string(),
                VariableMode::Hidden,
                "scoped".to_string(),
            ),
            (
                "API_HOST".to_string(),
                "https://example.com".to_string(),
                VariableMode::Placeholder,
                "global".to_string(),
            ),
        ];
        let mut table = VariableTable::new(&entries, false);

        assert_eq!(
            table.on(Event::Keyboard(KeyEvent::new(
                Key::Down,
                KeyModifiers::NONE,
            ))),
            Some(Msg::VariableSelectionChanged("API_HOST".to_string()))
        );
    }
}
