use pulldown_cmark::{Event, Parser, Tag};
use super::super::document::Document;

#[derive(Default)]
struct Chapter {
    pub name: String,
    pub index: i32,
    pub chapter_index: String, // 1.2.1.3
    //pub document: Document,
    pub sub_chapters: Vec<Chapter>
}

fn testme() {
    let content = r#"- [Intro](./test1/test1.md)
- [Another](./test/another.md)
    - [Level2.1](./test2/test2.md)
    - [Level2.2](./test2/test3.md)
        - [Level3.1](./test2/test3/test1.md)
        - [Level3.2](./test2/test3/test2.md)
- [Final](./final/final.md)
"#;
    //let mut buf = String::with_capacity(content.len());
    let parser = Parser::new(&content);
    // Recur go through chapters to build the repr
    //let mut chapter
    for event in parser {
        match event {
            Event::Start(Tag::Item) => println!("start item"),
            Event::End(Tag::Item) => println!("end item"),
            Event::Start(Tag::List(s)) => println!("start list {:?}", &s),
            Event::End(Tag::List(s)) => println!("end list {:?}", &s),
            _ => println!("other event: {:?}", &event)
        }
    }
    println!("klaus");
}

#[derive(Default)]
struct ChapterInfo {
    pub name: String,
    pub link: String,
    pub level: usize,
    pub sub_chapters: Vec<ChapterInfo>
}

fn parse_chapter(parser: &mut Parser) -> Vec<ChapterInfo> {
    let mut results: Vec<ChapterInfo> = Vec::new();
    //let mut chapter_stack: Vec<ChapterInfo> = Vec::new();
    let mut root: ChapterInfo = Default::default();
    for event in parser {
        match event {
            Event::Start(Tag::Item) => (),
            Event::End(Tag::Item) => (),
            Event::Start(Tag::List) =>
                chapter_stack.push(Default::default()),
            Event::End(Tag::List) =>
                if let Some(mut chapter) = chapter_stack.pop() {
                    chapter.level = chapter_stack.len();
                    results.push(chapter);
                },
            Event::Start(Tag::Link(url, _)) =>
                chapter_stack.last_mut().map(|c|c.link = url.to_string()),
            Event::Start(Tag::Text(text)) =>
                chapter_stack.last_mut().map(|c| c.name = text.to_string()),
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn superduper() {
        testme();
    }
}
