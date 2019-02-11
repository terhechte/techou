use pulldown_cmark::{html, Event, Parser, Tag};

pub trait EventHandler {
    fn handle(&mut self, event: &Event, result: &mut ParseResult, events: &mut Vec<Event>) -> bool;
}

pub struct ParseResult {
    pub content: String,
    pub sections: Vec<(i32, String)>,
}

pub mod highlight;
pub mod section;
