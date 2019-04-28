use rayon::prelude::*;

use crate::builder;
use crate::config::Config;
use crate::document::{Document, documents_in_folder};
use crate::document_operations::*;
use crate::error::Result;
use crate::feeds;
use crate::io_utils::*;
use crate::list::*;
use crate::template::Templates;
use crate::book::Book;
use crate::build_cache::BuildCache;
use crate::search::Searcher;
use crate::sitemap::SiteMap;

pub fn execute(ignore_errors: bool, config: &Config, cache: &BuildCache) -> Result<()> {
    match catchable_execute(&config, &cache) {
        Ok(_) => Ok(()),
        Err(e) => if ignore_errors {
            println!("Error: {}", &e);
            Ok(())
        } else {
            Err(e)
        },
    }
}

fn catchable_execute(config: &Config, cache: &BuildCache) -> Result<()> {
    let begin = std::time::Instant::now();
    // Clean the old output folder, if it still exists.
    // We don't want to remove the folder, so that static servers still work
    clear_directory(&config.folders.output_folder_path())?;

    // create a search engine
    let mut searcher = Searcher::new(&config);

    println!("Root folder: {:?}", &config.folders.root);
    let mut posts = documents_in_folder(&config.folders.posts_folder_path(), &config.folders.posts_folder_name, &config, &cache)?;
    posts.sort_by(|a1, a2| {
        a2.info
            .created_timestamp
            .partial_cmp(&a1.info.created_timestamp)
            .unwrap()
    });

    make_document_siblings(&mut posts);

    if config.search.enable {
        for document in &posts {
            searcher.index_document(document);
        }
    }

    // if we have more than 5 posts, start generating similarity
    if posts.len() >= 5 {
        // We want two similarity items for each post
        make_similarity(&mut posts, 2);
    }

    let mut template_writer = Templates::new(&config.folders.public_folder_path()).unwrap();

    let pages = documents_in_folder(&config.folders.pages_folder_path(), &config.folders.pages_folder_name, &config, &cache)?;

    if config.search.enable {
        for document in &pages {
            searcher.index_document(document);
        }
    }

    let books: Vec<Book> = config.folders.books.par_iter().filter_map(|filename| {
        match Book::new(&filename, &config, &cache) {
            Ok(book) => Some(book),
            Err(e) =>  {
                println!("Error generating book {}: {}", &filename, &e);
                None
            }
        }
    }).collect();
    let by_year = posts_by_date(&posts);
    let by_keyword = posts_by_array(&posts, |p| &p.info.keywords);
    let by_category = posts_by_array(&posts, |p| &p.info.category);

    let mut all_posts: Vec<&Document> = posts.iter().collect();
    for book in &books {
        // Temporarily awful
        for chapter in &book.chapters {
            all_posts.push(&chapter.document);
            for chapter in &chapter.sub_chapters {
                all_posts.push(&chapter.document);
                for chapter in &chapter.sub_chapters {
                    all_posts.push(&chapter.document);
                }
            }
        }
    }
    //let by_tag = posts_by_array(&posts, |p| &p.info.tags);
    let by_tag = posts_by_array(&all_posts, |p| &p.info.tags);

    if config.search.enable {
        for book in &books {
            searcher.index_book(book);
        }
    }

    let context = DocumentContext {
            posts: &posts,
            all_posts: &all_posts,
            pages: &pages,
            books: &books,
            by_date: &by_year,
            by_tag: &by_tag,
            by_keyword: &by_keyword,
            by_category: &by_category,
    };

    template_writer.register_url_functions(&context, &config);

    let builder = builder::Builder::with_context(
        context,
        &template_writer,
        &config,
    );

    builder.posts(&posts)?;
    builder.pages(&pages)?;
    builder.books(&books, &config.folders.books_folder_name)?;
    builder.category(&by_tag, &config.folders.tags_folder_name)?;
    builder.category(&by_keyword, &config.folders.keywords_folder_name)?;
    builder.category(&by_category, &config.folders.category_folder_name)?;

    // Write the indexed pages
    let title_fn = |index| match index {
        0 => ("index.html".to_string(), "Index".to_string()),
        _ => (
            format!("index-{}.html", index),
            format!("Index - Page {}", index),
        ),
    };
    builder.indexes_paged(&posts, config.project.posts_per_index as usize, title_fn, "")?;

    // Write the feed
    if let Some(rss) = &config.rss {
        feeds::write_posts_rss(
            &posts,
            &config.folders.output_folder_path().join("feed.rss"),
            &rss,
            &config.project.base_url
        )?;
    }

    // Write the assets
    copy_items_to_directory(
        &config.folders.public_copy_folders,
        &config.folders.public_folder_path(),
        &config.folders.output_folder_path(),
    )?;

    // Write the search index
    if config.search.enable {
        let search_contents = searcher.finalize()?;
        let search_index_output_path = config.folders.output_folder_path().join(&config.search.search_index_file);
        spit(search_index_output_path, &search_contents);
    }

    // create a site map
    if !config.project.base_url.is_empty() {
        let outfile = config.folders.output_folder_path().join("sitemap.xml");
        let mut sitemap = SiteMap::new(outfile, &config.project.base_url);
        for post in &all_posts {
            sitemap.add_document(&post);
        }

        // FIXME: Terrible, we need a better way to handle the recusrive book structure
        /*for book in books {
            book.map(|chapter| {
                sitemap.add_document(&chapter.document);
            })
        }*/

        for page in &pages {
            sitemap.add_document(&page);
        }

        sitemap.finish();
    }



    let end = std::time::Instant::now();
    println!("Execution time: {:?}", end - begin);

    Ok(())
}
