use serde::Serialize;
use tera::Tera;

use crate::config::Config;
use crate::document::Document;
use crate::book::{Book, Chapter};
use crate::error::*;
use crate::io_utils::spit;
use crate::list::*;
use crate::filters;
use crate::utils::{slugify, hash_string};

use std::path::Path;
use std::collections::{HashMap, BTreeMap};

struct UrlMaker {
    urls: BTreeMap<String, String>,
    context: String
}

impl UrlMaker {
    fn new(urls: BTreeMap<String, String>, context: &str) -> Self {
        UrlMaker { urls, context: context.to_string() }
    }
}

impl tera::Function for UrlMaker {
    fn call(&self, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let id = match args.get("id") {
            Some(val) => match tera::from_value::<String>(val.clone()) {
                Ok(v) =>  v,
                Err(_) => return Err(tera::Error::msg("Parameter not found")),
            },
            None => return Err(tera::Error::msg("Parameter not found")),
        };
        let entry = match self.urls.get(&id) {
            Some(v) => v,
            None => return Err(tera::Error::msg(format!("No entity with id `{}` in {}", &id, &self.context)))
        };
        tera::to_value(entry).map_err(|e| e.into())
    }
}

struct IdentifierHash;
impl tera::Function for IdentifierHash {
    fn call(&self, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let filename = match args.get("filename") {
            Some(val) => match tera::from_value::<String>(val.clone()) {
                Ok(v) =>  v,
                Err(_) => return Err(tera::Error::msg("Parameter not found")),
            },
            None => return Err(tera::Error::msg("Parameter not found")),
        };
        tera::Result::Ok(tera::Value::String(hash_string(&filename,  8)))
        
    }
}

fn recusive_chapter_urls(into_collection: &mut std::collections::BTreeMap<String, String>, chapter: &Chapter, config: &Config) {
    into_collection.insert(chapter.document.identifier.clone(), format!("/{}", chapter.slug));
    for sub_chapter in chapter.sub_chapters.iter() {
        into_collection.insert(sub_chapter.document.identifier.clone(),
                               format!("/{}", sub_chapter.slug));
        if !sub_chapter.sub_chapters.is_empty() {
            recusive_chapter_urls(into_collection, &sub_chapter, &config);
        }
    }
}

pub struct Templates {
    tera: Tera,
}
#[derive(Serialize, Debug)]
struct TemplateContext<'a, T>
where
    T: Serialize,
{
    config: &'a Config,
    context: &'a DocumentContext<'a>,
    content: &'a T,
}

impl Templates {
    pub fn new<A: AsRef<Path>>(directory: A) -> Result<Templates> {
        let folder_path = directory
            .as_ref()
            .to_str()
            .expect("Could not find template folder")
            .to_owned()
            // Not sure why, but tera doesn't load template paths begining with `./`
            .replace("./", "");
        let mut tera = Tera::new(&format!("{}/*.html", folder_path)).ctx(&folder_path)?;
        // We don't want to escape content. After all, this is a static engine
        tera.autoescape_on(vec![]);
        tera.register_filter("chunks", filters::chunks::Chunk);
        tera.register_filter("split", filters::split::Split);
        Ok(Templates { tera })
    }

    pub fn register_url_functions(&mut self, context: &DocumentContext, config: &Config) {
        let post_urls: std::collections::BTreeMap<String, String> = context.all_posts.iter()
            .map({ |d|
                // FIXME: Instead, make sure all slugs always start with / ! (i.e. are absolute)
                if let Some('/') = &d.slug.chars().nth(0) { 
                    (d.identifier.clone(), d.slug.clone())
                } else {
                    (d.identifier.clone(), format!("/{}", &d.slug))
                }
            }).collect();
        self.tera.register_function("url_post", UrlMaker::new(post_urls, "url_post"));

        let page_urls: std::collections::BTreeMap<String, String> = context.pages.iter()
            .map(|d|(d.identifier.clone(), d.slug.clone())).collect();
        self.tera.register_function("url_page", UrlMaker::new(page_urls, "url_page"));

        let tag_urls: std::collections::BTreeMap<String, String> = context.by_tag.iter()
            .map(|t| (t.name.to_string(), format!("/{}/{}.html", config.folders.tags_folder_name, &slugify(&t.name)))).collect();
        self.tera.register_function("url_tag", UrlMaker::new(tag_urls, "url_tag"));

        let keyword_urls: std::collections::BTreeMap<String, String> = context.by_keyword.iter()
            .map(|t| (t.name.to_string(), format!("/{}/{}.html", config.folders.keywords_folder_name, &slugify(&t.name)))).collect();
        self.tera.register_function("url_keyword", UrlMaker::new(keyword_urls, "url_keyword"));

        let category_urls: std::collections::BTreeMap<String, String> = context.by_category.iter()
            .map(|t| (t.name.to_string(), format!("/{}/{}.html", config.folders.category_folder_name, &slugify(&t.name)))).collect();
        self.tera.register_function("url_category", UrlMaker::new(category_urls, "url_category"));

        fn identifier_hash(i: &str) -> String {
            hash_string(i,  8)
        }
        self.tera.register_function("identifier_hash", IdentifierHash);

        let mut book_urls: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
        let mut chapter_urls: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
        for book in context.books.iter() {
            book_urls.insert(book.identifier.clone(), format!("/{}", book.slug));
            for chapter in book.chapters.iter() {
                recusive_chapter_urls(&mut chapter_urls, &chapter, &config);
            }
        }
        self.tera.register_function("url_chapter", UrlMaker::new(chapter_urls, "url_chapter"));
        self.tera.register_function("url_book", UrlMaker::new(book_urls, "url_book"));
    }

    pub fn write_post<'a, A: AsRef<Path>>(
        &self,
        context: &DocumentContext<'a>,
        post: &Document,
        path: A,
        config: &Config,
    ) -> Result<()> {
        let item = TemplateContext {
            config,
            context,
            content: post,
        };
        self.write_item(&config.templates.post_template, &item, path, config)
    }

    pub fn write_page<'a, A: AsRef<Path>>(
        &self,
        context: &DocumentContext<'a>,
        page: &Document,
        path: A,
        config: &Config,
    ) -> Result<()> {
        let item = TemplateContext {
            config,
            context,
            content: page,
        };
        self.write_item(&config.templates.page_template, &item, &path, config)
    }

    pub fn write_book<'a, A: AsRef<Path>>(
        &self,
        context: &DocumentContext<'a>,
        book: &Book,
        path: A,
        config: &Config,
    ) -> Result<()> {
        let item = TemplateContext {
            config,
            context,
            content: book,
        };
        self.write_item(&config.templates.book_template, &item, &path, config)
    }

    pub fn write_chapter<'a, A: AsRef<Path>>(
        &self,
        context: &DocumentContext<'a>,
        book: &Book,
        chapter: &Chapter,
        path: A,
        config: &Config,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct ChapterContext<'a> {
            book: &'a Book,
            chapter: &'a Chapter
        }
        let content = ChapterContext {
            book,
            chapter
        };
        let item = TemplateContext {
            config,
            context,
            content: &content,
        };
        self.write_item(&config.templates.chapter_template, &item, &path, config)
    }

    pub fn write_list<'a, A: AsRef<Path>, D: AsRef<Document>>(
        &self,
        context: &DocumentContext<'a>,
        list: &'a List<'a, D>,
        path: A,
        config: &Config,
    ) -> Result<()>
    where
        D: Serialize,
    {
        let item = TemplateContext {
            config,
            context,
            content: list,
        };
        self.write_item(&config.templates.list_template, &item, path, config)
    }

    pub fn write_year<'a, A: AsRef<Path>, D: AsRef<Document>>(
        &self,
        context: &DocumentContext<'a>,
        list: &'a List<'a, D>,
        path: A,
        config: &Config,
    ) -> Result<()>
    where
        D: Serialize,
    {
        let item = TemplateContext {
            config,
            context,
            content: list,
        };
        self.write_item(&config.templates.year_template, &item, path, config)
    }

    fn write_item<'a, A: AsRef<Path>, I: Serialize>(
        &self,
        template_name: &str,
        item: &'a I,
        path: A,
        _config: &Config,
    ) -> Result<()> {
        let context = tera::Context::from_serialize(&item)
        .map_err(|e| {
            crate::error::TechouError::Templating {
                source: e,
                context: "Write item".to_owned()
            }
        })?;
        let rendered = self.tera.render(template_name, &context).ctx(path.as_ref())?;
        spit(path.as_ref(), &rendered)
    }
}
