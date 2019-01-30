use std::path::Path;

use tera::{Tera, Context, compile_templates};
use serde::Serialize;

use crate::error::Result;
use crate::article::Article;
use crate::list::List;
use crate::io_utils::spit;
use crate::config::Config;
use crate::server::auto_reload_code;

pub struct Templates {
    tera: Tera
}

use toml::Value;

impl Templates {
    pub fn new<A: AsRef<Path>>(directory: A) -> Templates {
        let folder_path = directory.as_ref().to_str().expect("Could not find template folder");
        let mut tera = compile_templates!(&format!("{}/*.html", folder_path));
        // We don't want to escape content. After all, this is a static engine
        tera.autoescape_on(vec![]);
        Templates {
            tera
        }
    }

    pub fn write_article<A: AsRef<Path>>(&self, article: &Article, path: A, config: &Config) -> Result<()> {
        let mut rendered = self.tera.render(&config.article_template, &article)?;
        // FIXME: Make this config dependent
        rendered.push_str(&auto_reload_code(&config));
        spit(path.as_ref(), &rendered)
    }

    pub fn write_list<'a, A: AsRef<Path>>(&self, list: &'a List<'a>, path: A, config: &Config) -> Result<()> {
        let rendered = self.tera.render(&config.list_template, &list)?;
        spit(path.as_ref(), &rendered)
    }
}
