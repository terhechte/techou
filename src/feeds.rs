use rss::{extension, ChannelBuilder, Item, ItemBuilder};

use crate::config::Config;
use crate::document::Document;
use crate::error::Result;
use crate::io_utils::spit;

use std::path::Path;

pub fn write_posts_rss<A: AsRef<Path>>(
    posts: &Vec<Document>,
    to_path: A,
    config: &Config,
) -> Result<()> {
    let rss = match &config.rss {
        Some(rss) => rss,
        None => return Ok(()),
    };
    let items: Vec<Item> = posts
        .iter()
        .map(|post| {
            ItemBuilder::default()
                .itunes_ext(extension::itunes::ITunesItemExtension::default())
                .dublin_core_ext(extension::dublincore::DublinCoreExtension::default())
                .title(post.info.title.clone())
                .description(post.info.description.clone())
                .author(rss.author_email.clone())
                .pub_date(post.info.rfc2822())
                .build()
                .unwrap()
        })
        .collect();
    let channel = ChannelBuilder::default()
        .title(rss.title.clone())
        .link(rss.link.clone())
        .description(rss.description.clone())
        .items(items)
        .build()
        .unwrap();
    spit(to_path, &channel.to_string())
}
