use super::*;
use std::borrow::Cow;

use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Style};
use syntect::parsing::{SyntaxSet,ParseState};
use syntect::html::{ClassStyle,tokens_to_classed_html};

//use syntect::*;

pub struct HighlightEventHandler {
    next_text_is_code: bool,
    language: String,
    current_code: String,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet
}

impl HighlightEventHandler {
    pub fn new() -> HighlightEventHandler {
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        HighlightEventHandler {
            next_text_is_code: false,
            language: "text".to_owned(),
            current_code: String::new(),
            syntax_set: ps,
            theme_set: ts
        }
    }
}

impl EventHandler for HighlightEventHandler {

    fn handle(&mut self, event: &Event, result: &mut ParseResult, events: &mut Vec<Event>) -> bool {
        match &event {
            &Event::Start(Tag::CodeBlock(ref lang)) => {
                self.next_text_is_code = true;
                self.language = lang.to_string();
                return false;
            }
            &Event::Text(ref text) if self.next_text_is_code => {
                self.current_code.push_str(&text);
                return false;
            }
            &Event::End(Tag::CodeBlock(_)) => {
                self.next_text_is_code = false;
                // try to find a syntax
                let syntax = match self.syntax_set.find_syntax_by_name(&self.language) {
                    Some(s) => s,
                    None => match self.syntax_set.find_syntax_by_extension(&self.language) {
                        Some(s) => s,
                        None => self.syntax_set.find_syntax_plain_text()
                    }
                };
                //let mys = self.syntax_set.find_syntax_by_extension(&self.language).unwrap_or(self.syntax_set.find_syntax_plain_text());
                //let mys = self.syntax_set.find_syntax_by_name("Rust").unwrap();
                let mut ps = ParseState::new(&syntax);
                self.language = "text".to_owned();
                // FIXME: Highlight
                let mut html_str = String::new();
                for line in self.current_code.lines() {
                    println!("line: {}", &line);
                    let parsed_line = ps.parse_line(line, &self.syntax_set);
                    //println!("parsed: {:?}", &prsl);
                    match parsed_line.len() {
                        0 => html_str.push_str(&line),
                        _ => html_str.push_str(&tokens_to_classed_html(line, parsed_line.as_slice(), ClassStyle::Spaced).as_str())
                    }
                    //let tok = ;
                    //println!("tok: {}", &tok);
                    //html_str.push_str(tok.as_str());
                    html_str.push('\n');
                }

                events.push(Event::Html(Cow::Owned(format!("<code>{}</code>", &html_str))));
                println!("push the code: {}", &self.current_code);
                self.current_code = String::new();
            }
            _ => ()
        }
        true
    }
}