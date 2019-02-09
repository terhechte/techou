use std::path::Path;

use rayon::prelude::*;

use crate::builder;
use crate::error::Result;
use crate::io_utils::*;
use crate::config::Config;
use crate::document::{Document, documents_in_folder};
use crate::template::Templates;
use crate::list::*;
use crate::feeds;

pub fn execute(ignore_errors: bool, config: &Config) -> Result<()> {
    match catchable_execute(&config) {
        Ok(n) => Ok(n),
        Err(e) => match ignore_errors {
            true => {
                println!("Error: {}", &e);
                Ok(())
            },
            false => {
                Err(e)
            }
        }
    }
}

fn catchable_execute(config: &Config) -> Result<()> {

    // Clean the old output folder, if it still exists.
    // We don't want to remove the folder, so that static servers still work
    clear_directory(&config.folders.output_folder_path())?;

    println!("Root folder: {:?}", &config.folders.root);
    let mut posts = documents_in_folder(&config.folders.posts_folder_path(), &config)?;
    posts.sort_by(|a1, a2| a2.info.created_timestamp.partial_cmp(&a1.info.created_timestamp).unwrap());

    let template_writer = Templates::new(&config.folders.public_folder_path()).unwrap();


    // write all posts + slug
    builder::posts(&posts, &config.folders.posts_folder_name, &template_writer, &config)?;

    // write all pages
    let pages = documents_in_folder(&config.folders.pages_folder_path(), &config)?;
    builder::pages(&pages, &config.folders.pages_folder_name, &template_writer, &config)?;

    let title_fn = |index| {
        match index {
            0 => ("index.html".to_string(), "Index".to_string()),
            _ => (format!("index-{}.html", index), format!("Index - Page {}", index)),
        }
    };

    builder::indexes_paged(&posts, 3, title_fn, "", &template_writer, &config)?;

    // todo: write per tag

    // Write the feed
    feeds::write_posts_rss(&posts, &config.folders.output_folder_path().join("feed.rss"), &config)?;

    // Write the assets
    copy_items_to_directory(&config.folders.public_copy_folders, &config.folders.public_folder_path(), &config.folders.output_folder_path())?;

    Ok(())
}
