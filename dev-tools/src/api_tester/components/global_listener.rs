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
            Key::Char('v') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Msg::App(AppMsg::OpenScopedVariables))
            }
            Key::Char('V') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Msg::App(AppMsg::OpenScopedVariables))
            }
            Key::Char('v') => Some(Msg::App(AppMsg::SwitchView(ActiveView::VariableManager))),
            Key::Char('i') => Some(Msg::App(AppMsg::EnterInsertMode)),
            Key::Esc => Some(Msg::App(AppMsg::Cancel)),
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

    fn map_insert_key(key: KeyEvent) -> Option<Msg> {
        match key.code {
            Key::Esc => Some(Msg::App(AppMsg::EnterNormalMode)),
            _ => None,
        }
    }

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
            Event::WindowResize(width, height) => {
                Some(Msg::App(AppMsg::TerminalResize(width, height)))
            }
            _ => None,
        }
    }
}
