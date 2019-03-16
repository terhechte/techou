use pulldown_cmark::{html, Event, Parser, Options};
use crate::parse_event_handlers::{
    highlight::HighlightEventHandler, section::SectionEventHandler, EventHandler, 
};
pub use crate::parse_event_handlers::ParseResult;


// Transform the AST of the markdown to support custom markdown constructs
pub fn markdown_to_html(markdown: &str, section_identifier: &str) -> ParseResult {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(markdown, opts);
    let mut events: Vec<Event> = Vec::new();
    let mut result = ParseResult {
        content: String::new(),
        sections: Vec::new(),
    };

    let mut handlers: Vec<Box<dyn EventHandler>> = vec![
        Box::new(SectionEventHandler::new(section_identifier)),
        Box::new(HighlightEventHandler::new()),
    ];

    for event in parser {
        let mut ignore_event = false;
        for handler in handlers.iter_mut() {
            ignore_event = !handler.handle(&event, &mut result, &mut events);
        }
        if !ignore_event {
            events.push(event);
        }
    }
    html::push_html(&mut result.content, events.into_iter());
    result
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_sections() {
        use crate::document;
        use crate::markdown::*;
        let contents = r#"
# Section 1
Hello world
## Section 2
More text
## Another section
# Final section"#;
        let result = markdown_to_html(&contents, "");
        assert_eq!(result.sections.len(), 4);
        assert_eq!(result.sections[0].1, "Section 1");
    }

    #[test]
    fn test_syntax() {
        use crate::document;
        use crate::markdown::*;
        let contents = r#"
# Section 1
`printf()`

more code
``` Rust
if let Some(x) = variable {
  println!("{}", &x);
}

"#;
        let result = markdown_to_html(&contents, "");
        // Test for the CSS classes
        assert!(result.content.contains("source rust"));
    }
}
