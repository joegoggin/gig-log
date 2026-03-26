use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::time::Instant;

use anyhow::Context;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use sqlx::PgPool;
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::{Application, NoUserEvent};

use crate::db_viewer::db::{self, ColumnInfo, QueryOutput, TableInfo, TableRow};
use crate::db_viewer::paths;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    GlobalListener,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AppMsg {
    Key(KeyEvent),
    TerminalResize(u16, u16),
    QueryEditedExternally(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Msg {
    App(AppMsg),
}

#[derive(Debug, Clone)]
pub enum AppEffect {
    Close,
    OpenQueryEditor(String),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ActivePage {
    Tables,
    Grid,
    Query,
}

impl ActivePage {
    fn as_str(self) -> &'static str {
        match self {
            Self::Tables => "tables",
            Self::Grid => "grid",
            Self::Query => "query",
        }
    }

    fn from_str(raw: &str) -> Option<Self> {
        match raw {
            "tables" => Some(Self::Tables),
            "grid" => Some(Self::Grid),
            "query" => Some(Self::Query),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
enum TextEntryTarget {
    Query,
    GridCell {
        row_index: usize,
        column_name: String,
        column: ColumnInfo,
    },
    AddField {
        column_name: String,
        column: ColumnInfo,
    },
}

#[derive(Debug, Clone)]
struct TextEntryState {
    target: TextEntryTarget,
    buffer: String,
}

#[derive(Debug, Clone)]
struct DeleteConfirmation {
    ctids: Vec<String>,
    description: String,
}

#[derive(Debug, Clone)]
struct AddFormState {
    selected_field: usize,
    values: BTreeMap<String, Option<Value>>,
}

#[derive(Debug, Clone)]
struct ValuePickerOption {
    label: String,
    value: Value,
}

#[derive(Debug, Clone)]
struct ValuePickerState {
    row_index: usize,
    column_name: String,
    column_type: String,
    options: Vec<ValuePickerOption>,
    selected_index: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum TemporalMode {
    Date,
    Time,
    DateTime,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum TemporalPart {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
}

#[derive(Debug, Clone)]
struct DateTimePickerState {
    row_index: usize,
    column_name: String,
    column_type: String,
    mode: TemporalMode,
    selected_part: TemporalPart,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    nullable: bool,
    is_null: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedState {
    selected_table: Option<String>,
    active_page: Option<String>,
}

struct GridRenderData {
    left_header: Line<'static>,
    right_header: Line<'static>,
    left_rows: Vec<Line<'static>>,
    right_rows: Vec<Line<'static>>,
    column_ranges: Vec<(usize, usize)>,
}

pub struct AppModel {
    pub app: Application<Id, Msg, NoUserEvent>,
    pool: PgPool,
    tables: Vec<TableInfo>,
    selected_table: usize,
    rows: Vec<TableRow>,
    selected_row: usize,
    selected_col: usize,
    selected_rows: BTreeSet<usize>,
    dirty_rows: BTreeMap<usize, BTreeMap<String, Value>>,
    active_page: ActivePage,
    query: String,
    query_result_lines: Vec<String>,
    status: String,
    error_popup: Option<String>,
    help_visible: bool,
    text_entry: Option<TextEntryState>,
    datetime_picker: Option<DateTimePickerState>,
    value_picker: Option<ValuePickerState>,
    confirm_delete: Option<DeleteConfirmation>,
    add_form: Option<AddFormState>,
    terminal_width: u16,
    terminal_height: u16,
    grid_h_scroll: usize,
    grid_v_scroll: usize,
    pending_g: bool,
}

impl AppModel {
    const ROW_LIMIT: i64 = 200;

    pub async fn new(app: Application<Id, Msg, NoUserEvent>, pool: PgPool) -> anyhow::Result<Self> {
        let state = Self::load_state().unwrap_or_default();
        let query = Self::load_query().unwrap_or_else(|| "SELECT * FROM users;".to_string());
        let tables = db::load_tables(&pool).await?;

        let mut selected_table = 0_usize;
        if let Some(selected_name) = state.selected_table.as_deref()
            && let Some(index) = tables
                .iter()
                .position(|table| format!("{}.{}", table.schema, table.name) == selected_name)
        {
            selected_table = index;
        }

        let active_page = state
            .active_page
            .as_deref()
            .and_then(ActivePage::from_str)
            .unwrap_or(ActivePage::Tables);

        let mut model = Self {
            app,
            pool,
            tables,
            selected_table,
            rows: vec![],
            selected_row: 0,
            selected_col: 0,
            selected_rows: BTreeSet::new(),
            dirty_rows: BTreeMap::new(),
            active_page,
            query,
            query_result_lines: vec!["Run a query with 'r'. Edit query with 'e'.".to_string()],
            status: String::new(),
            error_popup: None,
            help_visible: false,
            text_entry: None,
            datetime_picker: None,
            value_picker: None,
            confirm_delete: None,
            add_form: None,
            terminal_width: 120,
            terminal_height: 40,
            grid_h_scroll: 0,
            grid_v_scroll: 0,
            pending_g: false,
        };

        model.reload_rows().await?;
        model.status = "Loaded db-viewer".to_string();
        Ok(model)
    }

    pub async fn update(&mut self, msg: Msg) -> anyhow::Result<Option<AppEffect>> {
        match msg {
            Msg::App(AppMsg::TerminalResize(width, height)) => {
                self.terminal_width = width;
                self.terminal_height = height;
                return Ok(None);
            }
            Msg::App(AppMsg::QueryEditedExternally(query)) => {
                self.query = query;
                self.save_query();
                self.status = "Updated query from external editor".to_string();
                return Ok(None);
            }
            Msg::App(AppMsg::Key(key)) => return self.handle_key(key).await,
        }
    }

    async fn handle_key(&mut self, key: KeyEvent) -> anyhow::Result<Option<AppEffect>> {
        if self.error_popup.is_some() {
            return Ok(self.handle_error_popup_key(key));
        }

        if self.help_visible {
            return Ok(self.handle_help_key(key));
        }

        if self.confirm_delete.is_some() {
            return self.handle_delete_confirm_key(key).await;
        }

        if self.datetime_picker.is_some() {
            return self.handle_datetime_picker_key(key).await;
        }

        if self.value_picker.is_some() {
            return self.handle_value_picker_key(key).await;
        }

        if self.text_entry.is_some() {
            return self.handle_text_entry_key(key).await;
        }

        if self.add_form.is_some() {
            return self.handle_add_form_key(key).await;
        }

        if key.code == Key::Char('q') {
            return Ok(Some(AppEffect::Close));
        }

        if key.code == Key::Char('?') {
            self.help_visible = true;
            return Ok(None);
        }

        if key.code == Key::Char('1') {
            self.set_active_page(ActivePage::Tables);
            return Ok(None);
        }

        if key.code == Key::Char('2') {
            self.set_active_page(ActivePage::Grid);
            return Ok(None);
        }

        if key.code == Key::Char('3') {
            self.set_active_page(ActivePage::Query);
            return Ok(None);
        }

        match self.active_page {
            ActivePage::Tables => self.handle_table_keys(key).await,
            ActivePage::Grid => self.handle_grid_keys(key).await,
            ActivePage::Query => self.handle_query_keys(key).await,
        }
    }

    fn handle_error_popup_key(&mut self, key: KeyEvent) -> Option<AppEffect> {
        if matches!(key.code, Key::Esc | Key::Enter) {
            self.error_popup = None;
        }
        None
    }

    async fn handle_value_picker_key(
        &mut self,
        key: KeyEvent,
    ) -> anyhow::Result<Option<AppEffect>> {
        let Some(picker) = self.value_picker.as_mut() else {
            return Ok(None);
        };

        match key {
            KeyEvent {
                code: Key::Esc, ..
            } => {
                self.value_picker = None;
                self.status = "Selection cancelled".to_string();
            }
            KeyEvent {
                code: Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Down, ..
            } => {
                if !picker.options.is_empty() {
                    picker.selected_index =
                        (picker.selected_index + 1).min(picker.options.len().saturating_sub(1));
                }
            }
            KeyEvent {
                code: Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent { code: Key::Up, .. } => {
                picker.selected_index = picker.selected_index.saturating_sub(1);
            }
            KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Char('i'),
                modifiers: KeyModifiers::NONE,
            } => {
                if let Some(picker) = self.value_picker.take()
                    && let Some(option) = picker.options.get(picker.selected_index)
                {
                    let row_dirty = self.dirty_rows.entry(picker.row_index).or_default();
                    row_dirty.insert(picker.column_name.clone(), option.value.clone());
                    self.status = format!(
                        "Set {} = {} (Ctrl+S to save)",
                        picker.column_name, option.label
                    );
                }
            }
            _ => {}
        }

        Ok(None)
    }

    async fn handle_datetime_picker_key(
        &mut self,
        key: KeyEvent,
    ) -> anyhow::Result<Option<AppEffect>> {
        let Some(picker) = self.datetime_picker.as_mut() else {
            return Ok(None);
        };

        match key {
            KeyEvent {
                code: Key::Esc, ..
            } => {
                self.datetime_picker = None;
                self.status = "Date/time edit cancelled".to_string();
            }
            KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Right, ..
            } => {
                picker.selected_part = Self::next_temporal_part(picker.mode, picker.selected_part);
            }
            KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }
            | KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Left, ..
            } => {
                picker.selected_part =
                    Self::prev_temporal_part(picker.mode, picker.selected_part);
            }
            KeyEvent {
                code: Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Down, ..
            } => {
                Self::adjust_temporal_part(picker, 1);
                picker.is_null = false;
            }
            KeyEvent {
                code: Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent { code: Key::Up, .. } => {
                Self::adjust_temporal_part(picker, -1);
                picker.is_null = false;
            }
            KeyEvent {
                code: Key::Char('x'),
                modifiers: KeyModifiers::NONE,
            } => {
                if picker.nullable {
                    picker.is_null = !picker.is_null;
                }
            }
            KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Char('i'),
                modifiers: KeyModifiers::NONE,
            } => {
                if let Some(picker) = self.datetime_picker.take() {
                    let value = if picker.is_null {
                        Value::Null
                    } else {
                        Value::String(Self::format_temporal_value(&picker))
                    };

                    let row_dirty = self.dirty_rows.entry(picker.row_index).or_default();
                    row_dirty.insert(picker.column_name.clone(), value);
                    self.status =
                        format!("Set {} (Ctrl+S to save)", picker.column_name);
                }
            }
            _ => {}
        }

        Ok(None)
    }

    fn handle_help_key(&mut self, key: KeyEvent) -> Option<AppEffect> {
        if matches!(key.code, Key::Esc | Key::Char('?')) {
            self.help_visible = false;
        }
        None
    }

    async fn handle_delete_confirm_key(
        &mut self,
        key: KeyEvent,
    ) -> anyhow::Result<Option<AppEffect>> {
        match key.code {
            Key::Char('y') | Key::Char('Y') => {
                if let Some(confirm) = self.confirm_delete.take() {
                    if let Some(table) = self.selected_table().cloned() {
                        match db::delete_rows(&self.pool, &table, &confirm.ctids).await {
                            Ok(deleted) => {
                                self.status = format!("Deleted {deleted} row(s)");
                                if let Err(error) = self.reload_rows().await {
                                    self.show_error(format!(
                                        "Deleted row(s), but refresh failed: {}",
                                        Self::format_error_for_status(&error)
                                    ));
                                }
                            }
                            Err(error) => {
                                self.show_error(format!(
                                    "Delete failed: {}",
                                    Self::format_error_for_status(&error)
                                ));
                            }
                        }
                    }
                }
            }
            Key::Char('n') | Key::Char('N') | Key::Esc => {
                self.confirm_delete = None;
                self.status = "Delete cancelled".to_string();
            }
            _ => {}
        }

        Ok(None)
    }

    async fn handle_text_entry_key(&mut self, key: KeyEvent) -> anyhow::Result<Option<AppEffect>> {
        let Some(entry) = self.text_entry.as_mut() else {
            return Ok(None);
        };

        match key.code {
            Key::Esc => {
                self.text_entry = None;
                self.status = "Edit cancelled".to_string();
            }
            Key::Backspace => {
                entry.buffer.pop();
            }
            Key::Enter => {
                let entry = self.text_entry.take().unwrap_or(TextEntryState {
                    target: TextEntryTarget::Query,
                    buffer: String::new(),
                });
                if let Err(error) = self.commit_text_entry(entry.clone()).await {
                    self.text_entry = Some(entry);
                    self.show_error(Self::format_error_for_status(&error));
                }
            }
            Key::Char(ch) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(None);
                }

                entry.buffer.push(ch);
            }
            _ => {}
        }

        Ok(None)
    }

    async fn commit_text_entry(&mut self, entry: TextEntryState) -> anyhow::Result<()> {
        match entry.target {
            TextEntryTarget::Query => {
                self.query = entry.buffer;
                self.save_query();
                self.status = "Updated query".to_string();
            }
            TextEntryTarget::GridCell {
                row_index,
                column_name,
                column,
            } => {
                let parsed = self.parse_input_value(&entry.buffer, &column)?;
                let row_dirty = self.dirty_rows.entry(row_index).or_default();
                row_dirty.insert(column_name.clone(), parsed);

                self.status = format!(
                    "Edited {} row {} (Ctrl+S to save)",
                    column_name,
                    row_index + 1
                );
            }
            TextEntryTarget::AddField {
                column_name,
                column,
            } => {
                let parsed = self.parse_input_value(&entry.buffer, &column)?;
                if let Some(add_form) = &mut self.add_form {
                    add_form.values.insert(column_name, Some(parsed));
                }
                self.status = "Updated add-row field".to_string();
            }
        }

        Ok(())
    }

    async fn handle_add_form_key(&mut self, key: KeyEvent) -> anyhow::Result<Option<AppEffect>> {
        let Some(table) = self.selected_table().cloned() else {
            self.add_form = None;
            return Ok(None);
        };

        let Some(add_form) = self.add_form.as_mut() else {
            return Ok(None);
        };

        match key {
            KeyEvent { code: Key::Esc, .. } => {
                self.add_form = None;
                self.status = "Add row cancelled".to_string();
            }
            KeyEvent {
                code: Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Down, ..
            } => {
                if !table.columns.is_empty() {
                    add_form.selected_field = (add_form.selected_field + 1) % table.columns.len();
                }
            }
            KeyEvent {
                code: Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent { code: Key::Up, .. } => {
                if !table.columns.is_empty() {
                    add_form.selected_field = if add_form.selected_field == 0 {
                        table.columns.len().saturating_sub(1)
                    } else {
                        add_form.selected_field.saturating_sub(1)
                    };
                }
            }
            KeyEvent {
                code: Key::Char('x'),
                modifiers: KeyModifiers::NONE,
            } => {
                if let Some(column) = table.columns.get(add_form.selected_field) {
                    if column.is_nullable {
                        add_form
                            .values
                            .insert(column.name.clone(), Some(Value::Null));
                        self.status = format!("Set {} to NULL", column.name);
                    } else {
                        add_form.values.insert(column.name.clone(), None);
                        self.status = format!("Cleared {}", column.name);
                    }
                }
            }
            KeyEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                let mut payload = BTreeMap::new();
                for (column, value) in &add_form.values {
                    if let Some(value) = value {
                        payload.insert(column.clone(), value.clone());
                    }
                }

                match db::insert_row(&self.pool, &table, &payload).await {
                    Ok(()) => {
                        self.add_form = None;
                        self.status = "Inserted row".to_string();
                        if let Err(error) = self.reload_rows().await {
                            self.show_error(format!(
                                "Inserted row, but refresh failed: {}",
                                Self::format_error_for_status(&error)
                            ));
                        }
                    }
                    Err(error) => {
                        self.show_error(format!(
                            "Insert failed: {}",
                            Self::format_error_for_status(&error)
                        ));
                    }
                }
            }
            KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Char('i'),
                modifiers: KeyModifiers::NONE,
            } => {
                if let Some(column) = table.columns.get(add_form.selected_field).cloned() {
                    if column.is_enum() {
                        let next = Self::next_enum_value(
                            &column,
                            add_form.values.get(&column.name).cloned().flatten(),
                        );
                        add_form
                            .values
                            .insert(column.name.clone(), Some(Value::String(next.clone())));
                        self.status = format!("Set {} = {}", column.name, next);
                    } else {
                        let current = add_form
                            .values
                            .get(&column.name)
                            .and_then(|value| value.clone())
                            .map(|value| self.value_to_display(&value))
                            .unwrap_or_default();
                        self.text_entry = Some(TextEntryState {
                            target: TextEntryTarget::AddField {
                                column_name: column.name.clone(),
                                column,
                            },
                            buffer: current,
                        });
                    }
                }
            }
            _ => {}
        }

        Ok(None)
    }

    async fn handle_table_keys(&mut self, key: KeyEvent) -> anyhow::Result<Option<AppEffect>> {
        match key {
            KeyEvent {
                code: Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Down, ..
            } => {
                if !self.tables.is_empty() {
                    self.selected_table =
                        (self.selected_table + 1).min(self.tables.len().saturating_sub(1));
                    self.persist_selected_table();
                    self.reload_rows().await?;
                }
            }
            KeyEvent {
                code: Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent { code: Key::Up, .. } => {
                if self.selected_table > 0 {
                    self.selected_table = self.selected_table.saturating_sub(1);
                    self.persist_selected_table();
                    self.reload_rows().await?;
                }
            }
            KeyEvent {
                code: Key::Enter, ..
            }
            | KeyEvent {
                code: Key::Right, ..
            } => {
                self.set_active_page(ActivePage::Grid);
            }
            _ => {}
        }

        Ok(None)
    }

    async fn handle_grid_keys(&mut self, key: KeyEvent) -> anyhow::Result<Option<AppEffect>> {
        let Some(table) = self.selected_table().cloned() else {
            return Ok(None);
        };

        let is_plain_g = matches!(
            key,
            KeyEvent {
                code: Key::Char('g'),
                modifiers: KeyModifiers::NONE,
            }
        );

        if self.pending_g {
            self.pending_g = false;
            if is_plain_g {
                self.selected_row = 0;
                self.follow_selected_cell();
                return Ok(None);
            }
        }

        if is_plain_g {
            self.pending_g = true;
            return Ok(None);
        }

        self.pending_g = false;

        let mut moved_cursor = false;

        match key {
            KeyEvent {
                code: Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Down, ..
            } => {
                if !self.rows.is_empty() {
                    self.selected_row =
                        (self.selected_row + 1).min(self.rows.len().saturating_sub(1));
                    moved_cursor = true;
                }
            }
            KeyEvent {
                code: Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent { code: Key::Up, .. } => {
                self.selected_row = self.selected_row.saturating_sub(1);
                moved_cursor = true;
            }
            KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Left, ..
            } => {
                self.selected_col = self.selected_col.saturating_sub(1);
                moved_cursor = true;
            }
            KeyEvent {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Right, ..
            } => {
                if !table.columns.is_empty() {
                    self.selected_col =
                        (self.selected_col + 1).min(table.columns.len().saturating_sub(1));
                    moved_cursor = true;
                }
            }
            KeyEvent {
                code: Key::Char('0'),
                modifiers: KeyModifiers::NONE,
            } => {
                self.selected_col = 0;
                moved_cursor = true;
            }
            KeyEvent {
                code: Key::Char('$'),
                ..
            } => {
                if !table.columns.is_empty() {
                    self.selected_col = table.columns.len().saturating_sub(1);
                    moved_cursor = true;
                }
            }
            KeyEvent {
                code: Key::Char('G'),
                ..
            }
            | KeyEvent {
                code: Key::End, ..
            } => {
                if !self.rows.is_empty() {
                    self.selected_row = self.rows.len().saturating_sub(1);
                    moved_cursor = true;
                }
            }
            KeyEvent {
                code: Key::Char(' '),
                modifiers: KeyModifiers::NONE,
            } => {
                if self.selected_rows.contains(&self.selected_row) {
                    self.selected_rows.remove(&self.selected_row);
                } else {
                    self.selected_rows.insert(self.selected_row);
                }
            }
            KeyEvent {
                code: Key::Char('A'),
                modifiers: KeyModifiers::SHIFT,
            }
            | KeyEvent {
                code: Key::Char('a'),
                modifiers: KeyModifiers::SHIFT,
            } => {
                self.selected_rows = (0..self.rows.len()).collect();
                self.status = format!("Selected all {} loaded rows", self.selected_rows.len());
            }
            KeyEvent {
                code: Key::Char('c'),
                modifiers: KeyModifiers::SHIFT,
            }
            | KeyEvent {
                code: Key::Char('C'),
                modifiers: KeyModifiers::NONE,
            } => {
                self.selected_rows.clear();
                self.status = "Cleared row selection".to_string();
            }
            KeyEvent {
                code: Key::Char('d'),
                modifiers: KeyModifiers::NONE,
            } => {
                self.start_delete_confirmation();
            }
            KeyEvent {
                code: Key::Char('a'),
                modifiers: KeyModifiers::NONE,
            } => {
                let values = table
                    .columns
                    .iter()
                    .map(|column| (column.name.clone(), None))
                    .collect::<BTreeMap<_, _>>();
                self.add_form = Some(AddFormState {
                    selected_field: 0,
                    values,
                });
            }
            KeyEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                self.save_dirty_rows().await;
            }
            KeyEvent {
                code: Key::Char('r'),
                modifiers: KeyModifiers::NONE,
            } => {
                self.execute_query().await;
            }
            KeyEvent {
                code: Key::Char('e'),
                modifiers: KeyModifiers::NONE,
            } => {
                return Ok(Some(AppEffect::OpenQueryEditor(self.query.clone())));
            }
            KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Char('i'),
                modifiers: KeyModifiers::NONE,
            } => {
                self.begin_cell_edit();
            }
            _ => {}
        }

        if moved_cursor {
            self.follow_selected_cell();
        }

        Ok(None)
    }

    async fn handle_query_keys(&mut self, key: KeyEvent) -> anyhow::Result<Option<AppEffect>> {
        match key {
            KeyEvent {
                code: Key::Char('r'),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                self.execute_query().await;
            }
            KeyEvent {
                code: Key::Char('e'),
                modifiers: KeyModifiers::NONE,
            } => {
                return Ok(Some(AppEffect::OpenQueryEditor(self.query.clone())));
            }
            KeyEvent {
                code: Key::Char('i'),
                modifiers: KeyModifiers::NONE,
            } => {
                self.text_entry = Some(TextEntryState {
                    target: TextEntryTarget::Query,
                    buffer: self.query.clone(),
                });
            }
            _ => {}
        }

        Ok(None)
    }

    fn begin_cell_edit(&mut self) {
        let Some(table) = self.selected_table() else {
            return;
        };
        let Some(column) = table.columns.get(self.selected_col).cloned() else {
            return;
        };

        if self.open_datetime_picker_for_column(&column) {
            return;
        }

        if self.open_picker_for_column(&column) {
            return;
        }

        let current_value = self
            .current_cell_value(self.selected_row, &column.name)
            .map(|value| self.value_to_display(&value))
            .unwrap_or_default();
        self.text_entry = Some(TextEntryState {
            target: TextEntryTarget::GridCell {
                row_index: self.selected_row,
                column_name: column.name.clone(),
                column,
            },
            buffer: current_value,
        });
    }

    fn open_datetime_picker_for_column(&mut self, column: &ColumnInfo) -> bool {
        let mode = if Self::is_datetime_column(column) {
            TemporalMode::DateTime
        } else if Self::is_date_column(column) {
            TemporalMode::Date
        } else if Self::is_time_column(column) {
            TemporalMode::Time
        } else {
            return false;
        };

        let now = Utc::now();
        let mut picker = DateTimePickerState {
            row_index: self.selected_row,
            column_name: column.name.clone(),
            column_type: column.data_type.clone(),
            mode,
            selected_part: Self::initial_temporal_part(mode),
            year: now.year(),
            month: now.month(),
            day: now.day(),
            hour: now.hour(),
            minute: now.minute(),
            second: now.second(),
            nullable: column.is_nullable,
            is_null: false,
        };

        if let Some(value) = self.current_cell_value(self.selected_row, &column.name) {
            match value {
                Value::Null => {
                    picker.is_null = true;
                }
                Value::String(raw) => {
                    Self::apply_temporal_string(&mut picker, &raw);
                }
                other => {
                    Self::apply_temporal_string(&mut picker, &self.value_to_display(&other));
                }
            }
        }

        picker.day = picker.day.min(Self::days_in_month(picker.year, picker.month));
        self.datetime_picker = Some(picker);
        true
    }

    fn open_picker_for_column(&mut self, column: &ColumnInfo) -> bool {
        let mut options = Vec::new();

        if column.is_enum() {
            options = column
                .enum_options
                .iter()
                .map(|option| ValuePickerOption {
                    label: option.clone(),
                    value: Value::String(option.clone()),
                })
                .collect();
            if column.is_nullable {
                options.push(ValuePickerOption {
                    label: "NULL".to_string(),
                    value: Value::Null,
                });
            }
        } else if Self::is_boolean_column(column) {
            options.push(ValuePickerOption {
                label: "true".to_string(),
                value: Value::Bool(true),
            });
            options.push(ValuePickerOption {
                label: "false".to_string(),
                value: Value::Bool(false),
            });
            if column.is_nullable {
                options.push(ValuePickerOption {
                    label: "NULL".to_string(),
                    value: Value::Null,
                });
            }
        }

        if options.is_empty() {
            return false;
        }

        let current = self
            .current_cell_value(self.selected_row, &column.name)
            .unwrap_or(Value::Null);
        let selected_index = options
            .iter()
            .position(|option| option.value == current)
            .unwrap_or(0);

        self.value_picker = Some(ValuePickerState {
            row_index: self.selected_row,
            column_name: column.name.clone(),
            column_type: column.data_type.clone(),
            options,
            selected_index,
        });

        true
    }

    fn is_boolean_column(column: &ColumnInfo) -> bool {
        let data_type = column.data_type.to_ascii_lowercase();
        let udt_name = column.udt_name.to_ascii_lowercase();
        data_type.contains("boolean") || udt_name == "bool"
    }

    fn is_datetime_column(column: &ColumnInfo) -> bool {
        let data_type = column.data_type.to_ascii_lowercase();
        let udt_name = column.udt_name.to_ascii_lowercase();
        data_type.contains("timestamp") || udt_name == "timestamp" || udt_name == "timestamptz"
    }

    fn is_date_column(column: &ColumnInfo) -> bool {
        let data_type = column.data_type.to_ascii_lowercase();
        let udt_name = column.udt_name.to_ascii_lowercase();
        (data_type == "date" || udt_name == "date") && !Self::is_datetime_column(column)
    }

    fn is_time_column(column: &ColumnInfo) -> bool {
        let data_type = column.data_type.to_ascii_lowercase();
        let udt_name = column.udt_name.to_ascii_lowercase();
        (data_type.starts_with("time") || udt_name == "time" || udt_name == "timetz")
            && !Self::is_datetime_column(column)
    }

    fn initial_temporal_part(mode: TemporalMode) -> TemporalPart {
        match mode {
            TemporalMode::Date => TemporalPart::Day,
            TemporalMode::Time => TemporalPart::Hour,
            TemporalMode::DateTime => TemporalPart::Day,
        }
    }

    fn temporal_parts(mode: TemporalMode) -> &'static [TemporalPart] {
        match mode {
            TemporalMode::Date => &[TemporalPart::Year, TemporalPart::Month, TemporalPart::Day],
            TemporalMode::Time => &[TemporalPart::Hour, TemporalPart::Minute, TemporalPart::Second],
            TemporalMode::DateTime => &[
                TemporalPart::Year,
                TemporalPart::Month,
                TemporalPart::Day,
                TemporalPart::Hour,
                TemporalPart::Minute,
                TemporalPart::Second,
            ],
        }
    }

    fn next_temporal_part(mode: TemporalMode, current: TemporalPart) -> TemporalPart {
        let parts = Self::temporal_parts(mode);
        if let Some(index) = parts.iter().position(|part| *part == current) {
            parts[(index + 1) % parts.len()]
        } else {
            parts[0]
        }
    }

    fn prev_temporal_part(mode: TemporalMode, current: TemporalPart) -> TemporalPart {
        let parts = Self::temporal_parts(mode);
        if let Some(index) = parts.iter().position(|part| *part == current) {
            if index == 0 {
                parts[parts.len().saturating_sub(1)]
            } else {
                parts[index - 1]
            }
        } else {
            parts[0]
        }
    }

    fn adjust_temporal_part(picker: &mut DateTimePickerState, delta: i32) {
        match picker.selected_part {
            TemporalPart::Year => {
                picker.year = picker.year.saturating_add(delta);
                picker.day = picker.day.min(Self::days_in_month(picker.year, picker.month));
            }
            TemporalPart::Month => {
                let month = picker.month as i32 - 1 + delta;
                let wrapped = month.rem_euclid(12) + 1;
                picker.month = wrapped as u32;
                picker.day = picker.day.min(Self::days_in_month(picker.year, picker.month));
            }
            TemporalPart::Day => {
                let max_day = Self::days_in_month(picker.year, picker.month) as i32;
                let day = picker.day as i32 - 1 + delta;
                picker.day = (day.rem_euclid(max_day) + 1) as u32;
            }
            TemporalPart::Hour => {
                let hour = picker.hour as i32 + delta;
                picker.hour = hour.rem_euclid(24) as u32;
            }
            TemporalPart::Minute => {
                let minute = picker.minute as i32 + delta;
                picker.minute = minute.rem_euclid(60) as u32;
            }
            TemporalPart::Second => {
                let second = picker.second as i32 + delta;
                picker.second = second.rem_euclid(60) as u32;
            }
        }
    }

    fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
                if leap { 29 } else { 28 }
            }
            _ => 30,
        }
    }

    fn apply_temporal_string(picker: &mut DateTimePickerState, raw: &str) {
        match picker.mode {
            TemporalMode::Date => {
                if let Ok(value) = NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d") {
                    picker.year = value.year();
                    picker.month = value.month();
                    picker.day = value.day();
                    return;
                }

                if let Ok(value) = NaiveDateTime::parse_from_str(raw.trim(), "%Y-%m-%d %H:%M:%S") {
                    picker.year = value.year();
                    picker.month = value.month();
                    picker.day = value.day();
                }
            }
            TemporalMode::Time => {
                if let Ok(value) = NaiveTime::parse_from_str(raw.trim(), "%H:%M:%S") {
                    picker.hour = value.hour();
                    picker.minute = value.minute();
                    picker.second = value.second();
                    return;
                }

                if let Ok(value) = NaiveTime::parse_from_str(raw.trim(), "%H:%M") {
                    picker.hour = value.hour();
                    picker.minute = value.minute();
                    picker.second = 0;
                }
            }
            TemporalMode::DateTime => {
                if let Ok(value) = NaiveDateTime::parse_from_str(raw.trim(), "%Y-%m-%d %H:%M:%S") {
                    picker.year = value.year();
                    picker.month = value.month();
                    picker.day = value.day();
                    picker.hour = value.hour();
                    picker.minute = value.minute();
                    picker.second = value.second();
                    return;
                }

                if let Ok(value) = NaiveDateTime::parse_from_str(raw.trim(), "%Y-%m-%dT%H:%M:%S") {
                    picker.year = value.year();
                    picker.month = value.month();
                    picker.day = value.day();
                    picker.hour = value.hour();
                    picker.minute = value.minute();
                    picker.second = value.second();
                    return;
                }

                if let Ok(value) = DateTime::parse_from_rfc3339(raw.trim()) {
                    picker.year = value.year();
                    picker.month = value.month();
                    picker.day = value.day();
                    picker.hour = value.hour();
                    picker.minute = value.minute();
                    picker.second = value.second();
                }
            }
        }
    }

    fn format_temporal_value(picker: &DateTimePickerState) -> String {
        match picker.mode {
            TemporalMode::Date => format!("{:04}-{:02}-{:02}", picker.year, picker.month, picker.day),
            TemporalMode::Time => {
                format!("{:02}:{:02}:{:02}", picker.hour, picker.minute, picker.second)
            }
            TemporalMode::DateTime => format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                picker.year, picker.month, picker.day, picker.hour, picker.minute, picker.second
            ),
        }
    }

    fn parse_input_value(&self, raw: &str, column: &ColumnInfo) -> anyhow::Result<Value> {
        let trimmed = raw.trim();

        if trimmed.eq_ignore_ascii_case("null") {
            if column.is_nullable {
                return Ok(Value::Null);
            }
            anyhow::bail!("{} is not nullable", column.name);
        }

        if trimmed.is_empty() {
            if column.is_nullable {
                return Ok(Value::Null);
            }
            return Ok(Value::String(String::new()));
        }

        if column.is_enum() {
            if column.enum_options.iter().any(|option| option == trimmed) {
                return Ok(Value::String(trimmed.to_string()));
            }

            anyhow::bail!(
                "invalid enum value for {}. allowed: {}",
                column.name,
                column.enum_options.join(", ")
            );
        }

        let data_type = column.data_type.to_ascii_lowercase();
        let udt_name = column.udt_name.to_ascii_lowercase();

        if data_type == "uuid" || udt_name == "uuid" {
            let parsed = sqlx::types::Uuid::parse_str(trimmed)
                .with_context(|| format!("expected UUID for {}", column.name))?;
            return Ok(Value::String(parsed.to_string()));
        }

        if data_type.contains("boolean") {
            let parsed = trimmed
                .parse::<bool>()
                .with_context(|| format!("expected boolean for {}", column.name))?;
            return Ok(Value::Bool(parsed));
        }

        if data_type.contains("integer")
            || udt_name == "int2"
            || udt_name == "int4"
            || udt_name == "int8"
        {
            let parsed = trimmed
                .parse::<i64>()
                .with_context(|| format!("expected integer for {}", column.name))?;
            return Ok(Value::Number(Number::from(parsed)));
        }

        if data_type.contains("numeric")
            || data_type.contains("double")
            || data_type.contains("real")
            || udt_name == "numeric"
            || udt_name == "float4"
            || udt_name == "float8"
        {
            let parsed = trimmed
                .parse::<f64>()
                .with_context(|| format!("expected number for {}", column.name))?;
            let number = Number::from_f64(parsed)
                .with_context(|| format!("invalid number for {}", column.name))?;
            return Ok(Value::Number(number));
        }

        if data_type == "json" || data_type == "jsonb" || udt_name == "json" || udt_name == "jsonb"
        {
            return serde_json::from_str(trimmed)
                .with_context(|| format!("invalid JSON for {}", column.name));
        }

        Ok(Value::String(trimmed.to_string()))
    }

    fn start_delete_confirmation(&mut self) {
        if self.rows.is_empty() {
            self.status = "No rows to delete".to_string();
            return;
        }

        let indexes = if self.selected_rows.is_empty() {
            vec![self.selected_row]
        } else {
            self.selected_rows.iter().copied().collect::<Vec<_>>()
        };

        let ctids = indexes
            .into_iter()
            .filter_map(|index| self.rows.get(index).map(|row| row.ctid.clone()))
            .collect::<Vec<_>>();

        if ctids.is_empty() {
            self.status = "No rows selected for delete".to_string();
            return;
        }

        let description = if ctids.len() == 1 {
            "Delete 1 row?".to_string()
        } else {
            format!("Delete {} rows?", ctids.len())
        };

        self.confirm_delete = Some(DeleteConfirmation { ctids, description });
    }

    async fn save_dirty_rows(&mut self) {
        let Some(table) = self.selected_table().cloned() else {
            return;
        };

        let dirty_rows = self.dirty_rows.clone();
        if dirty_rows.is_empty() {
            self.status = "No pending edits".to_string();
            return;
        }

        let mut saved = 0_usize;
        let mut failed = 0_usize;
        let mut first_error: Option<String> = None;

        for (row_index, updates) in dirty_rows {
            let Some(row) = self.rows.get(row_index) else {
                continue;
            };

            match db::update_row_values(&self.pool, &table, &row.ctid, &updates).await {
                Ok(()) => {
                    saved += 1;
                    self.dirty_rows.remove(&row_index);
                    self.apply_updates_to_local_row(row_index, &updates);
                }
                Err(error) => {
                    failed += 1;
                    if first_error.is_none() {
                        first_error = Some(format!(
                            "row {}: {}",
                            row_index + 1,
                            Self::format_error_for_status(&error)
                        ));
                    }
                }
            }
        }

        let status = match (saved, failed) {
            (0, 0) => "No rows were updated".to_string(),
            (saved, 0) => format!("Saved {saved} row(s)"),
            (0, failed) => {
                let reason = first_error.unwrap_or_else(|| "unknown error".to_string());
                format!("Failed to save {failed} row(s): {reason}")
            }
            (saved, failed) => {
                let reason = first_error.unwrap_or_else(|| "unknown error".to_string());
                format!("Saved {saved} row(s), failed {failed}: {reason}")
            }
        };

        if failed > 0 {
            self.show_error(status);
        } else {
            self.status = status;
        }

        if failed == 0
            && saved > 0
            && let Err(error) = self.reload_rows().await
        {
            self.show_error(format!(
                "Saved {saved} row(s), but refresh failed: {}",
                Self::format_error_for_status(&error)
            ));
        }
    }

    fn apply_updates_to_local_row(&mut self, row_index: usize, updates: &BTreeMap<String, Value>) {
        let Some(row) = self.rows.get_mut(row_index) else {
            return;
        };

        for (column, value) in updates {
            row.values.insert(column.clone(), value.clone());
        }
    }

    fn format_error_for_status(error: &anyhow::Error) -> String {
        let chain = error.chain().map(|cause| cause.to_string()).collect::<Vec<_>>();
        let primary = chain
            .first()
            .cloned()
            .unwrap_or_else(|| "unexpected error".to_string());
        let root = chain.last().cloned().unwrap_or_else(|| primary.clone());

        if root.contains("invalid input syntax for type uuid") {
            return "invalid UUID value".to_string();
        }

        if primary == root {
            primary
        } else {
            format!("{primary} ({root})")
        }
    }

    fn show_error(&mut self, message: String) {
        self.status = message.clone();
        self.error_popup = Some(message);
    }

    pub fn report_runtime_error(&mut self, error: &anyhow::Error) {
        self.show_error(format!(
            "Unhandled error: {}",
            Self::format_error_for_status(error)
        ));
    }

    async fn execute_query(&mut self) {
        let started = Instant::now();
        match db::run_query(&self.pool, &self.query, 100).await {
            Ok(output) => {
                let elapsed_ms = started.elapsed().as_millis();
                self.query_result_lines = self.render_query_output(output);
                self.status = format!("Query executed in {elapsed_ms}ms");
            }
            Err(error) => {
                self.query_result_lines = vec![format!("Error: {error}")];
                self.show_error("Query failed".to_string());
            }
        }
    }

    fn render_query_output(&self, output: QueryOutput) -> Vec<String> {
        match output {
            QueryOutput::Command { rows_affected } => {
                vec![format!("rows affected: {rows_affected}")]
            }
            QueryOutput::Rows {
                columns,
                rows,
                row_count,
            } => {
                if columns.is_empty() {
                    return vec!["(0 rows)".to_string()];
                }

                let mut lines = vec![columns.join(" | ")];
                for row in rows.iter().take(5) {
                    lines.push(row.join(" | "));
                }

                if row_count > 5 {
                    lines.push(format!("... {} total rows (showing 5)", row_count));
                }

                lines
            }
        }
    }

    async fn reload_rows(&mut self) -> anyhow::Result<()> {
        let Some(table) = self.selected_table().cloned() else {
            self.rows.clear();
            return Ok(());
        };

        self.rows = db::fetch_rows(&self.pool, &table, Self::ROW_LIMIT).await?;
        self.selected_row = self.selected_row.min(self.rows.len().saturating_sub(1));
        self.selected_col = self.selected_col.min(table.columns.len().saturating_sub(1));
        self.selected_rows.clear();
        self.dirty_rows.clear();
        self.datetime_picker = None;
        self.value_picker = None;
        self.grid_h_scroll = 0;
        self.grid_v_scroll = 0;
        self.pending_g = false;
        Ok(())
    }

    fn selected_table(&self) -> Option<&TableInfo> {
        self.tables.get(self.selected_table)
    }

    fn set_active_page(&mut self, page: ActivePage) {
        self.active_page = page;
        self.pending_g = false;
        self.persist_selected_table();
    }

    fn estimated_grid_viewport(&self) -> (usize, usize) {
        let width = self.terminal_width.saturating_sub(4) as usize;
        let height = self.terminal_height.saturating_sub(5) as usize;
        (width.max(1), height.max(1))
    }

    fn ensure_selected_cell_visible(
        &mut self,
        column_ranges: &[(usize, usize)],
        viewport_width: usize,
        viewport_height: usize,
    ) {
        if let Some((start, end)) = column_ranges.get(self.selected_col).copied() {
            if self.grid_h_scroll > start {
                self.grid_h_scroll = start;
            } else {
                let viewport_end = self.grid_h_scroll.saturating_add(viewport_width);
                if end.saturating_add(1) > viewport_end {
                    self.grid_h_scroll = end.saturating_add(1).saturating_sub(viewport_width);
                }
            }
        } else {
            self.grid_h_scroll = 0;
        }

        if self.grid_v_scroll > self.selected_row {
            self.grid_v_scroll = self.selected_row;
        } else {
            let viewport_end = self.grid_v_scroll.saturating_add(viewport_height);
            if self.selected_row.saturating_add(1) > viewport_end {
                self.grid_v_scroll = self
                    .selected_row
                    .saturating_add(1)
                    .saturating_sub(viewport_height);
            }
        }
    }

    fn follow_selected_cell(&mut self) {
        let Some(table) = self.selected_table() else {
            return;
        };

        let column_ranges = self.compute_column_ranges(table);
        let (viewport_width, viewport_height) = self.estimated_grid_viewport();
        self.ensure_selected_cell_visible(&column_ranges, viewport_width, viewport_height);
    }

    fn current_cell_value(&self, row_index: usize, column_name: &str) -> Option<Value> {
        if let Some(dirty_row) = self.dirty_rows.get(&row_index)
            && let Some(value) = dirty_row.get(column_name)
        {
            return Some(value.clone());
        }

        self.rows
            .get(row_index)
            .and_then(|row| row.values.get(column_name).cloned())
    }

    fn value_to_display(&self, value: &Value) -> String {
        match value {
            Value::Null => "NULL".to_string(),
            Value::Bool(v) => v.to_string(),
            Value::Number(v) => v.to_string(),
            Value::String(v) => v.clone(),
            other => other.to_string(),
        }
    }

    fn sanitize_grid_text(raw: &str) -> String {
        raw.replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    fn text_width(raw: &str) -> usize {
        raw.chars().count()
    }

    fn pad_to_width(raw: &str, width: usize) -> String {
        let current = Self::text_width(raw);
        if current >= width {
            return raw.to_string();
        }

        format!("{raw}{}", " ".repeat(width - current))
    }

    fn compute_column_widths(&self, table: &TableInfo) -> Vec<usize> {
        table
            .columns
            .iter()
            .map(|column| {
                let header_width = Self::text_width(&column.name);
                let max_value = self
                    .rows
                    .iter()
                    .enumerate()
                    .map(|(row_index, _)| {
                        let text = self
                            .current_cell_value(row_index, &column.name)
                            .map(|value| self.value_to_display(&value))
                            .unwrap_or_else(|| "NULL".to_string());
                        Self::text_width(&Self::sanitize_grid_text(&text))
                    })
                    .max()
                    .unwrap_or(0);

                header_width.max(max_value).max(1)
            })
            .collect()
    }

    fn compute_column_ranges(&self, table: &TableInfo) -> Vec<(usize, usize)> {
        let widths = self.compute_column_widths(table);
        let mut cursor = 6_usize;
        widths
            .into_iter()
            .map(|width| {
                let start = cursor;
                let end = start.saturating_add(width.saturating_sub(1));
                cursor = end.saturating_add(4);
                (start, end)
            })
            .collect()
    }

    fn build_grid_render_data(&self, table: &TableInfo) -> GridRenderData {
        let widths = self.compute_column_widths(table);
        let mut column_ranges = Vec::with_capacity(widths.len());

        let left_header = Line::from(vec![Span::raw("    ")]);
        let mut right_header_spans = Vec::new();

        let mut cursor = 0_usize;
        for (index, (column, width)) in table.columns.iter().zip(widths.iter()).enumerate() {
            let start = cursor;
            let end = start.saturating_add(width.saturating_sub(1));
            column_ranges.push((start, end));

            let mut style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
            if index == self.selected_col {
                style = style.bg(Color::Rgb(35, 35, 80));
            }

            right_header_spans.push(Span::styled(
                Self::pad_to_width(&column.name, *width),
                style,
            ));

            if index + 1 < table.columns.len() {
                right_header_spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
            }

            cursor = end.saturating_add(4);
        }

        let mut left_rows = Vec::with_capacity(self.rows.len());
        let mut right_rows = Vec::with_capacity(self.rows.len());
        for (row_index, _) in self.rows.iter().enumerate() {
            let marker = if self.selected_rows.contains(&row_index) {
                "[x]"
            } else {
                "[ ]"
            };

            let left_style = if self.selected_rows.contains(&row_index) {
                Style::default().bg(Color::Rgb(25, 45, 25))
            } else {
                Style::default()
            };
            left_rows.push(Line::from(vec![Span::styled(
                format!("{:<4}", marker),
                left_style,
            )]));

            let row_bg = if self.selected_rows.contains(&row_index) {
                Some(Color::Rgb(25, 45, 25))
            } else {
                None
            };

            let mut spans = Vec::new();
            for (col_index, (column, width)) in table.columns.iter().zip(widths.iter()).enumerate() {
                let value = self
                    .current_cell_value(row_index, &column.name)
                    .unwrap_or(Value::Null);
                let raw = self.value_to_display(&value);
                let rendered = Self::sanitize_grid_text(&raw);
                let padded = Self::pad_to_width(&rendered, *width);

                let mut style = row_bg.map_or_else(Style::default, |bg| Style::default().bg(bg));
                if row_index == self.selected_row && col_index == self.selected_col {
                    style = style
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD);
                }

                spans.push(Span::styled(padded, style));

                if col_index + 1 < table.columns.len() {
                    spans.push(Span::styled(
                        " | ",
                        row_bg.map_or_else(
                            || Style::default().fg(Color::DarkGray),
                            |bg| Style::default().fg(Color::DarkGray).bg(bg),
                        ),
                    ));
                }
            }

            right_rows.push(Line::from(spans));
        }

        GridRenderData {
            left_header,
            right_header: Line::from(right_header_spans),
            left_rows,
            right_rows,
            column_ranges,
        }
    }

    fn next_enum_value(column: &ColumnInfo, current: Option<Value>) -> String {
        if column.enum_options.is_empty() {
            return String::new();
        }

        let current = current.and_then(|value| value.as_str().map(ToOwned::to_owned));
        if let Some(current) = current
            && let Some(index) = column
                .enum_options
                .iter()
                .position(|option| option == &current)
        {
            let next_index = (index + 1) % column.enum_options.len();
            return column.enum_options[next_index].clone();
        }

        column.enum_options[0].clone()
    }

    fn persist_selected_table(&self) {
        let selected_table = self
            .selected_table()
            .map(|table| format!("{}.{}", table.schema, table.name));
        let state = PersistedState {
            selected_table,
            active_page: Some(self.active_page.as_str().to_string()),
        };
        if let Err(error) = self.save_state(&state) {
            eprintln!("Warning: failed to persist db-viewer state: {error}");
        }
    }

    fn load_state() -> anyhow::Result<PersistedState> {
        let path = paths::state_path();
        if !path.exists() {
            return Ok(PersistedState::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read state file: {}", path.display()))?;
        toml::from_str(&content).context("failed to parse db-viewer state TOML")
    }

    fn save_state(&self, state: &PersistedState) -> anyhow::Result<()> {
        let path = paths::state_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content =
            toml::to_string_pretty(state).context("failed to serialize db-viewer state")?;
        fs::write(path, content).context("failed to write db-viewer state")
    }

    fn load_query() -> Option<String> {
        let path = paths::query_path();
        fs::read_to_string(path).ok()
    }

    fn save_query(&self) {
        let path = paths::query_path();
        let _ = path.parent().map(fs::create_dir_all);
        if let Err(error) = fs::write(path, &self.query) {
            eprintln!("Warning: failed to persist db-viewer query: {error}");
        }
    }

    pub fn view(&mut self, frame: &mut ratatui::Frame<'_>) {
        let area = frame.area();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(1)])
            .split(area);

        match self.active_page {
            ActivePage::Tables => self.render_tables_page(frame, layout[0]),
            ActivePage::Grid => self.render_grid(frame, layout[0]),
            ActivePage::Query => self.render_query(frame, layout[0]),
        }

        self.render_status(frame, layout[1]);

        if self.help_visible {
            self.render_help(frame);
        }
        if self.add_form.is_some() {
            self.render_add_form(frame);
        }
        if self.confirm_delete.is_some() {
            self.render_delete_confirm(frame);
        }
        if self.datetime_picker.is_some() {
            self.render_datetime_picker(frame);
        }
        if self.value_picker.is_some() {
            self.render_value_picker(frame);
        }
        if self.text_entry.is_some() {
            self.render_text_entry(frame);
        }
        if self.error_popup.is_some() {
            self.render_error_popup(frame);
        }
    }

    fn render_tables_page(&self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(40), Constraint::Min(20)])
            .split(area);

        self.render_tables(frame, chunks[0]);

        let details = if let Some(table) = self.selected_table() {
            let mut lines = vec![
                Line::from(format!("Table: {}.{}", table.schema, table.name)),
                Line::from(format!("Columns: {}", table.columns.len())),
                Line::from(""),
                Line::from("Columns:"),
            ];

            for column in &table.columns {
                let nullable = if column.is_nullable {
                    "nullable"
                } else {
                    "required"
                };
                let enum_hint = if column.is_enum() {
                    format!(" enum({})", column.enum_options.join("|"))
                } else {
                    String::new()
                };

                lines.push(Line::from(format!(
                    "- {}: {} ({nullable}{enum_hint})",
                    column.name, column.data_type
                )));
            }

            lines
        } else {
            vec![Line::from("No table selected")]
        };

        let details_widget = Paragraph::new(details).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green))
                .title("Table Details"),
        );

        frame.render_widget(details_widget, chunks[1]);
    }

    fn render_tables(&self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let title_style = if self.active_page == ActivePage::Tables {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let items = if self.tables.is_empty() {
            vec![ListItem::new("No tables found")]
        } else {
            self.tables
                .iter()
                .enumerate()
                .map(|(index, table)| {
                    let name = format!("{}.{}", table.schema, table.name);
                    let style = if index == self.selected_table {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(name).style(style)
                })
                .collect::<Vec<_>>()
        };

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(title_style)
                .title("Tables"),
        );
        frame.render_widget(list, area);
    }

    fn render_grid(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let Some(table) = self.selected_table() else {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Rows");
            let paragraph = Paragraph::new("No table selected").block(block);
            frame.render_widget(paragraph, area);
            return;
        };

        let border_style = if self.active_page == ActivePage::Grid {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(format!("Rows ({})", self.rows.len()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let render_data = self.build_grid_render_data(table);

        let chunks = if inner.height > 1 {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(1)])
                .split(inner)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1)])
                .split(inner)
        };

        let (header_area, body_area) = if chunks.len() == 1 {
            (chunks[0], Rect::new(0, 0, 0, 0))
        } else {
            (chunks[0], chunks[1])
        };

        let header_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(4), Constraint::Min(1)])
            .split(header_area);
        let body_cols = if body_area.width > 0 {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(4), Constraint::Min(1)])
                .split(body_area)
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(0), Constraint::Length(0)])
                .split(Rect::new(0, 0, 0, 0))
        };

        let header_left = header_cols[0];
        let header_right = header_cols[1];
        let body_left = if body_cols.is_empty() { Rect::new(0, 0, 0, 0) } else { body_cols[0] };
        let body_right = if body_cols.len() > 1 { body_cols[1] } else { Rect::new(0, 0, 0, 0) };

        self.ensure_selected_cell_visible(
            &render_data.column_ranges,
            body_right.width as usize,
            body_right.height as usize,
        );

        let scroll_x = u16::try_from(self.grid_h_scroll).unwrap_or(u16::MAX);
        let scroll_y = u16::try_from(self.grid_v_scroll).unwrap_or(u16::MAX);

        frame.render_widget(Paragraph::new(vec![render_data.left_header]), header_left);
        frame.render_widget(
            Paragraph::new(vec![render_data.right_header]).scroll((0, scroll_x)),
            header_right,
        );

        if body_right.height > 0 {
            let left_rows = if render_data.left_rows.is_empty() {
                vec![Line::from("    ")]
            } else {
                render_data.left_rows
            };
            frame.render_widget(Paragraph::new(left_rows).scroll((scroll_y, 0)), body_left);

            let right_rows = if render_data.right_rows.is_empty() {
                vec![Line::from("(no rows)")]
            } else {
                render_data.right_rows
            };

            let body = Paragraph::new(right_rows)
                .scroll((scroll_y, scroll_x));
            frame.render_widget(body, body_right);
        }
    }

    fn render_query(&self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(8), Constraint::Min(6)])
            .split(area);

        let query_border = if self.active_page == ActivePage::Query {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let query_widget = Paragraph::new(self.query.clone())
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(query_border)
                    .title("Query (r run, e external editor, i inline edit)"),
            );
        frame.render_widget(query_widget, chunks[0]);

        let result_widget = Paragraph::new(self.query_result_lines.join("\n"))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Result"),
            );
        frame.render_widget(result_widget, chunks[1]);
    }

    fn render_status(&self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let mode = if self.text_entry.is_some() {
            "INSERT"
        } else {
            "NORMAL"
        };

        let table = self
            .selected_table()
            .map(|table| format!("{}.{}", table.schema, table.name))
            .unwrap_or_else(|| "<none>".to_string());

        let status = format!(
            "-- {} -- | Page: {} (1 tables, 2 grid, 3 query) | Table: {} | Selected rows: {} | {}",
            mode,
            self.active_page.as_str(),
            table,
            self.selected_rows.len(),
            self.status
        );

        frame.render_widget(
            Paragraph::new(status).style(Style::default().fg(Color::Yellow)),
            area,
        );
    }

    fn render_help(&self, frame: &mut ratatui::Frame<'_>) {
        let area = centered_rect(80, 70, frame.area());
        let lines = vec![
            Line::from("Global"),
            Line::from("  q: quit | ?: close help"),
            Line::from("  1: tables page | 2: grid page | 3: query page"),
            Line::from(""),
            Line::from("Tables Page"),
            Line::from("  j/k or Up/Down: select table"),
            Line::from("  Enter/Right: open grid page for selected table"),
            Line::from(""),
            Line::from("Grid Page"),
            Line::from("  h/j/k/l, arrows: move cell"),
            Line::from("  0/$: jump to first/last column"),
            Line::from("  gg/G: jump to top/bottom row"),
            Line::from("  Enter/i: edit cell"),
            Line::from("  enum/bool fields open a selection popup"),
            Line::from("  date/time/datetime fields open a picker popup"),
            Line::from("  viewport follows active cell horizontally + vertically"),
            Line::from("  Ctrl+S: save pending edits"),
            Line::from("  Space: toggle row selection"),
            Line::from("  A: select all loaded rows | C: clear selection"),
            Line::from("  d: delete (always asks for confirmation)"),
            Line::from("  a: add row"),
            Line::from(""),
            Line::from("Query Page"),
            Line::from("  r: run query | e: external editor | i: inline edit"),
            Line::from("Add Row Form"),
            Line::from("  j/k: move field | Enter/i: edit field | x: clear/null field"),
            Line::from("  Ctrl+S: insert row | Esc: cancel"),
            Line::from("Delete Confirmation"),
            Line::from("  y: confirm delete | n/Esc: cancel"),
        ];

        frame.render_widget(Clear, area);
        let widget = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("db-viewer keymap"),
        );
        frame.render_widget(widget, area);
    }

    fn render_add_form(&self, frame: &mut ratatui::Frame<'_>) {
        let Some(table) = self.selected_table() else {
            return;
        };
        let Some(add_form) = self.add_form.as_ref() else {
            return;
        };

        let area = centered_rect(75, 70, frame.area());
        frame.render_widget(Clear, area);

        let mut lines = Vec::with_capacity(table.columns.len() + 1);
        lines.push(Line::from(
            "Ctrl+S: insert row | Esc: cancel | Enter/i: edit | x: clear/null",
        ));
        for (index, column) in table.columns.iter().enumerate() {
            let selected = index == add_form.selected_field;
            let value = add_form
                .values
                .get(&column.name)
                .and_then(|value| value.as_ref())
                .map(|value| self.value_to_display(value))
                .unwrap_or_else(|| "<unset>".to_string());

            let prefix = if selected { ">>" } else { "  " };
            let nullable = if column.is_nullable {
                " nullable"
            } else {
                " required"
            };
            let enum_hint = if column.is_enum() {
                format!(" enum({})", column.enum_options.join("|"))
            } else {
                String::new()
            };

            lines.push(Line::from(format!(
                "{prefix} {} = {} [{}{}{}]",
                column.name, value, column.data_type, nullable, enum_hint
            )));
        }

        let widget = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Add Row"),
        );
        frame.render_widget(widget, area);
    }

    fn render_delete_confirm(&self, frame: &mut ratatui::Frame<'_>) {
        let Some(confirm) = &self.confirm_delete else {
            return;
        };

        let area = centered_rect(60, 30, frame.area());
        frame.render_widget(Clear, area);
        let text = format!(
            "{}\n\nThis action cannot be undone.\n\nPress 'y' to confirm, 'n' or Esc to cancel.",
            confirm.description
        );
        let widget = Paragraph::new(text).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Red))
                .title("Confirm Delete"),
        );
        frame.render_widget(widget, area);
    }

    fn render_text_entry(&self, frame: &mut ratatui::Frame<'_>) {
        let Some(entry) = &self.text_entry else {
            return;
        };

        let title = match entry.target {
            TextEntryTarget::Query => "Edit Query",
            TextEntryTarget::GridCell { .. } => "Edit Cell",
            TextEntryTarget::AddField { .. } => "Edit Field",
        };

        let area = centered_rect(70, 20, frame.area());
        frame.render_widget(Clear, area);
        let widget = Paragraph::new(entry.buffer.clone())
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(format!("{title} (Enter save, Esc cancel)")),
            );
        frame.render_widget(widget, area);
    }

    fn render_datetime_picker(&self, frame: &mut ratatui::Frame<'_>) {
        let Some(picker) = &self.datetime_picker else {
            return;
        };

        let area = centered_rect(60, 55, frame.area());
        frame.render_widget(Clear, area);

        let mode_label = match picker.mode {
            TemporalMode::Date => "date",
            TemporalMode::Time => "time",
            TemporalMode::DateTime => "datetime",
        };

        let date_line = format!(
            "Date: {}{}{}",
            if picker.selected_part == TemporalPart::Year {
                format!("[{:04}]", picker.year)
            } else {
                format!("{:04}", picker.year)
            },
            if picker.selected_part == TemporalPart::Month {
                format!("-[{:02}]", picker.month)
            } else {
                format!("-{:02}", picker.month)
            },
            if picker.selected_part == TemporalPart::Day {
                format!("-[{:02}]", picker.day)
            } else {
                format!("-{:02}", picker.day)
            }
        );

        let time_line = format!(
            "Time: {}{}{}",
            if picker.selected_part == TemporalPart::Hour {
                format!("[{:02}]", picker.hour)
            } else {
                format!("{:02}", picker.hour)
            },
            if picker.selected_part == TemporalPart::Minute {
                format!(":[{:02}]", picker.minute)
            } else {
                format!(":{:02}", picker.minute)
            },
            if picker.selected_part == TemporalPart::Second {
                format!(":[{:02}]", picker.second)
            } else {
                format!(":{:02}", picker.second)
            }
        );

        let mut lines = vec![Line::from(
            "h/l or Left/Right: part | j/k or Up/Down: +/- | Enter: select | Esc: cancel",
        )];
        if picker.nullable {
            lines.push(Line::from("x: toggle NULL"));
        }
        lines.push(Line::from(""));

        match picker.mode {
            TemporalMode::Date => lines.push(Line::from(date_line)),
            TemporalMode::Time => lines.push(Line::from(time_line)),
            TemporalMode::DateTime => {
                lines.push(Line::from(date_line));
                lines.push(Line::from(time_line));
            }
        }

        lines.push(Line::from(""));
        if picker.is_null {
            lines.push(Line::from("Value: NULL"));
        } else {
            lines.push(Line::from(format!(
                "Value: {}",
                Self::format_temporal_value(picker)
            )));
        }

        let widget = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(
                    "Select {}: {} ({})",
                    mode_label, picker.column_name, picker.column_type
                )),
        );
        frame.render_widget(widget, area);
    }

    fn render_value_picker(&self, frame: &mut ratatui::Frame<'_>) {
        let Some(picker) = &self.value_picker else {
            return;
        };

        let area = centered_rect(50, 50, frame.area());
        frame.render_widget(Clear, area);

        let mut lines = vec![Line::from("j/k or Up/Down: move | Enter: select | Esc: cancel")];
        lines.push(Line::from(""));

        for (index, option) in picker.options.iter().enumerate() {
            let prefix = if index == picker.selected_index {
                ">>"
            } else {
                "  "
            };
            lines.push(Line::from(format!("{prefix} {}", option.label)));
        }

        let widget = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(
                    "Select value: {} ({})",
                    picker.column_name, picker.column_type
                )),
        );
        frame.render_widget(widget, area);
    }

    fn render_error_popup(&self, frame: &mut ratatui::Frame<'_>) {
        let Some(message) = &self.error_popup else {
            return;
        };

        let area = centered_rect(75, 30, frame.area());
        frame.render_widget(Clear, area);

        let lines = vec![
            Line::from(message.clone()),
            Line::from(""),
            Line::from("Press Enter or Esc to dismiss."),
        ];

        let widget = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .title(Span::styled(
                    "Error",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )),
        );
        frame.render_widget(widget, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
