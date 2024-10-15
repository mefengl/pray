use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use ratatui::layout::Position;

use crate::app::{App, FocusedPane};

// Main UI function to draw all panes at once
pub fn ui(frame: &mut Frame, app: &App) {
    let size = frame.area();

    if app.show_help {
        draw_help_screen(frame, size);
        return;
    }

    if app.renaming_collection {
        draw_rename_prompt(frame, app, size);
        return;
    }

    // Create the main layout with a vertical split for content and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Footer
        ])
        .split(size);

    // Split the main content horizontally into files and collections panes
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left: Files pane
            Constraint::Percentage(50), // Right: Collections pane
        ])
        .split(chunks[0]);

    // Draw the files pane
    draw_files_pane(frame, app, main_chunks[0]);
    // Draw the collections pane
    draw_collections_pane(frame, app, main_chunks[1]);

    // Footer with basic commands or messages
    let footer_text = if let Some(message) = &app.footer_message {
        Span::styled(message, Style::default().fg(Color::Green))
    } else {
        match app.focused_pane {
            FocusedPane::FilesPane => Span::raw(
                "[j/k] Up/Down [h] Back [l/Enter] Enter \
                 [Space] Select [a] All [c] Copy [q] Quit",
            ),
            FocusedPane::CollectionsPane => {
                Span::raw("[j/k] Up/Down [d] Delete [c] Copy [r] Rename [q] Quit")
            }
            FocusedPane::SelectedFilesPane => Span::raw("[j/k] Up/Down [Space] Unselect [q] Quit"),
        }
    };

    let footer = Paragraph::new(Line::from(footer_text))
        .style(Style::default().fg(Color::White))
        .block(Block::default());

    frame.render_widget(footer, chunks[1]);
}

// Draw the files pane
fn draw_files_pane(frame: &mut Frame, app: &App, area: Rect) {
    // Determine the style based on focus
    let is_focused = matches!(app.focused_pane, FocusedPane::FilesPane);

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    // Create a block with title and border
    let title = Line::from("[1] Files");

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Create list items for the directory entries
    let items: Vec<ListItem> = app
        .directory_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let file_name = entry.file_name().unwrap().to_string_lossy();
            let is_selected = app.selected_items.contains(entry);
            let is_cursor = is_focused && i == app.selected_file_index;

            let style = match (is_selected, is_cursor) {
                (true, true) => Style::default().fg(Color::Black).bg(Color::LightGreen),
                (true, false) => Style::default().fg(Color::Black).bg(Color::Green),
                (false, true) => Style::default().fg(Color::White).bg(Color::Blue),
                (false, false) => Style::default(),
            };

            let symbol = if entry.is_dir() { "[D]" } else { "   " };
            ListItem::new(Line::from(Span::styled(
                format!("{} {}", symbol, file_name),
                style,
            )))
        })
        .collect();

    let items_list =
        List::new(items).highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(items_list, inner_area);
}

// Draw the collections pane
fn draw_collections_pane(frame: &mut Frame, app: &App, area: Rect) {
    // Split the collections pane vertically into list and details
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Top: Collection list
            Constraint::Percentage(50), // Bottom: Selected files
        ])
        .split(area);

    // Draw the collection list
    draw_collection_list(frame, app, chunks[0]);
    // Draw the selected files
    draw_selected_files_pane(frame, app, chunks[1]);
}

// Draw the collection list
fn draw_collection_list(frame: &mut Frame, app: &App, area: Rect) {
    // Determine the style based on focus
    let is_focused = matches!(app.focused_pane, FocusedPane::CollectionsPane);

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    // Create a block with title and border
    let title = Line::from("[2] Collections");

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Create list items for the collections
    let items: Vec<ListItem> = app
        .collections
        .iter()
        .enumerate()
        .map(|(i, collection)| {
            let is_cursor = is_focused && i == app.selected_collection_index;

            let style = if is_cursor {
                Style::default().fg(Color::White).bg(Color::Blue)
            } else {
                Style::default()
            };

            let item_text = format!(
                "{} - {} files - {}",
                collection.name,
                collection.num_files,
                collection.timestamp.format("%Y-%m-%d %H:%M:%S")
            );

            ListItem::new(Line::from(Span::styled(item_text, style)))
        })
        .collect();

    let collections_list =
        List::new(items).highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(collections_list, inner_area);
}

// Draw the selected files pane
fn draw_selected_files_pane(frame: &mut Frame, app: &App, area: Rect) {
    // Determine the style based on focus
    let is_focused = matches!(app.focused_pane, FocusedPane::SelectedFilesPane);

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title("[3] Selected Files")
        .border_style(border_style);
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem>;

    match app.focused_pane {
        FocusedPane::FilesPane => {
            // Display selected items from the FilesPane
            if app.selected_items.is_empty() {
                // Display a message if there are no selected files
                let text = Paragraph::new("No selected files").alignment(Alignment::Center);
                frame.render_widget(text, inner_area);
                return;
            }

            let base_dir = &app.base_dir;

            items = app
                .selected_items
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let display_path = entry.strip_prefix(base_dir).unwrap_or(entry);
                    let file_name = display_path.to_string_lossy();
                    let is_cursor = is_focused && i == app.selected_file_in_collection_index;

                    let style = if is_cursor {
                        Style::default().fg(Color::White).bg(Color::Blue)
                    } else {
                        Style::default()
                    };

                    ListItem::new(Line::from(Span::styled(file_name, style)))
                })
                .collect();
        }
        FocusedPane::CollectionsPane | FocusedPane::SelectedFilesPane => {
            // Display files from the selected collection
            if app.collections.is_empty() {
                // Display a message if there are no collections
                let text = Paragraph::new("No collections").alignment(Alignment::Center);
                frame.render_widget(text, inner_area);
                return;
            }

            let collection = &app.collections[app.selected_collection_index];

            if collection.files.is_empty() {
                let text =
                    Paragraph::new("No files in selected collection").alignment(Alignment::Center);
                frame.render_widget(text, inner_area);
                return;
            }

            let base_dir = &app.base_dir;

            items = collection
                .files
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let display_path = entry.strip_prefix(base_dir).unwrap_or(entry);
                    let file_name = display_path.to_string_lossy();
                    let is_cursor = is_focused && i == app.selected_file_in_collection_index;

                    let style = if is_cursor {
                        Style::default().fg(Color::White).bg(Color::Blue)
                    } else {
                        Style::default()
                    };

                    ListItem::new(Line::from(Span::styled(file_name, style)))
                })
                .collect();
        }
    }

    let files_list =
        List::new(items).highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(files_list, inner_area);
}

// Draw the help screen
fn draw_help_screen(frame: &mut Frame, size: Rect) {
    use ratatui::widgets::Wrap;

    let help_text = vec![
        Line::from(Span::styled(
            "Help - Available Commands",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::raw("[1] Switch to Files Pane")),
        Line::from(Span::raw("[2] Switch to Collections Pane")),
        Line::from(Span::raw("[3] Switch to Selected Files Pane")),
        Line::from(Span::raw("[h] Go back to parent directory")),
        Line::from(Span::raw("[l/Enter] Enter directory")),
        Line::from(Span::raw("[j/k] Move down/up")),
        Line::from(Span::raw("[Space] Select/Deselect item")),
        Line::from(Span::raw("[a] Select/Deselect all items")),
        Line::from(Span::raw("[c] Copy selected files' contents to clipboard")),
        Line::from(Span::raw("[d] Delete selected collection or unselect file")),
        Line::from(Span::raw("[r] Rename selected collection")),
        Line::from(Span::raw("[ESC] Cancel renaming")),
        Line::from(Span::raw("[q] Quit the application")),
        Line::from(Span::raw("[?] Show this help screen")),
        Line::from(""),
        Line::from(Span::raw("Press any key to return")),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    // Clear the background before rendering the popup
    frame.render_widget(Clear, size);
    frame.render_widget(help_paragraph, size);
}

// Draw the rename prompt
fn draw_rename_prompt(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Rename Collection");

    let input = Paragraph::new(app.new_collection_name.as_str())
        .block(block.clone())
        .style(Style::default().fg(Color::Yellow));

    // Center the popup
    let popup_area = centered_rect(60, 20, area);

    // Clear the background before rendering the popup
    frame.render_widget(Clear, popup_area);
    frame.render_widget(input, popup_area);

    // Add a hint below the input box
    let hint = Paragraph::new("[Enter] Confirm, [Esc] Cancel")
        .style(Style::default())
        .alignment(Alignment::Center);

    let hint_area = Rect {
        x: popup_area.x,
        y: popup_area.y + popup_area.height - 1,
        width: popup_area.width,
        height: 1,
    };

    frame.render_widget(hint, hint_area);

    // Put cursor past the end of the input text
    frame.set_cursor_position(Position::new(
        popup_area.x + app.new_collection_name.len() as u16 + 1,
        popup_area.y + 1,
    ));
}

// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    let vertical_chunk = popup_layout[1];

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical_chunk);

    horizontal_layout[1]
}
