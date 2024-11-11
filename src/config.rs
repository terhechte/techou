use serde_derive::{Deserialize, Serialize};

use crate::error::*;
use crate::io_utils::slurp;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

fn default_posts_per_index() -> u32 {
    8
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct ConfigProject {
    #[serde(default, rename = "baseURL")]
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
    // Fast rendering means we don't write tags, archives, search indexes etc.
    // just the necessary stuff required to work on an article
    #[serde(default)]
    pub fast_render: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct ConfigRenderer {
    // Should code syntax be highlighted
    #[serde(default)]
    pub highlight_syntax: bool,
    // Should custom syntax highlighting (via Splash) be used for Swift?
    #[serde(default)]
    pub swift_use_splash: bool,
    // Markdown table support
    #[serde(default)]
    pub markdown_tables: bool,
    // Markdown footnotes support
    #[serde(default)]
    pub markdown_footnotes: bool,
    // Detect headers and generate small identifiers and a list
    // of all headers, so that they can be listed in a sidebar
    #[serde(default)]
    pub parse_headers: bool,
    // Link parsing support
    // lnk::link-id -> replace with the id from the Shortlinks
    // rel::link -> replace with the absolute link within the current guide root
    #[serde(default)]
    pub parse_links: bool,
    // The HTML to use for the header sections that are parsed
    // out of `h1` tags and can be used to populate a sidebar for longer articles or a toc
    #[serde(default)]
    pub section_header_identifier_template: String,
    /// If this is true, we save the buildcache to disk
    /// The filename will be `buildcache.techou`
    #[serde(default)]
    pub store_build_cache: bool,
    // FIXME: Currently unused because it has to be static in syntect
    #[serde(default)]
    pub highlight_prefix: String,
}

impl Default for ConfigRenderer {
    fn default() -> Self {
        ConfigRenderer {
            highlight_syntax: true,
            swift_use_splash: false,
            markdown_tables: true,
            markdown_footnotes: true,
            parse_headers: true,
            parse_links: true,
            section_header_identifier_template: "<span id=\"{identifier}-{number}\"></span>"
                .to_owned(),
            store_build_cache: true,
            highlight_prefix: "techou".to_string(),
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
    #[serde(default)]
    pub posts_folder: String,
    #[serde(default)]
    pub pages_folder: String,
    #[serde(default)]
    pub books_folder: String,
    #[serde(default)]
    pub output_folder: String,
    #[serde(default)]
    pub public_folder: String,
    #[serde(default)]
    pub public_copy_folders: Vec<String>,

    /// Name of book folders including the summary toml file
    #[serde(default)]
    pub books: Vec<String>,

    /// Folder names in the generated structure
    #[serde(default)]
    pub posts_folder_name: String,
    #[serde(default)]
    pub tags_folder_name: String,
    #[serde(default)]
    pub keywords_folder_name: String,
    #[serde(default)]
    pub category_folder_name: String,
    #[serde(default)]
    pub pages_folder_name: String,
    #[serde(default)]
    pub books_folder_name: String,
    #[serde(default)]
    pub years_folder_name: String,
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
            public_copy_folders: vec!["css".to_string(), "img".to_string(), "js".to_string()],

            books: Vec::new(),

            posts_folder_name: "posts".to_string(),
            pages_folder_name: "pages".to_string(),
            books_folder_name: "books".to_string(),
            tags_folder_name: "tags".to_string(),
            keywords_folder_name: "keywords".to_string(),
            category_folder_name: "category".to_string(),
            years_folder_name: "years".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct ConfigTemplates {
    #[serde(default)]
    pub post_template: String,
    #[serde(default)]
    pub page_template: String,
    #[serde(default)]
    pub list_template: String,
    #[serde(default)]
    pub book_template: String,
    #[serde(default)]
    pub chapter_template: String,
    #[serde(default)]
    pub year_template: String,
}

impl Default for ConfigTemplates {
    fn default() -> Self {
        ConfigTemplates {
            post_template: "post.html".to_string(),
            page_template: "page.html".to_string(),
            list_template: "list.html".to_string(),
            book_template: "book.html".to_string(),
            chapter_template: "chapter.html".to_string(),
            year_template: "year.html".to_string(),
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
            auto_reload_browser_via_websocket_on_change: true,
            auto_reload_websocket_path: "/ws/".to_string(),
        }
    }
}

impl Default for ConfigProject {
    fn default() -> Self {
        ConfigProject {
            base_url: "https://example.com".to_owned(),
            keywords: Default::default(),
            title: Default::default(),
            description: Default::default(),
            posts_per_index: default_posts_per_index(),
            render_one_page_books: false,
            debug_instrumentation: false,
            fast_render: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct ConfigDates {
    #[serde(default)]
    pub date_format: String,
    #[serde(default)]
    pub date_time_format: String,
    #[serde(default)]
    pub output_date_time_format: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct ConfigServer {
    #[serde(default)]
    pub server_address: String, // usually "127.0.0.1:8001"
    // Insert websocket javascript to automatically reload
    // when a change is detected
    #[serde(default)]
    pub auto_reload_browser_via_websocket_on_change: bool,
    #[serde(default)]
    pub auto_reload_websocket_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRSS {
    #[serde(default)]
    pub feed_address: String, // the feed file name
    #[serde(default)]
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub author_email: String,
    pub author_name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct ConfigSearch {
    /// Enable the search feature. Default: `true`.
    #[serde(default)]
    pub enable: bool,
    /// The name / path of the `.js` file that the search index will be written to
    #[serde(default)]
    pub search_index_file: String,
    /// Maximum number of visible results. Default: `30`.
    #[serde(default)]
    pub limit_results: u32,
    /// The number of words used for a search result teaser. Default: `30`.
    #[serde(default)]
    pub teaser_word_count: u32,
    /// Define the logical link between multiple search words.
    /// If true, all search words must appear in each result. Default: `true`.
    #[serde(default)]
    pub use_boolean_and: bool,
    /// Boost factor for the search result score if a search word appears in the header.
    /// Default: `2`.
    #[serde(default)]
    pub boost_title: u8,
    /// Boost factor for the search result score if a search word appears in the hierarchy.
    /// The hierarchy contains all titles of the parent documents and all parent headings.
    /// Default: `1`.
    #[serde(default)]
    pub boost_hierarchy: u8,
    /// Boost factor for the search result score if a search word appears in the text.
    /// Default: `1`.
    #[serde(default)]
    pub boost_paragraph: u8,
    /// True if the searchword `micro` should match `microwave`. Default: `true`.
    #[serde(default)]
    pub expand: bool,
    /// Documents are split into smaller parts, seperated by headings. This defines, until which
    /// level of heading documents should be split. Default: `3`. (`### This is a level 3 heading`)
    #[serde(default)]
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
    #[serde(rename = "Search", default)]
    pub search: ConfigSearch,

    /// Rendering
    #[serde(rename = "Render", default)]
    pub render: ConfigRenderer,

    /// Shortlinks
    #[serde(default, rename = "Shortlinks")]
    pub short_links: Option<HashMap<String, String>>,

    /// Meta
    #[serde(default, rename = "Meta")]
    pub meta: HashMap<String, String>,
}

impl Config {
    /// Create a default in-memory config for this folder
    pub fn new<A: AsRef<std::path::Path>>(folder: A) -> Config {
        Config::from_toml("", folder.as_ref().to_path_buf()).unwrap()
    }

    /// Open a config with this file
    pub fn from_file<A: AsRef<std::path::Path>>(toml_file: A) -> Result<Config> {
        let parent = match &toml_file.as_ref().parent() {
            Some(root) => root.to_path_buf(),
            None => panic!(
                "The toml file {:?} is invalid. No Parent Folder.",
                &toml_file.as_ref()
            ),
        };
        let contents = slurp(toml_file)?;
        Config::from_toml(&contents, parent)
    }

    /// Parse the given input toml from a config in the given `in_folder`
    pub fn from_toml(input: &str, in_folder: PathBuf) -> Result<Config> {
        // The default toml is parsed from the default config
        let default_config = Config::default();
        let parsed = toml::to_string(&default_config).unwrap();

        // Parse the config based on multiple sources:
        use config::{Environment, File, FileFormat};
        let s = config::Config::builder()
            .add_source(File::from_str(&parsed, FileFormat::Toml))
            .add_source(File::from_str(input, FileFormat::Toml))
            .add_source(Environment::with_prefix("techou"))
            .build()
            .map_err(|e| crate::error::TechouError::ConfigBuilding {
                source: e,
                context: "Could not build config".to_owned(),
            })?;

        let mut configuration: Config = s.try_deserialize().unwrap();
        configuration.folders.root = in_folder.clone();
        Ok(configuration)
    }

    pub fn example_config(folder: impl AsRef<std::path::Path>) -> String {
        // It would be great if we could just serialize `Config::default` to toml, but that would
        // not include the helpful comments:
        // https://github.com/alexcrichton/toml-rs/issues/274
        // So instead the default config will be `Config::default` + the disabled options
        // and then we comment everything out
        let mut config = Config::new(folder);
        config.rss = Some(ConfigRSS {
            feed_address: "feed.rss".to_string(),
            title: "My RSS Feed".to_string(),
            description: Some("".to_string()),
            author_email: "".to_string(),
            author_name: Some("".to_string()),
        });

        // documentation strings
        let mut docs = HashMap::new();
        docs.insert(
            "fastRender",
            "Fast rendering means we don't write tags, archives, search indexes etc.",
        );
        docs.insert(
            "debugInstrumentation",
            "Add additional debug information to the HTML",
        );
        docs.insert(
            "swiftUseSplash",
            "Proper Swift rendering doesn't work well with Syntect. Better rendering is provided by Splash (github.com/JohnSundell/Splash). Enabling this requires a Splash install on the system.",
        );
        docs.insert("parseHeaders", "Detect headers and generate small identifiers and a list of all headers, so that they can be listed in a sidebar");
        docs.insert("parseLinks", "convert `lnk::link-id` with the shortlink and `rel::link` with the absolute link to the current root");
        docs.insert("sectionHeaderIdentifierTemplate", "The HTML to use for the header sections that are parsed out of `h1` tags and can be used to populate a sidebar for longer articles or a toc");
        docs.insert("storeBuildCache", "If this is true, we save the buildcache to disk. This will enable faster rendering. The filename will be `buildcache.techou`");
        docs.insert("postsFolder", "Where are your posts");
        docs.insert(
            "pagesFolder",
            "Where are additional pages (if you intend to write them)",
        );
        docs.insert("publicFolder", "Where are the templates and public items");
        docs.insert(
            "publicCopyFolders",
            "The file and folders that should be copied over from within the public folder",
        );
        docs.insert(
            "dateFormat",
            "The input date format that should be used for your posts and apges",
        );
        docs.insert(
            "dateTimeFormat",
            "The input date time format. Has priority over the date format",
        );

        let parsed = toml::to_string_pretty(&config).unwrap();
        let mut lines = Vec::new();
        for line in parsed.lines() {
            for key in docs.keys() {
                if line.starts_with(key) {
                    if let Some(value) = docs.get(key) {
                        lines.push(format!("# {}", value));
                        break;
                    }
                }
            }
            lines.push(format!("# {}", &line));
        }

        let mut combined = lines.join("\n");

        // Add the short links and meta blurbs
        let blurs = r#"
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
        combined.push_str(blurs);
        combined
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
        let parsed =
            Config::from_toml(&contents, std::path::PathBuf::from("/tmp/test.toml")).unwrap();
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
        let parsed =
            Config::from_toml(&contents, std::path::PathBuf::from("/tmp/test.toml")).unwrap();
        assert!(parsed.rss.is_some());
        assert_eq!(parsed.rss.unwrap().title, "klaus");
    }

    #[test]
    fn test_default_project_toml() {
        use crate::config::*;
        let default = Config::example_config("/tmp/");
        let parsed =
            Config::from_toml(&default, std::path::PathBuf::from("/tmp/test.toml")).unwrap();
        assert_eq!(parsed.project.base_url, "https://example.com");
    }
}
