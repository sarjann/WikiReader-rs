use fst::automaton::{Automaton, StartsWith, StartsWithState, Str, Subsequence};
use fst::{IntoStreamer, Map, MapBuilder, Streamer};
use std::io::BufWriter;
use std::fs::File;
use crate::page::Page;

pub trait Searchable {
    fn new() -> Self;
    fn get(&self, key: &str) -> Option<u64>;
    fn len(&self) -> usize;
    fn search(&self, query: &str) -> std::io::Result<Vec<(String, u64)>>;
    fn open_searcher(&mut self, path: &str) -> std::io::Result<()>;
    fn create_searcher(&mut self, pages: &Vec<Page>, output_path: &str) -> std::io::Result<()>;
}

#[derive(Debug)]
pub struct Searcher {
    map: Option<Map<Vec<u8>>>,
}

impl Searchable for Searcher {
    fn new() -> Searcher {
        let searcher = Searcher { map: None };
        return searcher;
    }

    fn len(&self) -> usize {
        if self.map.is_none() {
            return 0;
        }
        let len = self.map.as_ref().unwrap().len();
        return len;
    }

    fn get(&self, key: &str) -> Option<u64> {
        if self.map.is_none() {
            return None;
        }
        let val = self.map.as_ref().unwrap().get(key);
        return val;
    }

    fn open_searcher(&mut self, path: &str) -> std::io::Result<()> {
        self.map = Some(Map::new(std::fs::read(path).unwrap()).unwrap());
        return Ok(());
    }

    fn create_searcher(&mut self, pages: &Vec<Page>, output_path: &str) -> std::io::Result<()> {
        println!("Creating Searcher");
        let mut key_val_tuple = pages
            .iter()
            .map(|page| {
                let key = page.title.as_bytes();
                let block_id = page.block_id.unwrap() as u64;
                let page_id = page.id as u64;

                // Store block_id and page_id (u32) in a u64
                let value = (block_id << 32) | page_id;
                (key, value)
            })
            .collect::<Vec<_>>();

        key_val_tuple.sort_by(|a, b| a.0.cmp(b.0));
        let mut wtr = BufWriter::new(File::create(output_path).unwrap());
        let mut build = MapBuilder::new(&mut wtr).unwrap();

        let mut set = std::collections::HashSet::new();

        for (key, value) in key_val_tuple.iter() {
            if set.contains(key) {
                continue;
            }
            set.insert(key);
            build.insert(key, *value).unwrap();
        }

        build.finish().unwrap();
        let map = Map::new(std::fs::read(output_path).unwrap()).unwrap();

        self.map = Some(map);
        return Ok(());
    }

    fn search(&self, query: &str) -> std::io::Result<Vec<(String, u64)>> {
        if self.map.is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Searcher data not initialised",
            ));
        }

        let matcher = Str::new(query);

        let mut results: Vec<(String, u64)> = Vec::new();

        let mut stream = self
            .map
            .as_ref()
            .unwrap()
            .search_with_state(&matcher)
            .into_stream();

        while let Some((k, v, _s)) = stream.next() {
            let datstr = String::from_utf8(k.to_vec()).unwrap();
            results.push((datstr, v));
        }

        return Ok(results);
    }
}
