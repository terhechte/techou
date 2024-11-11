use serde::Serialize;

use crate::book::Book;
use crate::document::Document;

#[derive(Serialize, Debug)]
pub struct Year<'a> {
    pub name: i32,
    pub months: Vec<Month<'a>>,
    pub count: i32,
}

#[derive(Serialize, Debug)]
pub struct Month<'a> {
    pub name: u32,
    pub posts: Vec<&'a Document>,
}

#[derive(Serialize, Debug)]
pub struct Category<'a> {
    pub name: &'a str,
    pub count: u32,
    pub posts: Vec<&'a Document>,
}

impl<'a> From<(i32, Vec<Month<'a>>)> for Year<'a> {
    fn from(entry: (i32, Vec<Month<'a>>)) -> Self {
        Year {
            name: entry.0,
            count: entry.1.iter().map(|e| e.posts.len() as i32).sum(),
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
    pub all_posts: &'a Vec<&'a Document>,
    pub books: &'a Vec<Book>,
    pub by_date: &'a Vec<Year<'a>>,
    pub by_tag: &'a Vec<Category<'a>>,
    pub by_keyword: &'a Vec<Category<'a>>,
    pub by_category: &'a Vec<Category<'a>>,
}

#[derive(Serialize, Debug, Clone)]
pub enum ListType {
    Index,
    Category,
    Year,
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
    pub list_type: ListType,
}
