use ratatui_explorer::{FileExplorer, Theme};
use ratatui::{widgets::*, prelude::*};
use std::io::Result;

pub fn create_file_explorer() -> Result<FileExplorer> {
    let theme = Theme::default()
        .add_default_title()
        .with_title_bottom(|fe| format!("[{} files]", fe.files().len()).into())
        .with_block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded))
        .with_highlight_item_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .with_highlight_dir_style(
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )
        .with_highlight_symbol("> ".into());

    FileExplorer::with_theme(theme)
}
