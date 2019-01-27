use std::collections::HashMap;
use std::path::Path;
use toml::de::from_str;
use chrono::{NaiveDate, NaiveDateTime};
use serde;
use serde_derive::{Serialize, Deserialize};

use crate::config::Config;
use crate::error::{Result, Error};

fn default_nativetime() -> NaiveDateTime {
    NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FrontMatter {
    pub title: String,
    pub tags: Vec<String>,
    pub created: String,
    pub description: String,
    pub published: bool,

    // The Meta Information will be injected
    #[serde(default)]
    pub meta: HashMap<String, String>,
    // The unix timestamp will be injected
    #[serde(default)]
    pub created_timestamp: i64,
    // The unique identifier will be injected (based on the title)
    #[serde(default="default_nativetime", skip)]
    pub date: NaiveDateTime,
    #[serde(default)]
    pub identifier: String
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ParsedFrontMatter {
    front_matter: FrontMatter,
    meta: HashMap<String, String>
}

pub fn parse_front_matter<'a, A: AsRef<Path>>(input: &'a str, filename: A, config: &Config) -> Result<(FrontMatter, &'a str)> {
    let (front_matter_raw, article) = detect_front_matter(&input, &filename, &config)?;
    let parsed_front_matter: ParsedFrontMatter = match from_str(front_matter_raw) {
        Ok(s) => s,
        Err(e) => return Err(Error::FrontMatter(format!("{:?}: Invalid Front Matter: {}", &filename.as_ref(), &e)))
    };

    let ParsedFrontMatter { mut front_matter, meta } = parsed_front_matter;

    let (date_string, timestamp, date) = detect_date_time(&front_matter.created, &filename, &config)?;

    front_matter.meta = meta;
    front_matter.created_timestamp = timestamp;
    front_matter.created = date_string;
    front_matter.date = date;

    Ok((front_matter, article))
}

fn detect_front_matter<'a, A: AsRef<Path>>(input: &'a str, filename: A, config: &Config) -> Result<(&'a str, &'a str)> {
    let separator = "\n---\n";
    let index = match input.find(separator) {
        Some(r) => r,
        None => return Err(Error::FrontMatter(format!("{:?}: Missing Front Matter", &filename.as_ref())))
    };
    let (f, a) = input.split_at(index);
    Ok((f, &a[separator.len()..]))
}

fn detect_date_time<A: AsRef<Path>>(input: &str, filename: A, config: &Config) -> Result<(String, i64, NaiveDateTime)> {
    let parsed_date = NaiveDateTime::parse_from_str(&input, &config.date_time_format).or_else(|_| {
        NaiveDate::parse_from_str(&input, &config.date_format).and_then(|e| {
            Ok(e.and_hms(10, 30, 30))
        })
    }).map_err(|e| Error::FrontMatter(format!("{:?}: Invalid Date Format in Front Matter: {} {}", &filename.as_ref(), &input, &e)))?;
    Ok((parsed_date.format(&config.output_date_time_format).to_string(), parsed_date.timestamp(), parsed_date))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_splitting() {
        use crate::front_matter;
        let contents = r#"
[global]
name = "techou"
version = "0.1.0 || "
---
this is the actual article contents yeah."#;
        let result = front_matter::detect_front_matter(contents, "testfile.md", &Default::default());
        assert!(result.is_ok());
        let (front_matter, article) = result.unwrap();
        assert!(front_matter.len() > 0);
        assert!(article.len() > 0);
        assert_eq!(article, "this is the actual article contents yeah.");
    }

    #[test]
    fn test_parsing() {
        use crate::front_matter;
        let contents = r#"
[frontMatter]
title = "Hello World"
tags = ["first tag", "second tag"]
created = "2009-12-30"
description = "A run around the world"
published = true

[meta]
html_content = "<b>this is html</b>"
author = "Benedikt Terhechte"
---
this is the actual article contents yeah."#;
        let result = front_matter::parse_front_matter(&contents, "yeah.md", &Default::default());
        assert!(result.is_ok());
        let (fm, _) = result.unwrap();
        assert_eq!(fm.title, "Hello World");
    }
}
