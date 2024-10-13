use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, CurrentScreen};

pub fn ui(frame: &mut Frame, app: &App) {
    let size = frame.area();

    match app.current_screen {
        CurrentScreen::Main => draw_main_screen(frame, app, size),
        CurrentScreen::Help => draw_help_screen(frame, size),
    }
}

fn draw_main_screen(frame: &mut Frame, app: &App, size: ratatui::layout::Rect) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Footer
        ])
        .split(size);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Left: Directory list
            Constraint::Percentage(30), // Right: Selected items
        ])
        .split(chunks[0]);

    // Left: Directory list
    let items: Vec<ListItem> = app
        .directory_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let file_name = entry.file_name().unwrap().to_string_lossy();
            let is_selected = app.selected_items.contains(entry);
            let is_cursor = i == app.selected_index;

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

    let items_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("File List"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(items_list, main_chunks[0]);

    // Right: Selected items list
    let selected_items: Vec<ListItem> = app
        .selected_items
        .iter()
        .map(|entry| {
            let display_path = entry.strip_prefix(&app.base_dir).unwrap_or(entry);
            let file_name = display_path.to_string_lossy();
            ListItem::new(Line::from(Span::raw(file_name)))
        })
        .collect();

    let selected_list = List::new(selected_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Selected Items"),
    );

    frame.render_widget(selected_list, main_chunks[1]);

    // Footer with basic commands
    let footer_text = if let Some(message) = &app.footer_message {
        Span::styled(message, Style::default().fg(Color::Green))
    } else {
        Span::raw("[h]Back [l]Enter [j/k]Up/Down [Space]Select [a]All [c]Copy [q]Quit [?]Help")
    };

    let footer = Paragraph::new(Line::from(footer_text))
        .style(Style::default().fg(Color::White))
        .block(Block::default());

    frame.render_widget(footer, chunks[1]);
}

fn draw_help_screen(frame: &mut Frame, size: ratatui::layout::Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            "Help - Available Commands",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::raw("[h] Go back to parent directory")),
        Line::from(Span::raw("[l] Enter directory")),
        Line::from(Span::raw("[j/k] Move down/up")),
        Line::from(Span::raw("[Space] Select/Deselect item")),
        Line::from(Span::raw("[a] Select/Deselect all items")),
        Line::from(Span::raw("[c] Copy selected files' contents to clipboard")),
        Line::from(Span::raw("[q] Quit the application")),
        Line::from(Span::raw("[?] Show this help screen")),
        Line::from(""),
        Line::from(Span::raw("Press any key to return")),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(ratatui::layout::Alignment::Left);

    frame.render_widget(help_paragraph, size);
}
