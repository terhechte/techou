use std::env;
use std::path::PathBuf;

pub struct Config {
    /// Folders on Disk
    pub root: PathBuf,
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
    pub output_date_time_format: String
}

impl Config {

    pub fn new<A: AsRef<PathBuf>>(folder: A) -> Config {
        let mut config: Config = Default::default();
        config.root = folder.as_ref().to_path_buf();
        config
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

    pub fn template_folder_path(&self) -> PathBuf {
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
            output_date_time_format: "%Y-%m-%d %H:%M:%S".to_string()
        }
    }
}