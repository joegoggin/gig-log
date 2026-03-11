use ratatui::{layout::Rect, Frame};
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind},
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State,
};

use crate::api_tester::app::{ActiveView, InputMode, Msg};

pub struct GlobalListener {
    input_mode: InputMode,
    touch_drag_row: Option<u16>,
    pending_g: bool,
}

impl GlobalListener {
    const MAX_DRAG_STEP: isize = 3;

    pub fn new() -> Self {
        Self {
            input_mode: InputMode::Normal,
            touch_drag_row: None,
            pending_g: false,
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

    fn map_normal_key(&mut self, key: KeyEvent) -> Option<Msg> {
        if self.pending_g {
            self.pending_g = false;

            if Self::is_plain_g(&key) {
                return Some(Msg::PreviewScrollToTop);
            }
        }

        if Self::is_plain_g(&key) {
            self.pending_g = true;
            return None;
        }

        if Self::is_jump_to_end(&key) {
            return Some(Msg::PreviewScrollToBottom);
        }

        match key.code {
            Key::Char('q') => Some(Msg::AppClose),
            Key::Char('v') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Msg::OpenScopedVariables)
            }
            Key::Char('V') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Msg::OpenScopedVariables)
            }
            Key::Char('v') => Some(Msg::SwitchView(ActiveView::VariableManager)),
            Key::Char('i') => Some(Msg::EnterInsertMode),
            Key::Esc => Some(Msg::CancelEdit),
            Key::Char('s') if key.modifiers == KeyModifiers::CONTROL => Some(Msg::SaveRoute),
            Key::Char('s') if key.modifiers == KeyModifiers::NONE => {
                Some(Msg::ToggleSecretVisibility)
            }
            Key::Char('b') => Some(Msg::OpenBodyEditor),
            Key::Char('r') => Some(Msg::ExecutePreviewRequest),
            Key::Char('k') | Key::Up => Some(Msg::EditorScrollUp),
            Key::Char('j') | Key::Down => Some(Msg::EditorScrollDown),
            Key::PageUp => Some(Msg::EditorPageUp),
            Key::PageDown => Some(Msg::EditorPageDown),
            _ => None,
        }
    }

    fn map_insert_key(key: KeyEvent) -> Option<Msg> {
        match key.code {
            Key::Esc => Some(Msg::EnterNormalMode),
            _ => None, // All other keys pass through to focused component
        }
    }

    fn map_mouse(&mut self, mouse: MouseEvent) -> Option<Msg> {
        self.pending_g = false;

        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.touch_drag_row = None;
                Some(Msg::EditorScrollBy(-1))
            }
            MouseEventKind::ScrollDown => {
                self.touch_drag_row = None;
                Some(Msg::EditorScrollBy(1))
            }
            MouseEventKind::Down(MouseButton::Left) => {
                self.touch_drag_row = Some(mouse.row);
                None
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                let Some(last_row) = self.touch_drag_row else {
                    self.touch_drag_row = Some(mouse.row);
                    return None;
                };

                self.touch_drag_row = Some(mouse.row);
                let delta = last_row as isize - mouse.row as isize;
                if delta == 0 {
                    return None;
                }

                Some(Msg::EditorScrollBy(
                    delta.clamp(-Self::MAX_DRAG_STEP, Self::MAX_DRAG_STEP),
                ))
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.touch_drag_row = None;
                None
            }
            MouseEventKind::Down(_) | MouseEventKind::Up(_) => {
                self.touch_drag_row = None;
                None
            }
            MouseEventKind::Drag(_) | MouseEventKind::Moved => None,
            MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight => None,
        }
    }
}

impl MockComponent for GlobalListener {
    fn view(&mut self, _frame: &mut Frame, _area: Rect) {}

    fn query(&self, _attr: Attribute) -> Option<AttrValue> {
        None
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
        }
    }

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }
}

impl Component<Msg, NoUserEvent> for GlobalListener {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key) => match self.input_mode {
                InputMode::Normal => self.map_normal_key(key),
                InputMode::Insert => Self::map_insert_key(key),
            },
            Event::Mouse(mouse) => self.map_mouse(mouse),
            Event::WindowResize(width, height) => Some(Msg::TerminalResize(width, height)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mouse_event(kind: MouseEventKind, row: u16) -> Event<NoUserEvent> {
        Event::Mouse(MouseEvent {
            kind,
            modifiers: KeyModifiers::NONE,
            column: 0,
            row,
        })
    }

    fn key_event(code: Key, modifiers: KeyModifiers) -> Event<NoUserEvent> {
        Event::Keyboard(KeyEvent::new(code, modifiers))
    }

    #[test]
    fn wheel_events_map_to_scroll_messages() {
        let mut listener = GlobalListener::new();

        assert_eq!(
            listener.on(mouse_event(MouseEventKind::ScrollUp, 0)),
            Some(Msg::EditorScrollBy(-1))
        );
        assert_eq!(
            listener.on(mouse_event(MouseEventKind::ScrollDown, 0)),
            Some(Msg::EditorScrollBy(1))
        );
    }

    #[test]
    fn drag_events_map_to_phone_style_scroll_direction() {
        let mut listener = GlobalListener::new();

        assert_eq!(
            listener.on(mouse_event(MouseEventKind::Down(MouseButton::Left), 10)),
            None
        );
        assert_eq!(
            listener.on(mouse_event(MouseEventKind::Drag(MouseButton::Left), 8)),
            Some(Msg::EditorScrollBy(2))
        );
        assert_eq!(
            listener.on(mouse_event(MouseEventKind::Drag(MouseButton::Left), 12)),
            Some(Msg::EditorScrollBy(-3))
        );
    }

    #[test]
    fn drag_state_resets_on_left_button_release() {
        let mut listener = GlobalListener::new();

        listener.on(mouse_event(MouseEventKind::Down(MouseButton::Left), 10));
        listener.on(mouse_event(MouseEventKind::Up(MouseButton::Left), 10));

        assert_eq!(
            listener.on(mouse_event(MouseEventKind::Drag(MouseButton::Left), 7)),
            None
        );
        assert_eq!(
            listener.on(mouse_event(MouseEventKind::Drag(MouseButton::Left), 5)),
            Some(Msg::EditorScrollBy(2))
        );
    }

    #[test]
    fn gg_maps_to_top_and_uppercase_g_maps_to_bottom() {
        let mut listener = GlobalListener::new();

        assert_eq!(
            listener.on(key_event(Key::Char('g'), KeyModifiers::NONE)),
            None
        );
        assert_eq!(
            listener.on(key_event(Key::Char('g'), KeyModifiers::NONE)),
            Some(Msg::PreviewScrollToTop)
        );

        assert_eq!(
            listener.on(key_event(Key::Char('G'), KeyModifiers::SHIFT)),
            Some(Msg::PreviewScrollToBottom)
        );
    }

    #[test]
    fn uppercase_v_opens_scoped_variable_manager() {
        let mut listener = GlobalListener::new();

        assert_eq!(
            listener.on(key_event(Key::Char('V'), KeyModifiers::SHIFT)),
            Some(Msg::OpenScopedVariables)
        );
    }

    #[test]
    fn s_toggles_secret_visibility_in_normal_mode() {
        let mut listener = GlobalListener::new();

        assert_eq!(
            listener.on(key_event(Key::Char('s'), KeyModifiers::NONE)),
            Some(Msg::ToggleSecretVisibility)
        );
    }

    #[test]
    fn s_does_not_toggle_secret_visibility_in_insert_mode() {
        let mut listener = GlobalListener::new();
        listener.attr(Attribute::Custom("input_mode"), AttrValue::Flag(true));

        assert_eq!(
            listener.on(key_event(Key::Char('s'), KeyModifiers::NONE)),
            None
        );
    }
}
