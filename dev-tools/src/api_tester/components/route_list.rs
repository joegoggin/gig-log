use tui_realm_stdlib::List;
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent, KeyModifiers},
    props::{Alignment, BorderType, Borders, Color, PropPayload, PropValue, TextSpan},
};

use crate::api_tester::{
    app::Msg,
    collection::{DEFAULT_ROUTE_GROUP, HttpMethod, Route},
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
}

pub struct RouteList {
    component: List,
    routes: Vec<Route>,
    groups: Vec<RouteGroup>,
    row_kinds: Vec<RowKind>,
    pending_g: bool,
}

impl RouteList {
    pub fn new(routes: &[Route]) -> Self {
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

        let mut list = Self {
            component: List::default(),
            routes: routes.to_vec(),
            groups,
            row_kinds: vec![],
            pending_g: false,
        };
        list.rebuild_rows(None);
        list
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

    fn on_keyboard(&mut self, key: KeyEvent) -> Option<Msg> {
        if self.pending_g {
            self.pending_g = false;

            if Self::is_plain_g(&key) {
                self.move_selection_home();
                return None;
            }
        }

        if Self::is_plain_g(&key) {
            self.pending_g = true;
            return None;
        }

        if Self::is_jump_to_end(&key) {
            self.move_selection_end();
            return None;
        }

        if Self::is_reverse_group_cycle(&key) {
            self.move_selection_to_previous_group_header();
            return None;
        }

        if Self::is_forward_group_cycle(&key) {
            self.move_selection_to_next_group_header();
            return None;
        }

        match key {
            KeyEvent {
                code: Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            } => {
                self.move_selection_down();
                None
            }
            KeyEvent {
                code: Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            } => {
                self.move_selection_up();
                None
            }
            KeyEvent {
                code: Key::Down, ..
            } => {
                self.move_selection_down();
                None
            }
            KeyEvent { code: Key::Up, .. } => {
                self.move_selection_up();
                None
            }
            KeyEvent {
                code: Key::Home, ..
            } => {
                self.move_selection_home();
                None
            }
            KeyEvent { code: Key::End, .. } => {
                self.move_selection_end();
                None
            }
            KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            } => {
                if let Some(group_index) = self.selected_group_index() {
                    self.toggle_group(group_index);
                    None
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
            None => fallback,
        }
    }

    fn build_component(rows: Vec<Vec<TextSpan>>, selected_line: usize) -> List {
        List::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Cyan),
            )
            .title("Routes", Alignment::Left)
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
        self.component.view(frame, area);
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

    #[test]
    fn navigation_moves_between_group_headers_when_collapsed() {
        let routes = vec![route("group-a", "first"), route("group-b", "second")];
        let mut list = RouteList::new(&routes);

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
        let mut list = RouteList::new(&routes);

        assert!(!list.groups[0].expanded);
        assert_eq!(selected_row_index(&list), 0);

        assert_eq!(
            list.on(Event::Keyboard(KeyEvent::new(
                Key::Enter,
                KeyModifiers::NONE,
            ))),
            None
        );
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

        assert_eq!(
            list.on(Event::Keyboard(KeyEvent::new(
                Key::Enter,
                KeyModifiers::NONE,
            ))),
            None
        );
        assert!(!list.groups[0].expanded);
        assert_eq!(selected_row_index(&list), 0);
    }

    #[test]
    fn route_actions_only_apply_to_route_rows() {
        let routes = vec![route("group-a", "first")];
        let mut list = RouteList::new(&routes);

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
        let mut list = RouteList::new(&routes);

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
        let mut list = RouteList::new(&routes);

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
        let mut list = RouteList::new(&routes);

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
        let mut list = RouteList::new(&routes);

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
    fn empty_route_list_has_no_selectable_rows() {
        let mut list = RouteList::new(&[]);

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
