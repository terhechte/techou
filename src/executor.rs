use std::path::Path;

use rayon::prelude::*;

use crate::error::Result;
use crate::io_utils::*;
use crate::config::Config;
use crate::article::Article;
use crate::template::Templates;
use crate::list::List;

pub fn execute<A: AsRef<Path>>(folder: A) -> Result<()> {
    let config: Config = Default::default();

    // Clean the old output folder, if it still exists
    std::fs::remove_dir_all(&config.output_folder_path())?;

    println!("Root folder: {:?}", &config.root);
    let files = contents_of_directory(config.posts_folder_path(), "md")?;
    let articles: Vec<Article> = files.par_iter().filter_map(|path| {
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

    let template_writer = Templates::new(&config.template_folder_path());

    // write all posts + slug
    articles.par_iter().for_each(|article| {
        let path = config.articles_folder_path().join(&article.slug);
        match template_writer.write_article(&article, &path, &config) {
            Ok(_) => (), Err(e) => println!("Could not write article {}: {:?}", &article.filename, &e)
        }
        println!("Wrote '{:?}'", &path);
    });

    // write the index
    let path = config.output_folder_path().join("index.html");
    match template_writer.write_list(&List {
        title: "Main Index".to_string(),
        articles: &articles
    }, &path, &config) {
        Ok(_) => (), Err(e) => println!("Could not write index {:?}: {:?}", &path, &e)
    }

    // todo: write per tag

    // todo: write per year / month
    Ok(())
}