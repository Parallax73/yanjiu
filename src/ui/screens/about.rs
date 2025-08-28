use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{Block, Paragraph},
    Frame,
};

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
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
            ])
            .split(area);

        self.draw(frame, chunks[0]);
    }

    fn render_medium(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
            ])
            .split(area);

        self.draw(frame, chunks[0]);
    }

    fn render_compact(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
            ])
            .split(area);

        self.draw(frame, chunks[0]);
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from("Yanjiu is a CLI Study/Memorization Tool..."),
            // TODO! Write an about.

        ];

        let paragraph = Paragraph::new(text)
            .block(Block::default().title("Yanjiu "))
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }
}


