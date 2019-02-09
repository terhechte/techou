use std::path::Path;

use tera::{Tera, Context};
use serde::Serialize;

use crate::error::*;
use crate::document::Document;
use crate::list::List;
use crate::io_utils::spit;
use crate::config::Config;
use crate::server::auto_reload_code;

pub struct Templates {
    tera: Tera
}

use toml::Value;

impl Templates {
    pub fn new<A: AsRef<Path>>(directory: A) -> Result<Templates> {
        let folder_path = directory.as_ref().to_str().expect("Could not find template folder");
        let mut tera = Tera::new(&format!("{}/*.html", folder_path))
            .ctx(&folder_path)?;
        // We don't want to escape content. After all, this is a static engine
        tera.autoescape_on(vec![]);
        Ok(Templates {
            tera
        })
    }

    pub fn write_post<A: AsRef<Path>>(&self, post: &Document, path: A, config: &Config) -> Result<()> {
        self.write_item(&config.templates.post_template, post, path, config)
    }

    pub fn write_page<A: AsRef<Path>>(&self, post: &Document, path: A, config: &Config) -> Result<()> {
        self.write_item(&config.templates.page_template, post, path, config)
    }

    pub fn write_list<'a, A: AsRef<Path>>(&self, list: &'a List<'a>, path: A, config: &Config)
        -> Result<()> {
        self.write_item(&config.templates.list_template, list, path, config)
    }

    fn write_item<'a, A: AsRef<Path>, I: Serialize>(&self, template_name: &str, item: &'a I, path: A, config: &Config)
    -> Result<()> {
        let mut rendered = self.tera.render(template_name, &item)
            .ctx(path.as_ref())?;
        if config.server.auto_reload_browser_via_websocket_on_change {
            rendered.push_str(&auto_reload_code(&config));
        }
        spit(path.as_ref(), &rendered)
    }
}
