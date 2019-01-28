use std::collections::HashMap;
use std::path::Path;

use crate::front_matter::{parse_front_matter, FrontMatter};
use crate::error::{Result, Error};
use crate::config::Config;
use crate::utils;
use crate::parse_event_handlers::{section::SectionEventHandler, highlight::HighlightEventHandler, EventHandler, ParseResult};

use chrono::Datelike;
use pulldown_cmark::{Event, Parser, Tag, html};
use serde_derive::{Serialize};

#[derive(Serialize, Debug)]
pub struct Article {
    pub identifier: String,
    pub filename: String,
    pub info: FrontMatter,
    pub slug: String,
    pub content: String,
    pub sections: Vec<(i32, String)>
}

impl Article {
    pub fn new<A: AsRef<Path>>(contents: &str, path: A, config: &Config) -> Result<Article> {
        let filename = path.as_ref().file_name().and_then(|e| e.to_str())
            .ok_or(Error::Other(format!("Path {:?} has no filename. Can't read it.", path.as_ref())))?
            .to_string();
        let identifier = utils::hash_string(&filename, 8);
        let (info, article) = parse_front_matter(&contents, &path.as_ref(), &config)?;
        let slug = slug_from_frontmatter(&info);
        let ParseResult { content, sections } = markdown_to_html(article);
        Ok(Article { identifier, filename, info, slug, content, sections } )
    }
}

fn slug_from_frontmatter(front_matter: &FrontMatter) -> String {
    // make lowercase ascii-only title
    let title: String = front_matter.title.to_lowercase()
        .replace(|c: char| !c.is_ascii_alphanumeric() && !c.is_ascii_whitespace(), "")
        .split_whitespace().collect::<Vec<&str>>().join("-");
    let d = &front_matter.date;
    format!("{}-{}-{}-{}.html", d.year(), d.month(), d.day(), title)
}



// Transform the AST of the markdown to support custom markdown constructs
fn markdown_to_html(markdown: &str) -> ParseResult {
    let parser = Parser::new(markdown);
    let mut events: Vec<Event> = Vec::new();
    let mut result = ParseResult {
        content: String::new(),
        sections: Vec::new()
    };

    let mut handlers: Vec<Box<dyn EventHandler>> = vec![
        Box::new(SectionEventHandler::new()),
        Box::new(HighlightEventHandler::new())
    ];

    for event in parser {
        println!("event: {:?}", &event);
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
        use crate::article;
        let contents = r#"
# Section 1
Hello world
## Section 2
More text
## Another section
# Final section"#;
        let result = article::markdown_to_html(&contents);
        assert_eq!(result.sections.len(), 4);
        assert_eq!(result.sections[0].1, "Section 1");
    }

    #[test]
    fn test_naming() {
        use crate::front_matter;
        use crate::article;
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
        let (frontmatter, _) = front_matter::parse_front_matter(&contents, "yeah.md", &Default::default()).unwrap();
        let slug = article::slug_from_frontmatter(&frontmatter);
        assert_eq!(slug, "2009-12-30-hello-world.html");
    }

    #[test]
    fn test_syntax() {
        use crate::article;
        let contents = r#"
# Section 1
`printf()`

more code
``` Rust
if let Some(x) = variable {
  println!("{}", &x);
}

"#;
        let result = article::markdown_to_html(&contents);
        println!("{}", result.content);
    }
}