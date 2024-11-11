use rayon::prelude::*;

use crate::book::{Book, Chapter};
use crate::config::Config;
use crate::document::Document;
use crate::error::Result;
use crate::list::*;
use crate::template::Templates;
use crate::utils;

use std::path::{Path, PathBuf};

trait NonAdjoinedPush {
    #[allow(non_snake_case)]
    fn nonAdjoinedPush(&self, path: &str) -> PathBuf;
}

impl NonAdjoinedPush for PathBuf {
    fn nonAdjoinedPush(&self, path: &str) -> PathBuf {
        let mut iter = path.chars();
        if let Some(first) = iter.next() {
            if first == '/' {
                let path: String = iter.collect();
                return self.join(&path);
            }
        }
        self.join(path)
    }
}

pub struct Builder<'a> {
    context: DocumentContext<'a>,
    config: &'a Config,
    template_writer: &'a Templates,
}

impl<'a> Builder<'a> {
    /// Create a new builder with the template context for the template to access
    /// `pages`, `posts`, `posts by tag` and `posts by date`
    pub fn with_context(
        context: DocumentContext<'a>,
        template_writer: &'a Templates,
        config: &'a Config,
    ) -> Builder<'a> {
        Builder {
            context,
            template_writer,
            config,
        }
    }

    /// Write the posts to the folder `folder` with template writer `writer`
    /// `folder` has to be a name / path within the output folder, but without the
    /// output folder as part of the name.
    /// If `html/posts` is your output folder / posts folder, then `posts` would be
    /// the correct value for `folder`
    pub fn posts(&self, posts: &[Document]) -> Result<()> {
        let folder = self.config.folders.output_folder_path();
        posts.par_iter().for_each(|post| {
            if post.updated == false {
                return;
            }
            let path = folder.nonAdjoinedPush(&post.slug);
            match self
                .template_writer
                .write_post(&self.context, &post, &path, &self.config)
            {
                Ok(_) => (), /*println!("Wrote '{:?}'", &path)*/
                Err(e) => println!("Could not write article {}: {:?}", &post.filename, &e),
            }
        });
        Ok(())
    }

    pub fn pages(&self, pages: &[Document]) -> Result<()> {
        let folder = self.config.folders.output_folder_path();
        pages.par_iter().for_each(|page| {
            if page.updated == false {
                return;
            }
            let path = folder.nonAdjoinedPush(&page.slug);
            match self
                .template_writer
                .write_page(&self.context, &page, &path, &self.config)
            {
                Ok(_) => (), /*println!("Wrote '{:?}'", &path)*/
                Err(e) => println!("Could not write article {}: {:?}", &page.filename, &e),
            }
        });
        Ok(())
    }

    /// Write a number of posts as chunks into multiple index files.
    /// `make_title(i32) -> (String, String)` is a function that returns the
    /// filename and the title of an index page based on the index. 0 being the first
    /// `per_page` is the number of posts that should be on one page before a new one begins
    pub fn indexes_paged<A: AsRef<Path>, TitleFn>(
        &self,
        posts: &[Document],
        per_page: usize,
        make_title: TitleFn,
        folder: A,
    ) -> Result<()>
    where
        TitleFn: Fn(usize) -> (String, String),
    {
        if posts.iter().all(|d| d.updated == false) {
            return Ok(());
        }
        let folder = self
            .config
            .folders
            .output_folder_path()
            .join(folder.as_ref());
        let mut state: (Option<Page>, Option<Page>) = (None, None);
        let mut iter = posts.chunks(per_page).enumerate().peekable();
        while let Some((index, chunk)) = iter.next() {
            println!("Index {index}. Chunk liength {}", chunk.len());
            // dbg!(&index, chunk);
            let (filename, title) = make_title(index);

            let (_, future_title) = make_title(index + 1);
            state.0 = iter.peek().map(|(index, chunk)| Page {
                title: future_title,
                index: *index,
                items: chunk.len(),
                path: make_title(*index).0,
            });

            let pagination = Pagination {
                current: index,
                next: state.0.take(),
                previous: state.1.take(),
            };

            let path = folder.join(&filename);

            match self.template_writer.write_list(
                &self.context,
                &List {
                    title: &title,
                    posts: chunk,
                    pagination,
                    list_type: ListType::Index,
                },
                &path,
                self.config,
            ) {
                Ok(_) => (),
                Err(e) => println!("Could not write index {:?}: {:?}", &path, &e),
            }
            state.1 = Some(Page {
                title,
                index,
                items: chunk.len(),
                path: make_title(index).0,
            });
        }
        Ok(())
    }

    /// Write out documents for each category with the articles for that name
    pub fn category<A: AsRef<Path>>(&self, tag_posts: &[Category<'a>], folder: A) -> Result<()> {
        let folder = self
            .config
            .folders
            .output_folder_path()
            .join(folder.as_ref());
        for tag in tag_posts {
            let slug = format!("{}.html", &utils::slugify(&tag.name));
            let path = folder.join(&slug);
            match self.template_writer.write_list(
                &self.context,
                &List {
                    title: tag.name,
                    posts: tag.posts.as_slice(),
                    pagination: Pagination {
                        current: 0,
                        next: None,
                        previous: None,
                    },
                    list_type: ListType::Category,
                },
                &path,
                &self.config,
            ) {
                Ok(_) => (), /*println!("Wrote tag index: {:?}", &path)*/
                Err(e) => println!("Could not write index {:?}: {:?}", &path, &e),
            };
        }
        Ok(())
    }

    /// Write out documents for each category with the articles for that name
    pub fn years<A: AsRef<Path>>(&self, year_posts: &[Year<'a>], folder: A) -> Result<()> {
        let folder = self
            .config
            .folders
            .output_folder_path()
            .join(folder.as_ref());
        for year in year_posts {
            let slug = format!("{}.html", &year.name);
            let path = folder.join(&slug);
            let mut posts: Vec<&Document> = Vec::new();
            for x in year.months.iter() {
                posts.extend(&x.posts);
            }
            match self.template_writer.write_year(
                &self.context,
                &List {
                    title: &format!("{}", year.name),
                    posts: posts.as_slice(),
                    pagination: Pagination {
                        current: 0,
                        next: None,
                        previous: None,
                    },
                    list_type: ListType::Category,
                },
                &path,
                self.config,
            ) {
                Ok(_) => (), /*println!("Wrote tag index: {:?}", &path)*/
                Err(e) => println!("Could not write index {:?}: {:?}", &path, &e),
            };
        }
        Ok(())
    }

    /// Write out a book as a recursive tree of chapters.
    /// We basically replicate the book directory structure as HTML
    pub fn books<A: AsRef<Path>>(&self, books: &[Book], folder: A) -> Result<()> {
        let folder = self
            .config
            .folders
            .output_folder_path()
            .join(folder.as_ref());
        books.par_iter().for_each(|book| {
            let path = folder.join(&book.folder);

            // for each book, we need to write out all the chapters recursively
            self.chapter(&book, &book.chapters).unwrap();

            let path = path.join("index.html");
            match self
                .template_writer
                .write_book(&self.context, &book, &path, &self.config)
            {
                Ok(_) => (), /*println!("Wrote '{:?}'", &path)*/
                Err(e) => println!("Could not write book {}: {:?}", &book.identifier, &e),
            }

            // If we have the book as one document, write that out
            if let Some(ref whole_book) = book.complete_book {
                let path = &self
                    .config
                    .folders
                    .output_folder_path()
                    .join(&whole_book.slug);
                match self.template_writer.write_post(
                    &self.context,
                    whole_book,
                    &path,
                    &self.config,
                ) {
                    Ok(_) => (),
                    Err(e) => println!(
                        "Could not write whole book {}: {:?} {:?}",
                        &whole_book.identifier, &path, &e
                    ),
                }
            }
        });
        Ok(())
    }

    pub fn chapter(&self, book: &Book, chapters: &[Chapter]) -> Result<()> {
        for chapter in chapters {
            if chapter.document.updated {
                let output_path = std::path::PathBuf::from(&self.config.folders.output_folder)
                    .join(&chapter.slug);
                match self.template_writer.write_chapter(
                    &self.context,
                    &book,
                    &chapter,
                    &output_path,
                    &self.config,
                ) {
                    Ok(_) => (), /*println!("write chapter to: {:?}", &chapter.slug)*/
                    Err(e) => println!("Could not write {}: {}", &chapter.name, &e),
                };
            }
            if !chapter.sub_chapters.is_empty() {
                self.chapter(&book, &chapter.sub_chapters)?;
            }
        }
        Ok(())
    }
}
