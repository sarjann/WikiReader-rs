use std::error;
use std::fmt::Display;
use wiki;
use fst::Set;
use ratatui::widgets::ListState;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub enum State {
    Command,
    Search,
    Browse,
    Read,
    Normal,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Search => write!(f, "Search Mode"),
            State::Browse => write!(f, "Browse Mode"),
            State::Read => write!(f, "Reading Mode"),
            State::Normal => write!(f, "Normal Mode"),
            State::Command => write!(f, "Command Mode"),
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub state: State,
    pub search: String,
    pub command: String,
    pub fst: Set<Vec<u8>>,
    pub search_results: Vec<String>,
    pub list_state: ListState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            state: State::Normal,
            search: String::new(),
            command: String::new(),
            fst: wiki::open_fst().unwrap(),
            search_results: Vec::new(),
            list_state: ListState::default(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub fn tick(&self) {}

    pub fn execute_command(&mut self) {
        match self.command.as_str() {
            ":q" => self.quit(),
            _ => {}
        }
        self.command.clear();
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn search(&mut self) {
        self.search_results = wiki::search(&self.fst, &self.search).unwrap();
    }

    pub fn unselect(&mut self) {
        self.list_state.select(None);
    }

    pub fn previous(&mut self) {
        let length = self.search_results.len();
        let i = match self.list_state.selected() {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    length - 1
                }
            },
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn next(&mut self) {
        let length = self.search_results.len();
        let i = match self.list_state.selected() {
            Some(i) => {
                if i < length - 1 {
                    i + 1
                } else {
                    0
                }
            },
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn up(&mut self) {
    }

    pub fn down(&mut self) {}

    pub fn left(&mut self) {}

    pub fn right(&mut self) {}
}
