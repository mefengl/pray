use std::{error::Error, io, time::Duration};

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

mod app;
mod ui;
use crate::{app::App, ui::ui};

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Set a timeout for the event reading
        if crossterm::event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match app.current_screen {
                    app::CurrentScreen::Main => match key.code {
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            if app.selected_index + 1 < app.directory_entries.len() {
                                app.selected_index += 1;
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if app.selected_index > 0 {
                                app.selected_index -= 1;
                            }
                        }
                        KeyCode::Char('h') => {
                            app.go_back();
                        }
                        KeyCode::Char('l') => {
                            app.enter_directory();
                        }
                        KeyCode::Char(' ') => {
                            app.toggle_selection();
                        }
                        KeyCode::Char('a') => {
                            app.toggle_select_all();
                        }
                        KeyCode::Char('c') => {
                            app.copy_selected_items_to_clipboard();
                        }
                        KeyCode::Char('?') => {
                            app.current_screen = app::CurrentScreen::Help;
                        }
                        _ => {}
                    },
                    app::CurrentScreen::Help => {
                        // Any key press returns to the main screen
                        app.current_screen = app::CurrentScreen::Main;
                    }
                }
            }
        }

        // Decrement message counter if needed
        app.decrement_message_counter();
    }
}
