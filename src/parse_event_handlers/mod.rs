use pulldown_cmark::{Event, Parser, Tag, html};

pub trait EventHandler {
    fn handle(&mut self, event: &Event, result: &mut ParseResult, events: &mut Vec<Event>) -> bool;
}

pub struct ParseResult {
    pub content: String,
    pub sections: Vec<(i32, String)>
}

pub mod section;
pub mod highlight;
