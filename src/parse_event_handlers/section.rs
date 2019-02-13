use super::*;

use std::borrow::Cow;

pub struct SectionEventHandler {
    next_text_is_section: bool,
    current_header: String,
}

impl SectionEventHandler {
    pub fn new() -> SectionEventHandler {
        SectionEventHandler {
            next_text_is_section: false,
            current_header: String::new(),
        }
    }
}

impl EventHandler for SectionEventHandler {
    fn handle(&mut self, event: &Event, result: &mut ParseResult, events: &mut Vec<Event>) -> bool {
        match event {
            Event::Start(Tag::Header(_)) => {
                self.next_text_is_section = true;
            }
            Event::Text(ref text) if self.next_text_is_section => {
                self.current_header.push_str(&text);
            }
            Event::End(Tag::Header(_)) => {
                self.next_text_is_section = false;
                let header_number = (result.sections.len() as u32) + 1;
                let text = std::mem::replace(&mut self.current_header, String::new());
                result.sections.push((header_number, text));
                // we insert a small identifier so that the header can be linked to
                events.push(Event::Html(Cow::Owned(format!(
                    "<span id=\"header-section-{}\"></span>",
                    header_number
                ))));
            }
            _ => (),
        }
        true
    }
}
