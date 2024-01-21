use crossterm::event::KeyCode;
use fst::Map;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use serde_json;
use std::error;
use std::fmt::Display;
use std::path::Path;
use wiki;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub enum State {
    Command,
    Search,
    Browse,
    Read,
    Normal,
}

#[derive(Debug)]
pub struct SearchElement<T> {
    pub title: String,
    pub val: T,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct WikiConfig {
    pub wiki_bzip_path: String,
    pub meta_directory: String,
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub state: State,
    pub search: String,
    pub command: String,
    pub page: Option<wiki::DetailedPage>,
    pub selected_page: Option<usize>,
    pub search_results: Vec<SearchElement<u64>>,
    pub list_state: ListState,
    pub scroll: u16,
    // Internals
    pub fst: Map<Vec<u8>>,
    pub base_path: std::path::PathBuf,
    pub bztable: wiki::BZipTable,

    // Crossterm
    pub last_key: Option<KeyCode>,
}

impl Default for App {
    fn default() -> Self {
        // Open config
        let _config_path = "~/.config/wikiterm/config.json";
        let config_path = _config_path.replace("~", std::env::var("HOME").unwrap().as_str());

        let config = serde_json::from_str::<WikiConfig>(
            std::fs::read_to_string(config_path).unwrap().as_str(),
        )
        .expect(
            "Could not parse config file or doesn't exist at
            ~/.config/wikiterm/config.json",
        );

        let _bzpath = &config
            .wiki_bzip_path
            .replace("~", std::env::var("HOME").unwrap().as_str());
        let bzpath = Path::new(_bzpath);

        let _meta_path = &config
            .meta_directory
            .replace("~", std::env::var("HOME").unwrap().as_str());
        let meta_path = Path::new(_meta_path);

        let table_path = meta_path.join("table.json");
        let fst_path = meta_path.join("map.fst");
        if !fst_path.exists() {
            println!(
                "Could not find map.fst in meta directory,
                running indexing"
            );
            match wiki::initial_indexing(
                bzpath.to_str().unwrap().into(),
                meta_path.to_str().unwrap().into(),
            ) {
                Ok(_) => {}
                Err(e) => {
                    panic!("Failed to index: {}", e);
                }
            }
        } else {
            println!("Found map.fst in meta directory, assuming indexed");
        }

        return Self {
            running: true,
            state: State::Normal,
            search: String::new(),
            command: String::new(),
            page: None,
            selected_page: None,
            search_results: Vec::new(),
            list_state: ListState::default(),
            scroll: 0,
            // Internals
            fst: wiki::open_fst(fst_path.to_str().unwrap()).unwrap(),
            base_path: bzpath.to_path_buf(),
            bztable: wiki::open_bz_table(table_path.to_str().unwrap()).unwrap(),
            last_key: None,
        };
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
        let out_search = wiki::search(&self.fst, &self.search).unwrap();
        self.search_results = Vec::new();
        for (key, value) in out_search.iter() {
            self.search_results.push(SearchElement::<u64> {
                title: key.clone(),
                val: *value,
            });
        }
    }

    pub fn get_page(&mut self) {
        self.selected_page = self.list_state.selected();
        let _title = self.search_results[self.selected_page.unwrap()]
            .title
            .clone();
        let val = self.search_results[self.selected_page.unwrap()].val;

        // Extract page_id and block_id
        let page_id = val & 0xffffffff;
        let block_id = val >> 32;
        self.page = wiki::get_detailed_page(&self.bztable, page_id, block_id, &self.base_path);

        if self.page.is_some() {
            self.state = State::Read;
            self.scroll = 0;
        }
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
            }
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
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn up(&mut self) {
        match self.state {
            State::Browse => self.previous(),
            State::Read => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
            }
            _ => {}
        }
    }

    pub fn down(&mut self) {
        match self.state {
            State::Browse => self.next(),
            State::Read => {
                self.scroll += 1;
            }
            _ => {}
        }
    }

    pub fn left(&mut self) {}

    pub fn right(&mut self) {}
}
