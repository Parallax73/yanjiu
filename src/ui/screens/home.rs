use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};

const LOGO_LINES: &[&str] = &[
    " █████ █████                           ███   ███            ",
    "▒▒███ ▒▒███                           ▒▒▒   ▒▒▒             ",
    " ▒▒███ ███    ██████   ████████       █████ ████  █████ ████",
    "  ▒▒█████    ▒▒▒▒▒███ ▒▒███▒▒███     ▒▒███ ▒▒███ ▒▒███ ▒███ ",
    "   ▒▒███      ███████  ▒███ ▒███      ▒███  ▒███  ▒███ ▒███ ",
    "    ▒███     ███▒▒███  ▒███ ▒███      ▒███  ▒███  ▒███ ▒███ ",
    "    █████   ▒▒████████ ████ █████     ▒███  █████ ▒▒████████",
    "   ▒▒▒▒▒     ▒▒▒▒▒▒▒▒ ▒▒▒▒ ▒▒▒▒▒      ▒███ ▒▒▒▒▒   ▒▒▒▒▒▒▒▒ ",
    "                                  ███ ▒███                  ",
    "                                 ▒▒██████                   ",
    "                                  ▒▒▒▒▒▒                    ",
];


struct MenuItem {
    key: char,
    label: &'static str,
}

const MENU_ITEMS: &[MenuItem] = &[
    MenuItem {
        key: 'f',
        label: "Find file",
    },
    MenuItem {
        key: 's',
        label: "Stats",
    },
    MenuItem {
        key: 'c',
        label: "Config",
    },
    MenuItem {
        key: 'a',
        label: "About",
    },
    MenuItem {
        key: 'q',
        label: "Quit",
    },
];

pub struct HomeScreen;

impl HomeScreen {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let is_small = area.height < 25 || area.width < 80;
        let is_very_small = area.height < 20 || area.width < 60;

        if is_very_small {
            self.render_compact(frame, area);
        } else if is_small {
            self.render_medium(frame, area);
        } else {
            self.render_full(frame, area);
        }
    }

    fn render_full(&self, frame: &mut Frame, area: Rect) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(5),
                Constraint::Min(7),
                Constraint::Min(1),
                Constraint::Min(10),
                Constraint::Min(0),
            ])
            .split(area);

        self.render_logo(frame, main_chunks[1]);
        self.render_subtitle(frame, main_chunks[2]);
        self.render_menu(frame, main_chunks[3]);
    }

    fn render_medium(&self, frame: &mut Frame, area: Rect) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(2),
                Constraint::Min(7),
                Constraint::Min(1),
                Constraint::Min(10),
                Constraint::Min(0),
            ])
            .split(area);

        self.render_logo(frame, main_chunks[1]);
        self.render_subtitle(frame, main_chunks[2]);
        self.render_menu(frame, main_chunks[3]);
    }

    fn render_compact(&self, frame: &mut Frame, area: Rect) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Min(1),
                Constraint::Min(6),
                Constraint::Length(1),
            ])
            .split(area);

        self.render_logo_compact(frame, main_chunks[1]);
        self.render_subtitle_compact(frame, main_chunks[2]);
        self.render_menu(frame, main_chunks[3]);
    }

    fn render_logo(&self, frame: &mut Frame, area: Rect) {
        let logo_width = LOGO_LINES.iter().map(|line| line.len()).max().unwrap_or(0) as u16;

        let logo_area = if area.width > logo_width + 4 {
            let margin = (area.width - logo_width) / 2;
            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(margin),
                    Constraint::Length(logo_width),
                    Constraint::Min(0),
                ])
                .split(area);
            horizontal_chunks[1]
        } else {
            area
        };

        let logo_text = Text::from_iter(
            LOGO_LINES
                .iter()
                .map(|line| Line::from(Span::styled(*line, Style::default().fg(Color::Cyan)))),
        );

        let logo_paragraph = Paragraph::new(logo_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan));

        frame.render_widget(logo_paragraph, logo_area);
    }

    fn render_logo_compact(&self, frame: &mut Frame, area: Rect) {
        let compact_logo = vec![Line::from(Span::styled(
            "YANJIU",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];

        let paragraph = Paragraph::new(compact_logo).alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn render_subtitle(&self, frame: &mut Frame, area: Rect) {
        let subtitle = vec![Line::from(Span::styled(
            "CLI Study/Memorization Tool",
            Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
        ))];

        let paragraph = Paragraph::new(subtitle).alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn render_subtitle_compact(&self, frame: &mut Frame, area: Rect) {
        let subtitle = vec![Line::from(Span::styled(
            "Study Tool",
            Style::default().fg(Color::Gray),
        ))];

        let paragraph = Paragraph::new(subtitle).alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn render_menu(&self, frame: &mut Frame, area: Rect) {
        let max_label_len = MENU_ITEMS.iter().map(|item| item.label.len()).max().unwrap_or(0);

        let menu_lines: Vec<Line> = MENU_ITEMS
            .iter()
            .map(|item| {
                let padded_label = format!("{:<width$}", item.label, width = max_label_len + 2);
                Line::from(vec![
                    Span::styled(padded_label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled(item.key.to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(menu_lines).alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    pub fn handle_key(&self, key: char) -> Option<HomeAction> {
        match key {
            'f' => Some(HomeAction::FindFiles),
            's' => Some(HomeAction::Stats),
            'c' => Some(HomeAction::Config),
            'a' => Some(HomeAction::About),
            'q' => Some(HomeAction::Quit),
            _ => None,
        }
    }
}

pub enum HomeAction {
    FindFiles,
    Stats,
    Config,
    About,
    Quit,
}