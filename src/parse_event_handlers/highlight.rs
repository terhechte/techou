use syntect::html::{tokens_to_classed_html, ClassStyle};
use syntect::parsing::{ParseState, SyntaxSet};

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
                let mut ps = ParseState::new(&syntax);
                let mut html_str = String::new();
                for line in self.current_code.lines() {
                    let parsed_line = ps.parse_line(line, &self.syntax_set);
                    // If there was nothing to parse, we just add the line as is
                    match parsed_line.len() {
                        0 => html_str.push_str(&line),
                        _ => html_str.push_str(
                            &tokens_to_classed_html(
                                line,
                                parsed_line.as_slice(),
                                ClassStyle::Spaced,
                            )
                            .as_str(),
                        ),
                    }
                    html_str.push('\n');
                }

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
