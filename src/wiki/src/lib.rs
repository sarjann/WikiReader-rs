use bzip2::read::MultiBzDecoder;
use std::fmt::Debug;
use std::fs::{read_to_string, File};
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, Cursor, SeekFrom};

//
use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Map, MapBuilder, Set, SetBuilder, Streamer};
use quick_xml;
use quick_xml::{events::Event, Reader};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use serde::de;
//

// pub struct Source {
//     path: String,
// }
//
// impl Source {
//     pub fn new(path: &str) -> Source {
//         Source {
//             path: path.to_string(),
//         }
//     }
// }

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
struct Redirect {
    #[serde(rename = "@title")]
    title: String,
}

#[derive(Deserialize, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct Revision {
    id: u32,
    parentid: Option<u32>,
    timestamp: String,
    format: Option<String>,
    model: String,
}

#[derive(Deserialize, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct Page {
    title: String,
    ns: u32,
    id: u32,
    block_id: Option<usize>,
    redirect: Option<Redirect>,
    revision: Option<Revision>,
}

#[derive(Deserialize, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct DetailedPage {
    title: String,
    ns: u32,
    id: u32,
    block_id: Option<usize>,
    redirect: Option<Redirect>,
    revision: Option<Revision>,
    text: String,
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

pub fn open_fst() -> std::io::Result<Map<Vec<u8>>> {
    let fst_path = format!("/home/s/Documents/wiki/meta/big.fst");
    let map: Map<Vec<u8>> = Map::new(std::fs::read(fst_path).unwrap()).unwrap();
    return Ok(map);
}

pub fn create_fst(pages: &Vec<Page>) -> Option<Map<Vec<u8>>> {
    let fst_path = format!("/home/s/Documents/wiki/meta/big.fst");
    // let map: Map<Vec<u8>> = Map::new(std::fs::read(fst_path).unwrap()).unwrap();

    let mut key_val_tuple = pages
        .into_par_iter()
        .map(|page| {
            let key = page.title.as_bytes();
            let value = page.id;
            (key, value)
        })
        .collect::<Vec<_>>();

    key_val_tuple.sort_by(|a, b| a.0.cmp(b.0));
    let mut wtr = io::BufWriter::new(File::create("map.fst").unwrap());
    let mut build = MapBuilder::memory();
    
    for (key, value) in key_val_tuple.iter() {
        build.insert(key, *value as u64).unwrap();
    }
    build.finish().unwrap();

    let bytes = build.into_inner().unwrap();

    let map = Map::new(bytes).unwrap();
    return Some(map);
}

pub fn search(set: &Set<Vec<u8>>, query: &str) -> std::io::Result<Vec<String>> {
    let lev = Levenshtein::new(query, 2).unwrap();
    let mut results: Vec<String> = Vec::new();

    let mut stream = set.search_with_state(lev).into_stream();
    while let Some((v, _s)) = stream.next() {
        let datstr = String::from_utf8(v.to_vec()).unwrap();
        results.push(datstr);
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

fn use_bzip_block(table: &BZipTable, buffer: &[u8], index: usize) -> Option<Vec<Page>> {
    let size = table.blocks[index].size;
    let offset = table.blocks[index].offset;

    let decoder = MultiBzDecoder::new(buffer);

    let mut reader = BufReader::new(decoder);
    let mut parser: std::result::IntoIter<Vec<Page>> =
        quick_xml::de::from_reader(&mut reader).into_iter();

    let pages = match parser.next() {
        Some(p) => Some(p),
        _ => None,
    };
    return pages;
}

fn use_bzip_block_n(table: &BZipTable, reader: &mut BufReader<File>, block_id: usize) -> Option<Vec<DetailedPage>> {
    let size = table.blocks[block_id].size;
    let offset = table.blocks[block_id].offset;
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

fn thing() {
    // let input_path = "/home/s/Documents/wiki/enwiki-20230820-pages-articles-multistream.xml.bz2";
    let input_path = "/home/s/Documents/wiki/simplewiki-20230820-pages-articles-multistream.xml.bz2";

    let f = File::open(input_path).unwrap();
    let mut reader = BufReader::new(f);

    let table: BZipTable = create_bz_table(&mut reader).unwrap();
    for i in 0..table.length {
        println!("{}: {:?}", i, table.blocks[i]);
    }

    println!("-bzip use_block");
    //
    let f = File::open(input_path).unwrap();
    let mut reader = BufReader::new(f);

    let pages: Vec<Page> = indexing_bzip_blocks(&table, &mut reader).unwrap();
}

fn get_detailed_page(table: &BZipTable, page: &Page, reader: &mut BufReader<File>) -> Option<Page> {
    let block_id = page.block_id.unwrap();

    let block_pages = use_bzip_block_n(&table, reader, block_id as usize);
    for block_page in block_pages.unwrap() {
        if page.id == block_page.id {
            return Some(block_page);
        } 
    }
    return None;
}
fn indexing_bzip_blocks(
    table: &BZipTable,
    reader: &mut BufReader<File>,
) -> std::io::Result<Vec<Page>> {
    let block_count = table.length;
    let mut pages: Vec<Page> = Vec::new();
    let interval = 1000;

    println!("Block Count: {}", block_count);
    let mut previous_time = std::time::Instant::now();

    let (sender, receiver) = std::sync::mpsc::channel();

    println!("got to the block bit");
    (1..block_count - 1)
        .into_par_iter()
        .for_each_with(sender, |s, i| {
            let offset = table.blocks[i].offset;
            let size = table.blocks[i].size;

            // let input_path =
            //     "/home/s/Documents/wiki/enwiki-20230820-pages-articles-multistream.xml.bz2";
            let input_path = "/home/s/Documents/wiki/simplewiki-20230820-pages-articles-multistream.xml.bz2";

            let f = File::open(input_path).unwrap();
            let mut reader = BufReader::new(f);

            reader.seek(SeekFrom::Start(offset as u64)).unwrap();
            let mut reader = reader.take(size as u64);
            let buf = reader.fill_buf().unwrap();

            let pages_block = use_bzip_block(&table, buf, i);
            if pages_block.is_none() {
                return ();
                // return Vec::new();
            }
            let mut pages = pages_block.unwrap();
            for page in pages.iter_mut() {
                page.block_id = Some(i);
            }

            s.send(pages).unwrap()
            // pages
        });

    let pages: Vec<Page> = receiver
        .iter()
        .collect::<Vec<Vec<Page>>>()
        .into_iter()
        .flatten()
        .collect();
    
    println!("pages_len: {}", pages.len());
    // Write pages
    // let path_output = "/home/s/Documents/wiki/meta/pages_w.json";
    let path_output = "/home/s/Documents/wiki/meta/pages_n.json";
    let _ = serde_json::ser::to_writer(File::create(path_output).unwrap(), &pages);

    // pages.sort();
    // let vec: Vec<Page> = par_iter.collect();
    // for i in 1..block_count - 1 {
    //     if i % interval== 0 {
    //         print!(
    //             "\rCount: {}k / {}k, Percentage {}%, ETA: {}m",
    //             i as f32 / 1000.0,
    //             (i / block_count) * 100,
    //             block_count as f32 / 1000.0,
    //             (std::time::Instant::now() - previous_time).as_secs()
    //                 * (block_count - i) as u64
    //                 / (interval as u64)*60
    //         );
    //         previous_time = std::time::Instant::now();
    //         std::io::stdout().flush().unwrap();
    //     }
    //
    //     let pages_block = use_bzip_block(&table, reader, i);
    //     if pages_block.is_none() {
    //         continue;
    //     }
    //     for mut page in pages_block.unwrap() {
    //         page.block_id = Some(i as u32);
    //         pages.push(page);
    //     }
    // }
    println!("pages_len: {}", pages.len());
    return Ok(pages);
}

fn create_bz_table(reader: &mut BufReader<File>) -> std::io::Result<BZipTable> {
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

    let path_output = "/home/s/Documents/wiki/meta/bzip_table.json";
    let _ = serde_json::ser::to_writer(File::create(path_output).unwrap(), &table);
    return Ok(table);
}

pub fn debugging() {
    println!("-bzip get_block");
    // let input_path =
    //     "/home/s/Documents/wiki/simplewiki-20230820-pages-articles-multistream.xml.bz2";
    let input_path = "/home/s/Documents/wiki/enwiki-20230820-pages-articles-multistream.xml.bz2";

    let f = File::open(input_path).unwrap();
    let mut reader = BufReader::new(f);

    let table: BZipTable = create_bz_table(&mut reader).unwrap();
    for i in 0..table.length {
        println!("{}: {:?}", i, table.blocks[i]);
    }

    println!("-bzip use_block");
    //
    let f = File::open(input_path).unwrap();
    let mut reader = BufReader::new(f);
    // let _ = use_bzip_block(&table, &mut reader, 1);
    let _ = indexing_bzip_blocks(&table, &mut reader);
}

pub fn load() {
    // let path_output = "/home/s/Documents/wiki/meta/bzip_table.json";
    // let table: BZipTable = serde_json::de::from_reader(
    //     File::open(path_output).unwrap(),
    // ).unwrap();
    //
    // let input_path = "/home/s/Documents/wiki/enwiki-20230820-pages-articles-multistream.xml.bz2";
    // let pages: Vec<Page> = indexing_bzip_blocks(&table, &mut BufReader::new(File::open(input_path).unwrap())).unwrap();
    // let path_output = "/home/s/Documents/wiki/meta/pages.json";
    // let _ = serde_json::ser::to_writer(
    //     File::create(path_output).unwrap(),
    //     &pages,
    // );
    // println!("table: {:?}", table);

    // let input_path = "/home/s/Documents/wiki/meta/pages.json";
    // let pages: Pages = serde_json::de::from_reader(File::open(input_path).unwrap()).unwrap();
    //
    // serde_cbor::ser::to_writer(
    //     File::create("/home/s/Documents/wiki/meta/pages.cbor").unwrap(),
    //     &pages,
    // )
    // .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn other() {
        println!("thing");
        thing();
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
