use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::{Application, NoUserEvent};

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
    None,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LayoutMode {
    Wide,
    Medium,
    Narrow,
}

pub struct AppModel {
    pub app: Application<Id, Msg, NoUserEvent>,
    pub collection: Collection,
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
                self.selected_route = Some(index);
                // Execution will be handled in #124
            }
            Msg::EditRoute(index) => {
                self.editing_route = Some(index);
                self.editor_draft = Some(self.collection.routes[index].clone());
                self.active_view = ActiveView::RouteEditor;
            }
            Msg::NewRoute => {
                self.editing_route = None;
                self.editor_draft = Some(Route {
                    group: DEFAULT_ROUTE_GROUP.to_string(),
                    name: String::new(),
                    method: crate::api_tester::collection::HttpMethod::Get,
                    url: String::new(),
                    headers: vec![],
                    body: None,
                });
                self.active_view = ActiveView::RouteEditor;
            }
            Msg::DeleteRoute(index) => {
                self.collection.delete_route(index)?;
                self.collection.save()?;
                self.refresh_route_list()?;
            }
            Msg::RefreshList | Msg::None => {}
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
                // Draw RouteEditor component.
            }
            ActiveView::ResponseViewer => {
                // Draw ResponseViewer component.
            }
            ActiveView::VariableManager => {
                // Draw VariableManager component.
            }
        }
    }
}
