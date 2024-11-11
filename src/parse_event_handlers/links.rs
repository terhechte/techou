use super::*;
use std::collections::HashMap;

use crate::utils;
use pulldown_cmark::CowStr;
use std::borrow::Cow;

pub struct LinksEventHandler<'a> {
    link_database: &'a HashMap<String, String>,
    base_folder: Option<&'a str>,
}

enum LinkType<'a> {
    Link,
    ShortLink(Cow<'a, str>),
    RelLink(Cow<'a, str>),
    Id(Cow<'a, str>, Cow<'a, str>),
}

impl<'a> LinksEventHandler<'a> {
    pub fn new(
        links: &'a HashMap<String, String>,
        base_folder: Option<&'a str>,
    ) -> LinksEventHandler<'a> {
        LinksEventHandler {
            link_database: links,
            base_folder: base_folder,
        }
    }

    fn detect_link_type(link: &'a CowStr) -> LinkType<'a> {
        let items: Vec<&str> = link.split("::").collect();
        // a normal link
        if items.len() < 2 {
            return LinkType::Link;
        }
        if items[0].len() > 3 {
            return LinkType::Link;
        }
        match items[0] {
            "lnk" => return LinkType::ShortLink(Cow::Borrowed(items[1])),
            "rel" => return LinkType::RelLink(Cow::Borrowed(items[1])),
            "id" if items.len() == 3 => {
                return LinkType::Id(Cow::Borrowed(items[1]), Cow::Borrowed(items[2]))
            }
            _ => return LinkType::Link,
        }
    }
}

impl<'a> EventHandler for LinksEventHandler<'a> {
    /// Return `true` if the event should be kept
    fn handle<'b>(
        &mut self,
        event: &'b Event,
        _result: &mut ParseResult,
        events: &'b mut Vec<Event>,
    ) -> bool {
        match event {
            Event::Start(Tag::Link(a, b, c)) => match LinksEventHandler::detect_link_type(b) {
                LinkType::Link => true,
                LinkType::RelLink(tag) => {
                    let base = self.base_folder.unwrap_or_default();
                    let full_path = format!("/{}/{}", &base, &tag.replace(".md", ".html"));
                    events.push(Event::Start(Tag::Link(
                        a.clone(),
                        CowStr::Boxed(full_path.into_boxed_str()),
                        CowStr::Boxed(c.clone().into_string().into_boxed_str()),
                    )));
                    return false;
                }
                LinkType::Id(tag, i) => {
                    let hashed = utils::hash_string(&tag, 8);
                    events.push(Event::Start(Tag::Link(
                        a.clone(),
                        CowStr::Boxed(format!("#{hashed}-{i}").into_boxed_str()),
                        CowStr::Boxed(c.clone().into_string().into_boxed_str()),
                    )));
                    return false;
                }
                LinkType::ShortLink(tag) => {
                    if let Some(link) = self.link_database.get(&format!("{}", &tag)) {
                        events.push(Event::Start(Tag::Link(
                            pulldown_cmark::LinkType::Inline,
                            CowStr::Boxed(link.clone().into_boxed_str()),
                            CowStr::Boxed(b.to_string().into_boxed_str()),
                        )));
                        return false;
                    } else {
                        println!("Could not find short-link for tag {}", &tag);
                        dbg!(&self.link_database);
                        return true;
                    }
                }
            },
            _ => return true,
        }
    }
}
