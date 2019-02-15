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

use std::path::Path;

fn make_url_for(urls: std::collections::BTreeMap<String, String>) -> tera::GlobalFn {
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
            None => return Err(format!("No entity with id `{}` found", &id).into()),
        };
        tera::to_value(entry).map_err(|e| e.into())
    })
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
        let post_urls: std::collections::BTreeMap<String, String> = context.posts.iter()
            .map(|d|(d.identifier.clone(), format!("/{}/{}", config.folders.posts_folder_name, d.slug))).collect();
        self.tera.register_function("url_post", make_url_for(post_urls));
        let page_urls: std::collections::BTreeMap<String, String> = context.pages.iter()
            .map(|d|(d.identifier.clone(), format!("/{}/{}", config.folders.pages_folder_name, d.slug))).collect();
        self.tera.register_function("url_page", make_url_for(page_urls));
        let tag_urls: std::collections::BTreeMap<String, String> = context.by_tag.iter()
            .map(|t| (t.name.to_string(), format!("/{}/{}", config.folders.tags_folder_name, &t.name))).collect();
        self.tera.register_function("url_tag", make_url_for(tag_urls));
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
        if config.server.auto_reload_browser_via_websocket_on_change {
            rendered.push_str(&auto_reload_code(&config));
        }
        spit(path.as_ref(), &rendered)
    }
}
