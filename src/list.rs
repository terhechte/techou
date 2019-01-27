use serde_derive::{Serialize};

use crate::article::Article;
use crate::config::Config;

#[derive(Serialize, Debug)]
pub struct List<'a> {
    pub title: String,
    pub articles: &'a Vec<Article>
}
