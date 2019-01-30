use std::env;
use std::path::PathBuf;
use serde_derive::{Deserialize};

use crate::io_utils::slurp;
use crate::error::Result;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RSS {
    #[serde(default)]
    pub absolute_feed_address: String, // the absolute URL of the feed
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author_email: String,
    #[serde(default)]
    pub author_name: String,
}

#[derive(Deserialize)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// The root folder of the project
    #[serde(skip)]
    pub root: PathBuf,

    /// Folders on Disk
    pub posts_folder: String,
    pub output_folder: String,
    pub public_folder: String,
    pub public_copy_folders: Vec<String>,

    /// Folder names in the generated structure
    pub articles_folder_name: String,
    pub tags_folder_name: String,

    /// Templates
    pub article_template: String,
    pub list_template: String,

    /// Date Formats
    pub date_format: String,
    pub date_time_format: String,
    pub output_date_time_format: String,

    /// Server configuration
    pub server_port: i32,
    // Insert websocket javascript to automatically reload
    // when a change is detected
    pub auto_reload_browser_via_websocket_on_change: bool,
    pub auto_reload_websocket_path: String,

    /// RSS
    #[serde(default)]
    #[serde(alias = "RSS")]
    pub rss_settings: Option<RSS>
}

impl Config {

    pub fn new<A: AsRef<std::path::Path>>(folder: A) -> Config {
        let mut config: Config = Default::default();
        config.root = folder.as_ref().to_path_buf();
        config
    }

    pub fn toml(input: &str, in_folder: &PathBuf) -> Result<Config> {
        let mut config: Config = toml::from_str(&input)?;
        config.root = in_folder.clone();
        Ok(config)
    }

    pub fn file<A: AsRef<std::path::Path>>(toml_file: A) -> Result<Config> {
        let parent = match &toml_file.as_ref().parent() {
            Some(root) => root.to_path_buf(),
            None => panic!("The toml file {:?} is invalid. No Parent Folder.", &toml_file.as_ref())
        };
        let contents = slurp(toml_file)?;
        Config::toml(&contents, &parent)
    }

    pub fn posts_folder_path(&self) -> PathBuf {
        self.root.join(&self.posts_folder)
    }
    pub fn output_folder_path(&self) -> PathBuf {
        self.root.join(&self.output_folder)
    }

    pub fn articles_folder_path(&self) -> PathBuf {
        self.output_folder_path().join(&self.articles_folder_name)
    }

    pub fn tags_folder_path(&self) -> PathBuf {
        self.output_folder_path().join(&self.tags_folder_name)
    }

    pub fn public_folder_path(&self) -> PathBuf {
        self.root.join(&self.public_folder)
    }
}

impl Default for Config {
    fn default() -> Self {
        let root = env::current_dir().expect("something is rotten in the state of your disk. No Current Dir found.");
        Config {
            root,
            posts_folder: "posts".to_string(),
            output_folder: "html".to_string(),
            public_folder: "public".to_string(),
            public_copy_folders: vec!["css".to_string(), "img".to_string()],

            articles_folder_name: "articles".to_string(),
            tags_folder_name: "tags".to_string(),

            article_template: "article.html".to_string(),
            list_template: "list.html".to_string(),

            date_format: "%Y-%m-%d".to_string(),
            date_time_format: "%Y-%m-%d %H:%M:%S".to_string(),
            output_date_time_format: "%Y-%m-%d %H:%M:%S".to_string(),

            server_port: 8001,
            auto_reload_browser_via_websocket_on_change: true,
            auto_reload_websocket_path: "ws".to_string(),

            rss_settings: None
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_from_toml() {
        use crate::config::Config;
        let contents = r#"
postsFolder = "jochen/"
outputFolder = "franz/"
"#;
        let parsed = Config::toml(&contents, &std::path::PathBuf::from("/tmp/test.toml")).unwrap();
        assert_eq!(parsed.posts_folder, "jochen/");

    }

    #[test]
    fn test_parse_rss() {
        use crate::config::Config;
        let contents = r#"
postsFolder = "jochen/"
outputFolder = "franz/"

[RSS]
title = "klaus"
"#;
        let parsed = Config::toml(&contents, &std::path::PathBuf::from("/tmp/test.toml")).unwrap();
        assert!(parsed.rss_settings.is_some());
        assert_eq!(parsed.rss_settings.unwrap().title, "klaus");
    }
}
