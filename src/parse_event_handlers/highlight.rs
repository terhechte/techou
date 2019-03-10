use syntect::html::ClassedHTMLGenerator;
use syntect::util::LinesWithEndings;
use syntect::parsing::SyntaxSet;

use super::*;

use std::borrow::Cow;


pub struct HighlightEventHandler {
    next_text_is_code: bool,
    language: String,
    current_code: String,
    syntax_set: SyntaxSet
}

impl HighlightEventHandler {
    pub fn new() -> HighlightEventHandler {
        let ps = SyntaxSet::load_defaults_newlines();
        HighlightEventHandler {
            next_text_is_code: false,
            language: "text".to_owned(),
            current_code: String::new(),
            syntax_set: ps
        }
    }
}

impl EventHandler for HighlightEventHandler {
    fn handle(&mut self, event: &Event, _result: &mut ParseResult, events: &mut Vec<Event>) -> bool {
        match event {
            Event::Start(Tag::CodeBlock(ref lang)) => {
                self.next_text_is_code = true;
                self.language = lang.to_string();
                return false;
            }
            Event::Text(ref text) if self.next_text_is_code => {
                self.current_code.push_str(&text);
                return false;
            }
            Event::End(Tag::CodeBlock(_)) => {
                let syntax = match self.syntax_set.find_syntax_by_name(&self.language) {
                    Some(s) => s,
                    None => match self.syntax_set.find_syntax_by_extension(&self.language) {
                        Some(s) => s,
                        None => self.syntax_set.find_syntax_plain_text(),
                    },
                };

                let mut html_generator = ClassedHTMLGenerator::new(&syntax, &self.syntax_set, Some("apv"));
                let mut lines = LinesWithEndings::from(&self.current_code);
                for line in lines {
                    html_generator.parse_html_for_line(&line);
                }
                let html_str = html_generator.finalize();

                events.push(Event::Html(Cow::Owned(format!(
                    "<pre class=\"{}\"><code>{}</code></pre>",
                    &syntax.name, &html_str
                ))));
                self.current_code = String::new();
                self.language = "text".to_owned();
                self.next_text_is_code = false;
            }
            _ => (),
        }
        true
    }
}
