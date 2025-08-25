use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

mod ui;

use ui::screens::home::{HomeScreen, HomeAction};

enum ActiveScreen {
    Home,
    FindFiles,
    Stats,
    Config,
    About,
}

struct App {
    should_quit: bool,
    active_screen: ActiveScreen,
    home_screen: HomeScreen,
}

impl App {
    fn new() -> App {
        App {
            should_quit: false,
            active_screen: ActiveScreen::Home,
            home_screen: HomeScreen::new(),
        }
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn Error>> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(250);

        loop {
            terminal.draw(|f| self.ui(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                self.should_quit = true;
                            }
                            KeyCode::Char(c) => {
                                if let ActiveScreen::Home = self.active_screen {
                                    if let Some(action) = self.home_screen.handle_key(c) {
                                        self.handle_home_action(action);
                                    }
                                } else if c == 'h' {
                                    self.active_screen = ActiveScreen::Home;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = Instant::now();
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn ui(&mut self, f: &mut ratatui::Frame) {
        let area = f.area();
        match self.active_screen {
            ActiveScreen::Home => self.home_screen.render(f, area),
            ActiveScreen::FindFiles => {
                let block = ratatui::widgets::Paragraph::new("Find Files Screen (press 'h' to go back)")
                    .alignment(ratatui::layout::Alignment::Center);
                f.render_widget(block, area);
            }
            ActiveScreen::Stats => {
                let block = ratatui::widgets::Paragraph::new("Stats Screen (press 'h' to go back)")
                    .alignment(ratatui::layout::Alignment::Center);
                f.render_widget(block, area);
            }
            ActiveScreen::Config => {
                let block = ratatui::widgets::Paragraph::new("Config Screen (press 'h' to go back)")
                    .alignment(ratatui::layout::Alignment::Center);
                f.render_widget(block, area);
            }
            ActiveScreen::About => {
                let block = ratatui::widgets::Paragraph::new("About Screen (press 'h' to go back)")
                    .alignment(ratatui::layout::Alignment::Center);
                f.render_widget(block, area);
            }
        }
    }

    fn handle_home_action(&mut self, action: HomeAction) {
        match action {
            HomeAction::FindFiles => self.active_screen = ActiveScreen::FindFiles,
            HomeAction::Stats => self.active_screen = ActiveScreen::Stats,
            HomeAction::Config => self.active_screen = ActiveScreen::Config,
            HomeAction::About => self.active_screen = ActiveScreen::About,
            HomeAction::Quit => self.should_quit = true,
        }
    }

    fn on_tick(&mut self) {}
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {err:?}");
    }

    Ok(())
}


