use serde_derive::{Serialize};
use serde::Serialize;
use std::collections::BTreeMap;

use crate::document::Document;
use crate::config::Config;
use rayon::prelude::*;

#[derive(Serialize, Debug)]
pub struct Year<'a> {
    pub name: i32,
    pub months: Vec<Month<'a>>
}

#[derive(Serialize, Debug)]
pub struct Month<'a> {
    pub name: u32,
    pub posts: Vec<&'a Document>
}

#[derive(Serialize, Debug)]
pub struct Tag<'a> {
    pub name: &'a str,
    pub count: u32,
    pub posts: Vec<&'a Document>
}

impl<'a> From<(i32, Vec<Month<'a>>)> for Year<'a> {
    fn from(entry: (i32, Vec<Month<'a>>)) -> Self {
        Year {
            name: entry.0,
            months: entry.1
        }
    }
}

impl<'a> From<(u32, Vec<&'a Document>)> for Month<'a> {
    fn from(entry: (u32, Vec<&'a Document>)) -> Self {
        Month {
            name: entry.0,
            posts: entry.1
        }
    }
}

#[derive(Serialize, Debug)]
pub struct DocumentContext<'a> {
    pub pages: &'a Vec<Document>,
    pub posts: &'a Vec<Document>,
    pub posts_by_date: Vec<Year<'a>>
}

#[derive(Serialize, Debug)]
pub struct TemplateContext<'a, T> {
    config: &'a Config,
    posts: DocumentContext<'a>,
    content: &'a T,
}

#[derive(Serialize, Debug, Clone)]
pub struct Page {
    pub index: usize,
    pub title: String,
    pub items: usize,
    pub path: String
}

#[derive(Serialize, Debug, Default)]
pub struct Pagination {
    pub current: usize,
    pub next: Option<Page>,
    pub previous: Option<Page>
}

#[derive(Serialize, Debug)]
pub struct List<'a, D: AsRef<Document>>
where D: Serialize
{
    pub title: &'a str,
    pub posts: &'a [D],
    pub pagination: Pagination
}

impl<'a, D: AsRef<Document>> List<'a, D>
where D: Serialize
{
    pub fn index(title: &'a str, posts: &'a [D]) -> Self {
        List {
            title: title,
            posts: posts,
            pagination: Default::default()
        }
    }
}

pub fn posts_by_date<'a>(posts: &'a Vec<Document>) -> Vec<Year<'a>> {
    let mut date_map: BTreeMap<i32, BTreeMap<u32, Vec<&'a Document>>> = BTreeMap::new();
    for post in posts {
        let mut year = date_map.entry(post.info.date_info.year)
            .or_insert( BTreeMap::new(), );
        let mut month = year.entry(post.info.date_info.month)
            .or_insert(Vec::new());
        month.push(post);
    }
    date_map.into_iter().rev().map(|(year, entries)| {
        Year::from((year, entries.into_iter().rev()
            .map(|m| Month::from(m)).collect()))
    }).collect()
}

pub fn posts_by_tag<'a, D: AsRef<Document>>(posts: &'a Vec<D>) -> Vec<Tag<'a>> {
    let mut tag_map: BTreeMap<&'a str, Vec<&'a Document>> = BTreeMap::new();
    for post in posts {
        let post = post.as_ref();
        for tag in &post.info.tags {
            let mut tags = tag_map.entry(&tag)
                .or_insert(Vec::new());
            tags.push(post);
        }
    }
    tag_map.into_iter().rev().map(|(tag, entries)| {
        Tag {
            name: tag,
            count: entries.len() as u32,
            posts: entries
        }
    }).collect()
}

pub fn documents_by_similarity<'a>(to_document: &'a Document, in_documents: &'a Vec<Document>, nr: u32) -> Vec<&'a Document> {
    // sort by similarity index
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use strsim::normalized_damerau_levenshtein;
    let mut items = HashSet::new();
    let max_tags: usize = 10; // if we have more than 10 matches, we have a winner
    for tag in &to_document.info.tags {
        items.insert(tag);
    }
    let mut sorted: Vec<(u32, &Document)> = in_documents.into_par_iter().map(|item| {
        let levdn = normalized_damerau_levenshtein(&item.info.title, &to_document.info.title);
        let tagd = HashSet::from_iter(item.info.tags.iter()).intersection(&items).count();
        let tagdn = ::std::cmp::max(max_tags, tagd) as f64 / max_tags as f64;
        (((levdn + tagdn) * 100.0) as u32, item)
    }).collect();
    sorted.sort_by_key(|k| k.0);
    sorted.into_iter().map(|(_, d)|d).collect()
}
