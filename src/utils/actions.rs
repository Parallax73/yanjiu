pub enum Actions {
    FindFiles,
    Stats,
    Config,
    About,
    Quit,
}

pub struct HomeAction;

impl HomeAction {
    pub fn new() -> HomeAction {
        Self
    }
    
    pub fn handle_actions(&self, key: char) -> Option<Actions> {
        match key {
            'f' => Some(Actions::FindFiles),
            's' => Some(Actions::Stats),
            'c' => Some(Actions::Config),
            'a' => Some(Actions::About),
            'q' => Some(Actions::Quit),
            _ => None,
        }
    }
    
}



