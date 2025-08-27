use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io, time::{Duration, Instant}};

mod ui;
mod utils;

use ui::screens::home::{HomeAction, HomeScreen};
use ui::screens::file::FindFilesScreen;

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
    find_files_screen: FindFilesScreen,
}

impl App {
    fn new() -> App {
        App {
            should_quit: false,
            active_screen: ActiveScreen::Home,
            home_screen: HomeScreen::new(),
            find_files_screen: FindFilesScreen::new(),
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
                    self.handle_key_event(key);
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

    fn handle_key_event(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.should_quit = true;
                    return;
                }
                _ => {}
            }
        }

        match self.active_screen {
            ActiveScreen::Home => {
                if key.kind == KeyEventKind::Press {
                    if let KeyCode::Char(c) = key.code {
                        if let Some(action) = self.home_screen.handle_key(c) {
                            self.handle_home_action(action);
                        }
                    }
                }
            }
            ActiveScreen::FindFiles => {
                if key.kind == KeyEventKind::Press {
                    if let KeyCode::Char('h') = key.code {
                        self.active_screen = ActiveScreen::Home;
                        return;
                    }
                }
                self.find_files_screen.handle_key_event(key);
            }
            ActiveScreen::Stats | ActiveScreen::Config | ActiveScreen::About => {
                if key.kind == KeyEventKind::Press {
                    if let KeyCode::Char('h') = key.code {
                        self.active_screen = ActiveScreen::Home;
                    }
                }
            }
        }
    }

    fn ui(&mut self, f: &mut ratatui::Frame) {
        let area = f.area();
        match self.active_screen {
            ActiveScreen::Home => self.home_screen.render(f, area),
            ActiveScreen::FindFiles => self.find_files_screen.render(f, area),
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
            HomeAction::FindFiles => {
                self.active_screen = ActiveScreen::FindFiles;
            }
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
