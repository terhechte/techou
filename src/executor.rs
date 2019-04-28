use rayon::prelude::*;

use crate::builder;
use crate::config::Config;
use crate::document::{Document, documents_in_folder};
use crate::document_operations::*;
use crate::error::Result;
use crate::feeds;
use crate::io_utils::*;
use crate::utils::DebugTimer;
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
    let mut timer = DebugTimer::begin(0, &config);
    // Clean the old output folder, if it still exists.
    // We don't want to remove the folder, so that static servers still work
    clear_directory(&config.folders.output_folder_path())?;
    timer.sub_step("Clean");

    // create a search engine
    let mut searcher = Searcher::new(&config);

    println!("Root folder: {:?}", &config.folders.root);
    let mut posts = documents_in_folder(&config.folders.posts_folder_path(), &config.folders.posts_folder_name, &config, &cache)?;
    timer.sub_step("Posts");

    posts.sort_by(|a1, a2| {
        a2.info
            .created_timestamp
            .partial_cmp(&a1.info.created_timestamp)
            .unwrap()
    });
    timer.sub_step("sort_by");

    make_document_siblings(&mut posts);

    timer.sub_step("Make Siblings");

    if config.search.enable {
        for document in &posts {
            searcher.index_document(document);
        }
    }

    timer.sub_step("Search Documents");

    // if we have more than 5 posts, start generating similarity
    if posts.len() >= 5 {
        // We want two similarity items for each post
        make_similarity(&mut posts, 2);
    }

    timer.sub_step("Similarity");

    let mut template_writer = Templates::new(&config.folders.public_folder_path()).unwrap();

    let pages = documents_in_folder(&config.folders.pages_folder_path(), &config.folders.pages_folder_name, &config, &cache)?;

    timer.sub_step("Load Pages");

    if config.search.enable {
        for document in &pages {
            searcher.index_document(document);
        }
    }

    timer.sub_step("Search Pages");

    let books: Vec<Book> = config.folders.books.par_iter().filter_map(|filename| {
        match Book::new(&filename, &config, &cache) {
            Ok(book) => Some(book),
            Err(e) =>  {
                println!("Error generating book {}: {}", &filename, &e);
                None
            }
        }
    }).collect();

    timer.sub_step("Books");

    let by_year = posts_by_date(&posts);
    timer.sub_step("posts_by_date");
    let by_keyword = posts_by_array(&posts, |p| &p.info.keywords);
    timer.sub_step("by_keyword");
    let by_category = posts_by_array(&posts, |p| &p.info.category);
    timer.sub_step("by_category");

    let mut all_posts: Vec<&Document> = posts.iter().collect();
    timer.sub_step("all_posts");
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
    timer.sub_step("Recursive Books");
    //let by_tag = posts_by_array(&posts, |p| &p.info.tags);
    let by_tag = posts_by_array(&all_posts, |p| &p.info.tags);
    timer.sub_step("All Posts");

    if config.search.enable {
        for book in &books {
            searcher.index_book(book);
        }
    }

    timer.sub_step("Search Books");

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
    timer.sub_step("Write Posts");
    builder.pages(&pages)?;
    timer.sub_step("Write Pages");
    builder.books(&books, &config.folders.books_folder_name)?;
    timer.sub_step("Write Books");
    builder.category(&by_tag, &config.folders.tags_folder_name)?;
    timer.sub_step("Write Tags");
    builder.category(&by_keyword, &config.folders.keywords_folder_name)?;
    timer.sub_step("Write Keywords");
    builder.category(&by_category, &config.folders.category_folder_name)?;
    timer.sub_step("Write Categories");

    // Write the indexed pages
    let title_fn = |index| match index {
        0 => ("index.html".to_string(), "Index".to_string()),
        _ => (
            format!("index-{}.html", index),
            format!("Index - Page {}", index),
        ),
    };
    builder.indexes_paged(&posts, config.project.posts_per_index as usize, title_fn, "")?;
    timer.sub_step("Write Indexses");

    // Write the feed
    if let Some(rss) = &config.rss {
        feeds::write_posts_rss(
            &posts,
            &config.folders.output_folder_path().join("feed.rss"),
            &rss,
            &config.project.base_url
        )?;
        timer.sub_step("Write Feed");
    }

    // Write the assets
    copy_items_to_directory(
        &config.folders.public_copy_folders,
        &config.folders.public_folder_path(),
        &config.folders.output_folder_path(),
    )?;
    timer.sub_step("Write Assets");

    // Write the search index
    if config.search.enable {
        let search_contents = searcher.finalize()?;
        let search_index_output_path = config.folders.output_folder_path().join(&config.search.search_index_file);
        spit(search_index_output_path, &search_contents);
        timer.sub_step("Write Search");
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
        timer.sub_step("Write Sitemap");
    }

    timer.end();

    Ok(())
}
