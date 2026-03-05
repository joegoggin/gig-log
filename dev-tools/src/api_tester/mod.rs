use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use tuirealm::event::Key;
use tuirealm::{NoUserEvent, Sub, SubClause, SubEventClause};

use crate::api_tester::app::{AppModel, Id, Msg};
use crate::api_tester::components::global_listener::GlobalListener;

pub mod app;
pub mod collection;
pub mod components;
pub mod executor;
pub mod variables;

type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

fn init_terminal() -> anyhow::Result<AppTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut AppTerminal) -> anyhow::Result<()> {
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}

fn install_panic_hook() {
    let default_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        default_hook(panic_info);
    }));
}

pub async fn run() -> anyhow::Result<()> {
    use tuirealm::{Application, EventListenerCfg, PollStrategy};

    install_panic_hook();
    let mut terminal = init_terminal()?;
    let mut model = AppModel::init()?;
    let listener =
        EventListenerCfg::default().crossterm_input_listener(Duration::from_millis(10), 10);
    let mut app: Application<Id, Msg, NoUserEvent> = Application::init(listener);

    app.mount(
        Id::GlobalListener,
        Box::new(GlobalListener::default()),
        vec![
            Sub::new(
                SubEventClause::Keyboard(Key::Char('q').into()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(Key::Char('v').into()),
                SubClause::Always,
            ),
            Sub::new(SubEventClause::Keyboard(Key::Esc.into()), SubClause::Always),
        ],
    )?;
    // app.mount(Id::RouteList, RouteList::default(), vec![]);

    let run_result = (|| -> anyhow::Result<()> {
        loop {
            let messages = app.tick(PollStrategy::Once)?;

            for msg in messages {
                if matches!(msg, Msg::AppClose) {
                    return Ok(());
                }

                if matches!(model.update(msg)?, Some(Msg::AppClose)) {
                    return Ok(());
                }
            }

            terminal.draw(|frame| model.view(frame))?;
        }
    })();

    restore_terminal(&mut terminal)?;
    run_result
}
