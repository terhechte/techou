use rss::{extension, ChannelBuilder, Item, ItemBuilder};

use crate::config::ConfigRSS;
use crate::document::Document;
use crate::error::Result;
use crate::io_utils::spit;

use std::path::Path;

pub fn write_posts_rss<A: AsRef<Path>>(
    posts: &[Document],
    to_path: A,
    rss: &ConfigRSS,
    base_url: &str,
) -> Result<()> {
    let author = match &rss.author_name {
        Some(name) => format!("{} ({})", &rss.author_email, name),
        None => rss.author_email.clone(),
    };
    let items: Vec<Item> = posts
        .iter()
        .map(|post| {
            let link = format!("{}/{}", &base_url, &post.slug);
            ItemBuilder::default()
                .itunes_ext(extension::itunes::ITunesItemExtension::default())
                .dublin_core_ext(extension::dublincore::DublinCoreExtension::default())
                .title(post.info.title.clone())
                .link(link)
                .description(post.info.description.clone())
                .author(author.clone())
                .pub_date(post.info.rfc2822())
                .build()
        })
        .collect();
    let link = format!("{}/{}", &base_url, &rss.feed_address);
    let mut channel = ChannelBuilder::default()
        .title(rss.title.clone())
        .link(link)
        .items(items)
        .build();
    if let Some(desc) = &rss.description {
        channel.set_description(desc.clone())
    }
    spit(to_path, &channel.to_string())
}
