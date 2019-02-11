use chrono::Datelike;
use pulldown_cmark::{html, Event, Parser};
use rayon::prelude::*;
use serde_derive::Serialize;

use crate::config::Config;
use crate::error::{Result, TechouError};
use crate::front_matter::{parse_front_matter, FrontMatter};
use crate::parse_event_handlers::{
    highlight::HighlightEventHandler, section::SectionEventHandler, EventHandler, ParseResult,
};
use crate::utils;

use std::path::Path;

#[derive(Serialize, Debug)]
pub struct SimilarDocument {
    pub identifier: String,
    pub title: String,
    pub desc: String,
    pub slug: String,
    pub score: u32,
}

#[derive(Serialize, Debug)]
pub struct Document {
    pub identifier: String,
    pub filename: String,
    pub info: FrontMatter,
    pub slug: String,
    pub content: String,
    pub sections: Vec<(i32, String)>,
    pub similar_documents: Vec<SimilarDocument>,
}

impl AsRef<Document> for Document {
    #[inline]
    fn as_ref(&self) -> &Document {
        self
    }
}

impl Document {
    pub fn new<A: AsRef<Path>>(contents: &str, path: A, config: &Config) -> Result<Document> {
        let filename = path
            .as_ref()
            .file_name()
            .and_then(|e| e.to_str())
            .ok_or(TechouError::Other {
                issue: format!("Path {:?} has no filename. Can't read it.", path.as_ref()),
            })?
            .to_string();
        let identifier = utils::hash_string(&filename, 8);
        let (info, article) = parse_front_matter(&contents, &path.as_ref(), &config)?;
        let slug = slug_from_frontmatter(&info);
        let ParseResult { content, sections } = markdown_to_html(article);
        Ok(Document {
            identifier,
            filename,
            info,
            slug,
            content,
            sections,
            similar_documents: Vec::new(),
        })
    }
}

pub fn documents_in_folder<A: AsRef<Path>>(folder: A, config: &Config) -> Result<Vec<Document>> {
    use crate::io_utils::{contents_of_directory, slurp};
    let files = contents_of_directory(folder.as_ref(), "md")?;
    let posts: Vec<Document> = files
        .par_iter()
        .filter_map(|path| {
            let contents = match slurp(path) {
                Ok(c) => c,
                Err(e) => {
                    println!("Can't read {:?}: {:?}", &path, &e);
                    return None;
                }
            };
            let post = match Document::new(&contents, &path, &config) {
                Ok(a) => a,
                Err(e) => {
                    println!("Invalid Format {:?}: {:?}", &path, &e);
                    return None;
                }
            };
            Some(post)
        })
        .collect();
    Ok(posts)
}

fn slug_from_frontmatter(front_matter: &FrontMatter) -> String {
    // If we have a slug in the meta attributes, use that (Document this!)
    if let Some(slug) = front_matter.meta.get("slug") {
        return slug.clone();
    }
    // make lowercase ascii-only title
    let title = utils::slugify(&front_matter.title);
    let d = &front_matter.date;
    format!("{}-{}-{}-{}.html", d.year(), d.month(), d.day(), title)
}

// Transform the AST of the markdown to support custom markdown constructs
fn markdown_to_html(markdown: &str) -> ParseResult {
    let parser = Parser::new(markdown);
    let mut events: Vec<Event> = Vec::new();
    let mut result = ParseResult {
        content: String::new(),
        sections: Vec::new(),
    };

    let mut handlers: Vec<Box<dyn EventHandler>> = vec![
        Box::new(SectionEventHandler::new()),
        Box::new(HighlightEventHandler::new()),
    ];

    for event in parser {
        let mut ignore_event = false;
        for handler in handlers.iter_mut() {
            if handler.handle(&event, &mut result, &mut events) == false {
                ignore_event = true;
            }
        }
        if !ignore_event {
            events.push(event);
        }
    }
    html::push_html(&mut result.content, events.into_iter());
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sections() {
        use crate::document;
        let contents = r#"
# Section 1
Hello world
## Section 2
More text
## Another section
# Final section"#;
        let result = document::markdown_to_html(&contents);
        assert_eq!(result.sections.len(), 4);
        assert_eq!(result.sections[0].1, "Section 1");
    }

    #[test]
    fn test_naming() {
        use crate::document;
        use crate::front_matter;
        let contents = r#"
[frontMatter]
title = "Hello World"
tags = ["first tag", "second tag"]
created = "2009-12-30"
description = "A run around the world"
published = true
[meta]
---
this is the actual article contents yeah."#;
        let (frontmatter, _) =
            front_matter::parse_front_matter(&contents, "yeah.md", &Default::default()).unwrap();
        let slug = document::slug_from_frontmatter(&frontmatter);
        assert_eq!(slug, "2009-12-30-hello-world.html");
    }

    #[test]
    fn test_syntax() {
        use crate::document;
        let contents = r#"
# Section 1
`printf()`

more code
``` Rust
if let Some(x) = variable {
  println!("{}", &x);
}

"#;
        let result = document::markdown_to_html(&contents);
        // Test for the CSS classes
        assert!(result.content.contains("source rust"));
    }
}
