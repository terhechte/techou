use serde::Serialize;
use serde_derive::Serialize;
use tera::Tera;

use crate::config::Config;
use crate::document::Document;
use crate::book::{Book, Chapter};
use crate::error::*;
use crate::io_utils::spit;
use crate::list::*;
use crate::server::auto_reload_code;
use crate::filters;
use crate::utils::slugify;

use std::path::Path;

fn make_url_for(urls: std::collections::BTreeMap<String, String>, context: &'static str) -> tera::GlobalFn {
    Box::new(move |args| -> tera::Result<tera::Value> {
        let id = match args.get("id") {
            Some(val) => match tera::from_value::<String>(val.clone()) {
                Ok(v) =>  v,
                Err(_) => return Err("id parameter not found".into()),
            },
            None => return Err("id parameter not found".into()),
        };
        let entry = match urls.get(&id) {
            Some(v) => v,
            None => return Err(format!("No entity with id `{}` found ({})", &id, &context).into()),
        };
        tera::to_value(entry).map_err(|e| e.into())
    })
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
            .expect("Could not find template folder");
        let mut tera = Tera::new(&format!("{}/*.html", folder_path)).ctx(&folder_path)?;
        // We don't want to escape content. After all, this is a static engine
        tera.autoescape_on(vec![]);
        tera.register_filter("chunks", filters::chunks::chunk);
        tera.register_filter("split", filters::split::split);
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
        self.tera.register_function("url_post", make_url_for(post_urls, "url_post"));

        let page_urls: std::collections::BTreeMap<String, String> = context.pages.iter()
            .map(|d|(d.identifier.clone(), d.slug.clone())).collect();
        self.tera.register_function("url_page", make_url_for(page_urls, "url_page"));

        let tag_urls: std::collections::BTreeMap<String, String> = context.by_tag.iter()
            .map(|t| (t.name.to_string(), format!("/{}/{}.html", config.folders.tags_folder_name, &slugify(&t.name)))).collect();
        self.tera.register_function("url_tag", make_url_for(tag_urls, "url_tag"));

        let keyword_urls: std::collections::BTreeMap<String, String> = context.by_keyword.iter()
            .map(|t| (t.name.to_string(), format!("/{}/{}.html", config.folders.keywords_folder_name, &slugify(&t.name)))).collect();
        self.tera.register_function("url_keyword", make_url_for(keyword_urls, "url_keyword"));

        let category_urls: std::collections::BTreeMap<String, String> = context.by_category.iter()
            .map(|t| (t.name.to_string(), format!("/{}/{}.html", config.folders.category_folder_name, &slugify(&t.name)))).collect();
        self.tera.register_function("url_category", make_url_for(category_urls, "url_category"));

        let mut book_urls: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
        let mut chapter_urls: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
        for book in context.books.iter() {
            book_urls.insert(book.identifier.clone(), format!("/{}", book.slug));
            for chapter in book.chapters.iter() {
                recusive_chapter_urls(&mut chapter_urls, &chapter, &config);
            }
        }
        self.tera.register_function("url_chapter", make_url_for(chapter_urls, "url_chapter"));
        self.tera.register_function("url_book", make_url_for(book_urls, "url_book"));
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

    fn write_item<'a, A: AsRef<Path>, I: Serialize>(
        &self,
        template_name: &str,
        item: &'a I,
        path: A,
        config: &Config,
    ) -> Result<()> {
        let mut rendered = self.tera.render(template_name, &item).ctx(path.as_ref())?;
        //if config.server.auto_reload_browser_via_websocket_on_change {
        //    rendered.push_str(&auto_reload_code(&config));
        //}
        spit(path.as_ref(), &rendered)
    }
}
