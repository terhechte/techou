use pulldown_cmark::{Event, Tag};

pub trait EventHandler {
    fn handle(&mut self, event: &Event, result: &mut ParseResult, events: &mut Vec<Event>) -> bool;
}

pub struct ParseResult {
    pub content: String,
    pub sections: Vec<(u32, String)>,
}

pub mod highlight;
pub mod section;
pub mod links;