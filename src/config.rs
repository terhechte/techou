use serde_derive::{Deserialize, Serialize};

use crate::error::*;
use crate::io_utils::slurp;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;


static DEFAULT_PROJECT_TOML: &str = r#"
[Project]
keywords = ["nam", "nom", "grah"]

# How many posts per index (default: 8)
# postsPerIndex = 8

[Folders]
# Where are your posts
postsFolder = "posts"

# Where are additional pages (if you intend to write them)
# pagesFolder = "pages"

# Where should the static site be stored
outputFolder = "html"

# Where are the templates and public items
publicFolder = "public"

# The file and folders that should be copied over from within the public folder
publicCopyFolders = ["css", "img", "js"]

[Dates]
# The input date format that should be used for your posts and apges
dateFormat = "%Y-%m-%d"
# The input date time format. Has priority over the date format
# dateTimeFormat = "%Y-%m-%d %H:%M:%S"

[Server]
# On which address to run the dev server
serverAddress = "127.0.0.1:8001"

# [RSS]
# absoluteFeedAddress = "https://example.com/feed.rss"
# title = "My Blog"
# authorEmail = "john@doe.com"
# authorName = "John Doe"

# This is where you can add additional meta information.
# they're available in all templates
# [Meta]
# twitter = "https://twitter.com/johndoe"
"#;

fn default_posts_per_index() -> u32 {
    8
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct ConfigProject {
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_posts_per_index")]
    pub posts_per_index: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct ConfigFolders {
    /// The root folder of the project
    #[serde(skip)]
    pub root: PathBuf,

    /// Folders on Disk
    pub posts_folder: String,
    pub pages_folder: String,
    pub output_folder: String,
    pub public_folder: String,
    pub public_copy_folders: Vec<String>,

    /// Name of book folders including the summary toml file
    pub books: Vec<String>,

    /// Folder names in the generated structure
    pub posts_folder_name: String,
    pub tags_folder_name: String,
    pub pages_folder_name: String,
    pub books_folder_name: String,
}

impl ConfigFolders {
    pub fn posts_folder_path(&self) -> PathBuf {
        self.root.join(&self.posts_folder)
    }
    pub fn pages_folder_path(&self) -> PathBuf {
        self.root.join(&self.pages_folder)
    }
    pub fn output_folder_path(&self) -> PathBuf {
        self.root.join(&self.output_folder)
    }

    pub fn output_posts_folder_path(&self) -> PathBuf {
        self.output_folder_path().join(&self.posts_folder_name)
    }

    pub fn output_pages_folder_path(&self) -> PathBuf {
        self.output_folder_path().join(&self.pages_folder_name)
    }

    pub fn tags_folder_path(&self) -> PathBuf {
        self.output_folder_path().join(&self.tags_folder_name)
    }

    pub fn books_folder_path(&self) -> PathBuf {
        self.output_folder_path().join(&self.books_folder_name)
    }

    pub fn public_folder_path(&self) -> PathBuf {
        self.root.join(&self.public_folder)
    }
}

impl Default for ConfigFolders {
    fn default() -> Self {
        let root = env::current_dir()
            .expect("something is rotten in the state of your disk. No Current Dir found.");
        ConfigFolders {
            root,
            posts_folder: "posts".to_string(),
            pages_folder: "pages".to_string(),
            output_folder: "html".to_string(),
            public_folder: "public".to_string(),
            public_copy_folders: vec!["css".to_string(), "img".to_string()],

            books: Vec::new(),

            posts_folder_name: "posts".to_string(),
            pages_folder_name: "pages".to_string(),
            books_folder_name: "books".to_string(),
            tags_folder_name: "tags".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct ConfigTemplates {
    pub post_template: String,
    pub page_template: String,
    pub list_template: String,
    pub book_template: String,
    pub chapter_template: String,
}

impl Default for ConfigTemplates {
    fn default() -> Self {
        ConfigTemplates {
            post_template: "post.html".to_string(),
            page_template: "page.html".to_string(),
            list_template: "list.html".to_string(),
            book_template: "book.html".to_string(),
            chapter_template: "chapter.html".to_string(),
        }
    }
}

impl Default for ConfigDates {
    fn default() -> Self {
        ConfigDates {
            date_format: "%Y-%m-%d".to_string(),
            date_time_format: "%Y-%m-%d %H:%M:%S".to_string(),
            output_date_time_format: "%Y-%m-%d %H:%M:%S".to_string(),
        }
    }
}

impl Default for ConfigServer {
    fn default() -> Self {
        ConfigServer {
            server_address: "127.0.0.1:8001".to_string(),
            auto_reload_browser_via_websocket_on_change: false,
            auto_reload_websocket_path: "/ws/".to_string(),
        }
    }
}

impl Default for ConfigProject {
    fn default() -> Self {
        ConfigProject {
            keywords: Default::default(),
            title: Default::default(),
            description: Default::default(),
            posts_per_index: default_posts_per_index()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct ConfigDates {
    pub date_format: String,
    pub date_time_format: String,
    pub output_date_time_format: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct ConfigServer {
    pub server_address: String, // usually "127.0.0.1:8001"
    // Insert websocket javascript to automatically reload
    // when a change is detected
    pub auto_reload_browser_via_websocket_on_change: bool,
    pub auto_reload_websocket_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ConfigRSS {
    pub absolute_feed_address: String, // the absolute URL of the feed
    pub title: String,
    pub link: String,
    pub description: String,
    pub author_email: String,
    pub author_name: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Various Project Properties
    #[serde(rename = "Project", default)]
    pub project: ConfigProject,

    /// Folder configuration
    #[serde(rename = "Folders", default)]
    pub folders: ConfigFolders,

    /// Template configuration
    #[serde(rename = "Templates", default)]
    pub templates: ConfigTemplates,

    /// Date configuration
    #[serde(rename = "Dates", default)]
    pub dates: ConfigDates,

    /// Server configuration
    #[serde(rename = "Server", default)]
    pub server: ConfigServer,

    /// RSS
    #[serde(default, rename = "RSS")]
    pub rss: Option<ConfigRSS>,

    /// Meta
    #[serde(default, rename = "Meta")]
    pub meta: HashMap<String, String>,
}

impl Config {
    pub fn new<A: AsRef<std::path::Path>>(folder: A) -> Config {
        let mut config: Config = Default::default();
        config.folders.root = folder.as_ref().to_path_buf();
        config
    }

    pub fn toml(input: &str, in_folder: &PathBuf) -> Result<Config> {
        let mut config: Config =
            toml::from_str(&input).ctx(format!("toml file in folder {:?}", &in_folder))?;

        config.folders.root = in_folder.clone();
        Ok(config)
    }

    pub fn file<A: AsRef<std::path::Path>>(toml_file: A) -> Result<Config> {
        let parent = match &toml_file.as_ref().parent() {
            Some(root) => root.to_path_buf(),
            None => panic!(
                "The toml file {:?} is invalid. No Parent Folder.",
                &toml_file.as_ref()
            ),
        };
        let contents = slurp(toml_file)?;
        Config::toml(&contents, &parent)
    }

    pub fn example_config() -> &'static str {
        DEFAULT_PROJECT_TOML
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_from_toml() {
        use crate::config::Config;
        let contents = r#"
[Folders]
postsFolder = "jochen/"
outputFolder = "franz/"
"#;
        let parsed = Config::toml(&contents, &std::path::PathBuf::from("/tmp/test.toml")).unwrap();
        assert_eq!(parsed.folders.posts_folder, "jochen/");
    }

    #[test]
    fn test_parse_rss() {
        use crate::config::Config;
        let contents = r#"
[Folders]
postsFolder = "jochen/"
outputFolder = "franz/"

[RSS]
title = "klaus"
"#;
        let parsed = Config::toml(&contents, &std::path::PathBuf::from("/tmp/test.toml")).unwrap();
        assert!(parsed.rss.is_some());
        assert_eq!(parsed.rss.unwrap().title, "klaus");
    }

    #[test]
    fn test_default_project_toml() {
        use crate::config::*;
        let parsed = Config::toml(
            &DEFAULT_PROJECT_TOML,
            &std::path::PathBuf::from("/tmp/test.toml"),
        )
        .unwrap();
        assert_eq!(parsed.project.keywords, vec!["nam", "nom", "grah"]);
    }
}
