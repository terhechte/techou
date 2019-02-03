use std::path::Path;

use rayon::prelude::*;

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
    posts.par_iter().for_each(|post| {
        let path = config.folders.output_posts_folder_path().join(&post.slug);
        match template_writer.write_post(&post, &path, &config) {
            Ok(_) => println!("Wrote '{:?}'", &path),
            Err(e) => println!("Could not write article {}: {:?}", &post.filename, &e)
        }
    });

    // write all pages
    let pages = documents_in_folder(&config.folders.pages_folder_path(), &config)?;
    pages.par_iter().for_each(|page| {
        let path = config.folders.output_pages_folder_path().join(&page.slug);
        match template_writer.write_page(&page, &path, &config) {
            Ok(_) => println!("Wrote '{:?}'", &path),
            Err(e) => println!("Could not write article {}: {:?}", &page.filename, &e)
        }
    });

    // write the index
    let path = config.folders.output_folder_path().join("index.html");
    match template_writer.write_list(&List {
        title: "Main Index".to_string(),
        posts: &posts,
        posts_by_date: posts_by_date(&posts),
        pages: &pages
    }, &path, &config) {
        Ok(_) => println!("Wrote index: {:?}", &path),
        Err(e) => println!("Could not write index {:?}: {:?}", &path, &e)
    }

    // todo: write per tag

    // todo: write per year / month

    // Write the feed
    feeds::write_posts_rss(&posts, &config.folders.output_folder_path().join("feed.rss"), &config)?;

    // Write the assets
    copy_items_to_directory(&config.folders.public_copy_folders, &config.folders.public_folder_path(), &config.folders.output_folder_path())?;

    Ok(())
}
