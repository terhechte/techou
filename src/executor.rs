use crate::builder;
use crate::config::Config;
use crate::document::documents_in_folder;
use crate::document_operations::*;
use crate::error::Result;
use crate::feeds;
use crate::io_utils::*;
use crate::list::*;
use crate::template::Templates;

pub fn execute(ignore_errors: bool, config: &Config) -> Result<()> {
    match catchable_execute(&config) {
        Ok(_) => Ok(()),
        Err(e) => if ignore_errors {
            println!("Error: {}", &e);
            Ok(())
        } else {
            Err(e)
        },
    }
}

fn catchable_execute(config: &Config) -> Result<()> {
    // Clean the old output folder, if it still exists.
    // We don't want to remove the folder, so that static servers still work
    clear_directory(&config.folders.output_folder_path())?;

    println!("Root folder: {:?}", &config.folders.root);
    let mut posts = documents_in_folder(&config.folders.posts_folder_path(), &config)?;
    posts.sort_by(|a1, a2| {
        a2.info
            .created_timestamp
            .partial_cmp(&a1.info.created_timestamp)
            .unwrap()
    });

    make_document_siblings(&mut posts);

    // if we have more than 5 posts, start generating similarity
    if posts.len() >= 5 {
        // We want two similarity items for each post
        make_similarity(&mut posts, 2);
    }

    let template_writer = Templates::new(&config.folders.public_folder_path()).unwrap();

    let pages = documents_in_folder(&config.folders.pages_folder_path(), &config)?;
    let by_year = posts_by_date(&posts);
    let by_tag = posts_by_array(&posts, |p| &p.info.tags);
    let by_keyword = posts_by_array(&posts, |p| &p.info.keywords);

    let builder = builder::Builder::with_context(
        DocumentContext {
            posts: &posts,
            pages: &pages,
            by_date: &by_year,
            by_tag: &by_tag,
            by_keyword: &by_keyword,
        },
        &template_writer,
        &config,
    );

    builder.posts(&posts, &config.folders.posts_folder_name)?;
    builder.pages(&pages, &config.folders.pages_folder_name)?;
    builder.tags(&by_tag, &config.folders.tags_folder_name)?;

    // Write the indexed pages
    let title_fn = |index| match index {
        0 => ("index.html".to_string(), "Index".to_string()),
        _ => (
            format!("index-{}.html", index),
            format!("Index - Page {}", index),
        ),
    };
    builder.indexes_paged(&posts, 3, title_fn, "")?;

    // Write the feed
    feeds::write_posts_rss(
        &posts,
        &config.folders.output_folder_path().join("feed.rss"),
        &config,
    )?;

    // Write the assets
    copy_items_to_directory(
        &config.folders.public_copy_folders,
        &config.folders.public_folder_path(),
        &config.folders.output_folder_path(),
    )?;

    Ok(())
}
