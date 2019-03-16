use rayon::prelude::*;
use pulldown_cmark::{Event, Parser, Tag};
use serde_derive::Serialize;

use crate::document::Document;
use crate::error::Result;
use crate::io_utils::slurp;
use crate::front_matter::*;
use crate::config::Config;
use std::path::PathBuf;

#[derive(Serialize, Debug)]
pub struct Book {
    pub identifier: String,
    pub slug: String,
    pub folder: String,
    pub info: FrontMatter,
    pub chapters: Vec<Chapter>,
    pub complete_book: Option<Document>
}

impl Book {
    pub fn new<A: AsRef<std::path::Path>>(file: A, config: &Config) -> Result<Book> {
        let contents = slurp(&config.folders.books_folder_path().join(&file))?;
        let (info, md) = parse_front_matter(&contents, &file, &config)?;
        let folder = file.as_ref().parent().expect("Expect the path for the book to have a parent folder");
        let book_folder = std::path::PathBuf::from(&config.folders.books_folder_name).join(folder);
        let chapter_info = parse_chapter(md, &config.folders.books_folder_path().join(&folder),
                                         &book_folder);
        let chapters: Vec<Chapter> = chapter_info.into_par_iter().filter_map(|c| match c.convert(&config) {
            Ok(s) => Some(s),
            Err(e) => {
                println!("{:?}", &e);
                None
            }
        }).collect();
        // A book needs chapters
        if chapters.is_empty() {
            return Err(crate::error::TechouError::Other {
                issue: format!("Empty book {} will not be included", &info.title)
            });
        }
        let mut book = Book {
            identifier: format!("{:?}", file.as_ref()),
            slug: chapters[0].slug.clone(),
            folder: file.as_ref().parent().expect("Proper book path").to_str().expect("Proper book path").to_string(),
            info,
            chapters,
            complete_book: None
        };

        if config.project.render_one_page_books {
            let complete_book = book.as_one_document();
            book.complete_book = Some(complete_book);
        }

        Ok(book)
    }

    /// Render the whole book (i.e. all chapters) as one one document
    /// This is currently a not-so-nice solution.
    /// It writes all the html together into one document with the
    /// frontMatter of the original document
    /// Then, it merges them with <h1> headlines
    pub fn as_one_document(&self) -> Document {
        let mut buffer: String = String::new();
        let mut sections: Vec<(String, String)> = Vec::new();
        Book::recursive_add(&self.chapters, &mut buffer, &mut sections);

        let slug_path = PathBuf::from(&self.slug);
        let parent = slug_path.parent().expect("Expect a parent for a book");
        let filename = "complete_book.html";
        let slug = parent.join(&filename);
        let slug = slug.to_str().expect("Proper String");
        Document::from_multiple(
            buffer,
            "",
            slug,
            &filename,
            &self.info,
            sections
        )
    }

    fn recursive_add(chapters: &Vec<Chapter>, into_buffer: &mut String, sections: &mut Vec<(String, String)>) {
        //let mut counter = 1;
        for chapter in chapters.iter() {
            // This is not needed as chapters always start with their name anyway
            //into_buffer.push_str(&format!("<h1 id=\"header-section-{}\">{}</h1>", &counter, chapter.name));
            //sections.push((format!("header-section-{}", counter), chapter.name.clone()));
            //counter += 1;
            into_buffer.push_str(&chapter.document.content);
            let mut cloned = chapter.document.sections.clone();
            sections.append(&mut cloned);
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ChapterLink {
    pub name: String,
    pub slug: String
}

#[derive(Serialize, Debug)]
pub struct Chapter {
    pub name: String,
    pub slug: String,
    pub file_url: std::path::PathBuf,
    pub level: usize,
    pub document: Document,
    pub sub_chapters: Vec<Chapter>,
    pub next: Option<ChapterLink>,
    pub previous: Option<ChapterLink>,
    pub parent: Option<ChapterLink>
}

#[derive(Default, Debug)]
/// A chapter without any loaded document
pub struct ChapterInfo {
    pub name: String,
    pub level: usize,
    // The file this was read from
    pub file_url: std::path::PathBuf,
    pub sub_chapters: Vec<ChapterInfo>,
    // The slug / absolute address to locate this on the server (with subfolders)
    pub slug: String,
    // The absolute filename this will be written to on disk
    pub output: String,
    // The slug of the next chapter
    pub next: Option<ChapterLink>,
    // The slug of the previosu chapter
    pub previous: Option<ChapterLink>,
    // The slug of the parent chapter
    pub parent: Option<ChapterLink>
}

impl ChapterInfo {
    fn convert(self, config: &Config) -> Result<Chapter> {
        let contents = slurp(&self.file_url)?;
        let mut doc = Document::new(&contents, &self.file_url, "", &config)?;
        doc.slug = self.slug.clone();
        let chapters: Vec<Chapter> = self.sub_chapters.into_par_iter().filter_map(|c| match c.convert(&config) {
            Ok(s) => Some(s),
            Err(e) => {
                println!("{:?}", &e);
                None
            }
        }).collect();
        Ok(Chapter {
            name: self.name,
            slug: self.slug,
            file_url: self.file_url,
            level: self.level,
            document: doc,
            sub_chapters: chapters,
            previous: self.previous,
            next: self.next,
            parent: self.parent
        })
    }
}

/// Return the path where the chapter will be written to (including folders) and the absolute link to it
fn make_link(chapter: &ChapterInfo) -> Option<ChapterLink> {
    Some(ChapterLink {
        name: chapter.name.clone(),
        slug: chapter.slug.clone(),
    })
}

/// `in_folder` is the folder where the md file was loaded from. (i.e. /books/book1/ for /books/book1/summary.toml)
/// `out_folder` is the absolute base folder for html (i.e. `/book1/` for `/book1/index.html` or `/books/book1/` for `/books/book1/index.html`)
pub fn parse_chapter<A: AsRef<std::path::Path>, B: AsRef<std::path::Path>>(content: &str, in_folder: A, out_folder: B) -> Vec<ChapterInfo> {
    // A non-recursive parsing of a tree data structure
    let mut parser = Parser::new(&content);
    let mut chapter_stack: Vec<ChapterInfo> = vec![Default::default()];
    let mut last_chapter_link: Option<ChapterLink> = None;
    for event in parser {
        match event {
            Event::Start(Tag::Item) => {
                let mut chapter: ChapterInfo = Default::default();
                chapter.level = chapter_stack.len();
                chapter.previous = last_chapter_link.clone();
                chapter_stack.last_mut().map(|c| c.sub_chapters.push(chapter));
            },
            Event::End(Tag::Item) => {
                // We always have at least one in the stack, so this will never underflow
                let idx = chapter_stack.len() - 1;
                // We just inserted one in 'start item' so this should never underflow
                let uidx = chapter_stack[idx].sub_chapters.len() - 1;

                // The parent is always the one on the chapter stack
                chapter_stack[idx].sub_chapters[uidx].parent = make_link(&chapter_stack[idx]);

                // if we're the first in the sub-chapters, the previous is the parent
                if uidx == 0 {
                    chapter_stack[idx].sub_chapters[uidx].previous = make_link(&chapter_stack[idx]);
                    chapter_stack[idx].next = make_link(&chapter_stack[idx].sub_chapters[uidx]);
                } else {
                    // if we already have a next, then don't set it. Otherwise we would override
                    // the next set from the first item in the sub_chapter
                    if chapter_stack[idx].sub_chapters[uidx - 1].next.is_none() {
                        chapter_stack[idx].sub_chapters[uidx - 1].next = make_link(&chapter_stack[idx].sub_chapters[uidx]);
                    }
                }
            },
            Event::Start(Tag::List(_)) => {
                if let Some(cur) = chapter_stack.last_mut() {
                    if let Some(sb) = cur.sub_chapters.pop() {
                        chapter_stack.push(sb);
                    }
                }
            }
            Event::End(Tag::List(_)) if chapter_stack.len() > 1 =>
                if let Some(mut chapter) = chapter_stack.pop() {
                    if let Some(sb) = chapter_stack.last_mut() {
                        sb.sub_chapters.push(chapter);
                    }
                },
            Event::Start(Tag::Link(url, _)) => {
                let path = out_folder.as_ref().join(&url.to_string());
                // FIXME: Set out_folder + url as slug for chapter_stack.last
                chapter_stack.last_mut().map(|c| {
                    c.sub_chapters.last_mut().map(|c2| {
                        c2.slug = path.to_str().unwrap().replace(".md", ".html").to_string();
                        c2.file_url = in_folder.as_ref().join(&url.to_string());
                    })
                });
            },
            Event::End(Tag::Link(url, _)) => {
                if let Some(item) = chapter_stack.last() {
                    if let Some(inner) = item.sub_chapters.last() {
                        last_chapter_link = make_link(&inner);
                    }
                }
            }
            Event::Text(text) => {
                chapter_stack.last_mut().map(|c|c.sub_chapters.last_mut().map(|c2| c2.name = text.to_string()));
            },
            _ => ()
        }
    }
    if chapter_stack.is_empty() {
        return Vec::new();
    }
    let mut chapters = chapter_stack.pop().take().unwrap().sub_chapters;
    // remove the artificial parent
    for chapter in &mut chapters {
        chapter.parent = None;
    }
    chapters.first_mut().map(|x| x.previous = None);
    chapters.last_mut().map(|x| x.next = None);

    chapters
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_book_summary() {
        let content = r#"- [Intro](test1/test1.md)
- [Another](test/another.md)
    - [Level2.1](test2/test2.md)
    - [Level2.2](test2/test3.md)
        - [Level3.1](test2/test3/test1.md)
        - [Level3.2](test2/test3/test2.md)
- [Final](final/final.md)
"#;
        let r = parse_chapter(&content, "/home/books", "/html/book");
        assert_eq!(r.len(), 3);
        assert_eq!(r[1].sub_chapters.len(), 2);
        assert_eq!(r[0].next.as_ref().unwrap().name, "Another");
        assert_eq!(r[1].next.as_ref().unwrap().name, "Level2.1");
        assert_eq!(r[1].previous.as_ref().unwrap().name, "Intro");
        assert_eq!(r[1].sub_chapters[0].previous.as_ref().unwrap().name, "Another");
        assert_eq!(r[1].sub_chapters[0].next.as_ref().unwrap().name, "Level2.2");
        assert_eq!(r[1].sub_chapters[1].parent.as_ref().unwrap().name, "Another");
        assert_eq!(r[1].sub_chapters[0].parent.as_ref().unwrap().name, "Another");
        assert_eq!(r[2].previous.as_ref().unwrap().name, "Level3.2");
    }
}
