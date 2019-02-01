use serde_derive::{Serialize};
use std::collections::BTreeMap;

use crate::article::Article;
use crate::config::Config;

#[derive(Serialize, Debug)]
pub struct Year<'a> {
    pub name: i32,
    pub months: Vec<Month<'a>>
}

#[derive(Serialize, Debug)]
pub struct Month<'a> {
    pub name: u32,
    pub articles: Vec<&'a Article>
}

impl<'a> From<(i32, Vec<Month<'a>>)> for Year<'a> {
    fn from(entry: (i32, Vec<Month<'a>>)) -> Self {
        Year {
            name: entry.0,
            months: entry.1
        }
    }
}

impl<'a> From<(u32, Vec<&'a Article>)> for Month<'a> {
    fn from(entry: (u32, Vec<&'a Article>)) -> Self {
        Month {
            name: entry.0,
            articles: entry.1
        }
    }
}

#[derive(Serialize, Debug)]
pub struct List<'a> {
    pub title: String,
    pub articles: &'a Vec<Article>,
    pub articles_by_date: Vec<Year<'a>>
}

pub fn articles_by_date<'a>(articles: &'a Vec<Article>) -> Vec<Year<'a>> {
    let mut date_map: BTreeMap<i32, BTreeMap<u32, Vec<&'a Article>>> = BTreeMap::new();
    for article in articles {
        let mut year = date_map.entry(article.info.date_info.year).or_insert( BTreeMap::new(), );
        let mut month = year.entry(article.info.date_info.month).or_insert(Vec::new());
        month.push(article);
    }
    date_map.into_iter().rev().map(|(year, entries)| {
        Year::from((year, entries.into_iter().rev().map(|m| Month::from(m)).collect()))
    }).collect()
}