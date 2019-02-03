use serde_derive::{Serialize};
use std::collections::BTreeMap;

use crate::document::Document;
use crate::config::Config;

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

#[derive(Serialize, Debug)]
pub struct List<'a> {
    pub title: String,
    pub posts: &'a Vec<Document>,
    pub posts_by_date: Vec<Year<'a>>,
    pub pages: &'a Vec<Document>
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
