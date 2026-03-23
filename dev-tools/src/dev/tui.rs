use std::io::{self, stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{ExecutableCommand, execute};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;

use super::log_store::{LogEntry, LogStore, Service};
use super::ui::{self, AppState};

pub enum TuiEvent {
    Log(LogEntry),
    ClearLogs,
    ServiceStarted(Service),
    ServiceExited(Service),
}

pub async fn run_tui(mut event_rx: mpsc::Receiver<TuiEvent>) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut log_store = LogStore::new();
    let mut state = AppState::new();

    let result = run_event_loop(&mut terminal, &mut log_store, &mut state, &mut event_rx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    log_store: &mut LogStore,
    state: &mut AppState,
    event_rx: &mut mpsc::Receiver<TuiEvent>,
) -> Result<()> {
    loop {
        while let Ok(event) = event_rx.try_recv() {
            apply_tui_event(log_store, state, event);
        }

        // Draw and capture frame size
        let entries = log_store.filtered(state.filter);
        let mut metrics = ui::ViewMetrics::default();
        terminal.draw(|frame| {
            metrics = ui::render(frame, &entries, state);
        })?;

        let max_offset = metrics.max_offset;
        let page_jump = (metrics.viewport_height / 2).max(1);

        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if handle_key_event(
                    log_store,
                    state,
                    key.code,
                    key.modifiers,
                    max_offset,
                    page_jump,
                ) {
                    return Ok(());
                }
            }
        }

        tokio::select! {
            event = event_rx.recv() => {
                match event {
                    Some(event) => {
                        apply_tui_event(log_store, state, event);
                        while let Ok(next_event) = event_rx.try_recv() {
                            apply_tui_event(log_store, state, next_event);
                        }
                    }
                    None => {
                        // All senders dropped, all services exited
                        return Ok(());
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(16)) => {}
        }
    }
}

fn handle_key_event(
    log_store: &mut LogStore,
    state: &mut AppState,
    key_code: KeyCode,
    modifiers: KeyModifiers,
    max_offset: usize,
    page_jump: usize,
) -> bool {
    if state.pending_g {
        state.pending_g = false;
        if key_code == KeyCode::Char('g') {
            state.follow = false;
            state.scroll_offset = 0;
            return false;
        }
    }

    match (key_code, modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => true,
        (KeyCode::Char('c'), _) => {
            log_store.clear_filtered(state.filter);
            state.scroll_offset = 0;
            state.follow = true;
            false
        }
        (KeyCode::Char('1'), _) => {
            state.filter = Some(Service::Api);
            state.scroll_offset = 0;
            state.follow = true;
            false
        }
        (KeyCode::Char('2'), _) => {
            state.filter = Some(Service::Web);
            state.scroll_offset = 0;
            state.follow = true;
            false
        }
        (KeyCode::Char('3'), _) => {
            state.filter = Some(Service::Common);
            state.scroll_offset = 0;
            state.follow = true;
            false
        }
        (KeyCode::Char('4'), _) => {
            state.filter = Some(Service::DevTools);
            state.scroll_offset = 0;
            state.follow = true;
            false
        }
        (KeyCode::Char('5'), _) => {
            state.filter = Some(Service::Docs);
            state.scroll_offset = 0;
            state.follow = true;
            false
        }
        (KeyCode::Char('6'), _) => {
            state.filter = Some(Service::System);
            state.scroll_offset = 0;
            state.follow = true;
            false
        }
        (KeyCode::Char('x'), _) => {
            state.auto_clear = !state.auto_clear;
            false
        }
        (KeyCode::Char('a'), _) => {
            state.filter = None;
            state.scroll_offset = 0;
            state.follow = true;
            false
        }
        (KeyCode::Char('j') | KeyCode::Down, _) => {
            if state.follow {
                state.scroll_offset = max_offset;
            }
            state.follow = false;
            state.scroll_offset = state.scroll_offset.saturating_add(1).min(max_offset);
            false
        }
        (KeyCode::Char('k') | KeyCode::Up, _) => {
            if state.follow {
                state.scroll_offset = max_offset;
            }
            state.follow = false;
            state.scroll_offset = state.scroll_offset.saturating_sub(1);
            false
        }
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
            if state.follow {
                state.scroll_offset = max_offset;
            }
            state.follow = false;
            state.scroll_offset = state
                .scroll_offset
                .saturating_add(page_jump)
                .min(max_offset);
            false
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            if state.follow {
                state.scroll_offset = max_offset;
            }
            state.follow = false;
            state.scroll_offset = state.scroll_offset.saturating_sub(page_jump);
            false
        }
        (KeyCode::Char('g'), _) => {
            state.pending_g = true;
            false
        }
        (KeyCode::Char('G'), _) => {
            state.follow = true;
            false
        }
        _ => false,
    }
}

fn apply_tui_event(log_store: &mut LogStore, state: &mut AppState, event: TuiEvent) {
    match event {
        TuiEvent::Log(entry) => {
            log_store.push(entry);
        }
        TuiEvent::ClearLogs => {
            if state.auto_clear {
                log_store.clear();
                state.scroll_offset = 0;
                state.follow = true;
            }
        }
        TuiEvent::ServiceStarted(service) => match service {
            Service::Api => state.services_running[0] = true,
            Service::Web => state.services_running[1] = true,
            Service::Common => state.services_running[2] = true,
            Service::DevTools => state.services_running[3] = true,
            Service::Docs => state.services_running[4] = true,
            Service::System => state.services_running[5] = true,
        },
        TuiEvent::ServiceExited(service) => match service {
            Service::Api => state.services_running[0] = false,
            Service::Web => state.services_running[1] = false,
            Service::Common => state.services_running[2] = false,
            Service::DevTools => state.services_running[3] = false,
            Service::Docs => state.services_running[4] = false,
            Service::System => state.services_running[5] = false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{TuiEvent, apply_tui_event, handle_key_event};
    use crate::dev::log_store::{LogEntry, LogStore, Service};
    use crate::dev::ui::AppState;
    use crossterm::event::{KeyCode, KeyModifiers};

    fn sample_entry() -> LogEntry {
        LogEntry {
            service: Service::Api,
            line: "sample".to_string(),
        }
    }

    #[test]
    fn clear_logs_clears_store_when_auto_clear_enabled() {
        let mut log_store = LogStore::new();
        log_store.push(sample_entry());

        let mut state = AppState::new();
        state.follow = false;
        state.scroll_offset = 7;
        state.auto_clear = true;

        apply_tui_event(&mut log_store, &mut state, TuiEvent::ClearLogs);

        assert!(log_store.filtered(None).is_empty());
        assert_eq!(state.scroll_offset, 0);
        assert!(state.follow);
    }

    #[test]
    fn clear_logs_is_noop_when_auto_clear_disabled() {
        let mut log_store = LogStore::new();
        log_store.push(sample_entry());

        let mut state = AppState::new();
        state.follow = false;
        state.scroll_offset = 7;
        state.auto_clear = false;

        apply_tui_event(&mut log_store, &mut state, TuiEvent::ClearLogs);

        assert_eq!(log_store.filtered(None).len(), 1);
        assert_eq!(state.scroll_offset, 7);
        assert!(!state.follow);
    }

    #[test]
    fn x_key_toggles_auto_clear() {
        let mut log_store = LogStore::new();
        let mut state = AppState::new();
        assert!(state.auto_clear);

        let should_quit = handle_key_event(
            &mut log_store,
            &mut state,
            KeyCode::Char('x'),
            KeyModifiers::NONE,
            0,
            1,
        );

        assert!(!should_quit);
        assert!(!state.auto_clear);
    }
}
