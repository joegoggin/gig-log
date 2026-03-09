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
use crate::api_tester::components::body_editor::open_external_editor;
use crate::api_tester::components::global_listener::GlobalListener;
use crate::utils::sub::SubUtils;

pub mod app;
pub mod body_preview;
pub mod collection;
pub mod components;
pub mod executor;
pub mod paths;
pub mod route_list_state;
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
            Key::Char('b').into(),
            Key::Char('j').into(),
            Key::Char('k').into(),
            Key::Up.into(),
            Key::Down.into(),
            Key::PageUp.into(),
            Key::PageDown.into(),
        ]),
    )?;

    model.refresh_route_list()?;

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

                match model.update(msg)? {
                    Some(Msg::AppClose) => return Ok(()),
                    Some(Msg::OpenBodyEditor) => {
                        restore_terminal(&mut terminal)?;

                        let current_body = model.editor_draft_body();
                        let editor_result = open_external_editor(current_body);

                        terminal = init_terminal()?;

                        match editor_result {
                            Ok(new_body) => {
                                model.update(Msg::BodyEditorResult(new_body))?;
                            }
                            Err(error) => {
                                eprintln!("Editor error: {error}");
                            }
                        }
                    }
                    _ => {}
                }
            }

            terminal.draw(|frame| model.view(frame))?;
        }
    })();

    restore_terminal(&mut terminal)?;
    run_result
}
