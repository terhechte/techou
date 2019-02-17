use std::collections::HashMap;
use std::cell::Cell;
use std::sync::{Arc, Mutex};

use crate::document::Document;
use crate::utils;

/// Simple in-memory build-cache that
/// Keeps the rendered markdown for a document
#[derive(Clone)]
pub struct BuildCache {
    cache: Arc<Mutex<HashMap<String, (String, Document)>>>
}

impl BuildCache {
    pub fn new() -> BuildCache {
        BuildCache { cache: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn set_item(&self, path: &str, document: &Document) {
        let hashed = utils::hash_string(&document.raw_content, 16);
        self.cache.lock().unwrap().insert(path.to_string(), (hashed, document.clone()));
    }

    pub fn get_item(self, path: &str, contents: &str) -> Option<Document> {
        let hashed = utils::hash_string(&contents, 16);
        if let Some(item) = self.cache.lock().unwrap().get(path) {
            if hashed == item.0 {
                return Some(item.1.clone());
            }
        }
        return None
    }
}