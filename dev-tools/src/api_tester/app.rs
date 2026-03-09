use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::{Application, AttrValue, Attribute, NoUserEvent, State, StateValue};

use crate::api_tester::collection::HttpMethod;
use crate::api_tester::components::route_editor::group_selector::EditorGroupSelector;
use crate::api_tester::components::route_editor::headers_input::EditorHeadersInput;
use crate::api_tester::components::route_editor::method_radio::EditorMethodRadio;
use crate::api_tester::components::route_editor::name_input::EditorNameInput;
use crate::api_tester::components::route_editor::new_group_input::EditorNewGroupInput;
use crate::api_tester::components::route_editor::url_input::EditorUrlInput;
use crate::api_tester::{
    collection::{Collection, Route, DEFAULT_ROUTE_GROUP},
    components::route_list::RouteList,
    executor::CurlResponse,
    variables::Variables,
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
    EditorBody,
    ResponseTabs,
    ResponseStatus,
    ResponseHeaders,
    ResponseBody,
    VariableTable,
    VariableKeyInput,
    VariableValueInput,
    EditorGroup,
    EditorNewGroup,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActiveView {
    RouteList,
    RouteEditor,
    ResponseViewer,
    VariableManager,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Msg {
    AppClose,
    SwitchView(ActiveView),
    RunRoute(usize),
    EditRoute(usize),
    NewRoute,
    DeleteRoute(usize),
    SaveRoute,
    CancelEdit,
    OpenBodyEditor,
    RefreshList,
    RouteSelected(usize),
    FocusField(Id),
    MethodChanged(usize),
    BodyEditorResult(Option<String>),
    ResponseTabChanged(usize),
    ToggleSecretVisibility,
    AddVariable,
    DeleteVariable(String),
    UpdateVariable(String, String),
    TerminalResize(u16, u16),
    EnterInsertMode,
    EnterNormalMode,
    GroupSelected(usize),
    NewGroupEntered,
    None,
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

pub struct AppModel {
    pub app: Application<Id, Msg, NoUserEvent>,
    pub collection: Collection,
    pub input_mode: InputMode,
    variables: Variables,
    active_view: ActiveView,
    response: Option<CurlResponse>,
    selected_route: Option<usize>,
    editing_route: Option<usize>,
    editor_draft: Option<Route>,
    response_tab: usize,
    secrets_visible: bool,
    layout_mode: LayoutMode,
    terminal_width: u16,
}

impl AppModel {
    pub fn new(app: Application<Id, Msg, NoUserEvent>) -> anyhow::Result<Self> {
        Ok(Self {
            app,
            collection: Collection::load()?,
            input_mode: InputMode::Normal,
            variables: Variables::load()?,
            active_view: ActiveView::RouteList,
            response: None,
            selected_route: None,
            editing_route: None,
            editor_draft: None,
            response_tab: 0,
            secrets_visible: false,
            layout_mode: LayoutMode::Wide,
            terminal_width: 120,
        })
    }

    pub fn update(&mut self, msg: Msg) -> anyhow::Result<Option<Msg>> {
        match msg {
            Msg::AppClose => return Ok(Some(Msg::AppClose)),
            Msg::SwitchView(view) => self.active_view = view,
            Msg::RunRoute(index) => {
                if self.active_view != ActiveView::RouteList {
                    return Ok(None);
                }

                self.selected_route = Some(index);
                // Execution will be handled in #124
            }
            Msg::EditRoute(index) => {
                if self.active_view != ActiveView::RouteList {
                    return Ok(None);
                }

                let route = self.collection.routes[index].clone();
                self.editing_route = Some(index);
                self.editor_draft = Some(route.clone());
                self.input_mode = InputMode::Normal;
                self.mount_editor(&route)?;
                self.active_view = ActiveView::RouteEditor;
            }
            Msg::NewRoute => {
                if self.active_view != ActiveView::RouteList {
                    return Ok(None);
                }

                let route = Route {
                    group: DEFAULT_ROUTE_GROUP.to_string(),
                    name: String::new(),
                    method: HttpMethod::Get,
                    url: String::new(),
                    headers: vec![],
                    body: None,
                };

                self.editing_route = None;
                self.editor_draft = Some(route.clone());
                self.input_mode = InputMode::Normal;
                self.mount_editor(&route)?;
                self.active_view = ActiveView::RouteEditor;
            }
            Msg::DeleteRoute(index) => {
                if self.active_view != ActiveView::RouteList {
                    return Ok(None);
                }

                self.collection.delete_route(index)?;
                self.collection.save()?;
                self.refresh_route_list()?;
            }
            Msg::RefreshList | Msg::None => {}
            Msg::EnterInsertMode => {
                self.input_mode = InputMode::Insert;
                if self.active_view == ActiveView::RouteEditor {
                    self.sync_editor_input_mode()?;
                }
            }
            Msg::EnterNormalMode => {
                self.input_mode = InputMode::Normal;
                if self.active_view == ActiveView::RouteEditor {
                    self.sync_editor_input_mode()?;
                }
            }
            Msg::FocusField(id) => {
                let _ = self.app.active(&id);
            }
            Msg::GroupSelected(index) => {
                // If "New Group..." is selected (last item), the view will show the new group input
                // Otherwise, store the selected group name for use during save
            }

            Msg::NewGroupEntered => {
                // Focus moves to method after entering new group name
                self.app.active(&Id::EditorMethod)?;
            }
            Msg::SaveRoute => {
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

                    let body = self.editor_draft.as_ref().and_then(|d| d.body.clone());

                    let route = Route {
                        group,
                        name,
                        method,
                        url,
                        headers,
                        body,
                    };

                    if let Some(index) = self.editing_route {
                        self.collection.update_route(index, route)?;
                    } else {
                        self.collection.add_route(route);
                    }

                    self.collection.save()?;
                    self.editing_route = None;
                    self.editor_draft = None;
                    self.input_mode = InputMode::Normal;
                    self.active_view = ActiveView::RouteList;
                    self.refresh_route_list()?;
                }
            }
            Msg::CancelEdit => {
                if self.active_view == ActiveView::RouteEditor {
                    self.editing_route = None;
                    self.editor_draft = None;
                    self.input_mode = InputMode::Normal;
                    self.active_view = ActiveView::RouteList;
                }
            }
            _ => {}
        }

        Ok(None)
    }

    pub fn refresh_route_list(&mut self) -> anyhow::Result<()> {
        self.app.umount(&Id::RouteList)?;
        self.app.mount(
            Id::RouteList,
            Box::new(RouteList::new(&self.collection.routes)),
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
        match self.active_view {
            ActiveView::RouteList => {
                self.app.view(&Id::RouteList, frame, frame.area());
            }
            ActiveView::RouteEditor => {
                use ratatui::layout::{Constraint, Layout};

                // Check if "New Group..." is selected to show the new group input
                let show_new_group = if let Ok(State::One(StateValue::Usize(idx))) =
                    self.app.state(&Id::EditorGroup)
                {
                    let group_names = self.collection.group_names();
                    idx >= group_names.len()
                } else {
                    false
                };

                let mut constraints = vec![
                    Constraint::Length(3), // Name
                    Constraint::Length(3), // Group selector
                ];

                if show_new_group {
                    constraints.push(Constraint::Length(3)); // New group input
                }

                constraints.extend([
                    Constraint::Length(3), // Method
                    Constraint::Length(3), // URL
                    Constraint::Length(3), // Headers
                    Constraint::Length(1), // Body status
                    Constraint::Length(1), // Mode indicator
                    Constraint::Min(1),    // Help
                ]);

                let chunks = Layout::vertical(constraints).split(frame.area());

                let mut idx = 0;
                self.app.view(&Id::EditorName, frame, chunks[idx]);
                idx += 1;
                self.app.view(&Id::EditorGroup, frame, chunks[idx]);
                idx += 1;

                if show_new_group {
                    self.app.view(&Id::EditorNewGroup, frame, chunks[idx]);
                    idx += 1;
                }

                self.app.view(&Id::EditorMethod, frame, chunks[idx]);
                idx += 1;
                self.app.view(&Id::EditorUrl, frame, chunks[idx]);
                idx += 1;
                self.app.view(&Id::EditorHeaders, frame, chunks[idx]);
                idx += 1;

                // Body status
                let body_status = if self
                    .editor_draft
                    .as_ref()
                    .and_then(|d| d.body.as_ref())
                    .is_some()
                {
                    "Body: Set (press 'b' to edit in Normal mode)"
                } else {
                    "Body: Empty (press 'b' to add in Normal mode)"
                };
                let body_widget = ratatui::widgets::Paragraph::new(body_status)
                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray));
                frame.render_widget(body_widget, chunks[idx]);
                idx += 1;

                // Mode indicator
                let mode_label = match self.input_mode {
                    InputMode::Normal => "-- NORMAL --",
                    InputMode::Insert => "-- INSERT --",
                };
                let mode_widget = ratatui::widgets::Paragraph::new(mode_label)
                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
                frame.render_widget(mode_widget, chunks[idx]);
                idx += 1;

                // Help text
                let help = ratatui::widgets::Paragraph::new(
                    "i: Insert mode | Esc: Normal/Cancel | Ctrl+S: Save | Tab/Shift+Tab: Navigate",
                )
                .style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray));
                frame.render_widget(help, chunks[idx]);
            }
            ActiveView::ResponseViewer => {
                // Draw ResponseViewer component.
            }
            ActiveView::VariableManager => {
                // Draw VariableManager component.
            }
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
        self.app.active(&Id::EditorName)?;
        self.sync_editor_input_mode()?;

        Ok(())
    }

    fn sync_editor_input_mode(&mut self) -> anyhow::Result<()> {
        let is_insert_mode = self.input_mode == InputMode::Insert;

        for id in [
            Id::EditorName,
            Id::EditorUrl,
            Id::EditorHeaders,
            Id::EditorNewGroup,
        ] {
            self.app.attr(
                &id,
                Attribute::Custom("input_mode"),
                AttrValue::Flag(is_insert_mode),
            )?;
        }

        Ok(())
    }

    fn editor_input_value(&self, id: &Id) -> String {
        match self.app.state(id) {
            Ok(State::One(StateValue::String(value))) => value,
            _ => String::new(),
        }
    }
}
