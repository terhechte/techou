use crossbeam::channel::Receiver;

use std::sync::{Arc, Mutex};
use std::path::PathBuf;

#[derive(Debug)]
pub enum BrowserAction {
    Reload
}

/// The vec is an array of optional error messages
pub type BrowserResult = std::result::Result<BrowserAction, Option<Vec<String>>>;

pub struct ServerState<S> {
    pub receiver: Option<Mutex<Receiver<S>>>,
    pub websocket_payload: String,
    pub serve_dir: PathBuf
}
