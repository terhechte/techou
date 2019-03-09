use rayon::prelude::*;

use crate::config::Config;
use crate::document::Document;
use crate::book::{Book, Chapter};
use crate::error::Result;
use crate::list::*;
use crate::template::Templates;
use crate::utils;

use std::path::Path;

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
    pub fn posts<A: AsRef<Path>>(&self, posts: &[Document], folder: A) -> Result<()> {
        let folder = self
            .config
            .folders
            .output_folder_path()
            .join(folder.as_ref());
        posts.par_iter().for_each(|post| {
            let path = folder.join(&post.slug);
            match self
                .template_writer
                .write_post(&self.context, &post, &path, &self.config)
            {
                Ok(_) => () /*println!("Wrote '{:?}'", &path)*/,
                Err(e) => println!("Could not write article {}: {:?}", &post.filename, &e),
            }
        });
        Ok(())
    }

    pub fn pages<A: AsRef<Path>>(&self, pages: &[Document], folder: A) -> Result<()> {
        let folder = self
            .config
            .folders
            .output_folder_path()
            .join(folder.as_ref());
        pages.par_iter().for_each(|page| {
            let path = folder.join(&page.slug);
            match self
                .template_writer
                .write_page(&self.context, &page, &path, &self.config)
            {
                Ok(_) => () /*println!("Wrote '{:?}'", &path)*/,
                Err(e) => println!("Could not write article {}: {:?}", &page.filename, &e),
            }
        });
        Ok(())
    }

    /// Write all the posts into one long index file.
    pub fn index<A: AsRef<Path>>(&self, posts: &[Document], folder: A) -> Result<()> {
        let folder = self
            .config
            .folders
            .output_folder_path()
            .join(folder.as_ref());
        let path = folder.join("index.html");
        match self.template_writer.write_list(
            &self.context,
            &List {
                title: "Index",
                posts,
                pagination: Pagination {
                    current: 0,
                    next: None,
                    previous: None,
                },
                list_type: ListType::Index,
            },
            &path,
            &self.config,
        ) {
            Ok(_) => () /*println!("Wrote index: {:?}", &path)*/,
            Err(e) => println!("Could not write index {:?}: {:?}", &path, &e),
        };
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
        let folder = self
            .config
            .folders
            .output_folder_path()
            .join(folder.as_ref());
        let mut state: (Option<Page>, Option<Page>) = (None, None);
        let mut iter = posts.chunks(per_page).enumerate().peekable();
        while let Some((index, chunk)) = iter.next() {

            let (filename, title) = make_title(index);

            let (_, future_title) = make_title(index + 1);
            state.0 = iter.peek().map(|(index, chunk)| Page {
                title: future_title,
                index: *index,
                items: chunk.len(),
                path: filename.clone()
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
                &self.config,
            ) {
                Ok(_) => () /*println!("Wrote index: {:?}", &path)*/,
                Err(e) => println!("Could not write index {:?}: {:?}", &path, &e),
            }
            state.1 = Some(Page {
                title,
                index,
                items: chunk.len(),
                path: filename.clone(),
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
                Ok(_) => () /*println!("Wrote tag index: {:?}", &path)*/ ,
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
            self.chapter(&book, &book.chapters);

            let path = path.join("index.html");
            match self
                .template_writer
                .write_book(&self.context, &book, &path, &self.config)
                {
                    Ok(_) => () /*println!("Wrote '{:?}'", &path)*/,
                    Err(e) => println!("Could not write book {}: {:?}", &book.identifier, &e),
                }
        });
        Ok(())
    }

    pub fn chapter(&self, book: &Book, chapters: &[Chapter]) -> Result<()> {
        for chapter in chapters {
            let output_path = std::path::PathBuf::from(&self.config.folders.output_folder).join(&chapter.slug);
            match self.template_writer.write_chapter(&self.context, &book, &chapter, &output_path, &self.config) {
                Ok(_) => () /*println!("write chapter to: {:?}", &chapter.slug)*/,
                Err(e) => println!("Could not write {}: {}", &chapter.name, &e)
            };
            if !chapter.sub_chapters.is_empty() {
                self.chapter(&book, &chapter.sub_chapters)?;
            }
        }
        Ok(())
    }
}
