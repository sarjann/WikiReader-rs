use bzip2::read::MultiBzDecoder;
use std::fmt::Debug;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::path::Path;

//
use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Map, MapBuilder, Streamer};
use quick_xml;
use rayon::prelude::*;
use serde::de;
use serde::{Deserialize, Serialize};
use serde_json;
//

fn create_directory_if_not_exists(path: &str) {
    let path = path.replace("~", std::env::var("HOME").unwrap().as_str());
    if !std::path::Path::new(&path).exists() {
        std::fs::create_dir_all(path).unwrap();
    }
}

#[derive(Deserialize, Debug, Serialize)]
struct NameSpace {
    #[serde(rename = "@key")]
    key: String,
    #[serde(rename = "@case")]
    case: String,
    #[serde(rename = "$value")]
    value: Option<String>,
}

#[derive(Deserialize, Debug, Serialize)]
struct NameSpaces {
    namespace: Vec<NameSpace>,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename(serialize = "siteinfo"))]
struct SiteInfo {
    sitename: String,
    dbname: String,
    base: String,
    generator: String,
    case: String,
    namespaces: Vec<NameSpaces>,
}

#[derive(Deserialize, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Redirect {
    #[serde(rename = "@title")]
    pub title: String,
}

#[derive(Deserialize, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Text {
    #[serde(rename = "@bytes")]
    pub bytes: u32,
    #[serde(rename = "@xml:space")]
    pub xml_space: Option<String>,
    #[serde(rename = "$value")]
    pub value: Option<String>,
}

#[derive(Deserialize, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RevisionDetailedPage {
    pub id: u32,
    pub parentid: Option<u32>,
    pub timestamp: String,
    pub format: Option<String>,
    pub model: String,
    pub text: Option<Text>,
}

#[derive(Deserialize, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RevisionPage {
    pub id: u32,
    pub parentid: Option<u32>,
    pub timestamp: String,
    pub format: Option<String>,
    pub model: String,
}

#[derive(Deserialize, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    pub title: String,
    pub ns: u32,
    pub id: u32,
    pub block_id: Option<usize>,
    pub redirect: Option<Redirect>,
    pub revision: Option<RevisionPage>,
}

#[derive(Deserialize, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct DetailedPage {
    pub title: String,
    pub ns: u32,
    pub id: u32,
    pub block_id: Option<usize>,
    pub redirect: Option<Redirect>,
    pub revision: Option<RevisionDetailedPage>,
}

trait PageItem {}
impl PageItem for Page {}
impl PageItem for DetailedPage {}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename(serialize = "mediawiki"))]
struct MediaWiki {
    siteinfo: SiteInfo,
    page: Vec<Page>,
}

pub fn open_fst(path: &str) -> std::io::Result<Map<Vec<u8>>> {
    let map: Map<Vec<u8>> = Map::new(std::fs::read(path).unwrap()).unwrap();
    return Ok(map);
}

pub fn open_bz_table(path: &str) -> std::io::Result<BZipTable> {
    let bztable = serde_json::de::from_reader(File::open(path).unwrap()).unwrap();
    return Ok(bztable);
}

pub fn create_fst(pages: &Vec<Page>, output_path: &str) -> Option<Map<Vec<u8>>> {
    println!("Creating FST");
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
    let mut wtr = io::BufWriter::new(File::create(output_path).unwrap());
    let mut build = MapBuilder::new(&mut wtr).unwrap();

    for (key, value) in key_val_tuple.iter() {
        build.insert(key, *value).unwrap();
    }

    build.finish().unwrap();
    let map = Map::new(std::fs::read(output_path).unwrap()).unwrap();
    return Some(map);
}

pub fn search(map: &Map<Vec<u8>>, query: &str) -> std::io::Result<Vec<(String, u64)>> {
    let lev = Levenshtein::new(query, 2).unwrap();
    let mut results: Vec<(String, u64)> = Vec::new();

    let mut stream = map.search_with_state(lev).into_stream();
    while let Some((k, v, _s)) = stream.next() {
        let datstr = String::from_utf8(k.to_vec()).unwrap();
        results.push((datstr, v));
    }
    return Ok(results);
}

#[derive(Serialize, Deserialize, Debug)]
struct BZipBlock {
    offset: usize,
    size: usize,
}

#[derive(Serialize, Deserialize)]
pub struct BZipTable {
    blocks: Vec<BZipBlock>,
    length: usize,
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

    // panic!("Block ID: {}, Page ID: {}", block_id, page_id);
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

fn indexing_bzip_blocks(
    table: &BZipTable,
    path: &Path,
    output_path: &str,
) -> std::io::Result<Vec<Page>> {
    let block_count = table.length;

    println!("Block Count: {}", block_count);

    let (sender, receiver) = std::sync::mpsc::channel();

    println!("got to the block bit");
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

    println!("pages_len: {}", pages.len());

    // Write pages
    let _ = serde_json::ser::to_writer(File::create(output_path).unwrap(), &pages);

    println!("pages_len: {}", pages.len());
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

#[cfg(test)]
mod tests {
    use super::*;

    fn thing() {
        let base_path = Path::new("/home/s/Documents/wiki/simple/");
        // let base_path = Path::new("/home/s/Documents/wiki/bigger/");
        // let input_path =
        //     "/home/s/Documents/wiki/simple/meta/simplewiki-20230820-pages-articles-multistream.xml.bz2";

        let input_bz2_path = base_path.join("base.bz2");

        //// Index
        // let f = File::open(&input_bz2_path).unwrap();
        //
        // let mut reader = BufReader::new(f);
        //
        // let output_bzip_path = base_path.join("meta/bzip_table.json");
        // let table: BZipTable =
        //     create_bz_table(&mut reader, output_bzip_path.to_str().unwrap()).unwrap();
        // for i in 0..table.length {
        //     println!("{}: {:?}", i, table.blocks[i]);
        // }
        ////

        let table: BZipTable = serde_json::de::from_reader(
            File::open(base_path.join("meta/bzip_table.json")).unwrap(),
        )
        .unwrap();

        println!("-bzip use_block");

        // let f = File::open(&input_bz2_path).unwrap();
        // let mut reader = BufReader::new(f);

        let index_path = base_path.join("meta/indexed");
        // let pages: Vec<Page> =
        //     indexing_bzip_blocks(&table, &input_bz2_path, index_path.to_str().unwrap()).unwrap();

        let pages: Vec<Page> = serde_json::from_reader(File::open(index_path).unwrap()).unwrap();
        println!("Page Len {}", pages.len());

        let output_fst = base_path.join("meta/map.fst");
        let map = create_fst(&pages, &output_fst.to_str().unwrap()).unwrap();
        // let fst = open_fst("/home/s/Documents/wiki/meta/big.fst").unwrap();
    }

    fn thing2() {
        let base_path = Path::new("/home/s/Documents/wiki/simple/");
        let input_bz2_path = base_path.join("base.bz2");

        let table: BZipTable = serde_json::de::from_reader(
            File::open(base_path.join("meta/bzip_table.json")).unwrap(),
        )
        .unwrap();

        println!("-bzip use_block");
        let page = get_detailed_page(&table, 47955, 228, &input_bz2_path).unwrap();
        println!("Page: {:?}", page);
    }

    #[test]
    fn other() {
        thing2();
    }

    // #[test]
    // fn load_file() {
    //     println!("test load");
    //     load();
    // }

    // #[test]
    // fn run() {
    //     debugging();
    // }
}
