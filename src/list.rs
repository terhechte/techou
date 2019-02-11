use serde::Serialize;
use serde_derive::Serialize;
use std::collections::BTreeMap;

use crate::config::Config;
use crate::document::Document;
use rayon::prelude::*;

#[derive(Serialize, Debug)]
pub struct Year<'a> {
    pub name: i32,
    pub months: Vec<Month<'a>>,
}

#[derive(Serialize, Debug)]
pub struct Month<'a> {
    pub name: u32,
    pub posts: Vec<&'a Document>,
}

#[derive(Serialize, Debug)]
pub struct Tag<'a> {
    pub name: &'a str,
    pub count: u32,
    pub posts: Vec<&'a Document>,
}

impl<'a> From<(i32, Vec<Month<'a>>)> for Year<'a> {
    fn from(entry: (i32, Vec<Month<'a>>)) -> Self {
        Year {
            name: entry.0,
            months: entry.1,
        }
    }
}

impl<'a> From<(u32, Vec<&'a Document>)> for Month<'a> {
    fn from(entry: (u32, Vec<&'a Document>)) -> Self {
        Month {
            name: entry.0,
            posts: entry.1,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct DocumentContext<'a> {
    pub pages: &'a Vec<Document>,
    pub posts: &'a Vec<Document>,
    pub by_date: &'a Vec<Year<'a>>,
    pub by_tag: &'a Vec<Tag<'a>>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Page {
    pub index: usize,
    pub title: String,
    pub items: usize,
    pub path: String,
}

#[derive(Serialize, Debug, Default)]
pub struct Pagination {
    pub current: usize,
    pub next: Option<Page>,
    pub previous: Option<Page>,
}

#[derive(Serialize, Debug)]
pub struct List<'a, D: AsRef<Document>>
where
    D: Serialize,
{
    pub title: &'a str,
    pub posts: &'a [D],
    pub pagination: Pagination,
}

impl<'a, D: AsRef<Document>> List<'a, D>
where
    D: Serialize,
{
    pub fn index(title: &'a str, posts: &'a [D]) -> Self {
        List {
            title: title,
            posts: posts,
            pagination: Default::default(),
        }
    }
}
