use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::document::Document;
use crate::io_utils::slurp;
use crate::io_utils::spit;
use crate::utils;

/// Simple in-memory build-cache that
/// Keeps the rendered markdown for a document
#[derive(Clone)]
pub struct BuildCache {
    cache: Arc<Mutex<HashMap<String, (String, Document)>>>,
    filename: PathBuf,
}

impl BuildCache {
    pub fn new<A: AsRef<Path>>(from: A) -> BuildCache {
        if from.as_ref().exists() {
            let data = slurp(from.as_ref()).unwrap();
            let deserialized: HashMap<String, (String, Document)> =
                serde_json::from_str(&data).unwrap();
            BuildCache {
                cache: Arc::new(Mutex::new(deserialized)),
                filename: from.as_ref().to_owned(),
            }
        } else {
            BuildCache {
                cache: Arc::new(Mutex::new(HashMap::new())),
                filename: from.as_ref().to_owned(),
            }
        }
    }

    pub fn set_item(&self, path: &str, document: &Document) {
        let hashed = utils::hash_string(&document.raw_content, 16);
        self.cache
            .lock()
            .unwrap()
            .insert(path.to_string(), (hashed, document.clone()));
    }

    pub fn get_item(self, path: &str, contents: &str) -> Option<Document> {
        let hashed = utils::hash_string(&contents, 16);
        if let Some(item) = self.cache.lock().unwrap().get(path) {
            if hashed == item.0 {
                return Some(item.1.clone());
            }
        }
        return None;
    }

    pub fn write(&self) -> Result<(), Box<dyn Error>> {
        println!("Write Build Cache");
        let data = self.cache.lock().unwrap().clone();
        let serialized = serde_json::to_string(&data).unwrap();
        spit(self.filename.as_path(), &serialized).unwrap();
        Ok(())
    }
}
