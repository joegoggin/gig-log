use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
};
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
    body_preview,
    collection::{Collection, DEFAULT_ROUTE_GROUP, Route},
    components::route_list::RouteList,
    executor::CurlResponse,
    route_list_state::{RouteListState, RouteSelection, SelectedItem},
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
    RouteListStateChanged(RouteListState),
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
    route_list_state: RouteListState,
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
    const ROUTE_PREVIEW_MIN_WIDTH: u16 = 110;
    const BODY_STATUS_HEIGHT: u16 = 3;
    const BODY_PREVIEW_CHROME_HEIGHT: u16 = 4;

    pub fn new(app: Application<Id, Msg, NoUserEvent>) -> anyhow::Result<Self> {
        Ok(Self {
            app,
            collection: Collection::load()?,
            input_mode: InputMode::Normal,
            variables: Variables::load()?,
            route_list_state: RouteListState::load(),
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

                if index >= self.collection.routes.len() {
                    return Ok(None);
                }

                self.selected_route = Some(index);
                self.select_route_in_state(index, true);
                self.persist_route_list_state();
                // Execution will be handled in #124
            }
            Msg::EditRoute(index) => {
                if self.active_view != ActiveView::RouteList {
                    return Ok(None);
                }

                if index >= self.collection.routes.len() {
                    return Ok(None);
                }

                let route = self.collection.routes[index].clone();
                self.select_route_in_state(index, true);
                self.persist_route_list_state();
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

                if index >= self.collection.routes.len() {
                    return Ok(None);
                }

                self.collection.delete_route(index)?;
                self.collection.save()?;

                if self.collection.routes.is_empty() {
                    self.route_list_state.selected = None;
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
            Msg::RouteListStateChanged(state) => {
                self.route_list_state = state;
                self.persist_route_list_state();
            }
            Msg::TerminalResize(width, _height) => {
                self.terminal_width = width;
                self.layout_mode = Self::layout_mode_for_width(width);
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
            Msg::GroupSelected(_index) => {
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
                        self.select_route_in_state(index, true);
                    } else {
                        self.collection.add_route(route);
                        let new_index = self.collection.routes.len().saturating_sub(1);
                        self.select_route_in_state(new_index, true);
                    }

                    self.collection.save()?;
                    self.persist_route_list_state();
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
            Msg::OpenBodyEditor => {
                if self.active_view == ActiveView::RouteEditor {
                    // Signal to the event loop that we need to suspend for external editor
                    return Ok(Some(Msg::OpenBodyEditor));
                }
            }
            Msg::BodyEditorResult(body) => {
                if self.active_view == ActiveView::RouteEditor {
                    if let Some(draft) = &mut self.editor_draft {
                        draft.body = body;
                    }
                }
            }
            _ => {}
        }

        Ok(None)
    }

    pub fn editor_draft_body(&self) -> Option<&str> {
        self.editor_draft
            .as_ref()
            .and_then(|draft| draft.body.as_deref())
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
                    .route_list_state
                    .expanded_groups
                    .iter()
                    .any(|name| name == &group_name)
            {
                self.route_list_state.expanded_groups.push(group_name);
            }

            self.route_list_state.selected =
                Some(SelectedItem::Route(RouteSelection::from_route(route)));
        }
    }

    fn persist_route_list_state(&self) {
        if let Err(error) = self.route_list_state.save() {
            eprintln!("Warning: failed to persist route list state: {error}");
        }
    }

    pub fn refresh_route_list(&mut self) -> anyhow::Result<()> {
        let _ = self.app.umount(&Id::RouteList);
        self.app.mount(
            Id::RouteList,
            Box::new(RouteList::new(
                &self.collection.routes,
                &self.route_list_state,
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
        self.layout_mode = Self::layout_mode_for_width(self.terminal_width);

        match self.active_view {
            ActiveView::RouteList => {
                self.render_route_list(frame, content_area);
            }
            ActiveView::RouteEditor => {
                // Check if "New Group..." is selected to show the new group input
                let show_new_group = if let Ok(State::One(StateValue::Usize(idx))) =
                    self.app.state(&Id::EditorGroup)
                {
                    let group_names = self.collection.group_names();
                    idx >= group_names.len()
                } else {
                    false
                };

                let body_preview = self
                    .editor_draft
                    .as_ref()
                    .and_then(|draft| draft.body.as_deref())
                    .map(|body| self.variables.substitute_for_preview(body))
                    .map(|body| body_preview::build(&body))
                    .filter(|preview| !preview.lines.is_empty());

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
                    Constraint::Length(Self::BODY_STATUS_HEIGHT), // Body status
                ]);

                if let Some(preview) = body_preview.as_ref() {
                    let preview_height = preview.lines.len().clamp(1, u16::MAX as usize) as u16;
                    constraints.push(Constraint::Length(
                        preview_height.saturating_add(Self::BODY_PREVIEW_CHROME_HEIGHT),
                    ));
                }

                constraints.extend([
                    Constraint::Length(2), // Mode indicator + bottom margin
                    Constraint::Min(1),    // Help
                ]);

                let chunks = Layout::vertical(constraints).split(content_area);

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
                let body_status = if body_preview.is_some() {
                    "Body: Set (press 'b' to edit in Normal mode)"
                } else {
                    "Body: Empty (press 'b' to add in Normal mode)"
                };
                let body_widget =
                    Paragraph::new(body_status).style(Style::default().fg(Color::DarkGray));
                let body_status_area = chunks[idx].inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                });
                frame.render_widget(body_widget, body_status_area);
                idx += 1;

                if let Some(preview) = body_preview {
                    let title = match preview.format {
                        body_preview::BodyPreviewFormat::Json => "Body Preview (JSON)",
                        body_preview::BodyPreviewFormat::Text => "Body Preview",
                    };

                    let preview_widget = Paragraph::new(preview.lines)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_type(BorderType::Rounded)
                                .border_style(Style::default().fg(Color::Cyan))
                                .padding(Padding::new(1, 1, 1, 1))
                                .title(title),
                        )
                        .wrap(Wrap { trim: false });
                    frame.render_widget(preview_widget, chunks[idx]);
                    idx += 1;
                }

                // Mode indicator
                let mode_label = match self.input_mode {
                    InputMode::Normal => "-- NORMAL --",
                    InputMode::Insert => "-- INSERT --",
                };
                let mode_widget =
                    Paragraph::new(mode_label).style(Style::default().fg(Color::Yellow));
                frame.render_widget(mode_widget, chunks[idx]);
                idx += 1;

                // Help text
                let help = Paragraph::new(
                    "i: Insert mode | Esc: Normal/Cancel | Ctrl+S: Save | Tab/Shift+Tab: Navigate",
                )
                .style(Style::default().fg(Color::DarkGray));
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

    fn render_route_list(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        if self.layout_mode != LayoutMode::Wide {
            self.app.view(&Id::RouteList, frame, area);
            return;
        }

        let chunks = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let route_list_area = chunks[0].inner(Margin {
            vertical: 1,
            horizontal: 1,
        });

        self.app.view(&Id::RouteList, frame, route_list_area);
        self.render_route_preview(frame, chunks[1]);
    }

    fn render_route_preview(&self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        if let Some(SelectedItem::Route(selection)) = self.route_list_state.selected.as_ref()
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
        let body_preview = route
            .body
            .as_deref()
            .map(|body| self.variables.substitute_for_preview(body))
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
        match self.route_list_state.selected.as_ref() {
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
        match method {
            HttpMethod::Get => Color::Green,
            HttpMethod::Post => Color::Blue,
            HttpMethod::Put => Color::Yellow,
            HttpMethod::Patch => Color::Magenta,
            HttpMethod::Delete => Color::Red,
        }
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

    fn content_area(area: Rect) -> Rect {
        area.inner(Margin {
            vertical: 1,
            horizontal: 2,
        })
    }
}
