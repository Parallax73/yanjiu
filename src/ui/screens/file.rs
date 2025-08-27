use std::{fs, path::PathBuf, process::Command};
use ratatui::{
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph, ListState, Wrap},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};


use crate::utils::picker::{FindFilesLogic, InputMode};

pub struct FindFilesScreen {
    logic: FindFilesLogic,
    input: String,
    character_index: usize,
    input_mode: InputMode,
    show_preview: bool,
    list_state: ListState,
    preview_content: Vec<String>,
}

impl FindFilesScreen {
    pub fn new() -> Self {
        let mut screen = Self {
            logic: FindFilesLogic::new(),
            input: String::new(),
            input_mode: InputMode::Normal,
            character_index: 0,
            show_preview: true,
            list_state: ListState::default(),
            preview_content: Vec::new(),
        };

        screen.logic.load_initial_files();
        screen.update_preview();
        screen
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('i') | KeyCode::Char('/') => {
                            self.input_mode = InputMode::Editing;
                        }
                        KeyCode::Up | KeyCode::Char('k') => self.previous_file(),
                        KeyCode::Down | KeyCode::Char('j') => self.next_file(),
                        KeyCode::Enter => {
                            if let Some(file) = self.logic.get_selected_file(self.list_state.selected()) {
                                Command::new("nvim").arg(&file.path).spawn().ok();
                            }
                        }
                        KeyCode::Char('g') => {
                            if !self.logic.files.is_empty() {
                                self.list_state.select(Some(0));
                                self.update_preview();
                            }
                        }
                        KeyCode::Char('G') => {
                            if !self.logic.files.is_empty() {
                                self.list_state.select(Some(self.logic.files.len() - 1));
                                self.update_preview();
                            }
                        }
                        KeyCode::PageUp => {
                            if !self.logic.files.is_empty() {
                                let current = self.list_state.selected().unwrap_or(0);
                                let new_index = current.saturating_sub(10);
                                self.list_state.select(Some(new_index));
                                self.update_preview();
                            }
                        }
                        KeyCode::PageDown => {
                            if !self.logic.files.is_empty() {
                                let current = self.list_state.selected().unwrap_or(0);
                                let new_index = (current + 10).min(self.logic.files.len() - 1);
                                self.list_state.select(Some(new_index));
                                self.update_preview();
                            }
                        }
                        KeyCode::Char('p') => {
                            self.show_preview = !self.show_preview;
                            if self.show_preview {
                                self.update_preview();
                            }
                        }
                        _ => {}
                    }
                }
            }
            InputMode::Editing => {
                if key.kind == KeyEventKind::Press {
                    match (key.code, key.modifiers) {
                        (KeyCode::Enter, _) => {
                            self.input_mode = InputMode::Normal;
                        }
                        (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                            self.previous_file();
                        },
                        (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                            self.next_file();
                        },
                        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                            self.input.clear();
                            self.character_index = 0;
                            self.logic.search_files("");
                        },
                        (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                            self.delete_word_backwards();
                            let input = self.input.clone();
                            self.logic.search_files(&input);
                        },
                        (KeyCode::PageUp, _) => {
                            if !self.logic.files.is_empty() {
                                let current = self.list_state.selected().unwrap_or(0);
                                let new_index = current.saturating_sub(10);
                                self.list_state.select(Some(new_index));
                                self.update_preview();
                            }
                        }
                        (KeyCode::PageDown, _) => {
                            if !self.logic.files.is_empty() {
                                let current = self.list_state.selected().unwrap_or(0);
                                let new_index = (current + 10).min(self.logic.files.len() - 1);
                                self.list_state.select(Some(new_index));
                                self.update_preview();
                            }
                        }
                        (KeyCode::Char(to_insert), _) => {
                            self.enter_char(to_insert);
                            let input = self.input.clone();
                            self.logic.search_files(&input);
                        }
                        (KeyCode::Backspace, _) => {
                            self.delete_char();
                            let input = self.input.clone();
                            self.logic.search_files(&input);
                        }
                        (KeyCode::Left, _) => self.move_cursor_left(),
                        (KeyCode::Right, _) => self.move_cursor_right(),
                        (KeyCode::Home, _) => {
                            self.character_index = 0;
                        },
                        (KeyCode::End, _) => {
                            self.character_index = self.input.chars().count();
                        },
                        (KeyCode::Esc, _) => {
                            self.input_mode = InputMode::Normal;
                        }
                        (KeyCode::Tab, _) => {
                            self.input_mode = InputMode::Normal;
                        }
                        (KeyCode::F(5), _) => {
                            let input = self.input.clone();
                            self.logic.search_files(&input);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let main_layout = if self.show_preview {
            Layout::horizontal([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ])
        } else {
            Layout::horizontal([Constraint::Percentage(100)])
        };

        let main_areas = main_layout.split(area);
        let list_area = main_areas[0];

        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let chunks = vertical.split(list_area);
        let search_area = chunks[0];
        let files_area = chunks[1];

        let search_style = match self.input_mode {
            InputMode::Normal => Style::default().fg(Color::Gray),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        };

        let search_title = if self.logic.is_searching {
            "🔍 Searching..."
        } else if self.input.is_empty() {
            if self.logic.has_fd {
                "Find Files (fd) - i/: search, ↑↓/jk: navigate, g/G: top/bottom, p: preview"
            } else {
                "Find Files - i/: search, ↑↓/jk: navigate, g/G: top/bottom, p: preview"
            }
        } else {
            "Find Files - Esc: normal mode"
        };

        let search_input = Paragraph::new(self.input.as_str())
            .style(search_style)
            .block(Block::bordered().title(search_title));
        frame.render_widget(search_input, search_area);

        if matches!(self.input_mode, InputMode::Editing) {
            #[allow(clippy::cast_possible_truncation)]
            frame.set_cursor_position(Position::new(
                search_area.x + self.character_index as u16 + 1,
                search_area.y + 1,
            ));
        }

        let file_items: Vec<ListItem> = self
            .logic
            .files
            .iter()
            .map(|scored_file| {
                let path = &scored_file.path;
                let filename = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();

                let relative_path = if let Ok(rel) = path.strip_prefix(&self.logic.search_base_dir) {
                    rel.to_string_lossy().into_owned()
                } else {
                    path.to_string_lossy().into_owned()
                };

                let icon = match path.extension().and_then(|s| s.to_str()) {
                    Some("rs") => "🦀",
                    Some("py") => "🐍",
                    Some("js") | Some("ts") => "⚡",
                    Some("jsx") | Some("tsx") => "⚛️",
                    Some("md") => "📝",
                    Some("json") => "📋",
                    Some("toml") | Some("yaml") | Some("yml") => "⚙️",
                    Some("txt") => "📄",
                    Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("svg") => "🖼️",
                    Some("css") | Some("scss") | Some("sass") => "🎨",
                    Some("html") | Some("htm") => "🌐",
                    Some("go") => "🐹",
                    Some("cpp") | Some("c") | Some("h") => "⚡",
                    Some("java") => "☕",
                    Some("php") => "🐘",
                    Some("rb") => "💎",
                    Some("sh") | Some("bash") | Some("zsh") => "🐚",
                    _ => "📄",
                };

                let line = if !self.input.is_empty() && !scored_file.match_positions.is_empty() {
                    let mut spans = vec![Span::from(format!("{} ", icon))];
                    let filename_chars: Vec<char> = filename.chars().collect();
                    let mut last_pos = 0;

                    let mut match_iter = scored_file.match_positions.iter().peekable();
                    while let Some(&pos) = match_iter.next() {
                        let mut end_pos = pos;
                        while let Some(&&next_pos) = match_iter.peek() {
                            if next_pos == end_pos + 1 {
                                end_pos = *match_iter.next().unwrap();
                            } else {
                                break;
                            }
                        }

                        if pos > last_pos {
                            spans.push(Span::from(filename_chars[last_pos..pos].iter().collect::<String>()));
                        }

                        spans.push(Span::styled(
                            filename_chars[pos..=end_pos].iter().collect::<String>(),
                            Style::default().bg(Color::Yellow).fg(Color::Black),
                        ));

                        last_pos = end_pos + 1;
                    }

                    if last_pos < filename_chars.len() {
                        spans.push(Span::from(filename_chars[last_pos..].iter().collect::<String>()));
                    }

                    spans.push(Span::styled(
                        format!(" ({})", relative_path),
                        Style::default().fg(Color::Gray),
                    ));

                    Line::from(spans)
                } else {
                    Line::from(format!("{} {} ({})", icon, filename, relative_path))
                };

                ListItem::new(line)
            })
            .collect();

        let files_count = self.logic.files.len();
        let selected_info = if let Some(selected) = self.list_state.selected() {
            format!(" ({}/{})", selected + 1, files_count)
        } else {
            String::new()
        };

        let list_title = if self.input.is_empty() {
            format!("Recent Files ({}){}", files_count, selected_info)
        } else {
            format!("'{}' ({}){}", self.input, files_count, selected_info)
        };

        let files_list = List::new(file_items)
            .block(Block::bordered().title(list_title))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(files_list, files_area, &mut self.list_state);

        if self.show_preview && main_areas.len() > 1 {
            let preview_area = main_areas[1];
            let preview_title = if let Some(selected_file) = self.logic.get_selected_file(self.list_state.selected()) {
                format!("Preview: {}", selected_file.path.file_name().unwrap_or_default().to_string_lossy())
            } else {
                "Preview".to_string()
            };

            let preview_text = self.preview_content.join("\n");
            let preview_widget = Paragraph::new(preview_text)
                .block(Block::bordered().title(preview_title))
                .wrap(Wrap { trim: false })
                .scroll((0, 0));

            frame.render_widget(preview_widget, preview_area);
        }
    }

    fn update_preview(&mut self) {
        if !self.show_preview {
            return;
        }

        self.preview_content.clear();

        if let Some(selected_file) = self.logic.get_selected_file(self.list_state.selected()) {
            match fs::read_to_string(&selected_file.path) {
                Ok(content) => {
                    self.preview_content = content
                        .lines()
                        .take(100)
                        .map(|line| line.to_string())
                        .collect();
                },
                Err(_) => {
                    if let Ok(metadata) = fs::metadata(&selected_file.path) {
                        self.preview_content = vec![
                            format!("File: {}", selected_file.path.display()),
                            format!("Size: {} bytes", metadata.len()),
                            format!("Modified: {:?}", metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)),
                            String::new(),
                            "Cannot preview this file type".to_string(),
                        ];
                    } else {
                        self.preview_content = vec!["Cannot read file".to_string()];
                    }
                }
            }
        }
    }

    fn previous_file(&mut self) {
        let files = &self.logic.files;
        if files.is_empty() {
            return;
        }
        let selected = self.list_state.selected().unwrap_or(0);
        let new_index = if selected == 0 {
            files.len() - 1
        } else {
            selected - 1
        };
        self.list_state.select(Some(new_index));
        self.update_preview();
    }

    fn next_file(&mut self) {
        let files = &self.logic.files;
        if files.is_empty() {
            return;
        }
        let selected = self.list_state.selected().unwrap_or(0);
        let new_index = if selected >= files.len() - 1 {
            0
        } else {
            selected + 1
        };
        self.list_state.select(Some(new_index));
        self.update_preview();
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn delete_word_backwards(&mut self) {
        let cursor_pos = self.character_index;
        let chars: Vec<char> = self.input.chars().collect();

        if cursor_pos == 0 {
            return;
        }

        let mut new_cursor_pos = cursor_pos;

        while new_cursor_pos > 0 && chars[new_cursor_pos - 1].is_whitespace() {
            new_cursor_pos -= 1;
        }

        while new_cursor_pos > 0 && !chars[new_cursor_pos - 1].is_whitespace() {
            new_cursor_pos -= 1;
        }

        let before: String = chars[..new_cursor_pos].iter().collect();
        let after: String = chars[cursor_pos..].iter().collect();

        self.input = before + &after;
        self.character_index = new_cursor_pos;
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        if self.character_index == 0 {
            return;
        }
        let current_index = self.character_index;
        let from_left_to_current_index = current_index - 1;

        let before = self.input.chars().take(from_left_to_current_index);
        let after = self.input.chars().skip(current_index);

        self.input = before.chain(after).collect();
        self.move_cursor_left();
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }
}