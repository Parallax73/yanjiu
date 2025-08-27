use std::{fs, path::PathBuf, process::Command};

#[derive(Clone, Debug)]
pub struct ScoredFile {
    pub path: PathBuf,
    pub score: i32,
    pub match_positions: Vec<usize>,
}

#[derive(Clone, Debug)]
struct FuzzyMatch {
    score: i32,
    positions: Vec<usize>,
}

pub enum InputMode {
    Normal,
    Editing,
}

pub struct FindFilesLogic {
    pub files: Vec<ScoredFile>,
    pub is_searching: bool,
    pub search_base_dir: String,
    last_search: String,
    pub has_fd: bool,
}

impl FindFilesLogic {
    pub fn new() -> Self {
        let search_base_dir = "/".to_string();
        let has_fd = Self::check_fd_availability();

        Self {
            files: Vec::new(),
            is_searching: false,
            search_base_dir,
            last_search: String::new(),
            has_fd,
        }
    }

    fn check_fd_availability() -> bool {
        Command::new("fd")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn load_initial_files(&mut self) {
        let paths = if self.has_fd {
            self.search_with_fd_all().unwrap_or_else(|_| self.get_recent_files())
        } else {
            self.get_recent_files()
        };

        self.files = paths.into_iter().map(|path| ScoredFile {
            path,
            score: 0,
            match_positions: Vec::new(),
        }).collect();
    }

    fn get_recent_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.search_base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                    if files.len() >= 30 {
                        break;
                    }
                }
            }
        }

        files.sort_by(|a, b| {
            a.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
                .cmp(&b.file_name().unwrap_or_default().to_string_lossy().to_lowercase())
        });

        files
    }

    fn fuzzy_match(pattern: &str, text: &str) -> Option<FuzzyMatch> {
        if pattern.is_empty() {
            return Some(FuzzyMatch { score: 0, positions: Vec::new() });
        }

        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_lower: Vec<char> = text.to_lowercase().chars().collect();
        let text_chars: Vec<char> = text.chars().collect();

        let mut best_score = i32::MIN;
        let mut best_positions = Vec::new();

        for start_idx in 0..text_lower.len() {
            let mut current_positions = Vec::new();
            let mut pattern_idx = 0;
            let mut current_score = 0;
            let mut consecutive_bonus = 0;
            let mut last_match_idx = start_idx.saturating_sub(1);

            if text_lower[start_idx] != pattern_chars[0].to_lowercase().next().unwrap() {
                continue;
            }

            for text_idx in start_idx..text_lower.len() {
                if pattern_idx < pattern_chars.len()
                    && text_lower[text_idx] == pattern_chars[pattern_idx].to_lowercase().next().unwrap()
                {
                    let gap = text_idx - last_match_idx - 1;
                    current_score -= gap as i32 * 2;

                    if text_idx == 0 {
                        current_score += 25;
                    } else {
                        let prev_char = text_chars[text_idx - 1];
                        if !prev_char.is_alphanumeric() {
                            current_score += 20;
                        } else if prev_char.is_lowercase() && text_chars[text_idx].is_uppercase() {
                            current_score += 20;
                        }
                    }

                    if text_idx == last_match_idx + 1 {
                        consecutive_bonus += 15;
                        current_score += consecutive_bonus;
                    } else {
                        consecutive_bonus = 0;
                    }

                    if text_chars[text_idx] == pattern_chars[pattern_idx] {
                        current_score += 5;
                    }

                    current_positions.push(text_idx);
                    last_match_idx = text_idx;
                    pattern_idx += 1;
                }
            }

            if pattern_idx == pattern_chars.len() {
                current_score -= text.len() as i32 / 5;

                if current_score > best_score {
                    best_score = current_score;
                    best_positions = current_positions.clone();
                }
            }
        }

        if best_score > i32::MIN {
            Some(FuzzyMatch {
                score: best_score,
                positions: best_positions,
            })
        } else {
            None
        }
    }

    pub fn search_files(&mut self, query: &str) {
        if query == self.last_search {
            return;
        }

        self.last_search = query.to_string();

        if query.is_empty() {
            self.load_initial_files();
            return;
        }

        if query.len() < 2 {
            return;
        }

        self.is_searching = true;

        let paths = if self.has_fd {
            self.search_with_fd(query).unwrap_or_else(|_| Vec::new())
        } else {
            self.search_with_find(query).unwrap_or_else(|_| self.search_with_rust(query))
        };

        let mut scored_files: Vec<ScoredFile> = paths.into_iter()
            .filter_map(|path| {
                let filename = path.file_name()?.to_string_lossy();
                let full_path = path.to_string_lossy();

                let filename_match = Self::fuzzy_match(query, &filename);
                let path_match = Self::fuzzy_match(query, &full_path);

                match (filename_match, path_match) {
                    (Some(f_match), Some(p_match)) => {
                        if f_match.score >= p_match.score {
                            Some(ScoredFile {
                                path,
                                score: f_match.score + 50,
                                match_positions: f_match.positions,
                            })
                        } else {
                            Some(ScoredFile {
                                path,
                                score: p_match.score,
                                match_positions: p_match.positions,
                            })
                        }
                    },
                    (Some(f_match), None) => Some(ScoredFile {
                        path,
                        score: f_match.score + 50,
                        match_positions: f_match.positions,
                    }),
                    (None, Some(p_match)) => Some(ScoredFile {
                        path,
                        score: p_match.score,
                        match_positions: p_match.positions,
                    }),
                    (None, None) => None,
                }
            })
            .collect();

        scored_files.sort_by(|a, b| b.score.cmp(&a.score));

        self.files = scored_files;
        self.is_searching = false;
    }

    fn search_with_fd_all(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let output = Command::new("fd")
            .arg("--type")
            .arg("f")
            .arg("--max-results")
            .arg("50")
            .arg("--max-depth")
            .arg("10")
            .arg("--search-path")
            .arg(&self.search_base_dir)
            .arg("--exclude")
            .arg("/proc")
            .arg("--exclude")
            .arg("/sys")
            .arg("--exclude")
            .arg("/dev")
            .arg("--exclude")
            .arg("/tmp")
            .arg(".")
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let files: Vec<PathBuf> = stdout
                .lines()
                .map(PathBuf::from)
                .collect();
            Ok(files)
        } else {
            Err("fd command failed".into())
        }
    }

    fn search_with_fd(&self, query: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let output = Command::new("fd")
            .arg("--type")
            .arg("f")
            .arg("--ignore-case")
            .arg("--max-results")
            .arg("500")
            .arg("--max-depth")
            .arg("10")
            .arg("--search-path")
            .arg(&self.search_base_dir)
            .arg("--exclude")
            .arg("/proc")
            .arg("--exclude")
            .arg("/sys")
            .arg("--exclude")
            .arg("/dev")
            .arg("--exclude")
            .arg("/tmp")
            .arg(".")
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let files: Vec<PathBuf> = stdout
                .lines()
                .take(500)
                .map(PathBuf::from)
                .collect();
            Ok(files)
        } else {
            Err("fd command failed".into())
        }
    }

    fn search_with_find(&self, query: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let output = Command::new("find")
            .arg(&self.search_base_dir)
            .arg("-type")
            .arg("f")
            .arg("-not")
            .arg("-path")
            .arg("/proc/*")
            .arg("-not")
            .arg("-path")
            .arg("/sys/*")
            .arg("-not")
            .arg("-path")
            .arg("/dev/*")
            .arg("-not")
            .arg("-path")
            .arg("/tmp/*")
            .arg("-print")
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let files: Vec<PathBuf> = stdout
                .lines()
                .take(500)
                .map(PathBuf::from)
                .collect();
            Ok(files)
        } else {
            Err("find command failed".into())
        }
    }

    fn search_with_rust(&self, query: &str) -> Vec<PathBuf> {
        let mut files = Vec::new();
        self.search_directory_recursive(&PathBuf::from(&self.search_base_dir), &mut files, 10);
        files
    }

    fn search_directory_recursive(&self, dir: &PathBuf, files: &mut Vec<PathBuf>, depth: u32) {
        if depth == 0 || files.len() >= 200 {
            return;
        }

        let system_dirs = ["/proc", "/sys", "/dev", "/tmp"];
        if system_dirs.iter().any(|&d| dir.starts_with(d)) {
            return;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() {
                    files.push(path.clone());
                    if files.len() >= 200 {
                        return;
                    }
                } else if path.is_dir() && !path.file_name().unwrap_or_default().to_string_lossy().starts_with('.') {
                    self.search_directory_recursive(&path, files, depth - 1);
                }
            }
        }
    }

    pub fn get_selected_file(&self, index: Option<usize>) -> Option<&ScoredFile> {
        index.and_then(|i| self.files.get(i))
    }
}