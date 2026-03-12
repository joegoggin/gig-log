use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
};
use std::collections::{BTreeMap, BTreeSet};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{
    BorderType as InputBorderType, Borders as InputBorders, Color as InputColor,
    InputType as FieldInputType,
};
use tuirealm::{Application, AttrValue, Attribute, NoUserEvent, State, StateValue};

use crate::api_tester::components::response_viewer::details_view::ResponseDetailsView;
use crate::api_tester::components::route_editor::group_selector::EditorGroupSelector;
use crate::api_tester::components::route_editor::headers_input::EditorHeadersInput;
use crate::api_tester::components::route_editor::method_radio::EditorMethodRadio;
use crate::api_tester::components::route_editor::name_input::EditorNameInput;
use crate::api_tester::components::route_editor::new_group_input::EditorNewGroupInput;
use crate::api_tester::components::route_editor::url_input::EditorUrlInput;
use crate::api_tester::components::variable_manager::{
    VariableKeyInput, VariableModeSelector, VariableTable, VariableValueInput,
};
use crate::api_tester::{
    body_preview,
    collection::{Collection, DEFAULT_ROUTE_GROUP, HttpMethod, Route},
    components::{core::style as core_style, route_list::RouteList},
    executor::{CurlExecutor, CurlResponse},
    route_list_state::{RouteListState, RouteSelection, SelectedItem},
    variables::{VariableMode, Variables},
};
use crate::utils::sub::SubUtils;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    GlobalListener,
    RouteList,
    EditorName,
    EditorMethod,
    EditorUrl,
    EditorHeaders,
    ResponseDetails,
    VariableTable,
    VariableKeyInput,
    VariableValueInput,
    VariableMode,
    EditorGroup,
    EditorNewGroup,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActiveView {
    RouteList,
    RouteEditor,
    RequestPreview,
    ResponseViewer,
    VariableManager,
    KeymapHelp,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AppMsg {
    Close,
    SwitchView(ActiveView),
    NavigateBack,
    NavigateForward,
    OpenBodyEditor,
    ToggleSecretVisibility,
    OpenScopedVariables,
    TerminalResize(u16, u16),
    EnterInsertMode,
    EnterNormalMode,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    ScrollBy(isize),
    ScrollToTop,
    ScrollToBottom,
    ToggleKeymapHelp,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RouteListMsg {
    RunRoute(usize),
    EditRoute(usize),
    NewRoute,
    DeleteRoute(usize),
    StateChanged(RouteListState),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RouteEditorMsg {
    Save,
    FocusField(Id),
    MethodChanged(usize),
    GroupSelected(usize),
    NewGroupEntered,
    BodyEditorResult(Option<String>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RequestPreviewMsg {
    Execute,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResponseViewerMsg {
    RouteExecuted(usize, CurlResponse),
    RouteExecutionFailed(usize, String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResponseViewerResult {
    Success(CurlResponse),
    Error(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum VariableManagerMsg {
    Add,
    SelectionChanged(String),
    ModeChanged(VariableMode),
    Edit(String),
    Save,
    Delete(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Msg {
    App(AppMsg),
    RouteList(RouteListMsg),
    RouteEditor(RouteEditorMsg),
    RequestPreview(RequestPreviewMsg),
    ResponseViewer(ResponseViewerMsg),
    VariableManager(VariableManagerMsg),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AppEffect {
    Close,
    OpenBodyEditor,
    ExecutePreviewRequest,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LayoutMode {
    Wide,
    Medium,
    Narrow,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum EditorSectionKind {
    Name,
    Group,
    NewGroup,
    Method,
    Url,
    Headers,
    BodyStatus,
    BodyPreview,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct EditorSection {
    kind: EditorSectionKind,
    height: u16,
}

#[derive(Debug, Clone)]
struct RequestPreviewState {
    route_index: usize,
    route_scope_id: String,
    route_name: String,
    method: HttpMethod,
    display_url: String,
    execution_url: String,
    display_headers: Vec<String>,
    execution_headers: Vec<String>,
    display_body: Option<String>,
    execution_body: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum VariableContext {
    Global,
    Scoped { scope_id: String },
}

trait Screen {
    type Message;

    fn apply(&mut self, message: &Self::Message);
}

#[derive(Debug, Clone)]
struct RouteListScreenState {
    list_state: RouteListState,
    selected_route: Option<usize>,
}

impl Screen for RouteListScreenState {
    type Message = RouteListMsg;

    fn apply(&mut self, message: &Self::Message) {
        if let RouteListMsg::StateChanged(state) = message {
            self.list_state = state.clone();
        }
    }
}

#[derive(Debug, Clone)]
struct RouteEditorScreenState {
    editing_route: Option<usize>,
    draft: Option<Route>,
    scroll_offset: usize,
}

impl Screen for RouteEditorScreenState {
    type Message = RouteEditorMsg;

    fn apply(&mut self, _message: &Self::Message) {}
}

#[derive(Debug, Clone)]
struct RequestPreviewScreenState {
    preview: Option<RequestPreviewState>,
    scroll_offset: usize,
}

impl Screen for RequestPreviewScreenState {
    type Message = RequestPreviewMsg;

    fn apply(&mut self, _message: &Self::Message) {}
}

#[derive(Debug, Clone)]
struct ResponseViewerScreenState {
    response: Option<ResponseViewerResult>,
}

impl Screen for ResponseViewerScreenState {
    type Message = ResponseViewerMsg;

    fn apply(&mut self, message: &Self::Message) {
        match message {
            ResponseViewerMsg::RouteExecuted(_, response) => {
                self.response = Some(ResponseViewerResult::Success(response.clone()));
            }
            ResponseViewerMsg::RouteExecutionFailed(_, error) => {
                self.response = Some(ResponseViewerResult::Error(error.clone()));
            }
        }
    }
}

#[derive(Debug, Clone)]
struct VariableManagerScreenState {
    secrets_visible: bool,
    editing_variable: Option<String>,
    context: VariableContext,
}

impl Screen for VariableManagerScreenState {
    type Message = VariableManagerMsg;

    fn apply(&mut self, _message: &Self::Message) {}
}

#[derive(Debug, Clone)]
struct KeymapHelpScreenState {
    return_view: Option<ActiveView>,
    scroll_offset: usize,
    previous_focus: Option<Id>,
}

impl Screen for KeymapHelpScreenState {
    type Message = AppMsg;

    fn apply(&mut self, _message: &Self::Message) {}
}

#[derive(Debug, Clone)]
struct NavigationSnapshot {
    active_view: ActiveView,
    input_mode: InputMode,
    route_list: RouteListScreenState,
    route_editor: RouteEditorScreenState,
    request_preview: RequestPreviewScreenState,
    response_viewer: ResponseViewerScreenState,
    variable_manager: VariableManagerScreenState,
    focus: Option<Id>,
}

pub struct AppModel {
    pub app: Application<Id, Msg, NoUserEvent>,
    pub collection: Collection,
    pub input_mode: InputMode,
    variables: Variables,
    route_list: RouteListScreenState,
    active_view: ActiveView,
    route_editor: RouteEditorScreenState,
    request_preview: RequestPreviewScreenState,
    response_viewer: ResponseViewerScreenState,
    variable_manager: VariableManagerScreenState,
    keymap_help: KeymapHelpScreenState,
    navigation_history: Vec<NavigationSnapshot>,
    navigation_index: usize,
    layout_mode: LayoutMode,
    terminal_width: u16,
    terminal_height: u16,
}

impl AppModel {
    const ROUTE_PREVIEW_MIN_WIDTH: u16 = 110;
    const BODY_STATUS_HEIGHT: u16 = 3;
    const BODY_PREVIEW_CHROME_HEIGHT: u16 = 4;
    const EDITOR_FOOTER_MAX_HEIGHT: u16 = 1;
    const EDITOR_SCROLLBAR_WIDTH: u16 = 1;

    pub fn new(app: Application<Id, Msg, NoUserEvent>) -> anyhow::Result<Self> {
        let mut model = Self {
            app,
            collection: Collection::load()?,
            input_mode: InputMode::Normal,
            variables: Variables::load()?,
            route_list: RouteListScreenState {
                list_state: RouteListState::load(),
                selected_route: None,
            },
            active_view: ActiveView::RouteList,
            route_editor: RouteEditorScreenState {
                editing_route: None,
                draft: None,
                scroll_offset: 0,
            },
            request_preview: RequestPreviewScreenState {
                preview: None,
                scroll_offset: 0,
            },
            response_viewer: ResponseViewerScreenState { response: None },
            variable_manager: VariableManagerScreenState {
                secrets_visible: false,
                editing_variable: None,
                context: VariableContext::Global,
            },
            keymap_help: KeymapHelpScreenState {
                return_view: None,
                scroll_offset: 0,
                previous_focus: None,
            },
            navigation_history: vec![],
            navigation_index: 0,
            layout_mode: LayoutMode::Wide,
            terminal_width: 120,
            terminal_height: 40,
        };

        model.push_current_snapshot_to_history();
        Ok(model)
    }

    pub fn update(&mut self, msg: Msg) -> anyhow::Result<Option<AppEffect>> {
        if self.active_view == ActiveView::KeymapHelp {
            match msg {
                Msg::App(AppMsg::Close) => return Ok(Some(AppEffect::Close)),
                Msg::App(AppMsg::TerminalResize(width, height)) => {
                    self.terminal_width = width;
                    self.terminal_height = height.saturating_sub(2);
                    self.layout_mode = Self::layout_mode_for_width(width);
                    return Ok(None);
                }
                Msg::App(AppMsg::ToggleKeymapHelp) => {
                    self.hide_keymap_help();
                    return Ok(None);
                }
                Msg::App(AppMsg::ScrollUp) => {
                    self.scroll_keymap_help_by(-1);
                    return Ok(None);
                }
                Msg::App(AppMsg::ScrollDown) => {
                    self.scroll_keymap_help_by(1);
                    return Ok(None);
                }
                Msg::App(AppMsg::ScrollBy(delta)) => {
                    self.scroll_keymap_help_by(delta);
                    return Ok(None);
                }
                Msg::App(AppMsg::PageUp) => {
                    self.scroll_keymap_help_page(-1);
                    return Ok(None);
                }
                Msg::App(AppMsg::PageDown) => {
                    self.scroll_keymap_help_page(1);
                    return Ok(None);
                }
                Msg::App(AppMsg::ScrollToTop) => {
                    self.keymap_help.scroll_offset = 0;
                    return Ok(None);
                }
                Msg::App(AppMsg::ScrollToBottom) => {
                    self.keymap_help.scroll_offset = self.keymap_help_max_offset();
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }

        match &msg {
            Msg::RouteList(screen_msg) => self.route_list.apply(screen_msg),
            Msg::RouteEditor(screen_msg) => self.route_editor.apply(screen_msg),
            Msg::RequestPreview(screen_msg) => self.request_preview.apply(screen_msg),
            Msg::ResponseViewer(screen_msg) => self.response_viewer.apply(screen_msg),
            Msg::VariableManager(screen_msg) => self.variable_manager.apply(screen_msg),
            Msg::App(_) => {}
        }

        match msg {
            Msg::App(AppMsg::Close) => return Ok(Some(AppEffect::Close)),
            Msg::App(AppMsg::NavigateBack) => {
                self.navigate_back_history()?;
            }
            Msg::App(AppMsg::NavigateForward) => {
                self.navigate_forward_history()?;
            }
            Msg::App(AppMsg::SwitchView(ActiveView::VariableManager)) => {
                let open_scoped = self.active_view == ActiveView::RouteEditor;
                self.input_mode = InputMode::Normal;
                self.variable_manager.editing_variable = None;

                if open_scoped {
                    if let Some(scope_id) = self
                        .route_editor
                        .draft
                        .as_ref()
                        .map(|route| route.scope_id.clone())
                    {
                        self.variable_manager.context = VariableContext::Scoped { scope_id };
                    } else {
                        self.variable_manager.context = VariableContext::Global;
                    }
                } else {
                    self.variable_manager.context = VariableContext::Global;
                }

                self.mount_variable_manager()?;
                self.set_active_view_with_history(ActiveView::VariableManager);
            }
            Msg::App(AppMsg::SwitchView(view)) => self.set_active_view_with_history(view),
            Msg::RouteList(RouteListMsg::RunRoute(index)) => {
                if self.active_view != ActiveView::RouteList {
                    return Ok(None);
                }

                if index >= self.collection.routes.len() {
                    return Ok(None);
                }

                self.route_list.selected_route = Some(index);
                self.select_route_in_state(index, true);
                self.persist_route_list_state();
                self.request_preview.preview = self.build_request_preview_state(index);
                self.input_mode = InputMode::Normal;
                self.request_preview.scroll_offset = 0;
                self.set_active_view_with_history(ActiveView::RequestPreview);
            }
            Msg::ResponseViewer(ResponseViewerMsg::RouteExecuted(index, response)) => {
                self.route_list.selected_route = Some(index);
                self.response_viewer.response = Some(ResponseViewerResult::Success(response));
                self.input_mode = InputMode::Normal;
                self.mount_response_viewer()?;
                self.set_active_view_with_history(ActiveView::ResponseViewer);
            }
            Msg::ResponseViewer(ResponseViewerMsg::RouteExecutionFailed(index, error)) => {
                self.route_list.selected_route = Some(index);
                self.response_viewer.response = Some(ResponseViewerResult::Error(error));
                self.input_mode = InputMode::Normal;
                self.mount_response_viewer()?;
                self.set_active_view_with_history(ActiveView::ResponseViewer);
            }
            Msg::RouteList(RouteListMsg::EditRoute(index)) => {
                if self.active_view != ActiveView::RouteList {
                    return Ok(None);
                }

                if index >= self.collection.routes.len() {
                    return Ok(None);
                }

                let route = self.collection.routes[index].clone();
                self.select_route_in_state(index, true);
                self.persist_route_list_state();
                self.route_editor.editing_route = Some(index);
                self.route_editor.draft = Some(route.clone());
                self.input_mode = InputMode::Normal;
                self.route_editor.scroll_offset = 0;
                self.mount_editor(&route)?;
                self.set_active_view_with_history(ActiveView::RouteEditor);
            }
            Msg::RouteList(RouteListMsg::NewRoute) => {
                if self.active_view != ActiveView::RouteList {
                    return Ok(None);
                }

                let route = Route {
                    group: DEFAULT_ROUTE_GROUP.to_string(),
                    scope_id: self.collection.new_scope_id(),
                    name: String::new(),
                    method: HttpMethod::Get,
                    url: String::new(),
                    headers: vec![],
                    body: None,
                };

                self.route_editor.editing_route = None;
                self.route_editor.draft = Some(route.clone());
                self.input_mode = InputMode::Normal;
                self.route_editor.scroll_offset = 0;
                self.mount_editor(&route)?;
                self.set_active_view_with_history(ActiveView::RouteEditor);
            }
            Msg::RouteList(RouteListMsg::DeleteRoute(index)) => {
                if self.active_view != ActiveView::RouteList {
                    return Ok(None);
                }

                if index >= self.collection.routes.len() {
                    return Ok(None);
                }

                let scope_id = self.collection.routes[index].scope_id.clone();

                self.collection.delete_route(index)?;
                self.collection.save()?;

                self.variables.delete_scope(&scope_id);
                self.variables.save()?;

                if self.collection.routes.is_empty() {
                    self.route_list.list_state.selected = None;
                } else {
                    let next_index = if index < self.collection.routes.len() {
                        index
                    } else {
                        self.collection.routes.len() - 1
                    };
                    self.select_route_in_state(next_index, true);
                }

                self.persist_route_list_state();
                self.refresh_route_list()?;
            }
            Msg::RouteList(RouteListMsg::StateChanged(state)) => {
                self.route_list.list_state = state;
                self.persist_route_list_state();
            }
            Msg::App(AppMsg::OpenScopedVariables) => {
                if self.active_view != ActiveView::RouteEditor {
                    return Ok(None);
                }

                let Some(scope_id) = self
                    .route_editor
                    .draft
                    .as_ref()
                    .map(|route| route.scope_id.clone())
                else {
                    return Ok(None);
                };

                self.input_mode = InputMode::Normal;
                self.variable_manager.editing_variable = None;
                self.variable_manager.context = VariableContext::Scoped { scope_id };
                self.mount_variable_manager()?;
                self.set_active_view_with_history(ActiveView::VariableManager);
            }
            Msg::App(AppMsg::ToggleSecretVisibility) => {
                if self.active_view == ActiveView::VariableManager {
                    self.variable_manager.secrets_visible = !self.variable_manager.secrets_visible;
                    self.sync_variable_value_visibility()?;
                }
            }
            Msg::VariableManager(VariableManagerMsg::ModeChanged(_)) => {
                if self.active_view == ActiveView::VariableManager {
                    self.sync_variable_value_visibility()?;
                }
            }
            Msg::App(AppMsg::ToggleKeymapHelp) => {
                if self.input_mode == InputMode::Normal {
                    if self.active_view == ActiveView::KeymapHelp {
                        self.hide_keymap_help();
                    } else {
                        self.show_keymap_help()?;
                    }
                }
            }
            Msg::VariableManager(VariableManagerMsg::Add) => {
                if self.active_view != ActiveView::VariableManager {
                    return Ok(None);
                }

                self.input_mode = InputMode::Normal;
                self.variable_manager.editing_variable = None;
                self.mount_variable_inputs("", "", VariableMode::Placeholder)?;
                self.app.active(&Id::VariableKeyInput)?;
            }
            Msg::VariableManager(VariableManagerMsg::SelectionChanged(key)) => {
                if self.active_view != ActiveView::VariableManager {
                    return Ok(None);
                }

                self.load_variable_into_inputs(&key)?;
                self.app.active(&Id::VariableTable)?;
            }
            Msg::VariableManager(VariableManagerMsg::Edit(key)) => {
                if self.active_view != ActiveView::VariableManager {
                    return Ok(None);
                }

                self.input_mode = InputMode::Normal;
                self.load_variable_into_inputs(&key)?;
                self.app.active(&Id::VariableKeyInput)?;
            }
            Msg::VariableManager(VariableManagerMsg::Save) => {
                if self.active_view != ActiveView::VariableManager {
                    return Ok(None);
                }

                let key = self
                    .editor_input_value(&Id::VariableKeyInput)
                    .trim()
                    .to_string();
                let value = self.editor_input_value(&Id::VariableValueInput);
                let mode = self.variable_mode_value();

                if key.is_empty() {
                    return Ok(None);
                }

                if let Some(old_key) = self.variable_manager.editing_variable.take()
                    && old_key != key
                {
                    if let Some(scope_id) = self.active_scope_id().map(ToOwned::to_owned) {
                        self.variables.scoped_delete(&scope_id, &old_key);
                    } else {
                        self.variables.delete(&old_key);
                    }
                }

                if let Some(scope_id) = self.active_scope_id().map(ToOwned::to_owned) {
                    self.variables
                        .scoped_add_with_mode(scope_id, key, value, mode);
                } else {
                    self.variables.add_with_mode(key, value, mode);
                }
                self.variables.save()?;

                self.input_mode = InputMode::Normal;
                self.variable_manager.editing_variable = None;
                self.mount_variable_manager()?;
            }
            Msg::VariableManager(VariableManagerMsg::Delete(key)) => {
                if self.active_view != ActiveView::VariableManager {
                    return Ok(None);
                }

                if let Some(scope_id) = self.active_scope_id().map(ToOwned::to_owned) {
                    self.variables.scoped_delete(&scope_id, &key);
                } else {
                    self.variables.delete(&key);
                }
                self.variables.save()?;

                if self.variable_manager.editing_variable.as_deref() == Some(key.as_str()) {
                    self.variable_manager.editing_variable = None;
                }

                self.mount_variable_manager()?;
            }
            Msg::App(AppMsg::TerminalResize(width, height)) => {
                self.terminal_width = width;
                self.terminal_height = height.saturating_sub(2);
                self.layout_mode = Self::layout_mode_for_width(width);
            }
            Msg::App(AppMsg::EnterInsertMode) => {
                self.input_mode = InputMode::Insert;
                if self.active_view == ActiveView::RouteEditor {
                    self.ensure_editor_focus_visible();
                    self.sync_editor_input_mode()?;
                } else if self.active_view == ActiveView::VariableManager {
                    self.sync_variable_input_mode()?;
                }
            }
            Msg::App(AppMsg::EnterNormalMode) => {
                self.input_mode = InputMode::Normal;
                if self.active_view == ActiveView::RouteEditor {
                    self.sync_editor_input_mode()?;
                } else if self.active_view == ActiveView::VariableManager {
                    self.sync_variable_input_mode()?;
                }
            }
            Msg::RouteEditor(RouteEditorMsg::FocusField(id)) => {
                let focus_target =
                    if id == Id::EditorNewGroup && !self.editor_show_new_group_selected() {
                        match self.app.focus() {
                            Some(Id::EditorMethod) => Id::EditorGroup,
                            _ => Id::EditorMethod,
                        }
                    } else {
                        id
                    };

                let _ = self.app.active(&focus_target);
                if self.input_mode == InputMode::Insert
                    && let Some(section) = Self::editor_section_for_focus(&focus_target)
                {
                    self.ensure_editor_section_visible(section);
                }
            }
            Msg::App(AppMsg::ScrollUp) => match self.active_view {
                ActiveView::RouteEditor => self.scroll_editor_by(-1),
                ActiveView::RequestPreview => self.scroll_preview_by(-1),
                _ => {}
            },
            Msg::App(AppMsg::ScrollDown) => match self.active_view {
                ActiveView::RouteEditor => self.scroll_editor_by(1),
                ActiveView::RequestPreview => self.scroll_preview_by(1),
                _ => {}
            },
            Msg::App(AppMsg::PageUp) => match self.active_view {
                ActiveView::RouteEditor => self.scroll_editor_page(-1),
                ActiveView::RequestPreview => self.scroll_preview_page(-1),
                _ => {}
            },
            Msg::App(AppMsg::PageDown) => match self.active_view {
                ActiveView::RouteEditor => self.scroll_editor_page(1),
                ActiveView::RequestPreview => self.scroll_preview_page(1),
                _ => {}
            },
            Msg::App(AppMsg::ScrollBy(delta)) => match self.active_view {
                ActiveView::RouteEditor => self.scroll_editor_by(delta),
                ActiveView::RequestPreview => self.scroll_preview_by(delta),
                _ => {}
            },
            Msg::App(AppMsg::ScrollToTop) => match self.active_view {
                ActiveView::RouteEditor => {
                    self.route_editor.scroll_offset = 0;
                }
                ActiveView::RequestPreview => {
                    self.request_preview.scroll_offset = 0;
                }
                _ => {}
            },
            Msg::App(AppMsg::ScrollToBottom) => match self.active_view {
                ActiveView::RouteEditor => {
                    self.route_editor.scroll_offset =
                        self.current_editor_sections().len().saturating_sub(1);
                }
                ActiveView::RequestPreview => {
                    self.request_preview.scroll_offset = self.preview_scroll_max_offset();
                }
                _ => {}
            },
            Msg::RouteEditor(RouteEditorMsg::MethodChanged(_)) => {}
            Msg::RouteEditor(RouteEditorMsg::GroupSelected(_index)) => {
                // If "New Group..." is selected (last item), the view will show the new group input
                // Otherwise, store the selected group name for use during save
            }

            Msg::RouteEditor(RouteEditorMsg::NewGroupEntered) => {
                // Focus moves to method after entering new group name
                self.app.active(&Id::EditorMethod)?;
                self.ensure_editor_section_visible(EditorSectionKind::Method);
            }
            Msg::RouteEditor(RouteEditorMsg::Save) => {
                if self.active_view == ActiveView::RouteEditor {
                    let name = self.editor_input_value(&Id::EditorName);

                    // Determine group: check if "New Group..." was selected
                    let group = if let Ok(State::One(StateValue::Usize(group_idx))) =
                        self.app.state(&Id::EditorGroup)
                    {
                        let group_names = self.collection.group_names();
                        if group_idx >= group_names.len() {
                            // "New Group..." selected — read from new group input
                            let new_group_name = self.editor_input_value(&Id::EditorNewGroup);
                            if new_group_name.trim().is_empty() {
                                DEFAULT_ROUTE_GROUP.to_string()
                            } else {
                                new_group_name
                            }
                        } else {
                            group_names[group_idx].clone()
                        }
                    } else {
                        DEFAULT_ROUTE_GROUP.to_string()
                    };

                    let method_index = if let Ok(State::One(StateValue::Usize(i))) =
                        self.app.state(&Id::EditorMethod)
                    {
                        i
                    } else {
                        0
                    };

                    let method = match method_index {
                        0 => HttpMethod::Get,
                        1 => HttpMethod::Post,
                        2 => HttpMethod::Put,
                        3 => HttpMethod::Patch,
                        4 => HttpMethod::Delete,
                        _ => HttpMethod::Get,
                    };

                    let url = self.editor_input_value(&Id::EditorUrl);
                    let headers_raw = self.editor_input_value(&Id::EditorHeaders);

                    let headers: Vec<String> = headers_raw
                        .split(',')
                        .map(|h| h.trim().to_string())
                        .filter(|h| !h.is_empty())
                        .collect();

                    let body = self
                        .route_editor
                        .draft
                        .as_ref()
                        .and_then(|d| d.body.clone());
                    let scope_id = self
                        .route_editor
                        .draft
                        .as_ref()
                        .map(|draft| draft.scope_id.clone())
                        .unwrap_or_else(|| self.collection.new_scope_id());

                    let route = Route {
                        group,
                        scope_id,
                        name,
                        method,
                        url,
                        headers,
                        body,
                    };

                    if let Some(index) = self.route_editor.editing_route {
                        self.collection.update_route(index, route)?;
                        self.select_route_in_state(index, true);
                    } else {
                        self.collection.add_route(route);
                        let new_index = self.collection.routes.len().saturating_sub(1);
                        self.select_route_in_state(new_index, true);
                    }

                    self.collection.save()?;
                    self.persist_route_list_state();
                    self.route_editor.editing_route = None;
                    self.route_editor.draft = None;
                    self.input_mode = InputMode::Normal;
                    self.route_editor.scroll_offset = 0;
                    self.set_active_view_with_history(ActiveView::RouteList);
                    self.refresh_route_list()?;
                }
            }
            Msg::App(AppMsg::OpenBodyEditor) => {
                if matches!(
                    self.active_view,
                    ActiveView::RouteEditor | ActiveView::RequestPreview
                ) {
                    // Signal to the event loop that we need to suspend for external editor
                    return Ok(Some(AppEffect::OpenBodyEditor));
                }
            }
            Msg::RouteEditor(RouteEditorMsg::BodyEditorResult(body)) => {
                if self.active_view == ActiveView::RouteEditor {
                    if let Some(draft) = &mut self.route_editor.draft {
                        draft.body = body;
                    }
                } else if self.active_view == ActiveView::RequestPreview {
                    let scope_id = self
                        .request_preview
                        .preview
                        .as_ref()
                        .map(|preview| preview.route_scope_id.as_str());
                    let masked_body = body.as_deref().map(|content| {
                        self.variables
                            .redact_hidden_values_with_scope(content, scope_id)
                    });

                    if let Some(preview) = &mut self.request_preview.preview {
                        preview.execution_body = body;
                        preview.display_body = masked_body;
                    }
                }
            }
            Msg::RequestPreview(RequestPreviewMsg::Execute) => {
                if self.active_view == ActiveView::RequestPreview
                    && self.request_preview.preview.is_some()
                {
                    return Ok(Some(AppEffect::ExecutePreviewRequest));
                }
            }
        }

        Ok(None)
    }

    pub fn body_editor_initial_content(&self) -> Option<&str> {
        match self.active_view {
            ActiveView::RouteEditor => self
                .route_editor
                .draft
                .as_ref()
                .and_then(|draft| draft.body.as_deref()),
            ActiveView::RequestPreview => self
                .request_preview
                .preview
                .as_ref()
                .and_then(|preview| preview.execution_body.as_deref()),
            _ => None,
        }
    }

    pub fn build_preview_executor(&self) -> Option<(usize, CurlExecutor)> {
        let preview = self.request_preview.preview.as_ref()?;
        let route = Route {
            group: DEFAULT_ROUTE_GROUP.to_string(),
            scope_id: preview.route_scope_id.clone(),
            name: preview.route_name.clone(),
            method: preview.method.clone(),
            url: preview.execution_url.clone(),
            headers: preview.execution_headers.clone(),
            body: preview.execution_body.clone(),
        };

        Some((preview.route_index, CurlExecutor::from_prepared(route)))
    }

    fn build_request_preview_state(&self, index: usize) -> Option<RequestPreviewState> {
        let route = self.collection.routes.get(index)?;

        let scope_id = Some(route.scope_id.as_str());

        let display_url = self
            .variables
            .substitute_for_preview_with_scope(&route.url, scope_id);
        let execution_url = self
            .variables
            .substitute_for_execution_with_scope(&route.url, scope_id);
        let display_headers = route
            .headers
            .iter()
            .map(|header| {
                self.variables
                    .substitute_for_preview_with_scope(header, scope_id)
            })
            .collect::<Vec<_>>();
        let execution_headers = route
            .headers
            .iter()
            .map(|header| {
                self.variables
                    .substitute_for_execution_with_scope(header, scope_id)
            })
            .collect::<Vec<_>>();

        let display_body = route
            .body
            .as_deref()
            .map(|body| {
                self.variables
                    .substitute_for_preview_with_scope(body, scope_id)
            })
            .filter(|body| !body.trim().is_empty());
        let execution_body = route
            .body
            .as_deref()
            .map(|body| {
                self.variables
                    .substitute_for_execution_with_scope(body, scope_id)
            })
            .filter(|body| !body.trim().is_empty());

        Some(RequestPreviewState {
            route_index: index,
            route_scope_id: route.scope_id.clone(),
            route_name: route.name.clone(),
            method: route.method.clone(),
            display_url,
            execution_url,
            display_headers,
            execution_headers,
            display_body,
            execution_body,
        })
    }

    fn select_route_in_state(&mut self, route_index: usize, expand_group: bool) {
        if let Some(route) = self.collection.routes.get(route_index) {
            let group_name = if route.group.trim().is_empty() {
                DEFAULT_ROUTE_GROUP.to_string()
            } else {
                route.group.clone()
            };

            if expand_group
                && !self
                    .route_list
                    .list_state
                    .expanded_groups
                    .iter()
                    .any(|name| name == &group_name)
            {
                self.route_list.list_state.expanded_groups.push(group_name);
            }

            self.route_list.list_state.selected =
                Some(SelectedItem::Route(RouteSelection::from_route(route)));
        }
    }

    fn persist_route_list_state(&self) {
        if let Err(error) = self.route_list.list_state.save() {
            eprintln!("Warning: failed to persist route list state: {error}");
        }
    }

    pub fn refresh_route_list(&mut self) -> anyhow::Result<()> {
        let _ = self.app.umount(&Id::RouteList);
        self.app.mount(
            Id::RouteList,
            Box::new(RouteList::new(
                &self.collection.routes,
                &self.route_list.list_state,
            )),
            SubUtils::key_subs([
                Key::Char('j').into(),
                Key::Char('k').into(),
                Key::Char('g').into(),
                Key::Char('G').into(),
                KeyEvent::new(Key::Char('g'), KeyModifiers::SHIFT),
                KeyEvent::new(Key::Char('G'), KeyModifiers::SHIFT),
                Key::Up.into(),
                Key::Down.into(),
                Key::Home.into(),
                Key::End.into(),
                Key::Tab.into(),
                Key::BackTab.into(),
                KeyEvent::new(Key::Tab, KeyModifiers::SHIFT),
                KeyEvent::new(Key::BackTab, KeyModifiers::SHIFT),
                Key::Enter.into(),
                Key::Char('e').into(),
                Key::Char('d').into(),
                Key::Char('n').into(),
            ]),
        )?;
        Ok(())
    }

    pub fn view(&mut self, frame: &mut ratatui::Frame<'_>) {
        let content_area = Self::content_area(frame.area());
        self.terminal_width = content_area.width;
        self.terminal_height = content_area.height;
        self.layout_mode = Self::layout_mode_for_width(self.terminal_width);

        match self.active_view {
            ActiveView::RouteList => {
                self.render_route_list(frame, content_area);
            }
            ActiveView::RouteEditor => {
                self.render_route_editor(frame, content_area);
            }
            ActiveView::RequestPreview => {
                self.render_request_preview(frame, content_area);
            }
            ActiveView::ResponseViewer => {
                self.render_response_viewer(frame, content_area);
            }
            ActiveView::VariableManager => {
                self.render_variable_manager(frame, content_area);
            }
            ActiveView::KeymapHelp => {
                self.render_keymap_help_page(frame, content_area);
            }
        }
    }

    pub fn mount_variable_manager(&mut self) -> anyhow::Result<()> {
        self.variable_manager.editing_variable = None;
        let entries = self.variable_table_entries();
        let selected_key = entries.first().map(|(key, _, _, _)| key.clone());

        self.app.remount(
            Id::VariableTable,
            Box::new(VariableTable::new(
                &entries,
                self.variable_manager.secrets_visible,
            )),
            vec![],
        )?;

        if let Some(key) = selected_key {
            self.load_variable_into_inputs(&key)?;
        } else {
            self.mount_variable_inputs("", "", VariableMode::Placeholder)?;
        }

        self.app.active(&Id::VariableTable)?;
        self.sync_variable_input_mode()?;

        Ok(())
    }

    fn variable_table_entries(&self) -> Vec<(String, String, VariableMode, String)> {
        if let Some(scope_id) = self.active_scope_id() {
            let mut merged: BTreeMap<String, (String, VariableMode, String)> = BTreeMap::new();

            for (key, value) in self.variables.scoped_entries(scope_id) {
                merged.insert(
                    key.clone(),
                    (
                        value.clone(),
                        self.variables
                            .scoped_mode(scope_id, key)
                            .unwrap_or(VariableMode::Placeholder),
                        "scoped".to_string(),
                    ),
                );
            }

            for key in self.scoped_route_used_global_keys() {
                if merged.contains_key(&key) {
                    continue;
                }

                if let Some(value) = self.variables.get(&key) {
                    merged.insert(
                        key.clone(),
                        (
                            value.clone(),
                            self.variables
                                .mode(&key)
                                .unwrap_or(VariableMode::Placeholder),
                            "global".to_string(),
                        ),
                    );
                }
            }

            merged
                .into_iter()
                .map(|(key, (value, mode, source))| (key, value, mode, source))
                .collect()
        } else {
            self.variables
                .entries()
                .into_iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        value.clone(),
                        self.variables
                            .mode(key)
                            .unwrap_or(VariableMode::Placeholder),
                        "global".to_string(),
                    )
                })
                .collect()
        }
    }

    fn scoped_route_used_global_keys(&self) -> BTreeSet<String> {
        let Some(route) = self.route_editor.draft.as_ref() else {
            return BTreeSet::new();
        };

        let url_template = match self.app.state(&Id::EditorUrl) {
            Ok(State::One(StateValue::String(value))) => value,
            _ => route.url.clone(),
        };

        let headers_template = match self.app.state(&Id::EditorHeaders) {
            Ok(State::One(StateValue::String(value))) => value,
            _ => route.headers.join(", "),
        };

        let mut keys = BTreeSet::new();

        Self::collect_variable_tokens(&url_template, &mut keys);
        for header in headers_template
            .split(',')
            .map(str::trim)
            .filter(|header| !header.is_empty())
        {
            Self::collect_variable_tokens(header, &mut keys);
        }
        if let Some(body) = route.body.as_deref() {
            Self::collect_variable_tokens(body, &mut keys);
        }

        keys.into_iter()
            .filter(|key| self.variables.get(key).is_some())
            .collect()
    }

    fn collect_variable_tokens(template: &str, output: &mut BTreeSet<String>) {
        let mut rest = template;

        while let Some(start) = rest.find("{{") {
            let after_open = &rest[start + 2..];

            let Some(end) = after_open.find("}}") else {
                break;
            };

            let token = after_open[..end].trim();
            if !token.is_empty() {
                output.insert(token.to_string());
            }

            rest = &after_open[end + 2..];
        }
    }

    fn variable_value_mode_for_key(&self, key: &str) -> Option<(String, VariableMode)> {
        if let Some(scope_id) = self.active_scope_id()
            && let Some(value) = self.variables.scoped_get(scope_id, key).cloned()
        {
            return Some((
                value,
                self.variables
                    .scoped_mode(scope_id, key)
                    .unwrap_or(VariableMode::Placeholder),
            ));
        }

        self.variables.get(key).cloned().map(|value| {
            (
                value,
                self.variables
                    .mode(key)
                    .unwrap_or(VariableMode::Placeholder),
            )
        })
    }

    fn load_variable_into_inputs(&mut self, key: &str) -> anyhow::Result<()> {
        let Some((value, mode)) = self.variable_value_mode_for_key(key) else {
            return Ok(());
        };

        self.variable_manager.editing_variable = Some(key.to_string());
        self.mount_variable_inputs(key, &value, mode)?;

        Ok(())
    }

    fn mount_variable_inputs(
        &mut self,
        key: &str,
        value: &str,
        mode: VariableMode,
    ) -> anyhow::Result<()> {
        self.app.remount(
            Id::VariableKeyInput,
            Box::new(VariableKeyInput::new(key)),
            vec![],
        )?;
        self.app.remount(
            Id::VariableValueInput,
            Box::new(VariableValueInput::new(value)),
            vec![],
        )?;
        self.app.remount(
            Id::VariableMode,
            Box::new(VariableModeSelector::new(mode)),
            vec![],
        )?;

        self.sync_variable_input_mode()?;
        self.sync_variable_value_visibility()?;
        Ok(())
    }

    pub fn mount_response_viewer(&mut self) -> anyhow::Result<()> {
        if let Some(response) = &self.response_viewer.response {
            let _ = self.app.umount(&Id::ResponseDetails);

            let details_view = match response {
                ResponseViewerResult::Success(response) => ResponseDetailsView::new(response),
                ResponseViewerResult::Error(error) => ResponseDetailsView::new_error(error),
            };

            self.app.mount(
                Id::ResponseDetails,
                Box::new(details_view),
                SubUtils::key_subs([
                    Key::Char('j').into(),
                    Key::Char('k').into(),
                    Key::Char('g').into(),
                    Key::Char('G').into(),
                    KeyEvent::new(Key::Char('g'), KeyModifiers::SHIFT),
                    KeyEvent::new(Key::Char('G'), KeyModifiers::SHIFT),
                    Key::Down.into(),
                    Key::Up.into(),
                    Key::PageUp.into(),
                    Key::PageDown.into(),
                    Key::Home.into(),
                    Key::End.into(),
                ]),
            )?;

            self.app.active(&Id::ResponseDetails)?;
        }

        Ok(())
    }

    fn render_response_viewer(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        self.app.view(&Id::ResponseDetails, frame, area);
    }

    fn render_variable_manager(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let chunks = Layout::vertical([
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

        self.app.view(&Id::VariableTable, frame, chunks[0]);
        self.app.view(&Id::VariableKeyInput, frame, chunks[1]);
        self.app.view(&Id::VariableValueInput, frame, chunks[2]);
        self.app.view(&Id::VariableMode, frame, chunks[3]);

        let mode_label = match self.input_mode {
            InputMode::Normal => "-- NORMAL --",
            InputMode::Insert => "-- INSERT --",
        };
        let context_label = if self.is_scoped_variable_context() {
            "Context: Route-scoped"
        } else {
            "Context: Global"
        };
        let status = Paragraph::new(format!("{mode_label} | {context_label}"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(status, chunks[4]);
    }

    fn show_keymap_help(&mut self) -> anyhow::Result<()> {
        if self.active_view == ActiveView::KeymapHelp {
            return Ok(());
        }

        self.keymap_help.return_view = Some(self.active_view);
        self.active_view = ActiveView::KeymapHelp;
        self.keymap_help.scroll_offset = 0;
        self.keymap_help.previous_focus = self
            .app
            .focus()
            .cloned()
            .filter(|id| *id != Id::GlobalListener);

        self.app.active(&Id::GlobalListener)?;
        self.app.lock_subs();
        Ok(())
    }

    fn hide_keymap_help(&mut self) {
        if self.active_view == ActiveView::KeymapHelp {
            self.active_view = self
                .keymap_help
                .return_view
                .take()
                .unwrap_or(ActiveView::RouteList);
        }

        self.keymap_help.scroll_offset = 0;
        self.app.unlock_subs();

        if let Some(previous_focus) = self.keymap_help.previous_focus.take()
            && self.app.mounted(&previous_focus)
        {
            let _ = self.app.active(&previous_focus);
        }
    }

    fn render_keymap_help_page(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let lines = self.keymap_help_lines();
        let can_render_scrollbar = area.width > Self::EDITOR_SCROLLBAR_WIDTH;
        let (content_area, scrollbar_area) = if can_render_scrollbar {
            let chunks = Layout::horizontal([
                Constraint::Min(0),
                Constraint::Length(Self::EDITOR_SCROLLBAR_WIDTH),
            ])
            .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            (area, None)
        };

        let page_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow))
            .padding(Padding::new(1, 1, 1, 1))
            .title("Keymaps");
        let content_inner = page_block.inner(content_area);

        if content_inner.width == 0 || content_inner.height == 0 {
            return;
        }

        let (total_lines, viewport_height, max_offset) =
            Self::keymap_scroll_metrics(&lines, content_inner);
        self.keymap_help.scroll_offset = self.keymap_help.scroll_offset.min(max_offset);

        let scroll_y = u16::try_from(self.keymap_help.scroll_offset).unwrap_or(u16::MAX);
        let content = Paragraph::new(lines)
            .block(page_block)
            .scroll((scroll_y, 0))
            .wrap(Wrap { trim: false });
        frame.render_widget(content, content_area);

        if let Some(scrollbar_area) = scrollbar_area {
            self.render_keymap_help_scrollbar(
                frame,
                scrollbar_area,
                total_lines,
                viewport_height,
                max_offset,
            );
        }
    }

    fn keymap_help_lines(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Self::keymap_help_heading("Global"),
            Line::from("q: Quit"),
            Line::from("?: Open/close keymap helper"),
            Line::from("H/L: Back/forward in navigation history"),
            Line::from("j/k or Up/Down: Scroll helper"),
            Line::from("PgUp/PgDn or gg/G: Page/top/bottom"),
            Line::from(""),
        ];

        let (view_title, view_lines) = Self::view_keymap_entries(self.keymap_help_target_view());

        lines.push(Self::keymap_help_heading(view_title));
        lines.extend(
            view_lines
                .into_iter()
                .map(|line| Line::from(line.to_string())),
        );
        lines
    }

    fn keymap_help_target_view(&self) -> ActiveView {
        self.keymap_help
            .return_view
            .unwrap_or(match self.active_view {
                ActiveView::KeymapHelp => ActiveView::RouteList,
                view => view,
            })
    }

    fn view_keymap_entries(view: ActiveView) -> (&'static str, Vec<&'static str>) {
        match view {
            ActiveView::RouteList => (
                "Route List",
                vec![
                    "j/k or Up/Down: Move selection",
                    "Enter: Expand group or open request preview",
                    "Tab/Shift+Tab: Jump between groups",
                    "gg/G or Home/End: Jump top/bottom",
                    "e: Edit route | n: New route | d: Delete route",
                    "v: Open global variables",
                ],
            ),
            ActiveView::RouteEditor => (
                "Route Editor",
                vec![
                    "i: Enter insert mode",
                    "Ctrl+S: Save route",
                    "Esc: Return to normal mode (insert only)",
                    "Tab/Shift+Tab: Focus next/previous field",
                    "b: Edit request body",
                    "v: Global vars | V: Route-scoped vars",
                    "j/k or Up/Down/PgUp/PgDn: Scroll",
                    "gg/G: Jump top/bottom",
                ],
            ),
            ActiveView::RequestPreview => (
                "Request Preview",
                vec![
                    "r: Execute request",
                    "b: Edit request body",
                    "j/k or Up/Down/PgUp/PgDn: Scroll",
                    "gg/G: Jump top/bottom",
                ],
            ),
            ActiveView::ResponseViewer => (
                "Response Viewer",
                vec![
                    "j/k or Up/Down: Scroll",
                    "PgUp/PgDn: Page scroll",
                    "gg/G or Home/End: Jump top/bottom",
                ],
            ),
            ActiveView::VariableManager => (
                "Variable Manager",
                vec![
                    "a: Add variable | e: Edit variable | d: Delete variable",
                    "i: Enter insert mode",
                    "Ctrl+S: Save variable",
                    "Esc: Return to normal mode (insert only)",
                    "Tab/Shift+Tab: Focus next/previous field",
                    "s: Toggle hidden values",
                    "j/k or Up/Down: Move selection",
                    "gg/G or Home/End: Jump top/bottom",
                ],
            ),
            ActiveView::KeymapHelp => ("Keymaps", vec!["Press ? to return."]),
        }
    }

    fn is_history_view(view: ActiveView) -> bool {
        !matches!(view, ActiveView::KeymapHelp)
    }

    fn sync_route_editor_draft_from_inputs(&mut self) {
        if self.active_view != ActiveView::RouteEditor {
            return;
        }

        if self.route_editor.draft.is_none() {
            return;
        }

        let name = self.editor_input_value(&Id::EditorName);
        let group = if let Ok(State::One(StateValue::Usize(group_idx))) =
            self.app.state(&Id::EditorGroup)
        {
            let group_names = self.collection.group_names();
            if group_idx >= group_names.len() {
                let new_group_name = self.editor_input_value(&Id::EditorNewGroup);
                if new_group_name.trim().is_empty() {
                    DEFAULT_ROUTE_GROUP.to_string()
                } else {
                    new_group_name
                }
            } else {
                group_names[group_idx].clone()
            }
        } else {
            DEFAULT_ROUTE_GROUP.to_string()
        };

        let method_index =
            if let Ok(State::One(StateValue::Usize(i))) = self.app.state(&Id::EditorMethod) {
                i
            } else {
                0
            };
        let method = match method_index {
            0 => HttpMethod::Get,
            1 => HttpMethod::Post,
            2 => HttpMethod::Put,
            3 => HttpMethod::Patch,
            4 => HttpMethod::Delete,
            _ => HttpMethod::Get,
        };

        let url = self.editor_input_value(&Id::EditorUrl);
        let headers = self
            .editor_input_value(&Id::EditorHeaders)
            .split(',')
            .map(|h| h.trim().to_string())
            .filter(|h| !h.is_empty())
            .collect();

        if let Some(draft) = self.route_editor.draft.as_mut() {
            draft.name = name;
            draft.group = group;
            draft.method = method;
            draft.url = url;
            draft.headers = headers;
        }
    }

    fn sync_active_view_state_from_components(&mut self) {
        if self.active_view == ActiveView::RouteEditor {
            self.sync_route_editor_draft_from_inputs();
        }
    }

    fn capture_navigation_snapshot(&self) -> NavigationSnapshot {
        NavigationSnapshot {
            active_view: self.active_view,
            input_mode: self.input_mode,
            route_list: self.route_list.clone(),
            route_editor: self.route_editor.clone(),
            request_preview: self.request_preview.clone(),
            response_viewer: self.response_viewer.clone(),
            variable_manager: self.variable_manager.clone(),
            focus: self
                .app
                .focus()
                .cloned()
                .filter(|id| *id != Id::GlobalListener),
        }
    }

    fn replace_current_history_entry(&mut self) {
        if !Self::is_history_view(self.active_view) || self.navigation_history.is_empty() {
            return;
        }

        self.sync_active_view_state_from_components();
        self.navigation_history[self.navigation_index] = self.capture_navigation_snapshot();
    }

    fn push_current_snapshot_to_history(&mut self) {
        if !Self::is_history_view(self.active_view) {
            return;
        }

        self.sync_active_view_state_from_components();

        if self.navigation_index + 1 < self.navigation_history.len() {
            self.navigation_history.truncate(self.navigation_index + 1);
        }

        self.navigation_history
            .push(self.capture_navigation_snapshot());
        self.navigation_index = self.navigation_history.len().saturating_sub(1);
    }

    fn set_active_view_with_history(&mut self, view: ActiveView) {
        if view == self.active_view {
            return;
        }

        self.replace_current_history_entry();
        self.active_view = view;
        self.push_current_snapshot_to_history();
    }

    fn restore_navigation_snapshot(&mut self, snapshot: NavigationSnapshot) -> anyhow::Result<()> {
        let restored_focus = snapshot.focus.clone();
        let restored_editor_scroll = snapshot.route_editor.scroll_offset;
        let restored_variable_editing = snapshot.variable_manager.editing_variable.clone();

        self.active_view = snapshot.active_view;
        self.input_mode = snapshot.input_mode;
        self.route_list = snapshot.route_list;
        self.route_editor = snapshot.route_editor;
        self.request_preview = snapshot.request_preview;
        self.response_viewer = snapshot.response_viewer;
        self.variable_manager = snapshot.variable_manager;

        match self.active_view {
            ActiveView::RouteList => {
                self.refresh_route_list()?;
                self.app.active(&Id::RouteList)?;
            }
            ActiveView::RouteEditor => {
                if let Some(route) = self.route_editor.draft.clone() {
                    self.mount_editor(&route)?;
                }
                self.route_editor.scroll_offset = restored_editor_scroll;
                self.sync_editor_input_mode()?;
            }
            ActiveView::RequestPreview => {}
            ActiveView::ResponseViewer => {
                self.mount_response_viewer()?;
            }
            ActiveView::VariableManager => {
                self.mount_variable_manager()?;
                if let Some(key) = restored_variable_editing {
                    self.load_variable_into_inputs(&key)?;
                    self.variable_manager.editing_variable = Some(key);
                }
                self.sync_variable_input_mode()?;
            }
            ActiveView::KeymapHelp => {}
        }

        if let Some(focus) = restored_focus
            && self.app.mounted(&focus)
        {
            let _ = self.app.active(&focus);
        }

        Ok(())
    }

    fn navigate_back_history(&mut self) -> anyhow::Result<()> {
        if self.navigation_history.is_empty() || self.navigation_index == 0 {
            return Ok(());
        }

        self.replace_current_history_entry();
        self.navigation_index -= 1;
        let snapshot = self.navigation_history[self.navigation_index].clone();
        self.restore_navigation_snapshot(snapshot)
    }

    fn navigate_forward_history(&mut self) -> anyhow::Result<()> {
        if self.navigation_history.is_empty()
            || self.navigation_index + 1 >= self.navigation_history.len()
        {
            return Ok(());
        }

        self.replace_current_history_entry();
        self.navigation_index += 1;
        let snapshot = self.navigation_history[self.navigation_index].clone();
        self.restore_navigation_snapshot(snapshot)
    }

    fn keymap_help_heading(text: &'static str) -> Line<'static> {
        Line::from(Span::styled(
            text,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
    }

    fn keymap_scroll_metrics(lines: &[Line<'static>], content_area: Rect) -> (usize, usize, usize) {
        if content_area.width == 0 || content_area.height == 0 {
            return (0, 0, 0);
        }

        let total_lines = Paragraph::new(lines.to_vec())
            .wrap(Wrap { trim: false })
            .line_count(content_area.width);
        let viewport_height = content_area.height as usize;
        let max_offset = total_lines.saturating_sub(viewport_height);

        (total_lines, viewport_height, max_offset)
    }

    fn keymap_help_max_offset(&self) -> usize {
        if self.terminal_width == 0 || self.terminal_height == 0 {
            return 0;
        }

        let lines = self.keymap_help_lines();
        let area = Rect::new(0, 0, self.terminal_width, self.terminal_height);
        let content_area = if area.width > Self::EDITOR_SCROLLBAR_WIDTH {
            let chunks = Layout::horizontal([
                Constraint::Min(0),
                Constraint::Length(Self::EDITOR_SCROLLBAR_WIDTH),
            ])
            .split(area);
            chunks[0]
        } else {
            area
        };
        let content_inner = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(1, 1, 1, 1))
            .title("Keymaps")
            .inner(content_area);

        let (_, _, max_offset) = Self::keymap_scroll_metrics(&lines, content_inner);
        max_offset
    }

    fn render_keymap_help_scrollbar(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: Rect,
        content_length: usize,
        viewport_height: usize,
        max_offset: usize,
    ) {
        if area.width == 0 || area.height == 0 || viewport_height == 0 {
            return;
        }

        if content_length <= viewport_height {
            return;
        }

        let position = if self.keymap_help.scroll_offset >= max_offset {
            content_length.saturating_sub(1)
        } else {
            self.keymap_help
                .scroll_offset
                .min(content_length.saturating_sub(1))
        };

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_style(Style::default().fg(Color::DarkGray))
            .thumb_style(Style::default().fg(Color::Yellow));
        let mut state = ScrollbarState::new(content_length)
            .position(position)
            .viewport_content_length(viewport_height);

        frame.render_stateful_widget(scrollbar, area, &mut state);
    }

    fn scroll_keymap_help_by(&mut self, delta: isize) {
        let max_offset = self.keymap_help_max_offset() as isize;
        let next_offset = (self.keymap_help.scroll_offset as isize + delta).clamp(0, max_offset);
        self.keymap_help.scroll_offset = next_offset as usize;
    }

    fn scroll_keymap_help_page(&mut self, direction: isize) {
        let page_step = self.terminal_height.saturating_sub(4).max(1) as isize;
        self.scroll_keymap_help_by(direction * page_step);
    }

    fn render_request_preview(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let Some(preview) = self.request_preview.preview.as_ref() else {
            let placeholder =
                Paragraph::new("No request selected. Press Enter on a route to preview.")
                    .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(placeholder, area);
            return;
        };

        let can_render_scrollbar = area.width > Self::EDITOR_SCROLLBAR_WIDTH;
        let (content_area, scrollbar_area) = if can_render_scrollbar {
            let chunks = Layout::horizontal([
                Constraint::Min(0),
                Constraint::Length(Self::EDITOR_SCROLLBAR_WIDTH),
            ])
            .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            (area, None)
        };

        let lines = self.request_preview_lines(preview);
        let preview_block = Self::request_preview_block();
        let preview_inner = preview_block.inner(content_area);
        let (total_lines, viewport_height, max_offset) =
            Self::request_preview_scroll_metrics(&lines, preview_inner);

        self.request_preview.scroll_offset = self.request_preview.scroll_offset.min(max_offset);

        let scroll_y = u16::try_from(self.request_preview.scroll_offset).unwrap_or(u16::MAX);

        let preview_widget = Paragraph::new(lines)
            .block(preview_block)
            .scroll((scroll_y, 0))
            .wrap(Wrap { trim: false });

        frame.render_widget(preview_widget, content_area);

        if let Some(scrollbar_area) = scrollbar_area {
            self.render_preview_scrollbar(
                frame,
                scrollbar_area,
                total_lines,
                viewport_height,
                max_offset,
            );
        }
    }

    fn request_preview_lines(&self, preview: &RequestPreviewState) -> Vec<Line<'static>> {
        let body_preview = preview
            .display_body
            .as_deref()
            .map(body_preview::build)
            .filter(|rendered| !rendered.lines.is_empty());

        let mut lines = vec![];
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<6}", preview.method),
                Style::default()
                    .fg(Self::method_color(&preview.method))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::raw(preview.route_name.clone()),
        ]));
        lines.push(Line::from(format!("URL: {}", preview.display_url)));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Headers:",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )));

        if preview.display_headers.is_empty() {
            lines.push(Line::from(Span::styled(
                "  (none)",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            lines.extend(preview.display_headers.iter().map(|header| {
                Line::from(vec![Span::styled(
                    format!("  - {header}"),
                    Style::default().fg(Color::White),
                )])
            }));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Body (hidden values masked):",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )));

        if let Some(body_preview) = body_preview {
            lines.extend(body_preview.lines);
        } else {
            lines.push(Line::from(Span::styled(
                "  (empty)",
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines
    }

    fn request_preview_block() -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .padding(Padding::new(1, 1, 1, 1))
            .title("Request Preview")
    }

    fn request_preview_scroll_metrics(
        lines: &[Line<'static>],
        preview_inner: Rect,
    ) -> (usize, usize, usize) {
        if preview_inner.width == 0 || preview_inner.height == 0 {
            return (0, 0, 0);
        }

        let total_lines = Paragraph::new(lines.to_vec())
            .wrap(Wrap { trim: false })
            .line_count(preview_inner.width);
        let viewport_height = preview_inner.height as usize;
        let max_offset = total_lines.saturating_sub(viewport_height);

        (total_lines, viewport_height, max_offset)
    }

    fn preview_scroll_max_offset(&self) -> usize {
        let Some(preview) = self.request_preview.preview.as_ref() else {
            return 0;
        };

        if self.terminal_width == 0 || self.terminal_height == 0 {
            return 0;
        }

        let lines = self.request_preview_lines(preview);
        let area = Rect::new(0, 0, self.terminal_width, self.terminal_height);
        let content_area = if area.width > Self::EDITOR_SCROLLBAR_WIDTH {
            let chunks = Layout::horizontal([
                Constraint::Min(0),
                Constraint::Length(Self::EDITOR_SCROLLBAR_WIDTH),
            ])
            .split(area);
            chunks[0]
        } else {
            area
        };
        let preview_inner = Self::request_preview_block().inner(content_area);
        let (_, _, max_offset) = Self::request_preview_scroll_metrics(&lines, preview_inner);
        max_offset
    }

    fn render_preview_scrollbar(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: Rect,
        content_length: usize,
        viewport_height: usize,
        max_offset: usize,
    ) {
        if area.width == 0 || area.height == 0 || viewport_height == 0 {
            return;
        }

        if content_length <= viewport_height {
            return;
        }

        let position = if self.request_preview.scroll_offset >= max_offset {
            content_length.saturating_sub(1)
        } else {
            self.request_preview
                .scroll_offset
                .min(content_length.saturating_sub(1))
        };

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_style(Style::default().fg(Color::DarkGray))
            .thumb_style(Style::default().fg(Color::Cyan));
        let mut state = ScrollbarState::new(content_length)
            .position(position)
            .viewport_content_length(viewport_height);

        frame.render_stateful_widget(scrollbar, area, &mut state);
    }

    fn scroll_preview_by(&mut self, delta: isize) {
        if self.active_view != ActiveView::RequestPreview {
            return;
        }

        let max_offset = self.preview_scroll_max_offset() as isize;
        let next_offset =
            (self.request_preview.scroll_offset as isize + delta).clamp(0, max_offset);
        self.request_preview.scroll_offset = next_offset as usize;
    }

    fn scroll_preview_page(&mut self, direction: isize) {
        if self.active_view != ActiveView::RequestPreview {
            return;
        }

        let page_step = self.terminal_height.saturating_sub(4).max(1) as isize;
        self.scroll_preview_by(direction * page_step);
    }

    fn render_route_editor(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        if area.height == 0 {
            return;
        }

        if self.input_mode == InputMode::Insert {
            self.ensure_editor_focus_visible();
        }

        let body_preview = self.editor_body_preview();
        let sections =
            self.editor_sections(self.editor_show_new_group_selected(), body_preview.as_ref());

        if sections.is_empty() {
            self.render_editor_footer(frame, area, false, false);
            return;
        }

        self.clamp_editor_scroll_offset(sections.len());

        let footer_height = Self::editor_footer_height_for_height(area.height);
        let (editor_area, footer_area) = if footer_height > 0 {
            let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(footer_height)])
                .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            (area, None)
        };

        let total_content_height = Self::editor_total_height(&sections);
        let can_render_scrollbar = total_content_height > editor_area.height as usize
            && editor_area.width > Self::EDITOR_SCROLLBAR_WIDTH;
        let (editor_content_area, scrollbar_area) = if can_render_scrollbar {
            let chunks = Layout::horizontal([
                Constraint::Min(0),
                Constraint::Length(Self::EDITOR_SCROLLBAR_WIDTH),
            ])
            .split(editor_area);
            (chunks[0], Some(chunks[1]))
        } else {
            (editor_area, None)
        };

        let start = self
            .route_editor
            .scroll_offset
            .min(sections.len().saturating_sub(1));
        let start = Self::editor_aligned_start(&sections, start, editor_content_area.height);
        let (start, end) = Self::editor_visible_range(&sections, start, editor_content_area.height);
        let can_scroll_up = start > 0;
        let can_scroll_down = end < sections.len();

        if start == end {
            if editor_content_area.height > 0 {
                let warning = Paragraph::new("Increase terminal height to edit this route.")
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(warning, editor_content_area);
            }

            if let Some(scrollbar_area) = scrollbar_area {
                self.render_editor_scrollbar(
                    frame,
                    scrollbar_area,
                    &sections,
                    start,
                    editor_content_area.height,
                    !can_scroll_down,
                );
            }

            if let Some(footer_area) = footer_area {
                self.render_editor_footer(frame, footer_area, can_scroll_up, can_scroll_down);
            }

            return;
        }

        self.route_editor.scroll_offset = start;

        let constraints: Vec<Constraint> = sections[start..end]
            .iter()
            .map(|section| Constraint::Length(section.height))
            .collect();
        let chunks = Layout::vertical(constraints).split(editor_content_area);

        for (section, chunk) in sections[start..end].iter().zip(chunks.iter()) {
            match section.kind {
                EditorSectionKind::Name => self.app.view(&Id::EditorName, frame, *chunk),
                EditorSectionKind::Group => self.app.view(&Id::EditorGroup, frame, *chunk),
                EditorSectionKind::NewGroup => self.app.view(&Id::EditorNewGroup, frame, *chunk),
                EditorSectionKind::Method => self.app.view(&Id::EditorMethod, frame, *chunk),
                EditorSectionKind::Url => self.app.view(&Id::EditorUrl, frame, *chunk),
                EditorSectionKind::Headers => self.app.view(&Id::EditorHeaders, frame, *chunk),
                EditorSectionKind::BodyStatus => {
                    let body_status = if body_preview.is_some() {
                        "Body: Set (press 'b' to edit in Normal mode)"
                    } else {
                        "Body: Empty (press 'b' to add in Normal mode)"
                    };
                    let body_widget =
                        Paragraph::new(body_status).style(Style::default().fg(Color::DarkGray));
                    let body_status_area = if chunk.height > 2 {
                        chunk.inner(Margin {
                            vertical: 1,
                            horizontal: 0,
                        })
                    } else {
                        *chunk
                    };
                    frame.render_widget(body_widget, body_status_area);
                }
                EditorSectionKind::BodyPreview => {
                    if let Some(preview) = body_preview.as_ref() {
                        let title = match preview.format {
                            body_preview::BodyPreviewFormat::Json => "Body Preview (JSON)",
                            body_preview::BodyPreviewFormat::Text => "Body Preview",
                        };

                        let preview_widget = Paragraph::new(preview.lines.clone())
                            .block(
                                Block::default()
                                    .borders(Borders::ALL)
                                    .border_type(BorderType::Rounded)
                                    .border_style(Style::default().fg(Color::Cyan))
                                    .padding(Padding::new(1, 1, 1, 1))
                                    .title(title),
                            )
                            .wrap(Wrap { trim: false });
                        frame.render_widget(preview_widget, *chunk);
                    }
                }
            }
        }

        if let Some(scrollbar_area) = scrollbar_area {
            self.render_editor_scrollbar(
                frame,
                scrollbar_area,
                &sections,
                start,
                editor_content_area.height,
                !can_scroll_down,
            );
        }

        if let Some(footer_area) = footer_area {
            self.render_editor_footer(frame, footer_area, can_scroll_up, can_scroll_down);
        }
    }

    fn render_editor_scrollbar(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: Rect,
        sections: &[EditorSection],
        start_index: usize,
        viewport_height: u16,
        at_bottom: bool,
    ) {
        if area.width == 0 || area.height == 0 || viewport_height == 0 {
            return;
        }

        let content_length = Self::editor_total_height(sections);
        if content_length <= viewport_height as usize {
            return;
        }

        let start_position = sections
            .iter()
            .take(start_index)
            .map(|section| section.height as usize)
            .sum::<usize>()
            .min(content_length.saturating_sub(1));
        let position = if at_bottom {
            content_length.saturating_sub(1)
        } else {
            start_position
        };

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_style(Style::default().fg(Color::DarkGray))
            .thumb_style(Style::default().fg(Color::Cyan));
        let mut state = ScrollbarState::new(content_length)
            .position(position)
            .viewport_content_length(viewport_height as usize);

        frame.render_stateful_widget(scrollbar, area, &mut state);
    }

    fn render_editor_footer(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: Rect,
        can_scroll_up: bool,
        can_scroll_down: bool,
    ) {
        if area.height == 0 {
            return;
        }

        let mode_widget = Paragraph::new(self.editor_mode_text(can_scroll_up, can_scroll_down))
            .style(Style::default().fg(Color::Yellow));
        let mode_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        frame.render_widget(mode_widget, mode_area);
    }

    fn editor_mode_text(&self, can_scroll_up: bool, can_scroll_down: bool) -> String {
        let mode_label = match self.input_mode {
            InputMode::Normal => "-- NORMAL --",
            InputMode::Insert => "-- INSERT --",
        };

        if can_scroll_up || can_scroll_down {
            let direction = match (can_scroll_up, can_scroll_down) {
                (true, true) => "up/down",
                (true, false) => "up",
                (false, true) => "down",
                (false, false) => "",
            };
            return format!("{mode_label} | scroll: {direction}");
        }

        mode_label.to_string()
    }

    fn editor_show_new_group_selected(&self) -> bool {
        if let Ok(State::One(StateValue::Usize(index))) = self.app.state(&Id::EditorGroup) {
            let group_names = self.collection.group_names();
            index >= group_names.len()
        } else {
            false
        }
    }

    fn editor_body_preview(&self) -> Option<body_preview::BodyPreview> {
        let scope_id = self
            .route_editor
            .draft
            .as_ref()
            .map(|route| route.scope_id.as_str());

        self.route_editor
            .draft
            .as_ref()
            .and_then(|draft| draft.body.as_deref())
            .map(|body| {
                self.variables
                    .substitute_for_preview_with_scope(body, scope_id)
            })
            .map(|body| body_preview::build(&body))
            .filter(|preview| !preview.lines.is_empty())
    }

    fn current_editor_sections(&self) -> Vec<EditorSection> {
        let body_preview = self.editor_body_preview();
        self.editor_sections(self.editor_show_new_group_selected(), body_preview.as_ref())
    }

    fn editor_sections(
        &self,
        show_new_group: bool,
        body_preview: Option<&body_preview::BodyPreview>,
    ) -> Vec<EditorSection> {
        let mut sections = vec![
            EditorSection {
                kind: EditorSectionKind::Name,
                height: 3,
            },
            EditorSection {
                kind: EditorSectionKind::Group,
                height: 3,
            },
        ];

        if show_new_group {
            sections.push(EditorSection {
                kind: EditorSectionKind::NewGroup,
                height: 3,
            });
        }

        sections.extend([
            EditorSection {
                kind: EditorSectionKind::Method,
                height: 3,
            },
            EditorSection {
                kind: EditorSectionKind::Url,
                height: 3,
            },
            EditorSection {
                kind: EditorSectionKind::Headers,
                height: 3,
            },
            EditorSection {
                kind: EditorSectionKind::BodyStatus,
                height: Self::BODY_STATUS_HEIGHT,
            },
        ]);

        if let Some(preview) = body_preview {
            let preview_height = preview.lines.len().clamp(1, u16::MAX as usize) as u16;
            sections.push(EditorSection {
                kind: EditorSectionKind::BodyPreview,
                height: preview_height.saturating_add(Self::BODY_PREVIEW_CHROME_HEIGHT),
            });
        }

        sections
    }

    fn editor_section_for_focus(id: &Id) -> Option<EditorSectionKind> {
        match id {
            Id::EditorName => Some(EditorSectionKind::Name),
            Id::EditorGroup => Some(EditorSectionKind::Group),
            Id::EditorNewGroup => Some(EditorSectionKind::NewGroup),
            Id::EditorMethod => Some(EditorSectionKind::Method),
            Id::EditorUrl => Some(EditorSectionKind::Url),
            Id::EditorHeaders => Some(EditorSectionKind::Headers),
            _ => None,
        }
    }

    fn scroll_editor_by(&mut self, delta: isize) {
        if self.active_view != ActiveView::RouteEditor {
            return;
        }

        let section_count = self.current_editor_sections().len();
        if section_count == 0 {
            self.route_editor.scroll_offset = 0;
            return;
        }

        let max_offset = section_count.saturating_sub(1) as isize;
        let next = (self.route_editor.scroll_offset as isize + delta).clamp(0, max_offset);
        self.route_editor.scroll_offset = next as usize;
    }

    fn scroll_editor_page(&mut self, direction: isize) {
        if self.active_view != ActiveView::RouteEditor {
            return;
        }

        let sections = self.current_editor_sections();
        if sections.is_empty() {
            self.route_editor.scroll_offset = 0;
            return;
        }

        self.clamp_editor_scroll_offset(sections.len());

        let available_height = self.editor_content_viewport_height();
        if available_height == 0 {
            return;
        }

        let start = self
            .route_editor
            .scroll_offset
            .min(sections.len().saturating_sub(1));
        let (start, end) = Self::editor_visible_range(&sections, start, available_height);
        let visible_count = end.saturating_sub(start);
        let step = visible_count.saturating_sub(1).max(1) as isize;

        self.scroll_editor_by(direction * step);
    }

    fn ensure_editor_focus_visible(&mut self) {
        if let Some(focus_id) = self.app.focus().cloned()
            && let Some(section) = Self::editor_section_for_focus(&focus_id)
        {
            self.ensure_editor_section_visible(section);
        }
    }

    fn ensure_editor_section_visible(&mut self, section_kind: EditorSectionKind) {
        if self.active_view != ActiveView::RouteEditor {
            return;
        }

        let sections = self.current_editor_sections();
        if sections.is_empty() {
            self.route_editor.scroll_offset = 0;
            return;
        }

        self.clamp_editor_scroll_offset(sections.len());

        let target_index = sections
            .iter()
            .position(|section| section.kind == section_kind);

        let Some(target_index) = target_index else {
            return;
        };

        let available_height = self.editor_content_viewport_height();
        if available_height == 0 {
            self.route_editor.scroll_offset = target_index;
            return;
        }

        let (start, end) = Self::editor_visible_range(
            &sections,
            self.route_editor.scroll_offset,
            available_height,
        );

        if target_index < start {
            self.route_editor.scroll_offset = target_index;
            return;
        }

        if target_index >= end {
            let mut new_start = target_index;
            let mut used_height = sections[target_index].height;

            while new_start > 0 {
                let previous_height = sections[new_start - 1].height;
                if used_height.saturating_add(previous_height) > available_height {
                    break;
                }

                used_height = used_height.saturating_add(previous_height);
                new_start -= 1;
            }

            self.route_editor.scroll_offset = new_start;
        }
    }

    fn clamp_editor_scroll_offset(&mut self, section_count: usize) {
        if section_count == 0 {
            self.route_editor.scroll_offset = 0;
        } else {
            self.route_editor.scroll_offset =
                self.route_editor.scroll_offset.min(section_count - 1);
        }
    }

    fn editor_content_viewport_height(&self) -> u16 {
        self.terminal_height
            .saturating_sub(Self::editor_footer_height_for_height(self.terminal_height))
    }

    fn editor_footer_height_for_height(height: u16) -> u16 {
        if height == 0 {
            0
        } else if height < Self::EDITOR_FOOTER_MAX_HEIGHT + 1 {
            1
        } else {
            Self::EDITOR_FOOTER_MAX_HEIGHT
        }
    }

    fn editor_visible_range(
        sections: &[EditorSection],
        start: usize,
        available_height: u16,
    ) -> (usize, usize) {
        if sections.is_empty() || available_height == 0 {
            return (0, 0);
        }

        let start = start.min(sections.len().saturating_sub(1));
        let mut end = start;
        let mut used_height: u16 = 0;

        while end < sections.len() {
            let next_height = sections[end].height;
            if used_height.saturating_add(next_height) > available_height {
                break;
            }

            used_height = used_height.saturating_add(next_height);
            end += 1;
        }

        (start, end)
    }

    fn editor_aligned_start(
        sections: &[EditorSection],
        start: usize,
        available_height: u16,
    ) -> usize {
        if sections.is_empty() || available_height == 0 {
            return 0;
        }

        let start = start.min(sections.len().saturating_sub(1));
        let (mut aligned_start, end) =
            Self::editor_visible_range(sections, start, available_height);

        if end < sections.len() {
            return aligned_start;
        }

        let mut used_height: u16 = sections[aligned_start..end]
            .iter()
            .map(|section| section.height)
            .sum();

        while aligned_start > 0 {
            let previous_height = sections[aligned_start - 1].height;
            if used_height.saturating_add(previous_height) > available_height {
                break;
            }

            used_height = used_height.saturating_add(previous_height);
            aligned_start -= 1;
        }

        aligned_start
    }

    fn editor_total_height(sections: &[EditorSection]) -> usize {
        sections.iter().map(|section| section.height as usize).sum()
    }

    fn render_route_list(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        if self.layout_mode != LayoutMode::Wide {
            self.app.view(&Id::RouteList, frame, area);
            return;
        }

        let chunks = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let route_list_area = chunks[0].inner(Margin {
            vertical: 0,
            horizontal: 1,
        });

        self.app.view(&Id::RouteList, frame, route_list_area);
        self.render_route_preview(frame, chunks[1]);
    }

    fn render_route_preview(&self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        if let Some(SelectedItem::Route(selection)) = self.route_list.list_state.selected.as_ref()
            && let Some(route) = self.selected_route(selection)
        {
            self.render_selected_route_preview(frame, area, route);
            return;
        }

        let max_lines = area.height.saturating_sub(2) as usize;
        let (title, lines) = self.route_preview_content(max_lines);

        let preview = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan))
                    .padding(Padding::new(1, 1, 1, 1))
                    .title(title),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(preview, area);
    }

    fn render_selected_route_preview(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: Rect,
        route: &Route,
    ) {
        let preview_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .padding(Padding::new(1, 1, 1, 1))
            .title("Route Preview");
        let inner = preview_block.inner(area);
        frame.render_widget(preview_block, area);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let summary_lines =
            Self::interleave_with_blank_lines(Self::route_summary_lines(route), usize::MAX);
        let summary_height = summary_lines.len() as u16;
        let scope_id = Some(route.scope_id.as_str());
        let body_preview = route
            .body
            .as_deref()
            .map(|body| {
                self.variables
                    .substitute_for_preview_with_scope(body, scope_id)
            })
            .map(|body| body_preview::build(&body))
            .filter(|preview| !preview.lines.is_empty());

        if let Some(preview) = body_preview {
            let body_preview_height = preview.lines.len().clamp(1, u16::MAX as usize) as u16;
            let body_preview_block_height =
                body_preview_height.saturating_add(Self::BODY_PREVIEW_CHROME_HEIGHT);
            let required_height = summary_height
                .saturating_add(Self::BODY_STATUS_HEIGHT)
                .saturating_add(body_preview_block_height);

            if inner.height < required_height {
                let mut lines = summary_lines;
                lines.push(Self::route_body_status_line(route));
                let lines = Self::interleave_with_blank_lines(lines, inner.height as usize);
                frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
                return;
            }

            let chunks = Layout::vertical([
                Constraint::Length(summary_height),
                Constraint::Length(Self::BODY_STATUS_HEIGHT),
                Constraint::Length(body_preview_block_height),
            ])
            .split(inner);

            frame.render_widget(
                Paragraph::new(summary_lines.clone()).wrap(Wrap { trim: false }),
                chunks[0],
            );

            let body_status_area = chunks[1].inner(Margin {
                vertical: 1,
                horizontal: 0,
            });
            frame.render_widget(
                Paragraph::new(vec![Self::route_body_status_line(route)]),
                body_status_area,
            );

            let body_preview_title = match preview.format {
                body_preview::BodyPreviewFormat::Json => "Body Preview (JSON)",
                body_preview::BodyPreviewFormat::Text => "Body Preview",
            };
            let body_preview_lines = Self::truncate_preview_lines(
                preview.lines,
                chunks[2]
                    .height
                    .saturating_sub(Self::BODY_PREVIEW_CHROME_HEIGHT) as usize,
            );

            let body_widget = Paragraph::new(body_preview_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Cyan))
                        .padding(Padding::new(1, 1, 1, 1))
                        .title(body_preview_title),
                )
                .wrap(Wrap { trim: false });
            frame.render_widget(body_widget, chunks[2]);
            return;
        }

        let mut lines = summary_lines;
        lines.push(Self::route_body_status_line(route));
        let lines = Self::interleave_with_blank_lines(lines, inner.height as usize);
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }

    fn route_preview_content(&self, max_lines: usize) -> (String, Vec<Line<'static>>) {
        match self.route_list.list_state.selected.as_ref() {
            Some(SelectedItem::Route(selection)) => {
                let group_name = Self::normalized_group_name(&selection.group);
                let routes = self.group_routes(&group_name);

                if routes.is_empty() {
                    (
                        "Route Preview".to_string(),
                        Self::placeholder_preview_lines(
                            "Selected route is no longer available.",
                            max_lines,
                        ),
                    )
                } else {
                    (
                        "Group Preview".to_string(),
                        self.route_preview_for_group(&group_name, &routes, max_lines),
                    )
                }
            }
            Some(SelectedItem::Group { name }) => {
                let group_name = Self::normalized_group_name(name);
                let routes = self.group_routes(&group_name);

                (
                    "Group Preview".to_string(),
                    self.route_preview_for_group(&group_name, &routes, max_lines),
                )
            }
            None => {
                if let Some(first_route) = self.collection.routes.first() {
                    let group_name = Self::normalized_group_name(&first_route.group);
                    let routes = self.group_routes(&group_name);

                    (
                        "Group Preview".to_string(),
                        self.route_preview_for_group(&group_name, &routes, max_lines),
                    )
                } else {
                    (
                        "Preview".to_string(),
                        Self::placeholder_preview_lines(
                            "No routes. Press 'n' to create one.",
                            max_lines,
                        ),
                    )
                }
            }
        }
    }

    fn route_summary_lines(route: &Route) -> Vec<Line<'static>> {
        let method = route.method.to_string();
        let group_name = Self::normalized_group_name(&route.group);
        let headers_label = if route.headers.is_empty() {
            "Headers: none".to_string()
        } else {
            format!("Headers: {}", route.headers.len())
        };

        vec![
            Line::from(vec![
                Span::styled(
                    format!("{method:<6}"),
                    Style::default()
                        .fg(Self::method_color(&route.method))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::raw(route.name.clone()),
            ]),
            Line::from(format!("Group: {group_name}")),
            Line::from(format!("URL: {}", route.url)),
            Line::from(headers_label),
        ]
    }

    fn route_body_status_line(route: &Route) -> Line<'static> {
        let body_status = route
            .body
            .as_ref()
            .filter(|body| !body.trim().is_empty())
            .map(|body| format!("Body: Set ({} chars)", body.chars().count()))
            .unwrap_or_else(|| "Body: Empty".to_string());

        Line::from(Span::styled(
            body_status,
            Style::default().fg(Color::DarkGray),
        ))
    }

    fn route_preview_for_group(
        &self,
        group_name: &str,
        routes: &[&Route],
        max_lines: usize,
    ) -> Vec<Line<'static>> {
        if max_lines == 0 {
            return vec![];
        }

        let mut entries = vec![Line::from(vec![
            Span::styled(
                "Group: ",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                group_name.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ])];

        entries.push(Line::from(format!("Routes: {}", routes.len())));

        if routes.is_empty() {
            entries.push(Line::from(Span::styled(
                "No routes in this group.",
                Style::default().fg(Color::DarkGray),
            )));
            return Self::interleave_with_blank_lines(entries, max_lines);
        }

        let total_routes = routes.len();
        let mut visible_route_count = 0;

        for candidate in (0..=total_routes).rev() {
            let hidden = total_routes.saturating_sub(candidate);
            let entry_count = 2 + candidate + usize::from(hidden > 0);
            let required_lines = entry_count.saturating_mul(2).saturating_sub(1);

            if required_lines <= max_lines {
                visible_route_count = candidate;
                break;
            }
        }

        entries.extend(
            routes
                .iter()
                .copied()
                .take(visible_route_count)
                .map(Self::group_route_line),
        );

        let hidden_routes = total_routes.saturating_sub(visible_route_count);
        if hidden_routes > 0 {
            entries.push(Line::from(Span::styled(
                format!("... +{hidden_routes} more"),
                Style::default().fg(Color::DarkGray),
            )));
        }

        Self::interleave_with_blank_lines(entries, max_lines)
    }

    fn group_route_line(route: &Route) -> Line<'static> {
        let method = route.method.to_string();

        Line::from(vec![
            Span::styled(
                format!("{method:<6}"),
                Style::default()
                    .fg(Self::method_color(&route.method))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::raw(route.name.clone()),
        ])
    }

    fn selected_route<'a>(&'a self, selection: &RouteSelection) -> Option<&'a Route> {
        self.collection
            .routes
            .iter()
            .find(|route| Self::route_matches_selection(route, selection))
    }

    fn group_routes<'a>(&'a self, group_name: &str) -> Vec<&'a Route> {
        let normalized_group_name = Self::normalized_group_name(group_name);

        self.collection
            .routes
            .iter()
            .filter(|route| Self::normalized_group_name(&route.group) == normalized_group_name)
            .collect()
    }

    fn route_matches_selection(route: &Route, selection: &RouteSelection) -> bool {
        Self::normalized_group_name(&route.group) == Self::normalized_group_name(&selection.group)
            && route.name == selection.name
            && route
                .method
                .to_string()
                .eq_ignore_ascii_case(&selection.method)
            && route.url == selection.url
    }

    fn normalized_group_name(group_name: &str) -> String {
        let trimmed = group_name.trim();

        if trimmed.is_empty() {
            DEFAULT_ROUTE_GROUP.to_string()
        } else {
            trimmed.to_string()
        }
    }

    fn method_color(method: &HttpMethod) -> Color {
        core_style::method_tui_color(method)
    }

    fn truncate_preview_lines(
        mut lines: Vec<Line<'static>>,
        max_lines: usize,
    ) -> Vec<Line<'static>> {
        if max_lines == 0 {
            return vec![];
        }

        if lines.len() <= max_lines {
            return lines;
        }

        if max_lines == 1 {
            lines.truncate(1);
            return lines;
        }

        let hidden_lines = lines.len() - max_lines + 1;
        lines.truncate(max_lines - 1);
        lines.push(Line::from(Span::styled(
            format!("... +{hidden_lines} more"),
            Style::default().fg(Color::DarkGray),
        )));
        lines
    }

    fn placeholder_preview_lines(message: &str, max_lines: usize) -> Vec<Line<'static>> {
        if max_lines == 0 {
            return vec![];
        }

        vec![Line::from(Span::styled(
            message.to_string(),
            Style::default().fg(Color::DarkGray),
        ))]
    }

    fn interleave_with_blank_lines(
        lines: Vec<Line<'static>>,
        max_lines: usize,
    ) -> Vec<Line<'static>> {
        if max_lines == 0 {
            return vec![];
        }

        let mut spaced = Vec::with_capacity(lines.len().saturating_mul(2));
        let mut iter = lines.into_iter().peekable();

        while let Some(line) = iter.next() {
            if spaced.len() >= max_lines {
                break;
            }

            spaced.push(line);

            if iter.peek().is_some() && spaced.len() < max_lines {
                spaced.push(Line::from(""));
            }
        }

        spaced
    }

    fn layout_mode_for_width(width: u16) -> LayoutMode {
        if width >= Self::ROUTE_PREVIEW_MIN_WIDTH {
            LayoutMode::Wide
        } else if width >= 80 {
            LayoutMode::Medium
        } else {
            LayoutMode::Narrow
        }
    }

    pub fn mount_editor(&mut self, route: &Route) -> anyhow::Result<()> {
        let group_names = self.collection.group_names();

        self.app.remount(
            Id::EditorName,
            Box::new(EditorNameInput::new(&route.name)),
            vec![],
        )?;
        self.app.remount(
            Id::EditorGroup,
            Box::new(EditorGroupSelector::new(&group_names, &route.group)),
            vec![],
        )?;
        self.app.remount(
            Id::EditorNewGroup,
            Box::new(EditorNewGroupInput::new("")),
            vec![],
        )?;
        self.app.remount(
            Id::EditorMethod,
            Box::new(EditorMethodRadio::new(&route.method)),
            vec![],
        )?;
        self.app.remount(
            Id::EditorUrl,
            Box::new(EditorUrlInput::new(&route.url)),
            vec![],
        )?;
        self.app.remount(
            Id::EditorHeaders,
            Box::new(EditorHeadersInput::new(&route.headers)),
            vec![],
        )?;

        // Focus the name field initially
        self.route_editor.scroll_offset = 0;
        self.app.active(&Id::EditorName)?;
        self.sync_editor_input_mode()?;

        Ok(())
    }

    fn sync_editor_input_mode(&mut self) -> anyhow::Result<()> {
        let is_insert_mode = self.input_mode == InputMode::Insert;
        let border_color = if is_insert_mode {
            InputColor::Yellow
        } else {
            InputColor::Cyan
        };

        for id in [
            Id::EditorName,
            Id::EditorGroup,
            Id::EditorMethod,
            Id::EditorUrl,
            Id::EditorHeaders,
            Id::EditorNewGroup,
        ] {
            self.app.attr(
                &id,
                Attribute::Custom("input_mode"),
                AttrValue::Flag(is_insert_mode),
            )?;
            self.app.attr(
                &id,
                Attribute::Borders,
                AttrValue::Borders(
                    InputBorders::default()
                        .modifiers(InputBorderType::Rounded)
                        .color(border_color),
                ),
            )?;
        }

        Ok(())
    }

    fn sync_variable_input_mode(&mut self) -> anyhow::Result<()> {
        let is_insert_mode = self.input_mode == InputMode::Insert;
        let border_color = if is_insert_mode {
            InputColor::Yellow
        } else {
            InputColor::Cyan
        };

        for id in [
            Id::VariableKeyInput,
            Id::VariableValueInput,
            Id::VariableMode,
        ] {
            self.app.attr(
                &id,
                Attribute::Custom("input_mode"),
                AttrValue::Flag(is_insert_mode),
            )?;
            self.app.attr(
                &id,
                Attribute::Borders,
                AttrValue::Borders(
                    InputBorders::default()
                        .modifiers(InputBorderType::Rounded)
                        .color(border_color),
                ),
            )?;
        }

        Ok(())
    }

    fn sync_variable_value_visibility(&mut self) -> anyhow::Result<()> {
        let input_type = if self.variable_mode_value() == VariableMode::Hidden
            && !self.variable_manager.secrets_visible
        {
            FieldInputType::Password('*')
        } else {
            FieldInputType::Text
        };

        self.app.attr(
            &Id::VariableValueInput,
            Attribute::InputType,
            AttrValue::InputType(input_type),
        )?;

        Ok(())
    }

    fn active_scope_id(&self) -> Option<&str> {
        match &self.variable_manager.context {
            VariableContext::Global => None,
            VariableContext::Scoped { scope_id } => Some(scope_id.as_str()),
        }
    }

    fn is_scoped_variable_context(&self) -> bool {
        matches!(
            self.variable_manager.context,
            VariableContext::Scoped { .. }
        )
    }

    fn variable_mode_value(&self) -> VariableMode {
        match self.app.state(&Id::VariableMode) {
            Ok(State::One(StateValue::Usize(1))) => VariableMode::Hidden,
            _ => VariableMode::Placeholder,
        }
    }

    fn editor_input_value(&self, id: &Id) -> String {
        match self.app.state(id) {
            Ok(State::One(StateValue::String(value))) => value,
            _ => String::new(),
        }
    }

    fn content_area(area: Rect) -> Rect {
        area.inner(Margin {
            vertical: 1,
            horizontal: 2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_preview_scroll_metrics_detect_one_line_overflow() {
        let lines = vec![
            Line::from("line-1"),
            Line::from("line-2"),
            Line::from("line-3"),
            Line::from("line-4"),
            Line::from("line-5"),
        ];

        let (total_lines, viewport_height, max_offset) =
            AppModel::request_preview_scroll_metrics(&lines, Rect::new(0, 0, 40, 4));

        assert_eq!(total_lines, 5);
        assert_eq!(viewport_height, 4);
        assert_eq!(max_offset, 1);
    }

    #[test]
    fn request_preview_scroll_metrics_account_for_wrapping() {
        let lines = vec![Line::from("0123456789 0123456789 0123456789")];

        let (total_lines, viewport_height, max_offset) =
            AppModel::request_preview_scroll_metrics(&lines, Rect::new(0, 0, 10, 2));

        assert!(total_lines > lines.len());
        assert_eq!(viewport_height, 2);
        assert!(max_offset > 0);
    }

    #[test]
    fn keymap_entries_include_expected_route_editor_shortcuts() {
        let (title, entries) = AppModel::view_keymap_entries(ActiveView::RouteEditor);

        assert_eq!(title, "Route Editor");
        assert!(entries.iter().any(|entry| entry.contains("Ctrl+S")));
        assert!(entries.iter().any(|entry| entry.contains("v: Global vars")));
    }

    #[test]
    fn keymap_entries_include_keymap_help_fallback() {
        let (title, entries) = AppModel::view_keymap_entries(ActiveView::KeymapHelp);

        assert_eq!(title, "Keymaps");
        assert!(entries.iter().any(|entry| entry.contains("?")));
    }

    #[test]
    fn keymap_scroll_metrics_account_for_wrapping() {
        let lines = vec![Line::from(
            "a: Add variable | e: Edit variable | d: Delete variable | Ctrl+S: Save variable",
        )];

        let (total_lines, viewport_height, max_offset) =
            AppModel::keymap_scroll_metrics(&lines, Rect::new(0, 0, 16, 2));

        assert!(total_lines > lines.len());
        assert_eq!(viewport_height, 2);
        assert!(max_offset > 0);
    }
}
