// Standard Lib
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

// Third Party
use serde_json;

pub mod bzip;
pub mod page;
pub mod search;
use crate::bzip::{create_bz_table, indexing_bzip_blocks, BZipTable};
use crate::page::Page;
use crate::search::{Searchable, Searcher};

// In prod we don't want to store the intermediate pages used which use up a
// lot of memory
static DEBUG_SAVE_PAGES: bool = true;

fn create_directory_if_not_exists(path: &str) {
    let path = path.replace("~", std::env::var("HOME").unwrap().as_str());
    if !std::path::Path::new(&path).exists() {
        std::fs::create_dir_all(path).unwrap();
    }
}

pub fn initial_indexing(input_bz_path: PathBuf, meta_path: PathBuf) -> std::io::Result<()> {
    // Create meta directory if doesn't exist
    create_directory_if_not_exists(meta_path.to_str().unwrap());

    // Index bzip blocks
    let f = File::open(&input_bz_path.to_str().unwrap()).expect("No bzip file found");

    println!("Indexing bzip blocks");
    let mut reader = BufReader::new(f);

    let output_bzip_path = meta_path.join("table.json");
    let table: BZipTable =
        create_bz_table(&mut reader, output_bzip_path.to_str().unwrap()).unwrap();
    for i in 0..table.length {
        println!("{}: {:?}", i, table.blocks[i]);
    }
    //

    let table: BZipTable =
        serde_json::de::from_reader(File::open(meta_path.join("table.json")).unwrap()).unwrap();

    println!("Indexing pages in blocks");

    // Might be a bit memory hungry
    let pages: Vec<Page> = indexing_bzip_blocks(&table, &input_bz_path).unwrap();

    if DEBUG_SAVE_PAGES {
        let _ = serde_json::to_writer(File::create(meta_path.join("pages.json")).unwrap(), &pages);
    }

    let output_searcher = meta_path.join("map.index");
    let mut searcher = Searcher::new();
    searcher
        .create_searcher(&pages, output_searcher.to_str().unwrap())
        .unwrap();
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder() {}
}
