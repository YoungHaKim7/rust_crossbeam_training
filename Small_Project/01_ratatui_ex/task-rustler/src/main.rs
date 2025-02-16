use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use std::error::Error;
use std::io;
use task_rustler::app::{AppContext, InputMode};
use task_rustler::command::*;
use task_rustler::ui;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = AppContext::new(String::from("tasks.db"));
    app.refresh_task_list();
    let mut terminal = ratatui::init();
    let res = run_app(&mut terminal, app);
    ratatui::restore();

    if let Err(err) = res {
        println!("{:?}", err)
    }
    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: AppContext,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            // Capture only press key event
            if key.kind == event::KeyEventKind::Release {
                continue;
            }
            match app.input_mode {
                InputMode::View => match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    (KeyCode::Char('h'), KeyModifiers::NONE) => {
                        app.show_help = !app.show_help;
                    }
                    (KeyCode::Esc, KeyModifiers::NONE) => {
                        if app.show_help {
                            app.show_help = false;
                        }
                    }
                    _ => handle_key_event_view_mode(key, &mut app),
                },
                InputMode::Adding => handle_key_event_adding_mode(key.code, &mut app),
                InputMode::EditingExisting => handle_key_event_editing_existing_mode(key.code, &mut app),
                InputMode::Export => handle_key_event_export_mode(key.code, &mut app),
            }
        }
    }
}

fn handle_key_event_view_mode(key: KeyEvent, app: &mut AppContext) {
    match (key.code, key.modifiers) {
        (KeyCode::Char('a'), KeyModifiers::NONE) => {
            let _ = EnterAddModeCommand.execute(app);
        }
        (KeyCode::Down, KeyModifiers::NONE) => {
            app.select_next();
        }
        (KeyCode::Up, KeyModifiers::NONE) => {
            app.select_previous();
        }
        (KeyCode::Char(' '), KeyModifiers::NONE) => {
            let _ = ToggleTaskStatusCommand.execute(app);
        }
        (KeyCode::Char('m'), KeyModifiers::NONE) => {
            let _ = StartEditingExistingTaskCommand.execute(app);
        }
        (KeyCode::Char('p'), KeyModifiers::NONE) => {
            let _ = ToggleItemPriorityCommand.execute(app);
        }
        (KeyCode::Char('s'), KeyModifiers::NONE) => {
            app.sort_by_priority();
        }
        (KeyCode::Char('t'), KeyModifiers::NONE) => {
            app.sort_by_date();
        }
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
            let _ = DeleteTaskCommand.execute(app);
        }
        (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
            handle_errors(EnterExportModeCommand, app);
        }
        _ => {}
    }
}

fn handle_key_event_adding_mode(key: KeyCode, app: &mut AppContext) {
    match key {
        KeyCode::Enter => {
            handle_errors(AddTaskCommand, app);
            if app.error.is_none() {
                app.input_mode = InputMode::View;
            }
        }
        KeyCode::Tab => app.next_input_field(),
        KeyCode::Char(c) => app.handle_char_input(c),
        KeyCode::Backspace => app.handle_backspace(),
        KeyCode::Esc => {
            let _ = StopEditingCommand.execute(app);
        }
        _ => {}
    }
}

fn handle_key_event_editing_existing_mode(key: KeyCode, app: &mut AppContext) {
    match key {
        KeyCode::Tab => app.next_input_field(),
        KeyCode::Enter => {
            handle_errors(FinishEditingExistingTaskCommand, app);
            if app.error.is_none() {
                app.input_mode = InputMode::View;
            }
        }
        KeyCode::Char(c) => app.handle_char_input(c),
        KeyCode::Backspace => app.handle_backspace(),
        KeyCode::Esc => {
            let _ = StopEditingCommand.execute(app);
        }
        _ => {}
    }
}

fn handle_key_event_export_mode(key: KeyCode, app: &mut AppContext) {
    match key {
        KeyCode::Esc => ExitExportModeCommand.execute(app).unwrap(),
        KeyCode::Enter => {
            handle_errors(FinishingExportCommand, app);
            if app.error.is_none() {
                app.input_mode = InputMode::View;
            }
        }
        KeyCode::Char(c) => {
            app.input_export_path.push(c);
        }
        KeyCode::Backspace => {
            app.input_export_path.pop();
        }
        _=>{}
    }
}

fn handle_errors<T: Command>(command:T, app: &mut AppContext) {
    if let Err(e) = command.execute(app) {
        app.error= Some(e.to_string());
    } else {
        app.error = None;
    }
}
