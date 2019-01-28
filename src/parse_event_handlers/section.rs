use super::*;

pub struct SectionEventHandler {
    next_text_is_section: bool
}

impl SectionEventHandler {
    pub fn new() -> SectionEventHandler {
        SectionEventHandler {
            next_text_is_section: false
        }
    }
}

impl EventHandler for SectionEventHandler {

    fn handle(&mut self, event: &Event, result: &mut ParseResult, events: &mut Vec<Event>) -> bool {
        match &event {
            &Event::Start(Tag::Header(_)) => {
                self.next_text_is_section = true;
            }
            &Event::Text(ref text) if self.next_text_is_section => {
                result.sections.push(((result.sections.len() as i32) + 1, text.to_string()));
                self.next_text_is_section = false;
            }
            _ => ()
        }
        true
    }
}