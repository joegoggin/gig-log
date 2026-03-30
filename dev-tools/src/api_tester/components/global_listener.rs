//! Global input listener for API tester shortcuts.
//!
//! This component converts keyboard, mouse, and resize events into high-level
//! application messages while honoring the current input mode.

use ratatui::{Frame, layout::Rect};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State,
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind},
};

use crate::api_tester::app::{
    ActiveView, AppMsg, InputMode, Msg, RequestPreviewMsg, RouteEditorMsg,
};
use crate::api_tester::components::core::keymap::{is_jump_to_end, is_plain_g};

/// Global event mapper for normal/insert mode interactions.
pub struct GlobalListener {
    /// Current application input mode.
    input_mode: InputMode,
    /// Last drag row used for mouse drag scrolling.
    touch_drag_row: Option<u16>,
    /// Tracks pending first `g` for `gg` keymap support.
    pending_g: bool,
}

impl GlobalListener {
    /// Maximum number of rows moved per drag event.
    const MAX_DRAG_STEP: isize = 3;

    /// Creates a new global listener in normal mode.
    ///
    /// # Returns
    ///
    /// A [`GlobalListener`] with default key-sequence state.
    pub fn new() -> Self {
        Self {
            input_mode: InputMode::Normal,
            touch_drag_row: None,
            pending_g: false,
        }
    }

    /// Maps normal-mode keyboard input to application messages.
    ///
    /// # Arguments
    ///
    /// * `key` — Keyboard event from the terminal listener.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing a mapped [`Msg`] when applicable.
    fn map_normal_key(&mut self, key: KeyEvent) -> Option<Msg> {
        if self.pending_g {
            self.pending_g = false;

            if is_plain_g(&key) {
                return Some(Msg::App(AppMsg::ScrollToTop));
            }
        }

        if is_plain_g(&key) {
            self.pending_g = true;
            return None;
        }

        if is_jump_to_end(&key) {
            return Some(Msg::App(AppMsg::ScrollToBottom));
        }

        match key.code {
            Key::Char('q') => Some(Msg::App(AppMsg::Close)),
            Key::Char('?') => Some(Msg::App(AppMsg::ToggleKeymapHelp)),
            Key::Char('H') => Some(Msg::App(AppMsg::NavigateBack)),
            Key::Char('L') => Some(Msg::App(AppMsg::NavigateForward)),
            Key::Char('h') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Msg::App(AppMsg::NavigateBack))
            }
            Key::Char('l') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Msg::App(AppMsg::NavigateForward))
            }
            Key::Char('v') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Msg::App(AppMsg::OpenScopedVariables))
            }
            Key::Char('V') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Msg::App(AppMsg::OpenScopedVariables))
            }
            Key::Char('v') => Some(Msg::App(AppMsg::SwitchView(ActiveView::VariableManager))),
            Key::Char('i') => Some(Msg::App(AppMsg::EnterInsertMode)),
            Key::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                Some(Msg::RouteEditor(RouteEditorMsg::Save))
            }
            Key::Char('s') if key.modifiers == KeyModifiers::NONE => {
                Some(Msg::App(AppMsg::ToggleSecretVisibility))
            }
            Key::Char('b') => Some(Msg::App(AppMsg::OpenBodyEditor)),
            Key::Char('r') => Some(Msg::RequestPreview(RequestPreviewMsg::Execute)),
            Key::Char('k') | Key::Up => Some(Msg::App(AppMsg::ScrollUp)),
            Key::Char('j') | Key::Down => Some(Msg::App(AppMsg::ScrollDown)),
            Key::PageUp => Some(Msg::App(AppMsg::PageUp)),
            Key::PageDown => Some(Msg::App(AppMsg::PageDown)),
            _ => None,
        }
    }

    /// Maps insert-mode keyboard input to application messages.
    ///
    /// # Arguments
    ///
    /// * `key` — Keyboard event from the terminal listener.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing a mapped [`Msg`] when applicable.
    fn map_insert_key(key: KeyEvent) -> Option<Msg> {
        match key.code {
            Key::Esc => Some(Msg::App(AppMsg::EnterNormalMode)),
            _ => None,
        }
    }

    /// Maps mouse events to scroll-related application messages.
    ///
    /// # Arguments
    ///
    /// * `mouse` — Mouse event from the terminal listener.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing a mapped [`Msg`] when applicable.
    fn map_mouse(&mut self, mouse: MouseEvent) -> Option<Msg> {
        self.pending_g = false;

        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.touch_drag_row = None;
                Some(Msg::App(AppMsg::ScrollBy(-1)))
            }
            MouseEventKind::ScrollDown => {
                self.touch_drag_row = None;
                Some(Msg::App(AppMsg::ScrollBy(1)))
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

                Some(Msg::App(AppMsg::ScrollBy(
                    delta.clamp(-Self::MAX_DRAG_STEP, Self::MAX_DRAG_STEP),
                )))
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
    /// Renders the listener component.
    ///
    /// This component is non-visual and intentionally renders nothing.
    ///
    /// # Arguments
    ///
    /// * `_frame` — Frame to render into.
    /// * `_area` — Area allocated to the widget.
    fn view(&mut self, _frame: &mut Frame, _area: Rect) {}

    /// Queries a widget attribute value.
    ///
    /// # Arguments
    ///
    /// * `_attr` — Attribute to query.
    ///
    /// # Returns
    ///
    /// An empty [`Option`] because this component has no queryable attributes.
    fn query(&self, _attr: Attribute) -> Option<AttrValue> {
        None
    }

    /// Updates component attributes.
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
        }
    }

    /// Returns the current component state.
    ///
    /// # Returns
    ///
    /// A [`State::None`] value because this component is stateless.
    fn state(&self) -> State {
        State::None
    }

    /// Executes a command against the component.
    ///
    /// # Arguments
    ///
    /// * `_cmd` — Command to execute.
    ///
    /// # Returns
    ///
    /// A [`CmdResult::None`] value because this component has no command
    /// handling.
    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }
}

impl Component<Msg, NoUserEvent> for GlobalListener {
    /// Maps a terminal event into an application message.
    ///
    /// # Arguments
    ///
    /// * `ev` — Incoming terminal event.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing the mapped application [`Msg`].
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key) => match self.input_mode {
                InputMode::Normal => self.map_normal_key(key),
                InputMode::Insert => Self::map_insert_key(key),
            },
            Event::Mouse(mouse) => self.map_mouse(mouse),
            Event::WindowResize(width, height) => {
                Some(Msg::App(AppMsg::TerminalResize(width, height)))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_h_to_navigate_back() {
        let mut listener = GlobalListener::new();

        let msg = listener.map_normal_key(KeyEvent::new(Key::Char('H'), KeyModifiers::SHIFT));

        assert_eq!(msg, Some(Msg::App(AppMsg::NavigateBack)));
    }

    #[test]
    fn maps_l_to_navigate_forward() {
        let mut listener = GlobalListener::new();

        let msg = listener.map_normal_key(KeyEvent::new(Key::Char('L'), KeyModifiers::SHIFT));

        assert_eq!(msg, Some(Msg::App(AppMsg::NavigateForward)));
    }

    #[test]
    fn esc_does_not_navigate_in_normal_mode() {
        let mut listener = GlobalListener::new();

        let msg = listener.map_normal_key(KeyEvent::new(Key::Esc, KeyModifiers::NONE));

        assert_eq!(msg, None);
    }

    #[test]
    fn esc_exits_insert_mode() {
        let msg = GlobalListener::map_insert_key(KeyEvent::new(Key::Esc, KeyModifiers::NONE));

        assert_eq!(msg, Some(Msg::App(AppMsg::EnterNormalMode)));
    }
}
