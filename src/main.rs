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
        terminal.draw(|f| {
            ui(f, app);

            // Update scroll after rendering to get correct dimensions
            if let app::FocusedPane::FilesPane = app.focused_pane {
                let height = f.area().height.saturating_sub(3) as usize; // Subtract borders and footer
                app.update_scroll(height);
            }
        })?;

        // Set a timeout for the event reading
        if crossterm::event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if app.show_help {
                    // Hide help screen on any key press
                    app.show_help = false;
                    continue;
                }

                if app.renaming_collection {
                    match key.code {
                        KeyCode::Char(c) => {
                            app.new_collection_name.push(c);
                        }
                        KeyCode::Backspace => {
                            app.new_collection_name.pop();
                        }
                        KeyCode::Enter => {
                            app.confirm_rename();
                        }
                        KeyCode::Esc => {
                            app.cancel_rename();
                        }
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('g') => {
                        app.toggle_gitignore();
                    }
                    // Quit the application
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    // Switch focus between panes using numbers
                    KeyCode::Char('1') => {
                        app.focused_pane = app::FocusedPane::FilesPane;
                    }
                    KeyCode::Char('2') => {
                        app.focused_pane = app::FocusedPane::CollectionsPane;
                    }
                    KeyCode::Char('3') => {
                        app.focused_pane = app::FocusedPane::SelectedFilesPane;
                    }
                    // Show help screen
                    KeyCode::Char('?') => {
                        app.show_help = true;
                    }
                    _ => {
                        // Handle key events based on the focused pane
                        match app.focused_pane {
                            app::FocusedPane::FilesPane => match key.code {
                                KeyCode::Char('j') | KeyCode::Down => {
                                    if app.selected_file_index + 1 < app.directory_entries.len() {
                                        app.selected_file_index += 1;
                                    }
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    if app.selected_file_index > 0 {
                                        app.selected_file_index -= 1;
                                    }
                                }
                                KeyCode::Char('h') => {
                                    app.go_back();
                                }
                                KeyCode::Char('l') | KeyCode::Enter => {
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
                                _ => {}
                            },
                            app::FocusedPane::CollectionsPane => match key.code {
                                KeyCode::Char('j') | KeyCode::Down => {
                                    if app.selected_collection_index + 1 < app.collections.len() {
                                        app.selected_collection_index += 1;
                                        app.selected_file_in_collection_index = 0;
                                    }
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    if app.selected_collection_index > 0 {
                                        app.selected_collection_index -= 1;
                                        app.selected_file_in_collection_index = 0;
                                    }
                                }
                                KeyCode::Char('d') => {
                                    app.remove_selected_collection();
                                }
                                KeyCode::Char('c') => {
                                    app.copy_selected_collection_to_clipboard();
                                }
                                KeyCode::Char('r') => {
                                    app.start_rename();
                                }
                                _ => {}
                            },
                            app::FocusedPane::SelectedFilesPane => match key.code {
                                KeyCode::Char('j') | KeyCode::Down => {
                                    if app.collections.is_empty() {
                                        continue;
                                    }
                                    let collection =
                                        &app.collections[app.selected_collection_index];
                                    if app.selected_file_in_collection_index + 1
                                        < collection.files.len()
                                    {
                                        app.selected_file_in_collection_index += 1;
                                    }
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    if app.selected_file_in_collection_index > 0 {
                                        app.selected_file_in_collection_index -= 1;
                                    }
                                }
                                KeyCode::Char(' ') => {
                                    app.unselect_file_from_collection();
                                }
                                _ => {}
                            },
                        }
                    }
                }
            }
        }

        // Decrement message counter if needed
        app.decrement_message_counter();
    }
}
