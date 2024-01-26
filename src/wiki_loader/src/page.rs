// Standard Lib
use std::fmt::{Debug, Display};
use std::path::Path;

// Third Party
use serde::{Deserialize, Serialize};

use crate::bzip::{use_bzip_block_n_detailed, BZipTable};

#[derive(Deserialize, Debug, Serialize)]
pub struct NameSpace {
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
pub struct SiteInfo {
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
    pub id: u32,
    pub block_id: Option<usize>,
    // Shouldn't be needed for indexing so saving memory
    // pub ns: u32,
    // pub redirect: Option<Redirect>,
    // pub revision: Option<RevisionPage>,
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

impl Display for DetailedPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = format!("Title:{},ns:{},id:{}", self.title, self.ns, self.id);
        out = match &self.redirect {
            Some(redirect) => format!("{},redirect:{}", out, redirect.title),
            None => out,
        };
        return write!(f, "{}", out);
    }
}

trait PageItem {}
impl PageItem for Page {}
impl PageItem for DetailedPage {}

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
