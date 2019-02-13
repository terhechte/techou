use serde::Serialize;
use serde_derive::Serialize;
use tera::Tera;

use crate::config::Config;
use crate::document::Document;
use crate::error::*;
use crate::io_utils::spit;
use crate::list::*;
use crate::server::auto_reload_code;
use crate::filters;

use std::path::Path;

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
        Ok(Templates { tera })
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
