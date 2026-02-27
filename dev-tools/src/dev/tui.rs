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
        // Draw
        let entries = log_store.filtered(state.filter);
        terminal.draw(|frame| {
            ui::render(frame, &entries, state);
        })?;

        // Handle events with a short poll to keep UI responsive
        tokio::select! {
            // Check for crossterm input events
            _ = tokio::task::yield_now() => {
                while event::poll(Duration::from_millis(16))? {
                    if let Event::Key(key) = event::read()? {
                        match (key.code, key.modifiers) {
                            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                                return Ok(());
                            }
                            (KeyCode::Char('c'), _) => {
                                log_store.clear();
                                state.scroll_offset = 0;
                                state.follow = true;
                            }
                            (KeyCode::Char('1'), _) => {
                                state.filter = Some(Service::Api);
                                state.scroll_offset = 0;
                                state.follow = true;
                            }
                            (KeyCode::Char('2'), _) => {
                                state.filter = Some(Service::Web);
                                state.scroll_offset = 0;
                                state.follow = true;
                            }
                            (KeyCode::Char('3'), _) => {
                                state.filter = Some(Service::Docs);
                                state.scroll_offset = 0;
                                state.follow = true;
                            }
                            (KeyCode::Char('a'), _) => {
                                state.filter = None;
                                state.scroll_offset = 0;
                                state.follow = true;
                            }
                            (KeyCode::Char('j') | KeyCode::Down, _) => {
                                state.follow = false;
                                state.scroll_offset = state.scroll_offset.saturating_add(1);
                            }
                            (KeyCode::Char('k') | KeyCode::Up, _) => {
                                state.follow = false;
                                state.scroll_offset = state.scroll_offset.saturating_sub(1);
                            }
                            (KeyCode::Char('G'), _) => {
                                state.follow = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
            // Check for log events from child processes
            event = event_rx.recv() => {
                match event {
                    Some(TuiEvent::Log(entry)) => {
                        log_store.push(entry);
                    }
                    Some(TuiEvent::ServiceStarted(service)) => {
                        match service {
                            Service::Api => state.services_running[0] = true,
                            Service::Web => state.services_running[1] = true,
                            Service::Docs => state.services_running[2] = true,
                        }
                    }
                    Some(TuiEvent::ServiceExited(service)) => {
                        match service {
                            Service::Api => state.services_running[0] = false,
                            Service::Web => state.services_running[1] = false,
                            Service::Docs => state.services_running[2] = false,
                        }
                    }
                    None => {
                        // All senders dropped, all services exited
                        return Ok(());
                    }
                }
            }
        }
    }
}
