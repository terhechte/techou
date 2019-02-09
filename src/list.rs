use serde_derive::{Serialize};
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

/*
what I want the template to have: index
global:
    - config
    - all articles
    - all pages
    - all tags (+ the articles for the tags)
    - the articles-by-year
index:
- title
- articles
- next page + title
- previous page + title
*/

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
pub struct List<'a> {
    pub title: &'a str,
    pub posts: &'a [Document],
    pub pagination: Pagination
}

impl<'a> List<'a> {
    pub fn index(title: &'a str, posts: &'a [Document]) -> Self {
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
        let mut year = date_map.entry(post.info.date_info.year).or_insert( BTreeMap::new(), );
        let mut month = year.entry(post.info.date_info.month).or_insert(Vec::new());
        month.push(post);
    }
    date_map.into_iter().rev().map(|(year, entries)| {
        Year::from((year, entries.into_iter().rev().map(|m| Month::from(m)).collect()))
    }).collect()
}
