use serde_derive::{Deserialize, Serialize};

use crate::error::*;
use crate::io_utils::slurp;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;


static DEFAULT_PROJECT_TOML: &str = r#"
[Project]
keywords = ["nam", "nom", "grah"]
# baseURL = "https://example.com"

# How many posts per index (default: 8)
# postsPerIndex = 9

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
# feedAddress = "/feed.rss"
# title = "My Blog"
# authorEmail = "john@doe.com"
# authorName = "John Doe"

# [Shortlinks]
# Short map from a short link such as `lnk::bookarticle` to `/articles/book.html`
# or `lnk::article2` to `https://example.com/article2.html`
# article2 = "https://example.com/article2.html"
# bookarticle = "/articles/book.html"

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
    #[serde(rename = "baseURL")]
    pub base_url: String, // the base url of the website
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_posts_per_index")]
    pub posts_per_index: u32,
    #[serde(default)]
    pub render_one_page_books: bool,
    #[serde(default)]
    pub debug_instrumentation: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct ConfigRenderer {
    // The prefix to use for each entry in the class list for the syntax
    // highlighter
    #[serde(default)]
    pub syntax_highlight_code_class_prefix: Option<String>,
    // Should code syntax be highlighted
    #[serde(default)]
    pub highlight_syntax: bool,
    // Markdown table support
    #[serde(default)]
    pub markdown_tables: bool,
    // Markdown footnotes support
    #[serde(default)]
    pub markdown_footnotes: bool
}

impl Default for ConfigRenderer {
    fn default() -> Self {
        ConfigRenderer {
            syntax_highlight_code_class_prefix: None,
            highlight_syntax: true,
            markdown_tables: false,
            markdown_footnotes: true
        }
    }
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
    pub books_folder: String,
    pub output_folder: String,
    pub public_folder: String,
    pub public_copy_folders: Vec<String>,

    /// Name of book folders including the summary toml file
    pub books: Vec<String>,

    /// Folder names in the generated structure
    pub posts_folder_name: String,
    pub tags_folder_name: String,
    pub keywords_folder_name: String,
    pub category_folder_name: String,
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
    pub fn books_folder_path(&self) -> PathBuf {
        self.root.join(&self.books_folder)
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

    pub fn output_tags_folder_path(&self) -> PathBuf {
        self.output_folder_path().join(&self.tags_folder_name)
    }

    pub fn output_keywords_folder_path(&self) -> PathBuf {
        self.output_folder_path().join(&self.keywords_folder_name)
    }

    pub fn output_category_folder_path(&self) -> PathBuf {
        self.output_folder_path().join(&self.category_folder_name)
    }

    pub fn output_books_folder_path(&self) -> PathBuf {
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
            books_folder: "books".to_string(),
            output_folder: "html".to_string(),
            public_folder: "public".to_string(),
            public_copy_folders: vec!["css".to_string(), "img".to_string()],

            books: Vec::new(),

            posts_folder_name: "posts".to_string(),
            pages_folder_name: "pages".to_string(),
            books_folder_name: "books".to_string(),
            tags_folder_name: "tags".to_string(),
            keywords_folder_name: "keywords".to_string(),
            category_folder_name: "category".to_string(),
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
            base_url: Default::default(),
            keywords: Default::default(),
            title: Default::default(),
            description: Default::default(),
            posts_per_index: default_posts_per_index(),
            render_one_page_books: false,
            debug_instrumentation: false,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRSS {
    pub feed_address: String, // the feed file name
    pub title: String,
    pub description: Option<String>,
    pub author_email: String,
    pub author_name: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct ConfigSearch {
    /// Enable the search feature. Default: `true`.
    pub enable: bool,
    /// The name / path of the `.js` file that the search index will be written to
    pub search_index_file: String,
    /// Maximum number of visible results. Default: `30`.
    pub limit_results: u32,
    /// The number of words used for a search result teaser. Default: `30`.
    pub teaser_word_count: u32,
    /// Define the logical link between multiple search words.
    /// If true, all search words must appear in each result. Default: `true`.
    pub use_boolean_and: bool,
    /// Boost factor for the search result score if a search word appears in the header.
    /// Default: `2`.
    pub boost_title: u8,
    /// Boost factor for the search result score if a search word appears in the hierarchy.
    /// The hierarchy contains all titles of the parent documents and all parent headings.
    /// Default: `1`.
    pub boost_hierarchy: u8,
    /// Boost factor for the search result score if a search word appears in the text.
    /// Default: `1`.
    pub boost_paragraph: u8,
    /// True if the searchword `micro` should match `microwave`. Default: `true`.
    pub expand: bool,
    /// Documents are split into smaller parts, seperated by headings. This defines, until which
    /// level of heading documents should be split. Default: `3`. (`### This is a level 3 heading`)
    pub heading_split_level: u8,
}

impl Default for ConfigSearch {
    fn default() -> ConfigSearch {
        ConfigSearch {
            enable: true,
            search_index_file: "js/searchindex.js".to_string(),
            limit_results: 30,
            teaser_word_count: 30,
            use_boolean_and: false,
            boost_title: 2,
            boost_hierarchy: 1,
            boost_paragraph: 1,
            expand: true,
            heading_split_level: 3,
        }
    }
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

    /// Search
    #[serde(rename="Search", default)]
    pub search: ConfigSearch,

    /// Rendering
    #[serde(rename="Render", default)]
    pub render: ConfigRenderer,

    /// Shortlinks
    #[serde(default, rename = "Shortlinks")]
    pub short_links: Option<HashMap<String, String>>,

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
feedAddress = "https://example.com"
authorEmail = "example@example.com"
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
