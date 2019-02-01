use std::path::Path;

use rayon::prelude::*;

use crate::error::Result;
use crate::io_utils::*;
use crate::config::Config;
use crate::article::Article;
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
    clear_directory(&config.output_folder_path())?;

    println!("Root folder: {:?}", &config.root);
    let files = contents_of_directory(config.posts_folder_path(), "md")?;
    let mut articles: Vec<Article> = files.par_iter().filter_map(|path| {
        let contents = match slurp(path) {
            Ok(c) => c, Err(e) => {
                println!("Can't read {:?}: {:?}", &path, &e);
                return None;
            }
        };
        let article = match Article::new(&contents, &path, &config) {
            Ok(a) => a, Err(e) => {
                println!("Invalid Format {:?}: {:?}", &path, &e);
                return None;
            }
        };
        Some(article)
    }).collect();

    articles.sort_by(|a1, a2| a2.info.created_timestamp.partial_cmp(&a1.info.created_timestamp).unwrap());

    let template_writer = Templates::new(&config.public_folder_path()).unwrap();

    // write all posts + slug
    articles.par_iter().for_each(|article| {
        let path = config.articles_folder_path().join(&article.slug);
        match template_writer.write_article(&article, &path, &config) {
            Ok(_) => println!("Wrote '{:?}'", &path),
            Err(e) => println!("Could not write article {}: {:?}", &article.filename, &e)
        }
    });

    // write the index
    let path = config.output_folder_path().join("index.html");
    match template_writer.write_list(&List {
        title: "Main Index".to_string(),
        articles: &articles,
        articles_by_date: articles_by_date(&articles)
    }, &path, &config) {
        Ok(_) => println!("Wrote index: {:?}", &path),
        Err(e) => println!("Could not write index {:?}: {:?}", &path, &e)
    }

    // todo: write per tag

    // todo: write per year / month

    // Write the feed
    if let Some(ref rss) = &config.rss_settings {
        feeds::write_articles_rss(&articles, &config.output_folder_path().join("feed.rss"), &config, rss)?;
    }

    // Write the assets
    copy_items_to_directory(&config.public_copy_folders, &config.public_folder_path(), &config.output_folder_path())?;

    Ok(())
}
