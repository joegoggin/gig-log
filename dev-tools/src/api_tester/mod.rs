use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::{AttrValue, Attribute};

use crate::api_tester::app::{AppModel, Id, InputMode, Msg};
use crate::api_tester::components::global_listener::GlobalListener;
use crate::api_tester::components::route_list::RouteList;
use crate::utils::sub::SubUtils;

pub mod app;
pub mod collection;
pub mod components;
pub mod executor;
pub mod paths;
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
    let listener =
        EventListenerCfg::default().crossterm_input_listener(Duration::from_millis(10), 10);
    let mut model = AppModel::new(Application::init(listener))?;

    model.app.mount(
        Id::GlobalListener,
        Box::new(GlobalListener::new()),
        SubUtils::key_subs([
            Key::Char('q').into(),
            Key::Char('v').into(),
            Key::Char('i').into(),
            Key::Esc.into(),
            KeyEvent::new(Key::Char('s'), KeyModifiers::CONTROL),
        ]),
    )?;

    model.app.mount(
        Id::RouteList,
        Box::new(RouteList::new(&model.collection.routes)),
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

    let run_result = (|| -> anyhow::Result<()> {
        loop {
            model.app.attr(
                &Id::GlobalListener,
                Attribute::Custom("input_mode"),
                AttrValue::Flag(model.input_mode == InputMode::Insert),
            )?;

            let messages = model.app.tick(PollStrategy::Once)?;

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
