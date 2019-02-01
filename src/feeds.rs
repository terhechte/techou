use rss::{ ChannelBuilder, ItemBuilder, extension, Item };

use crate::article::Article;
use crate::config::{Config, RSS};
use crate::error::{Result, TechouError};
use crate::io_utils::spit;

use std::path::Path;

pub fn write_articles_rss<A: AsRef<Path>>(articles: &Vec<Article>, to_path: A, config: &Config, rss: &RSS) -> Result<()> {
    let items: Vec<Item> = articles.iter().map(|article| {
        ItemBuilder::default()
            .itunes_ext(extension::itunes::ITunesItemExtension::default())
            .dublin_core_ext(extension::dublincore::DublinCoreExtension::default())
            .title(article.info.title.clone())
            .description(article.info.description.clone())
            .author(rss.author_email.clone())
            .pub_date(article.info.rfc2822())
            .build()
            .unwrap()
    }).collect();
    let channel = ChannelBuilder::default()
        .title(rss.title.clone())
        .link(rss.link.clone())
        .description(rss.description.clone())
        .items(items)
        .build()
        .unwrap();
    spit(to_path, &channel.to_string());
    Ok(())
}
