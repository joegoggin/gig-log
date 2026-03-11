use ratatui::{
    layout::{Constraint, Layout},
    style::{Color as TuiColor, Style},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use tui_realm_stdlib::List;
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent, KeyModifiers},
    props::{BorderSides, Borders, Color, PropPayload, PropValue, TextSpan},
};

use crate::api_tester::{
    app::Msg,
    collection::{DEFAULT_ROUTE_GROUP, HttpMethod, Route},
    route_list_state::{RouteListState, RouteSelection, SelectedItem},
};

struct RouteGroup {
    name: String,
    route_indexes: Vec<usize>,
    expanded: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RowKind {
    GroupHeader(usize),
    Route(usize),
    Spacer,
    EmptyState,
}

impl RowKind {
    fn is_selectable(&self) -> bool {
        matches!(self, Self::GroupHeader(_) | Self::Route(_))
    }
}

enum SelectionTarget {
    GroupHeader(usize),
    Route(usize),
}

const SCROLLBAR_WIDTH: u16 = 1;

pub struct RouteList {
    component: List,
    routes: Vec<Route>,
    groups: Vec<RouteGroup>,
    row_kinds: Vec<RowKind>,
    pending_g: bool,
}

impl RouteList {
    pub fn new(routes: &[Route], persisted_state: &RouteListState) -> Self {
        let mut groups: Vec<RouteGroup> = vec![];

        for (route_index, route) in routes.iter().enumerate() {
            let group_name = Self::group_name(route);

            if let Some(group) = groups.iter_mut().find(|group| group.name == group_name) {
                group.route_indexes.push(route_index);
            } else {
                groups.push(RouteGroup {
                    name: group_name,
                    route_indexes: vec![route_index],
                    expanded: false,
                });
            }
        }

        for group in &mut groups {
            group.expanded = persisted_state
                .expanded_groups
                .iter()
                .any(|name| name == &group.name);
        }

        let initial_selection_target =
            Self::selection_target_from_state(persisted_state, routes, &mut groups);

        let mut list = Self {
            component: List::default(),
            routes: routes.to_vec(),
            groups,
            row_kinds: vec![],
            pending_g: false,
        };
        list.rebuild_rows(initial_selection_target);
        list
    }

    fn selection_target_from_state(
        persisted_state: &RouteListState,
        routes: &[Route],
        groups: &mut [RouteGroup],
    ) -> Option<SelectionTarget> {
        match persisted_state.selected.as_ref() {
            Some(SelectedItem::Route(saved_route)) => {
                if let Some(route_index) = routes
                    .iter()
                    .position(|route| Self::route_matches_selection(route, saved_route))
                {
                    if let Some(group_index) = groups
                        .iter()
                        .position(|group| group.route_indexes.contains(&route_index))
                    {
                        groups[group_index].expanded = true;
                    }

                    Some(SelectionTarget::Route(route_index))
                } else {
                    groups
                        .iter()
                        .position(|group| group.name == saved_route.group)
                        .map(SelectionTarget::GroupHeader)
                }
            }
            Some(SelectedItem::Group { name }) => groups
                .iter()
                .position(|group| &group.name == name)
                .map(SelectionTarget::GroupHeader),
            None => None,
        }
    }

    fn route_matches_selection(route: &Route, saved_route: &RouteSelection) -> bool {
        Self::group_name(route) == saved_route.group
            && route.name == saved_route.name
            && Self::method_label(&route.method).eq_ignore_ascii_case(&saved_route.method)
            && route.url == saved_route.url
    }

    fn group_name(route: &Route) -> String {
        if route.group.trim().is_empty() {
            DEFAULT_ROUTE_GROUP.to_string()
        } else {
            route.group.clone()
        }
    }

    fn selected_route_index(&self) -> Option<usize> {
        match self.selected_row_kind() {
            Some(RowKind::Route(route_index)) => Some(route_index),
            _ => None,
        }
    }

    fn selected_group_index(&self) -> Option<usize> {
        match self.selected_row_kind() {
            Some(RowKind::GroupHeader(group_index)) => Some(group_index),
            _ => None,
        }
    }

    fn selected_row_kind(&self) -> Option<RowKind> {
        if let State::One(StateValue::Usize(index)) = self.state() {
            self.row_kinds.get(index).copied()
        } else {
            None
        }
    }

    fn selected_row_index(&self) -> Option<usize> {
        if let State::One(StateValue::Usize(index)) = self.state() {
            Some(index)
        } else {
            None
        }
    }

    fn set_selected_row(&mut self, row_index: usize) {
        self.attr(
            Attribute::Value,
            AttrValue::Payload(PropPayload::One(PropValue::Usize(row_index))),
        );
    }

    fn first_selectable_row_index(&self) -> Option<usize> {
        self.row_kinds.iter().position(RowKind::is_selectable)
    }

    fn last_selectable_row_index(&self) -> Option<usize> {
        self.row_kinds.iter().rposition(RowKind::is_selectable)
    }

    fn first_group_header_row_index(&self) -> Option<usize> {
        self.row_kinds
            .iter()
            .position(|kind| matches!(kind, RowKind::GroupHeader(_)))
    }

    fn last_group_header_row_index(&self) -> Option<usize> {
        self.row_kinds
            .iter()
            .rposition(|kind| matches!(kind, RowKind::GroupHeader(_)))
    }

    fn move_selection_down(&mut self) {
        let current = self.selected_row_index().unwrap_or(0);

        if let Some(next) = self
            .row_kinds
            .iter()
            .enumerate()
            .skip(current.saturating_add(1))
            .find_map(|(index, kind)| kind.is_selectable().then_some(index))
        {
            self.set_selected_row(next);
        } else if let Some(first) = self.first_selectable_row_index() {
            self.set_selected_row(first);
        }
    }

    fn move_selection_up(&mut self) {
        let current = self.selected_row_index().unwrap_or(0);

        if let Some(previous) = (0..current)
            .rev()
            .find(|&index| self.row_kinds[index].is_selectable())
        {
            self.set_selected_row(previous);
        } else if let Some(last) = self.last_selectable_row_index() {
            self.set_selected_row(last);
        }
    }

    fn move_selection_to_next_group_header(&mut self) {
        let current = self.selected_row_index().unwrap_or(0);

        if let Some(next_group_header) = self
            .row_kinds
            .iter()
            .enumerate()
            .skip(current.saturating_add(1))
            .find_map(|(index, kind)| matches!(kind, RowKind::GroupHeader(_)).then_some(index))
        {
            self.set_selected_row(next_group_header);
        } else if let Some(first_group_header) = self.first_group_header_row_index() {
            self.set_selected_row(first_group_header);
        }
    }

    fn move_selection_to_previous_group_header(&mut self) {
        let current = self.selected_row_index().unwrap_or(0);

        if let Some(previous_group_header) = (0..current)
            .rev()
            .find(|&index| matches!(self.row_kinds[index], RowKind::GroupHeader(_)))
        {
            self.set_selected_row(previous_group_header);
        } else if let Some(last_group_header) = self.last_group_header_row_index() {
            self.set_selected_row(last_group_header);
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

    fn is_forward_group_cycle(key: &KeyEvent) -> bool {
        key.code == Key::Tab && !key.modifiers.intersects(KeyModifiers::SHIFT)
    }

    fn is_reverse_group_cycle(key: &KeyEvent) -> bool {
        key.code == Key::BackTab
            || (key.code == Key::Tab && key.modifiers.intersects(KeyModifiers::SHIFT))
    }

    fn snapshot_state(&self) -> RouteListState {
        let expanded_groups = self
            .groups
            .iter()
            .filter(|group| group.expanded)
            .map(|group| group.name.clone())
            .collect();

        let selected = if let Some(route_index) = self.selected_route_index() {
            self.routes
                .get(route_index)
                .map(|route| SelectedItem::Route(RouteSelection::from_route(route)))
        } else if let Some(group_index) = self.selected_group_index() {
            self.groups
                .get(group_index)
                .map(|group| SelectedItem::Group {
                    name: group.name.clone(),
                })
        } else {
            None
        };

        RouteListState {
            expanded_groups,
            selected,
        }
    }

    fn route_list_state_changed_msg(&self) -> Msg {
        Msg::RouteListStateChanged(self.snapshot_state())
    }

    fn on_keyboard(&mut self, key: KeyEvent) -> Option<Msg> {
        if self.pending_g {
            self.pending_g = false;

            if Self::is_plain_g(&key) {
                self.move_selection_home();
                return Some(self.route_list_state_changed_msg());
            }
        }

        if Self::is_plain_g(&key) {
            self.pending_g = true;
            return None;
        }

        if Self::is_jump_to_end(&key) {
            self.move_selection_end();
            return Some(self.route_list_state_changed_msg());
        }

        if Self::is_reverse_group_cycle(&key) {
            self.move_selection_to_previous_group_header();
            return Some(self.route_list_state_changed_msg());
        }

        if Self::is_forward_group_cycle(&key) {
            self.move_selection_to_next_group_header();
            return Some(self.route_list_state_changed_msg());
        }

        match key {
            KeyEvent {
                code: Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            } => {
                self.move_selection_down();
                Some(self.route_list_state_changed_msg())
            }
            KeyEvent {
                code: Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            } => {
                self.move_selection_up();
                Some(self.route_list_state_changed_msg())
            }
            KeyEvent {
                code: Key::Down, ..
            } => {
                self.move_selection_down();
                Some(self.route_list_state_changed_msg())
            }
            KeyEvent { code: Key::Up, .. } => {
                self.move_selection_up();
                Some(self.route_list_state_changed_msg())
            }
            KeyEvent {
                code: Key::Home, ..
            } => {
                self.move_selection_home();
                Some(self.route_list_state_changed_msg())
            }
            KeyEvent { code: Key::End, .. } => {
                self.move_selection_end();
                Some(self.route_list_state_changed_msg())
            }
            KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            } => {
                if let Some(group_index) = self.selected_group_index() {
                    self.toggle_group(group_index);
                    Some(self.route_list_state_changed_msg())
                } else if let Some(index) = self.selected_route_index() {
                    Some(Msg::RunRoute(index))
                } else {
                    None
                }
            }
            KeyEvent {
                code: Key::Char('e'),
                modifiers: KeyModifiers::NONE,
            } => {
                if let Some(index) = self.selected_route_index() {
                    Some(Msg::EditRoute(index))
                } else {
                    None
                }
            }
            KeyEvent {
                code: Key::Char('d'),
                modifiers: KeyModifiers::NONE,
            } => {
                if let Some(index) = self.selected_route_index() {
                    Some(Msg::DeleteRoute(index))
                } else {
                    None
                }
            }
            KeyEvent {
                code: Key::Char('n'),
                modifiers: KeyModifiers::NONE,
            } => Some(Msg::NewRoute),
            _ => None,
        }
    }

    fn move_selection_home(&mut self) {
        if let Some(first_selectable_row) = self.first_selectable_row_index() {
            self.set_selected_row(first_selectable_row);
        }
    }

    fn move_selection_end(&mut self) {
        if let Some(last_selectable_row) = self.last_selectable_row_index() {
            self.set_selected_row(last_selectable_row);
        }
    }

    fn toggle_group(&mut self, group_index: usize) {
        if let Some(group) = self.groups.get_mut(group_index) {
            group.expanded = !group.expanded;
            self.rebuild_rows(Some(SelectionTarget::GroupHeader(group_index)));
        }
    }

    fn rebuild_rows(&mut self, selection_target: Option<SelectionTarget>) {
        let (rows, row_kinds) = if self.routes.is_empty() {
            (
                vec![vec![
                    TextSpan::new("No routes. Press 'n' to create one.").fg(Color::DarkGray),
                ]],
                vec![RowKind::EmptyState],
            )
        } else {
            let max_method_width = self
                .routes
                .iter()
                .map(|route| Self::method_label(&route.method).len())
                .max()
                .unwrap_or(0);

            let mut rows = vec![];
            let mut row_kinds = vec![];
            let group_count = self.groups.len();

            for (group_index, group) in self.groups.iter().enumerate() {
                let visibility = if group.expanded { "-" } else { "+" };
                rows.push(vec![
                    TextSpan::new(format!("[{}] {}", visibility, group.name)).fg(Color::Cyan),
                ]);
                row_kinds.push(RowKind::GroupHeader(group_index));

                if group.expanded {
                    for route_index in &group.route_indexes {
                        let route = &self.routes[*route_index];
                        let method_label = Self::method_label(&route.method);
                        let route_name_padding =
                            max_method_width.saturating_sub(method_label.len()) + 1;

                        rows.push(vec![
                            TextSpan::new(method_label).fg(Self::method_color(&route.method)),
                            TextSpan::new(format!(
                                "{}{}",
                                " ".repeat(route_name_padding),
                                route.name
                            ))
                            .fg(Color::White),
                        ]);
                        row_kinds.push(RowKind::Route(*route_index));
                    }
                }

                if group_index + 1 < group_count {
                    rows.push(vec![TextSpan::new(" ")]);
                    row_kinds.push(RowKind::Spacer);
                }
            }

            (rows, row_kinds)
        };

        let selected_line = Self::resolve_selected_line(&row_kinds, selection_target);
        self.component = Self::build_component(rows, selected_line);
        self.row_kinds = row_kinds;
    }

    fn resolve_selected_line(
        row_kinds: &[RowKind],
        selection_target: Option<SelectionTarget>,
    ) -> usize {
        let fallback = row_kinds
            .iter()
            .position(RowKind::is_selectable)
            .unwrap_or(0);

        match selection_target {
            Some(SelectionTarget::GroupHeader(group_index)) => row_kinds
                .iter()
                .position(|kind| {
                    matches!(kind, RowKind::GroupHeader(current_index) if *current_index == group_index)
                })
                .unwrap_or(fallback),
            Some(SelectionTarget::Route(route_index)) => row_kinds
                .iter()
                .position(|kind| matches!(kind, RowKind::Route(current_index) if *current_index == route_index))
                .unwrap_or(fallback),
            None => fallback,
        }
    }

    fn build_component(rows: Vec<Vec<TextSpan>>, selected_line: usize) -> List {
        List::default()
            .borders(Borders::default().sides(BorderSides::NONE))
            .highlighted_color(Color::LightYellow)
            .highlighted_str(">> ")
            .scroll(true)
            .rows(rows)
            .selected_line(selected_line)
    }

    fn method_color(method: &crate::api_tester::collection::HttpMethod) -> Color {
        match method {
            HttpMethod::Get => Color::Green,
            HttpMethod::Post => Color::Blue,
            HttpMethod::Put => Color::Yellow,
            HttpMethod::Patch => Color::Magenta,
            HttpMethod::Delete => Color::Red,
        }
    }

    fn method_label(method: &HttpMethod) -> &'static str {
        match method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
        }
    }
}

impl MockComponent for RouteList {
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let block = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Green))
            .title("Routes")
            .padding(ratatui::widgets::Padding::new(1, 1, 0, 0));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let can_render_scrollbar = inner.width > SCROLLBAR_WIDTH;
        let (content_area, scrollbar_area) = if can_render_scrollbar {
            let chunks =
                Layout::horizontal([Constraint::Min(0), Constraint::Length(SCROLLBAR_WIDTH)])
                    .split(inner);
            (chunks[0], Some(chunks[1]))
        } else {
            (inner, None)
        };

        if content_area.width == 0 || content_area.height == 0 {
            return;
        }

        self.component.view(frame, content_area);

        if let Some(scrollbar_area) = scrollbar_area {
            let row_count = self.row_kinds.len();
            let viewport_height = content_area.height as usize;

            if row_count > viewport_height && viewport_height > 0 {
                let position = self
                    .selected_row_index()
                    .unwrap_or(0)
                    .min(row_count.saturating_sub(1));
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_style(Style::default().fg(TuiColor::DarkGray))
                    .thumb_style(Style::default().fg(TuiColor::Green));
                let mut state = ScrollbarState::new(row_count)
                    .position(position)
                    .viewport_content_length(viewport_height);

                frame.render_stateful_widget(scrollbar, scrollbar_area, &mut state);
            }
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.component.query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.component.attr(attr, value);
    }

    fn state(&self) -> tuirealm::State {
        self.component.state()
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        self.component.perform(cmd)
    }
}

impl Component<Msg, NoUserEvent> for RouteList {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key) => self.on_keyboard(key),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(group: &str, name: &str) -> Route {
        Route {
            group: group.to_string(),
            scope_id: format!("scope-{group}-{name}"),
            name: name.to_string(),
            method: HttpMethod::Get,
            url: "https://example.com".to_string(),
            headers: vec![],
            body: None,
        }
    }

    fn selected_row_index(list: &RouteList) -> usize {
        match list.state() {
            State::One(StateValue::Usize(index)) => index,
            _ => panic!("route list is always scrollable"),
        }
    }

    fn new_list(routes: &[Route]) -> RouteList {
        RouteList::new(routes, &RouteListState::default())
    }

    #[test]
    fn navigation_moves_between_group_headers_when_collapsed() {
        let routes = vec![route("group-a", "first"), route("group-b", "second")];
        let mut list = new_list(&routes);

        assert_eq!(selected_row_index(&list), 0);
        assert_eq!(list.selected_group_index(), Some(0));
        assert_eq!(list.selected_route_index(), None);

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        assert_eq!(selected_row_index(&list), 2);
        assert_eq!(list.selected_group_index(), Some(1));
        assert_eq!(list.selected_route_index(), None);

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Char('k'),
            KeyModifiers::NONE,
        )));
        assert_eq!(selected_row_index(&list), 0);
        assert_eq!(list.selected_group_index(), Some(0));
        assert_eq!(list.selected_route_index(), None);

        list.on(Event::Keyboard(KeyEvent::new(Key::End, KeyModifiers::NONE)));
        assert_eq!(selected_row_index(&list), 2);
        assert_eq!(list.selected_group_index(), Some(1));
        assert_eq!(list.selected_route_index(), None);

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Home,
            KeyModifiers::NONE,
        )));
        assert_eq!(selected_row_index(&list), 0);
        assert_eq!(list.selected_group_index(), Some(0));
        assert_eq!(list.selected_route_index(), None);
    }

    #[test]
    fn enter_toggles_group_visibility_and_runs_selected_route() {
        let routes = vec![
            route("group-a", "first"),
            route("group-a", "second"),
            route("group-b", "third"),
        ];
        let mut list = new_list(&routes);

        assert!(!list.groups[0].expanded);
        assert_eq!(selected_row_index(&list), 0);

        assert!(matches!(
            list.on(Event::Keyboard(KeyEvent::new(
                Key::Enter,
                KeyModifiers::NONE,
            ))),
            Some(Msg::RouteListStateChanged(_))
        ));
        assert!(list.groups[0].expanded);
        assert_eq!(selected_row_index(&list), 0);
        assert_eq!(list.selected_route_index(), None);

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        assert_eq!(list.selected_route_index(), Some(0));

        assert_eq!(
            list.on(Event::Keyboard(KeyEvent::new(
                Key::Enter,
                KeyModifiers::NONE,
            ))),
            Some(Msg::RunRoute(0))
        );

        list.on(Event::Keyboard(KeyEvent::new(Key::Up, KeyModifiers::NONE)));
        assert_eq!(selected_row_index(&list), 0);

        assert!(matches!(
            list.on(Event::Keyboard(KeyEvent::new(
                Key::Enter,
                KeyModifiers::NONE,
            ))),
            Some(Msg::RouteListStateChanged(_))
        ));
        assert!(!list.groups[0].expanded);
        assert_eq!(selected_row_index(&list), 0);
    }

    #[test]
    fn route_actions_only_apply_to_route_rows() {
        let routes = vec![route("group-a", "first")];
        let mut list = new_list(&routes);

        assert_eq!(
            list.on(Event::Keyboard(KeyEvent::new(
                Key::Char('e'),
                KeyModifiers::NONE,
            ))),
            None
        );
        assert_eq!(
            list.on(Event::Keyboard(KeyEvent::new(
                Key::Char('d'),
                KeyModifiers::NONE,
            ))),
            None
        );

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));
        list.on(Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        assert_eq!(list.selected_route_index(), Some(0));

        assert_eq!(
            list.on(Event::Keyboard(KeyEvent::new(
                Key::Char('e'),
                KeyModifiers::NONE,
            ))),
            Some(Msg::EditRoute(0))
        );
        assert_eq!(
            list.on(Event::Keyboard(KeyEvent::new(
                Key::Char('d'),
                KeyModifiers::NONE,
            ))),
            Some(Msg::DeleteRoute(0))
        );
    }

    #[test]
    fn navigation_wraps_between_first_and_last_selectable_rows() {
        let routes = vec![route("group-a", "first"), route("group-b", "second")];
        let mut list = new_list(&routes);

        assert_eq!(selected_row_index(&list), 0);

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        assert_eq!(selected_row_index(&list), 2);

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        assert_eq!(selected_row_index(&list), 0);

        list.on(Event::Keyboard(KeyEvent::new(Key::Up, KeyModifiers::NONE)));
        assert_eq!(selected_row_index(&list), 2);
    }

    #[test]
    fn vim_style_gg_and_g_jump_to_top_and_bottom() {
        let routes = vec![route("group-a", "first"), route("group-b", "second")];
        let mut list = new_list(&routes);

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        assert_eq!(selected_row_index(&list), 2);

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Char('g'),
            KeyModifiers::NONE,
        )));
        assert_eq!(selected_row_index(&list), 2);

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Char('g'),
            KeyModifiers::NONE,
        )));
        assert_eq!(selected_row_index(&list), 0);

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Char('G'),
            KeyModifiers::NONE,
        )));
        assert_eq!(selected_row_index(&list), 2);
    }

    #[test]
    fn tab_and_backtab_cycle_between_group_headers() {
        let routes = vec![route("group-a", "first"), route("group-b", "second")];
        let mut list = new_list(&routes);

        assert_eq!(selected_row_index(&list), 0);
        assert_eq!(list.selected_group_index(), Some(0));

        list.on(Event::Keyboard(KeyEvent::new(Key::Tab, KeyModifiers::NONE)));
        assert_eq!(selected_row_index(&list), 2);
        assert_eq!(list.selected_group_index(), Some(1));

        list.on(Event::Keyboard(KeyEvent::new(Key::Tab, KeyModifiers::NONE)));
        assert_eq!(selected_row_index(&list), 0);
        assert_eq!(list.selected_group_index(), Some(0));

        list.on(Event::Keyboard(KeyEvent::new(
            Key::BackTab,
            KeyModifiers::SHIFT,
        )));
        assert_eq!(selected_row_index(&list), 2);
        assert_eq!(list.selected_group_index(), Some(1));

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Tab,
            KeyModifiers::SHIFT,
        )));
        assert_eq!(selected_row_index(&list), 0);
        assert_eq!(list.selected_group_index(), Some(0));
    }

    #[test]
    fn tab_and_backtab_jump_to_group_headers_from_route_rows() {
        let routes = vec![
            route("group-a", "first"),
            route("group-a", "second"),
            route("group-b", "third"),
        ];
        let mut list = new_list(&routes);

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));
        list.on(Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        assert_eq!(list.selected_route_index(), Some(0));

        list.on(Event::Keyboard(KeyEvent::new(Key::Tab, KeyModifiers::NONE)));
        assert_eq!(selected_row_index(&list), 4);
        assert_eq!(list.selected_group_index(), Some(1));

        list.on(Event::Keyboard(KeyEvent::new(
            Key::BackTab,
            KeyModifiers::SHIFT,
        )));
        assert_eq!(selected_row_index(&list), 0);
        assert_eq!(list.selected_group_index(), Some(0));
    }

    #[test]
    fn restores_expanded_groups_and_selected_route_from_state() {
        let routes = vec![route("group-a", "first"), route("group-b", "second")];
        let state = RouteListState {
            expanded_groups: vec!["group-a".to_string()],
            selected: Some(SelectedItem::Route(RouteSelection {
                group: "group-a".to_string(),
                name: "first".to_string(),
                method: "GET".to_string(),
                url: "https://example.com".to_string(),
            })),
        };

        let list = RouteList::new(&routes, &state);

        assert!(list.groups[0].expanded);
        assert!(!list.groups[1].expanded);
        assert_eq!(list.selected_route_index(), Some(0));
    }

    #[test]
    fn falls_back_to_group_when_selected_route_is_missing() {
        let routes = vec![route("group-a", "first"), route("group-b", "second")];
        let state = RouteListState {
            expanded_groups: vec![],
            selected: Some(SelectedItem::Route(RouteSelection {
                group: "group-b".to_string(),
                name: "missing".to_string(),
                method: "GET".to_string(),
                url: "https://example.com/missing".to_string(),
            })),
        };

        let list = RouteList::new(&routes, &state);

        assert_eq!(list.selected_group_index(), Some(1));
        assert_eq!(list.selected_route_index(), None);
    }

    #[test]
    fn empty_route_list_has_no_selectable_rows() {
        let mut list = new_list(&[]);

        assert_eq!(selected_row_index(&list), 0);
        assert_eq!(list.selected_route_index(), None);

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        assert_eq!(selected_row_index(&list), 0);
        assert_eq!(list.selected_route_index(), None);

        list.on(Event::Keyboard(KeyEvent::new(Key::End, KeyModifiers::NONE)));
        assert_eq!(selected_row_index(&list), 0);
        assert_eq!(list.selected_route_index(), None);

        list.on(Event::Keyboard(KeyEvent::new(Key::Tab, KeyModifiers::NONE)));
        assert_eq!(selected_row_index(&list), 0);
        assert_eq!(list.selected_route_index(), None);

        list.on(Event::Keyboard(KeyEvent::new(
            Key::BackTab,
            KeyModifiers::SHIFT,
        )));
        assert_eq!(selected_row_index(&list), 0);
        assert_eq!(list.selected_route_index(), None);

        list.on(Event::Keyboard(KeyEvent::new(
            Key::Tab,
            KeyModifiers::SHIFT,
        )));
        assert_eq!(selected_row_index(&list), 0);
        assert_eq!(list.selected_route_index(), None);
    }
}
