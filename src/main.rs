use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io};
use ratatui::widgets::Paragraph;
use ratatui::layout::Alignment;

use ui::screens::home::HomeScreen;
use ui::screens::about::AboutScreen;
use utils::picker::create_file_explorer;
use utils::actions::{Actions, HomeAction};

mod ui;
mod utils;

enum ActiveScreen {
    Home,
    FilePicker,
    Stats,
    Config,
    About,
}

struct App {
    should_quit: bool,
    active_screen: ActiveScreen,
    home_screen: HomeScreen,
    last_selected_file: Option<String>,
    file_explorer: Option<ratatui_explorer::FileExplorer>,
    about_screen: AboutScreen,
}

impl App {
    fn new() -> App {
        App {
            should_quit: false,
            active_screen: ActiveScreen::Home,
            home_screen: HomeScreen::new(),
            last_selected_file: None,
            file_explorer: None,
            about_screen: AboutScreen::new(),
        }
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn Error>> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_key_event(key)?;
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<(), Box<dyn Error>> {
        match self.active_screen {
            ActiveScreen::Home => {
                if let KeyCode::Char(c) = key.code {
                    if let Some(action) = HomeAction::new().handle_actions(c) {
                        self.handle_home_action(action)?;
                    }
                }
            }
            ActiveScreen::FilePicker => {
                if let Some(explorer) = self.file_explorer.as_mut() {
                    explorer.handle(&Event::Key(key))?;

                    if key.code == KeyCode::Enter {
                        if let Some(path) = explorer.current().path().to_str() {
                            self.last_selected_file = Some(path.to_string());
                            self.file_explorer = None;
                            self.active_screen = ActiveScreen::Home;
                        }
                    }

                    if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                        self.file_explorer = None;
                        self.active_screen = ActiveScreen::Home;
                    }
                }
            }
            ActiveScreen::About | ActiveScreen::Stats | ActiveScreen::Config => {
                if let KeyCode::Char('q') | KeyCode::Esc = key.code  {
                    self.active_screen = ActiveScreen::Home;
                }
            }
        }
        Ok(())
    }

    fn ui(&mut self, f: &mut ratatui::Frame) {
        let area = f.area();
        match self.active_screen {
            ActiveScreen::Home => {
                self.home_screen.render(f, area);

                if let Some(selected) = &self.last_selected_file {
                    let text = Paragraph::new(format!("Last selected file: {}", selected))
                        .alignment(Alignment::Center);
                    f.render_widget(text, area);
                }
            }
            ActiveScreen::FilePicker => {
                if let Some(explorer) = &self.file_explorer {
                    f.render_widget(&explorer.widget(), area);
                }
            }
            ActiveScreen::Stats => {
                let block = Paragraph::new("Stats Screen (press 'h' or 'Esc' to go back)")
                    .alignment(Alignment::Center);
                f.render_widget(block, area);
            }
            ActiveScreen::Config => {
                let block = Paragraph::new("Config Screen (press 'h' or 'Esc' to go back)")
                    .alignment(Alignment::Center);
                f.render_widget(block, area);
            }
            ActiveScreen::About => {
                self.about_screen.render(f, area);
            }
        }
    }

    fn handle_home_action(&mut self, action: Actions) -> Result<(), Box<dyn Error>> {
        match action {
            Actions::FindFiles => {
                self.file_explorer = Some(create_file_explorer()?);
                self.active_screen = ActiveScreen::FilePicker;
            }
            Actions::Stats => self.active_screen = ActiveScreen::Stats,
            Actions::Config => self.active_screen = ActiveScreen::Config,
            Actions::About => self.active_screen = ActiveScreen::About,
            Actions::Quit => self.should_quit = true,
        }
        Ok(())
    }
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
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {err:?}");
    }

    Ok(())
}