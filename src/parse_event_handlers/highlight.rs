use rayon;
use std::process::Command;
use syntect::html::ClassedHTMLGenerator;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use super::*;

use std::borrow::Cow;

pub struct HighlightEventHandler {
    next_text_is_code: bool,
    language: String,
    current_code: String,
    syntax_set: SyntaxSet,
}

impl HighlightEventHandler {
    pub fn new() -> HighlightEventHandler {
        let ps = SyntaxSet::load_defaults_newlines();
        HighlightEventHandler {
            next_text_is_code: false,
            language: "text".to_owned(),
            current_code: String::new(),
            syntax_set: ps,
        }
    }

    fn non_swift_code(&self) -> (String, String) {
        let syntax = match self.syntax_set.find_syntax_by_name(&self.language) {
            Some(s) => s,
            None => match self.syntax_set.find_syntax_by_extension(&self.language) {
                Some(s) => s,
                None => self.syntax_set.find_syntax_plain_text(),
            },
        };

        let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
            &syntax,
            &self.syntax_set,
            syntect::html::ClassStyle::SpacedPrefixed { prefix: "techou" },
        );
        let lines = LinesWithEndings::from(&self.current_code);
        for line in lines {
            html_generator.parse_html_for_line_which_includes_newline(&line);
        }
        (syntax.name.clone(), html_generator.finalize())
    }

    fn swift_code(&self) -> String {
        use rayon::prelude::*;
        // it seems splash works line-based, so we just highlight by line
        let lines: Vec<String> = self
            .current_code
            .par_lines()
            .map(|l| {
                let mapped = l.replace("\n", "").replace("\"", "\\\"");

                let output = Command::new("/usr/local/bin/SplashHTMLGen")
                    .arg(mapped)
                    .output()
                    .expect("Please install Splash / SplashHTMLGen in /usr/local/bin")
                    .stdout;
                match String::from_utf8(output) {
                    Ok(n) => n.replace("class=\"", "class=\"swift-"),
                    Err(_) => l.to_owned(),
                }
            })
            .collect();

        lines.join("\n")
    }
}

impl EventHandler for HighlightEventHandler {
    fn handle(
        &mut self,
        event: &Event,
        _result: &mut ParseResult,
        events: &mut Vec<Event>,
    ) -> bool {
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
                let (syntax_name, html_str) = match self.language.as_str() {
                    "Swift" | "swift" => ("Swift".to_owned(), self.swift_code()),
                    _ => self.non_swift_code(),
                };
                events.push(Event::Html(Cow::Owned(format!(
                    "<pre class=\"{}\"><code>{}</code></pre>",
                    &syntax_name, &html_str
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
