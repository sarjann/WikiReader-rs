// Standard Lib
use std::fs::File;
use std::io::BufWriter;

// Third Party
use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Map, MapBuilder};
use regex::Regex;

// Local
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
        searcher
    }

    fn len(&self) -> usize {
        if self.map.is_none() {
            return 0;
        }
        let len = self.map.as_ref().unwrap().len();
        len
    }

    fn get(&self, key: &str) -> Option<u64> {
        if self.map.is_none() {
            return None;
        }
        let val = self.map.as_ref().unwrap().get(key);
        val
    }

    fn open_searcher(&mut self, path: &str) -> std::io::Result<()> {
        self.map = Some(Map::new(std::fs::read(path).unwrap()).unwrap());
        Ok(())
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
        Ok(())
    }

    fn search(&self, query: &str) -> std::io::Result<Vec<(String, u64)>> {
        let pattern_contains = format!(r"(?i){query}");
        let pattern_identical = format!(r"(?i)^{query}$");
        let re_contains = Regex::new(&pattern_contains).unwrap();
        let re_identical = Regex::new(&pattern_identical).unwrap();

        let Some(map) = &self.map else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Searcher data not initialised",
            ));
        };

        let matcher = Levenshtein::new(query, 1).unwrap();
        let matched = map.search(&matcher).into_stream().into_str_vec().unwrap();
        let mut results: Vec<(String, u64)> = Vec::new();
        matched.iter().for_each(|(k, v)| {
            if re_identical.is_match(k) {
                results.insert(0, (k.clone(), v.clone()));
            } else if re_contains.is_match(k) {
                results.push((k.clone(), v.clone()));
            }
        });
        return Ok(results);
    }
}
