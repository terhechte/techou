use std::path::Path;

use rayon::prelude::*;

use crate::error::Result;
use crate::io_utils::*;
use crate::config::Config;
use crate::document::{Document, documents_in_folder};
use crate::template::Templates;
use crate::list::*;
use crate::feeds;
use crate::utils;

pub struct Builder<'a> {
    context: DocumentContext<'a>,
    config: &'a Config,
    template_writer: &'a Templates
}

impl<'a> Builder<'a> {
    /// Initialize a new Builder without a context. This means that the templates
    /// Will not have access to `pages`, `posts`, `posts by tag` and `posts by date`
    /// If you wish to have these, please use `Builder::with_contex`
    /*pub fn new(template_writer: &'a Templates, config: &'a Config) -> Builder<'a> {
        let context = DocumentContext {
            pages: Vec::new(), posts: Vec::new(), by_tag: Vec::new(), by_date: Vec::new()
        };
        Builder { context, template_writer, config }
    }*/

    /// Create a new builder with the template context for the template to access
    /// `pages`, `posts`, `posts by tag` and `posts by date`
    pub fn with_context(context: DocumentContext<'a>, template_writer: &'a Templates, config: &'a Config) -> Builder<'a> {
        return Builder { context, template_writer, config }
    }

    /// Write the posts to the folder `folder` with template writer `writer`
    /// `folder` has to be a name / path within the output folder, but without the
    /// output folder as part of the name.
    /// If `html/posts` is your output folder / posts folder, then `posts` would be
    /// the correct value for `folder`
    pub fn posts<A: AsRef<Path>>(&self, posts: &Vec<Document>, folder: A) -> Result<()> {
        let folder = self.config.folders.output_folder_path().join(folder.as_ref());
        posts.par_iter().for_each(|post| {
            let path = folder.join(&post.slug);
            match self.template_writer.write_post(&self.context, &post, &path, &self.config) {
                Ok(_) => println!("Wrote '{:?}'", &path),
                Err(e) => println!("Could not write article {}: {:?}", &post.filename, &e)
            }
        });
        Ok(())
    }

    pub fn pages<A: AsRef<Path>>(&self, pages: &Vec<Document>, folder: A) -> Result<()> {
        let folder = self.config.folders.output_folder_path().join(folder.as_ref());
        pages.par_iter().for_each(|page| {
            let path = folder.join(&page.slug);
            match self.template_writer.write_page(&self.context, &page, &path, &self.config) {
                Ok(_) => println!("Wrote '{:?}'", &path),
                Err(e) => println!("Could not write article {}: {:?}", &page.filename, &e)
            }
        });
        Ok(())
    }

    /// Write all the posts into one long index file.
    pub fn index<A: AsRef<Path>>(&self, posts: &Vec<Document>, folder: A) -> Result<()> {
        let folder = self.config.folders.output_folder_path().join(folder.as_ref());
        let path = folder.join("index.html");
        match self.template_writer.write_list(&self.context, &List {
            title: "Index",
            posts: posts,
            pagination: Pagination {
                current: 0,
                next: None,
                previous: None
            }}, &path, &self.config) {
            Ok(_) => println!("Wrote index: {:?}", &path),
            Err(e) => println!("Could not write index {:?}: {:?}", &path, &e)
        };
        Ok(())
    }

    /// Write a number of posts as chunks into multiple index files.
    /// `make_title(i32) -> (String, String)` is a function that returns the
    /// filename and the title of an index page based on the index. 0 being the first
    /// `per_page` is the number of posts that should be on one page before a new one begins
    pub fn indexes_paged<A: AsRef<Path>, TitleFn>(&self, posts: &Vec<Document>,
                                                  per_page: usize,
                                                  make_title: TitleFn,
                                                  folder: A) -> Result<()>
        where TitleFn: Fn(usize) -> (String, String)
    {
        let folder = self.config.folders.output_folder_path().join(folder.as_ref());
        let mut state: (Option<Page>, Option<Page>) = (None, None);
        let mut iter = posts.chunks(per_page).enumerate().peekable();
        let mut index_page = 0;
        loop {
            let (index, chunk) = match iter.next() {
                Some(o) => o,
                None => break
            };

            let (filename, title) = match index {
                0 => ("index.html".to_string(), "Index".to_string()),
                _ => (format!("index-{}.html", &index), format!("Index - Page {}", index_page)),
            };

            state.0 = iter.peek().map(|(index, chunk)| {
                Page { title: format!("Index - Page {}", index + 1),
                    index: *index,
                    items: chunk.len(),
                    path: filename.clone() }
            });

            let pagination = Pagination {
                current: index,
                next: state.0.take(),
                previous: state.1.take()
            };

            let path = folder.join(&filename);

            match self.template_writer.write_list(&self.context, &List { title: &title, posts: chunk, pagination}, &path, &self.config) {
                Ok(_) => println!("Wrote index: {:?}", &path),
                Err(e) => println!("Could not write index {:?}: {:?}", &path, &e)
            }
            state.1 = Some(Page { title: title,
                index: index,
                items: chunk.len(),
                path: filename.clone()
            });
            index_page += 1;
        }
        Ok(())
    }

    /// Write out documents for each tag with the articles for that tag
    pub fn tags<A: AsRef<Path>>(&self, tag_posts: &Vec<Tag<'a>>, folder: A) -> Result<()> {
        let folder = self.config.folders.output_folder_path().join(folder.as_ref());
        for tag in tag_posts {
            let slug = format!("{}.html", &utils::slugify(&tag.name));
            let path = folder.join(&slug);
            match self.template_writer.write_list(&self.context, &List {
                title: tag.name,
                posts: tag.posts.as_slice(),
                pagination: Pagination {
                    current: 0,
                    next: None,
                    previous: None
                }}, &path, &self.config) {
                Ok(_) => println!("Wrote tag index: {:?}", &path),
                Err(e) => println!("Could not write index {:?}: {:?}", &path, &e)
            };
        }
        Ok(())
    }
}
