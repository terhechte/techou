use super::*;

use pulldown_cmark::CowStr;

pub struct SectionEventHandler<'a> {
    base_identifier: &'a str,
    header_section_html: &'a str,
    next_text_is_section: bool,
    current_header: String,
}

impl<'a> SectionEventHandler<'a> {
    pub fn new(base_identifier: &'a str, header_section_html: &'a str) -> SectionEventHandler<'a> {
        SectionEventHandler {
            base_identifier,
            header_section_html,
            next_text_is_section: false,
            current_header: String::new(),
        }
    }
}

impl<'a> EventHandler for SectionEventHandler<'a> {
    fn handle(&mut self, event: &Event, result: &mut ParseResult, events: &mut Vec<Event>) -> bool {
        match event {
            Event::Start(Tag::Heading(_, _, _)) => {
                self.next_text_is_section = true;
            }
            Event::Text(ref text) if self.next_text_is_section => {
                self.current_header.push_str(&text);
            }
            Event::End(Tag::Heading(_, _, _)) => {
                self.next_text_is_section = false;
                let header_number = (result.sections.len() as u32) + 1;
                let text = std::mem::replace(&mut self.current_header, String::new());
                result.sections.push((header_number, text));
                // we insert a small identifier so that the header can be linked to
                let string = self
                    .header_section_html
                    .replace("{identifier}", self.base_identifier)
                    .replace("{number}", &header_number.to_string());
                events.push(Event::Html(CowStr::Boxed(string.into_boxed_str())));
            }
            _ => (),
        }
        true
    }
}
