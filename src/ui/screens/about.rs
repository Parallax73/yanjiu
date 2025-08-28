use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span, Text},
    widgets::{Paragraph},
    Frame,
};
use ratatui::prelude::{Color, Style};
use crate::utils::logo::YanjiuLogo;

pub struct AboutScreen;

impl AboutScreen {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let is_small = area.height < 20 || area.width < 60;
        let is_very_small = area.height < 10 || area.width < 40;

        if is_very_small {
            self.render_compact(frame, area);
        } else if is_small {
            self.render_medium(frame, area);
        } else {
            self.render_full(frame, area);
        }
    }

    fn render_full(&self, frame: &mut Frame, area: Rect) {
        self.draw(frame, area);
    }

    fn render_medium(&self, frame: &mut Frame, area: Rect) {
        self.draw(frame, area);
    }

    fn render_compact(&self, frame: &mut Frame, area: Rect) {
        self.draw(frame, area);
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let logo_lines = YanjiuLogo::lines()
            .iter()
            .map(|line| Line::from(Span::styled(*line, Style::default().fg(Color::Cyan))))
            .collect::<Vec<Line>>();

        let text_lines = vec![
            Line::from("Yanjiu is a terminal flashcard tool for studying and memorization"),
            Line::from(""),
            Line::from(""),
            Line::from("How to use it"),
            Line::from(""),
            Line::from("To navigate inside the explorer you can use:"),
            Line::from(""),
            Line::from("'j' or <DownArrow> to move the selection down"),
            Line::from("'k' or <UpArrow>  to move the selection up"),
            Line::from("'h' or <LeftArrow>  to go the parent directory"),
            Line::from("'l' or <RightArrow>  to go the child directory"),
            Line::from("'Home'  Select the first entry"),
            Line::from("'End'  Select the last entry"),
            Line::from("'Enter'  Select the file"),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from("Made by Parallax"),
        ];

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(logo_lines.len() as u16),
                Constraint::Min(1),
            ])
            .split(area);

        let logo_text = Text::from(logo_lines);
        let paragraph_logo = Paragraph::new(logo_text)
            .alignment(Alignment::Center);

        let text = Text::from(text_lines);
        let paragraph_text = Paragraph::new(text)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph_logo, chunks[0]);
        frame.render_widget(paragraph_text, chunks[1]);
    }
}
