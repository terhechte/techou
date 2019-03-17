use chrono::Datelike;
use rayon::prelude::*;
use serde_derive::Serialize;

use crate::config::Config;
use crate::error::{Result, TechouError};
use crate::front_matter::{parse_front_matter, FrontMatter};
use crate::utils;
use crate::markdown::*;

use std::path::Path;

#[derive(Serialize, Debug, Clone)]
pub struct DocumentLink {
    pub identifier: String,
    pub title: String,
    pub desc: String,
    pub slug: String
}

#[derive(Serialize, Debug, Clone)]
pub struct Document {
    pub identifier: String,
    pub filename: String,
    pub info: FrontMatter,
    pub slug: String,
    pub content: String,
    pub raw_content: String,
    pub sections: Vec<(String, String)>,
    pub similar_documents: Vec<(u32, DocumentLink)>,
    pub previous_document: Option<DocumentLink>,
    pub next_document: Option<DocumentLink>
}

impl AsRef<Document> for Document {
    #[inline]
    fn as_ref(&self) -> &Document {
        self
    }
}

impl Document {
    pub fn new<A: AsRef<Path>>(contents: &str, path: A, slug_base: &str, config: &Config) -> Result<Document> {
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
        let slug = slug_from_frontmatter(&info, slug_base);
        let ParseResult { content, sections } =
            markdown_to_html(article, &identifier,
                             &config.short_links,
            config.project.code_class_prefix.clone());
        let sections = sections.into_iter().map(|(number, title)| (format!("{}-{}", &identifier, &number), title)).collect();
        Ok(Document {
            identifier,
            filename,
            info,
            slug,
            content,
            raw_content: contents.to_string(),
            sections,
            similar_documents: Vec::new(),
            next_document: None,
            previous_document: None
        })
    }

    pub fn from_multiple(html_contents: String, partial_markdown_contents: &str, slug: &str, filename: &str, info: &FrontMatter, sections: Vec<(String, String)>) -> Document {
        Document {
            identifier: utils::hash_string(&slug, 4),
            filename: filename.to_string(),
            info: info.clone(),
            slug: slug.to_string(),
            content: html_contents,
            raw_content: partial_markdown_contents.to_string(),
            sections: sections,
            similar_documents: Vec::new(),
            next_document: None,
            previous_document: None
        }
    }
}

impl Document {
    pub fn link(&self) -> DocumentLink {
        DocumentLink {
            identifier: self.identifier.clone(),
            title: self.info.title.clone(),
            desc: self.info.description.clone(),
            slug: self.slug.clone()
        }
    }
}

pub fn documents_in_folder<A: AsRef<Path>>(folder: A, base: &str, config: &Config, cache: &crate::build_cache::BuildCache) -> Result<Vec<Document>> {
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

            let clone = cache.clone();
            let cache_key = &path.to_str().unwrap();
            if let Some(existing) = clone.get_item(cache_key, &contents) {
                return Some(existing)
            }

            let post = match Document::new(&contents, &path, &base, &config) {
                Ok(a) => a,
                Err(e) => {
                    println!("Invalid Format {:?}: {:?}", &path, &e);
                    return None;
                }
            };
            if post.info.published == false {
                return None;
            }
            cache.set_item(&cache_key, &post);
            Some(post)
        })
        .collect();
    Ok(posts)
}

fn slug_from_frontmatter(front_matter: &FrontMatter, slug_base: &str) -> String {
    if let Some(slug) = &front_matter.slug {
        return format!("/{}/{}", slug_base, slug);
    }
    // make lowercase ascii-only title
    let title = utils::slugify(&front_matter.title);
    let d = &front_matter.date;
    format!("/{}/{}-{}-{}-{}.html", slug_base, d.year(), d.month(), d.day(), title)
}

#[cfg(test)]
mod tests {
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
        let slug = document::slug_from_frontmatter(&frontmatter, "posts");
        assert_eq!(slug, "/posts/2009-12-30-hello-world.html");
    }

}
