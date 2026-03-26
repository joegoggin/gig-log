use std::io::{self, Stdout};
use std::time::Duration;

use anyhow::Context;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use tuirealm::SubEventClause;

use crate::db_viewer::app::{AppEffect, AppModel, AppMsg, Id, Msg};
use crate::db_viewer::components::global_listener::GlobalListener;
use crate::db_viewer::editor::open_external_editor;
use crate::utils::env;
use crate::utils::sub::SubUtils;

mod app;
mod components;
mod db;
mod editor;
mod paths;

type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

fn init_terminal() -> anyhow::Result<AppTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut AppTerminal) -> anyhow::Result<()> {
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}

fn install_panic_hook() {
    let default_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        default_hook(panic_info);
    }));
}

pub async fn run() -> anyhow::Result<()> {
    use tuirealm::{Application, EventListenerCfg, PollStrategy};

    install_panic_hook();

    let database_url = env::required_var("DATABASE_URL")
        .context("db-viewer requires a database connection")?;
    let pool = db::connect(&database_url).await?;

    let mut terminal = init_terminal()?;

    let listener =
        EventListenerCfg::default().crossterm_input_listener(Duration::from_millis(10), 10);
    let mut model = AppModel::new(Application::init(listener), pool).await?;

    let global_subs = SubUtils::event_subs([SubEventClause::Any]);
    model.app.mount(
        Id::GlobalListener,
        Box::new(GlobalListener::new()),
        global_subs,
    )?;

    let run_result = 'app_loop: loop {
        let messages = model.app.tick(PollStrategy::Once)?;

        for msg in messages {
            match model.update(msg).await {
                Ok(Some(AppEffect::Close)) => break 'app_loop Ok(()),
                Ok(Some(AppEffect::OpenQueryEditor(initial_query))) => {
                    restore_terminal(&mut terminal)?;
                    let editor_result = open_external_editor(&initial_query, ".sql");
                    terminal = init_terminal()?;

                    match editor_result {
                        Ok(query) => {
                            if let Err(error) =
                                model.update(Msg::App(AppMsg::QueryEditedExternally(query))).await
                            {
                                eprintln!("db-viewer runtime error: {error:#}");
                                model.report_runtime_error(&error);
                            }
                        }
                        Err(error) => {
                            eprintln!("Editor error: {error}");
                        }
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    eprintln!("db-viewer runtime error: {error:#}");
                    model.report_runtime_error(&error);
                }
            }
        }

        terminal.draw(|frame| model.view(frame))?;
    };

    restore_terminal(&mut terminal)?;
    run_result
}
