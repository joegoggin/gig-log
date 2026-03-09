use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{
    BorderType as InputBorderType, Borders as InputBorders, Color as InputColor,
};
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
    EditorScrollUp,
    EditorScrollDown,
    EditorPageUp,
    EditorPageDown,
    EditorScrollBy(isize),
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
    terminal_height: u16,
    editor_scroll_offset: usize,
}

impl AppModel {
    const ROUTE_PREVIEW_MIN_WIDTH: u16 = 110;
    const BODY_STATUS_HEIGHT: u16 = 3;
    const BODY_PREVIEW_CHROME_HEIGHT: u16 = 4;
    const EDITOR_FOOTER_MAX_HEIGHT: u16 = 2;
    const EDITOR_SCROLLBAR_MIN_CONTENT_WIDTH: u16 = 30;
    const EDITOR_SCROLLBAR_WIDTH: u16 = 1;
    const EDITOR_HELP_TEXT: &'static str = "i: Insert | Esc: Normal/Cancel | Ctrl+S: Save | Tab/Shift+Tab: Navigate | h/l or Left/Right: Edit radio (Insert) or move text cursor | j/k or Up/Down: Scroll | Swipe/Mouse Wheel: Scroll";

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
            terminal_height: 40,
            editor_scroll_offset: 0,
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
                self.editor_scroll_offset = 0;
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
                self.editor_scroll_offset = 0;
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
            Msg::TerminalResize(width, height) => {
                self.terminal_width = width;
                self.terminal_height = height.saturating_sub(2);
                self.layout_mode = Self::layout_mode_for_width(width);
            }
            Msg::RefreshList | Msg::None => {}
            Msg::EnterInsertMode => {
                self.input_mode = InputMode::Insert;
                if self.active_view == ActiveView::RouteEditor {
                    self.ensure_editor_focus_visible();
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
                if self.input_mode == InputMode::Insert
                    && let Some(section) = Self::editor_section_for_focus(&id)
                {
                    self.ensure_editor_section_visible(section);
                }
            }
            Msg::EditorScrollUp => self.scroll_editor_by(-1),
            Msg::EditorScrollDown => self.scroll_editor_by(1),
            Msg::EditorPageUp => self.scroll_editor_page(-1),
            Msg::EditorPageDown => self.scroll_editor_page(1),
            Msg::EditorScrollBy(delta) => self.scroll_editor_by(delta),
            Msg::GroupSelected(_index) => {
                // If "New Group..." is selected (last item), the view will show the new group input
                // Otherwise, store the selected group name for use during save
            }

            Msg::NewGroupEntered => {
                // Focus moves to method after entering new group name
                self.app.active(&Id::EditorMethod)?;
                self.ensure_editor_section_visible(EditorSectionKind::Method);
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
                    self.editor_scroll_offset = 0;
                    self.active_view = ActiveView::RouteList;
                    self.refresh_route_list()?;
                }
            }
            Msg::CancelEdit => {
                if self.active_view == ActiveView::RouteEditor {
                    self.editing_route = None;
                    self.editor_draft = None;
                    self.input_mode = InputMode::Normal;
                    self.editor_scroll_offset = 0;
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
        self.terminal_height = content_area.height;
        self.layout_mode = Self::layout_mode_for_width(self.terminal_width);

        match self.active_view {
            ActiveView::RouteList => {
                self.render_route_list(frame, content_area);
            }
            ActiveView::RouteEditor => {
                self.render_route_editor(frame, content_area);
            }
            ActiveView::ResponseViewer => {
                // Draw ResponseViewer component.
            }
            ActiveView::VariableManager => {
                // Draw VariableManager component.
            }
        }
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
            && editor_area.width
                > Self::EDITOR_SCROLLBAR_MIN_CONTENT_WIDTH + Self::EDITOR_SCROLLBAR_WIDTH;
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
            .editor_scroll_offset
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

        self.editor_scroll_offset = start;

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

        if area.height < Self::EDITOR_FOOTER_MAX_HEIGHT {
            return;
        }

        let help_widget =
            Paragraph::new(Self::EDITOR_HELP_TEXT).style(Style::default().fg(Color::DarkGray));
        let help_area = Rect {
            x: area.x,
            y: area.y.saturating_add(1),
            width: area.width,
            height: 1,
        };
        frame.render_widget(help_widget, help_area);
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
        self.editor_draft
            .as_ref()
            .and_then(|draft| draft.body.as_deref())
            .map(|body| self.variables.substitute_for_preview(body))
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
            self.editor_scroll_offset = 0;
            return;
        }

        let max_offset = section_count.saturating_sub(1) as isize;
        let next = (self.editor_scroll_offset as isize + delta).clamp(0, max_offset);
        self.editor_scroll_offset = next as usize;
    }

    fn scroll_editor_page(&mut self, direction: isize) {
        if self.active_view != ActiveView::RouteEditor {
            return;
        }

        let sections = self.current_editor_sections();
        if sections.is_empty() {
            self.editor_scroll_offset = 0;
            return;
        }

        self.clamp_editor_scroll_offset(sections.len());

        let available_height = self.editor_content_viewport_height();
        if available_height == 0 {
            return;
        }

        let start = self
            .editor_scroll_offset
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
            self.editor_scroll_offset = 0;
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
            self.editor_scroll_offset = target_index;
            return;
        }

        let (start, end) =
            Self::editor_visible_range(&sections, self.editor_scroll_offset, available_height);

        if target_index < start {
            self.editor_scroll_offset = target_index;
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

            self.editor_scroll_offset = new_start;
        }
    }

    fn clamp_editor_scroll_offset(&mut self, section_count: usize) {
        if section_count == 0 {
            self.editor_scroll_offset = 0;
        } else {
            self.editor_scroll_offset = self.editor_scroll_offset.min(section_count - 1);
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
        self.editor_scroll_offset = 0;
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
