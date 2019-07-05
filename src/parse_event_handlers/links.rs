use super::*;
use std::collections::HashMap;

use std::borrow::Cow;

pub struct LinksEventHandler<'a > {
    link_database: &'a HashMap<String, String>,
    base_folder: Option<&'a str>
}

enum LinkType<'a> {
    Link,
    ShortLink(Cow<'a, str>),
    RelLink(Cow<'a, str>)
}

impl<'a> LinksEventHandler<'a> {
    pub fn new(links: &'a HashMap<String, String>, base_folder: Option<&'a str>) -> LinksEventHandler<'a> {
        LinksEventHandler {
            link_database: links,
            base_folder: base_folder
        }
    }

    fn detect_link_type(link: &'a Cow<str>) -> LinkType<'a> {
        let items: Vec<&str> = link.split("::").collect();
        // a normal link
        if items.len() != 2 {
            return LinkType::Link;
        }
        if items[0].len() > 3 {
            return LinkType::Link;
        }
        match items[0] {
            "lnk" =>
                return LinkType::ShortLink(Cow::Borrowed(items[1])),
            "rel" => 
                return LinkType::RelLink(Cow::Borrowed(items[1])),
            _ =>
                return LinkType::Link
        }
    }
}

impl<'a> EventHandler for LinksEventHandler<'a> {
    /// Return `true` if the event should be kept
    fn handle<'b>(&mut self, event: &'b Event, _result: &mut ParseResult, events: &'b mut Vec<Event>) -> bool {
        match event {
            Event::Start(Tag::Link(a, b)) =>  {
                match LinksEventHandler::detect_link_type(a) {
                    LinkType::Link => return true,
                    LinkType::RelLink(tag) => {
                        if let Some(base) = self.base_folder {
                            let full_path = format!("/{}/{}", &base, &tag.replace(".md", ".html"));
                            events.push(Event::Start(Tag::Link(Cow::Owned(full_path), Cow::Owned(b.to_string()))));
                            return false;
                        } else {
                            return true;
                        }
                    },
                    LinkType::ShortLink(tag) => {
                        if let Some(link) = self.link_database.get(&format!("{}", &tag)) {
                            events.push(Event::Start(Tag::Link(Cow::Owned(link.to_string()), Cow::Owned(b.to_string()))));
                            return false;
                        } else {
                            println!("Could not find short-link for tag {}", &tag);
                            return true;
                        }
                    }
                }
            },
            _ => return true
        }
    }
}
