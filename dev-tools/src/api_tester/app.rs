use crate::api_tester::{collection::Collection, executor::CurlResponse, variables::Variables};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    RouteList,
    RouteEditor,
    BodyEditor,
    ResponseViewer,
    VariableManager,
    GlobalListener,
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
    None,
}

pub struct AppModel {
    collection: Collection,
    variables: Variables,
    active_view: ActiveView,
    response: Option<CurlResponse>,
}

impl AppModel {
    pub fn init() -> anyhow::Result<Self> {
        Ok(Self {
            collection: Collection::load()?,
            variables: Variables::load()?,
            active_view: ActiveView::RouteList,
            response: None,
        })
    }

    pub fn update(&mut self, msg: Msg) -> anyhow::Result<Option<Msg>> {
        match msg {
            Msg::AppClose => return Ok(Some(Msg::AppClose)),
            Msg::SwitchView(view) => self.active_view = view,
            Msg::DeleteRoute(index) => {
                self.collection.delete_route(index)?;
                self.collection.save()?;
            }
            Msg::RefreshList | Msg::None => {}
            _ => {}
        }

        Ok(None)
    }

    pub fn view(&mut self, frame: &mut ratatui::Frame<'_>) {
        match self.active_view {
            ActiveView::RouteList => {
                // Draw RouteList component.
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
