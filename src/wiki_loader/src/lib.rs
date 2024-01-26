
// Standard Lib
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::path::{Path, PathBuf};

// Third Party
use bzip2::read::MultiBzDecoder;
use quick_xml;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;

pub mod page;
pub mod search;
use crate::page::{DetailedPage, Page, SiteInfo};
use crate::search::{Searcher, Searchable};

fn create_directory_if_not_exists(path: &str) {
    let path = path.replace("~", std::env::var("HOME").unwrap().as_str());
    if !std::path::Path::new(&path).exists() {
        std::fs::create_dir_all(path).unwrap();
    }
}

pub fn open_bz_table(path: &str) -> std::io::Result<BZipTable> {
    let bztable = serde_json::de::from_reader(File::open(path).unwrap()).unwrap();
    return Ok(bztable);
}

#[derive(Serialize, Deserialize, Debug)]
struct BZipBlock {
    offset: usize,
    size: usize,
}

#[derive(Serialize, Deserialize)]
pub struct BZipTable {
    blocks: Vec<BZipBlock>,
    pub length: usize,
}

impl std::fmt::Debug for BZipTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BZipTable")
            .field("blocks", &self.blocks)
            .field("length", &self.length)
            .finish()
    }
}

fn use_bzip_block_n_non_detailed(
    table: &BZipTable,
    path: &Path,
    block_id: usize,
) -> Option<Vec<Page>> {
    let size = table.blocks[block_id].size;
    let offset = table.blocks[block_id].offset;
    let mut reader = BufReader::new(File::open(path).unwrap());
    reader.seek(SeekFrom::Start(offset as u64)).unwrap();
    let mut reader = reader.take(size as u64);

    let decoder = MultiBzDecoder::new(&mut reader);

    let mut output_reader = BufReader::new(decoder);

    let mut parser: std::result::IntoIter<Vec<Page>> =
        quick_xml::de::from_reader(&mut output_reader).into_iter();

    let pages = match parser.next() {
        Some(p) => Some(p),
        _ => None,
    };
    return pages;
}

fn use_bzip_block_n_detailed(
    table: &BZipTable,
    path: &Path,
    block_id: usize,
) -> Option<Vec<DetailedPage>> {
    let size = table.blocks[block_id].size;
    let offset = table.blocks[block_id].offset;
    let mut reader = BufReader::new(File::open(path).unwrap());
    reader.seek(SeekFrom::Start(offset as u64)).unwrap();
    let mut reader = reader.take(size as u64);

    let decoder = MultiBzDecoder::new(&mut reader);

    let mut output_reader = BufReader::new(decoder);

    let mut parser: std::result::IntoIter<Vec<DetailedPage>> =
        quick_xml::de::from_reader(&mut output_reader).into_iter();

    let pages = match parser.next() {
        Some(p) => Some(p),
        _ => None,
    };
    return pages;
}

pub fn get_detailed_page(
    table: &BZipTable,
    page_id: u64,
    block_id: u64,
    path: &Path,
) -> Option<DetailedPage> {
    let pages_block = use_bzip_block_n_detailed(&table, path, block_id as usize);

    let mut pages = pages_block.unwrap();

    let mut selected_id: Option<usize> = None;
    for (index, page) in pages.iter().enumerate() {
        if page.id == page_id as u32 {
            selected_id = Some(index);
        }
    }
    return match selected_id {
        Some(id) => Some(pages.remove(id)),
        None => None,
    };
}

fn indexing_bzip_blocks(table: &BZipTable, path: &Path) -> std::io::Result<Vec<Page>> {
    let block_count = table.length;

    println!("Block Count: {}", block_count);

    let (sender, receiver) = std::sync::mpsc::channel();

    println!("Indexing pages in blocks");
    (1..block_count - 1)
        .into_par_iter()
        .for_each_with(sender, |s, i| {
            let pages_block = use_bzip_block_n_non_detailed(&table, &path, i);
            if pages_block.is_none() {
                return ();
            }
            let mut pages = pages_block.unwrap();
            for page in pages.iter_mut() {
                page.block_id = Some(i);
            }

            s.send(pages).unwrap()
        });

    let pages: Vec<Page> = receiver
        .iter()
        .collect::<Vec<Vec<Page>>>()
        .into_iter()
        .flatten()
        .collect();

    println!("Page Count: {}", pages.len());
    return Ok(pages);
}

pub fn create_bz_table(
    reader: &mut BufReader<File>,
    output_path: &str,
) -> std::io::Result<BZipTable> {
    let mut offsets: Vec<usize> = Vec::new();
    let mut count = 0;
    // Magic number in bzip
    // PI
    // let bz_sub = [49, 65, 89, 38, 83, 89];
    // let count_offset = 9;
    // BZ + h9
    let bz_sub = [66, 90, 104, 57];
    let count_offset = 3;
    let length_bz_sub = bz_sub.len();
    let mut bytes = reader.bytes();
    let mut search_buffer: Vec<u8> = vec![0; length_bz_sub];

    loop {
        let byte = bytes.next();
        if byte.is_none() {
            break;
        }
        let byte = byte.unwrap().unwrap();
        search_buffer.remove(0);
        search_buffer.push(byte);
        if search_buffer == bz_sub && count > count_offset {
            offsets.push(count - count_offset);
        }
        count += 1;
    }

    let mut sizes = offsets.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>();
    sizes.push(count - offsets[offsets.len() - 1]);

    let mut blocks: Vec<BZipBlock> = Vec::new();
    let length = offsets.len();
    for i in 0..length {
        let table = BZipBlock {
            offset: offsets[i],
            size: sizes[i],
        };
        blocks.push(table);
    }
    let table = BZipTable { blocks, length };

    let _ = serde_json::ser::to_writer(File::create(output_path).unwrap(), &table);
    return Ok(table);
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

    println!("Indeing pages in blocks");

    // Might be a bit memory hungry
    let pages: Vec<Page> = indexing_bzip_blocks(&table, &input_bz_path).unwrap();

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
