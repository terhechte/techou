use crate::config::ConfigRenderer;
pub use crate::parse_event_handlers::ParseResult;
use crate::parse_event_handlers::{
    highlight::HighlightEventHandler, links::LinksEventHandler, section::SectionEventHandler,
    EventHandler,
};
use pulldown_cmark::{html, Event, Options, Parser};

use std::collections::HashMap;

// Transform the AST of the markdown to support custom markdown constructs
pub fn markdown_to_html(
    markdown: &str,
    section_identifier: &str,
    links: &Option<HashMap<String, String>>,
    book_html_root: Option<&str>,
    config: &ConfigRenderer,
) -> ParseResult {
    let default_hashmap: HashMap<String, String> = HashMap::new();
    let mut opts = Options::empty();
    if config.markdown_tables {
        opts.insert(Options::ENABLE_TABLES);
    }
    if config.markdown_footnotes {
        opts.insert(Options::ENABLE_FOOTNOTES);
    }

    let parser = Parser::new_ext(markdown, opts);
    let mut events: Vec<Event> = Vec::new();
    let mut result = ParseResult {
        content: String::new(),
        sections: Vec::new(),
    };

    let mut handlers: Vec<Box<dyn EventHandler>> = Vec::new();

    if config.parse_headers {
        handlers.push(Box::new(SectionEventHandler::new(
            section_identifier,
            &config.section_header_identifier_template,
        )));
    }

    if config.highlight_syntax {
        handlers.push(Box::new(HighlightEventHandler::new(config.clone())));
    }

    if config.parse_links {
        if let Some(links) = links {
            handlers.insert(0, Box::new(LinksEventHandler::new(&links, book_html_root)))
        } else {
            handlers.insert(
                0,
                Box::new(LinksEventHandler::new(&default_hashmap, book_html_root)),
            )
        }
    }

    for event in parser {
        let mut ignore_event = false;
        for handler in handlers.iter_mut() {
            ignore_event = !handler.handle(&event, &mut result, &mut events);
            if ignore_event {
                break;
            }
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
        use crate::config::ConfigRenderer;
        use crate::markdown::*;
        let cfg = ConfigRenderer {
            highlight_syntax: true,
            markdown_tables: false,
            markdown_footnotes: true,
            parse_links: true,
            parse_headers: true,
            section_header_identifier_template: "".to_owned(),
            store_build_cache: false,
            swift_use_splash: false,
        };
        let contents = r#"
# Section 1
Hello world
## Section 2
More text
## Another section
# Final section"#;
        let result = markdown_to_html(&contents, "", &None, None, &cfg);
        assert_eq!(result.sections.len(), 4);
        assert_eq!(result.sections[0].1, "Section 1");
    }

    #[test]
    fn test_syntax() {
        use crate::config::ConfigRenderer;
        use crate::markdown::*;
        let cfg = ConfigRenderer::default();
        let contents = r#"
# Section 1
`printf()`

more code
``` rs
if let Some(x) = variable {
  println!("{}", &x);
}

"#;
        let result = markdown_to_html(&contents, "", &None, None, &cfg);
        // Test for the CSS classes
        println!("{}", result.content);
        assert!(result.content.contains("techoucontrol techourust"));
    }

    #[test]
    fn test_rellinks() {
        use crate::config::ConfigRenderer;
        use crate::markdown::*;
        let cfg = ConfigRenderer::default();
        let contents = r#"
[bonjour](rel::posts/post.md)
"#;
        let result = markdown_to_html(&contents, "", &None, Some("book"), &cfg);
        assert!(result.content.contains("/book/posts/post.html"));
        let result = markdown_to_html(&contents, "", &None, None, &cfg);
        assert!(result.content.contains("/posts/post.html"));
    }

    #[test]
    fn test_reflinks() {
        use crate::config::ConfigRenderer;
        use crate::markdown::*;
        let cfg = ConfigRenderer::default();
        let contents = r#"
# Section 1
[hello](lnk::yahoo)
and another link
[another](lnk::drm)
and a non-link
[non-link](http://example.com)
yep
"#;
        let reflinks: HashMap<String, String> = [("yahoo", "jojo"), ("drm", "jaja")]
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        let result = markdown_to_html(&contents, "", &Some(reflinks), None, &cfg);
        // Test for the CSS classes
        println!("{}", &result.content);
        assert!(result.content.contains("hello"));
    }
}
