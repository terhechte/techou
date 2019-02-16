use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};
use serde_derive::{Deserialize, Serialize};
use toml::de::from_str;
use pulldown_cmark::{html, Parser};

use crate::config::Config;
use crate::error::{Result, TechouError};

use std::collections::HashMap;
use std::path::Path;

static DEFAULT_FRONT_MATTER_SEP: &str = "\n---\n";

fn default_nativetime() -> NaiveDateTime {
    NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11)
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DateInfo {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl From<NaiveDateTime> for DateInfo {
    fn from(date_time: NaiveDateTime) -> Self {
        DateInfo {
            year: date_time.year(),
            month: date_time.month(),
            day: date_time.day(),
            hour: date_time.hour(),
            minute: date_time.minute(),
            second: date_time.second(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FrontMatter {
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub description_html: String,
    #[serde(default)]
    pub published: bool,

    // If this is non-empty, use it instead of the generated one
    #[serde(default)]
    pub slug: Option<String>,

    // The Meta Information will be injected
    #[serde(default, skip)]
    pub meta: HashMap<String, String>,
    // The unix timestamp will be injected
    #[serde(default)]
    pub created_timestamp: i64,
    #[serde(default = "default_nativetime", skip)]
    pub date: NaiveDateTime,
    #[serde(default)]
    pub date_info: DateInfo, // FIXME: Move all date/time info into this struct.
    // The unique identifier will be injected (based on the title)
    #[serde(default)]
    pub identifier: String,
}

impl FrontMatter {
    pub fn rfc2822(&self) -> String {
        use chrono::{DateTime, Utc};
        let dt = DateTime::<Utc>::from_utc(self.date, Utc);
        dt.to_rfc2822()
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ParsedFrontMatter {
    front_matter: FrontMatter,
    #[serde(default)]
    meta: HashMap<String, String>,
}

pub fn parse_front_matter<'a, A: AsRef<Path>>(
    input: &'a str,
    filename: A,
    config: &Config,
) -> Result<(FrontMatter, &'a str)> {
    let (front_matter_raw, article) = detect_front_matter(&input, &filename, &config)?;
    let parsed_front_matter: ParsedFrontMatter = match from_str(front_matter_raw) {
        Ok(s) => s,
        Err(e) => {
            return Err(TechouError::FrontMatter {
                issue: format!("{:?}: Invalid Front Matter: {}", &filename.as_ref(), &e),
            });
        }
    };

    let ParsedFrontMatter {
        mut front_matter,
        meta,
    } = parsed_front_matter;

    if front_matter.created.is_empty() {
        front_matter.created = default_date_time(&config);
    }

    let (date_string, timestamp, date) = detect_date_time(&front_matter.created, &config)?;

    front_matter.meta = meta;
    front_matter.created_timestamp = timestamp;
    front_matter.created = date_string;
    front_matter.date = date;
    front_matter.date_info = DateInfo::from(date);

    // Parse the description into html
    let parser = Parser::new(&front_matter.description);

    let mut html_buf = String::new();
    html::push_html(&mut html_buf, parser);
    front_matter.description_html = html_buf;

    Ok((front_matter, article))
}

pub fn default_front_matter(title: &str, date: &str) -> String {
    format!(r#"[frontMatter]
title = "{}"
tags = []
created = "{}"
description = ""
published = false
"#, &title, &date)
}

pub fn join_front_matter_with_content(front_matter: &str, content: &str) -> String {
    format!("{}{}{}", &front_matter, DEFAULT_FRONT_MATTER_SEP, &content)
}

fn detect_front_matter<'a, A: AsRef<Path>>(
    input: &'a str,
    filename: A,
    _config: &Config,
) -> Result<(&'a str, &'a str)> {
    let index = match input.find(DEFAULT_FRONT_MATTER_SEP) {
        Some(r) => r,
        None => {
            return Err(TechouError::FrontMatter {
                issue: format!("{:?}: Missing Front Matter", &filename.as_ref()),
            });
        }
    };
    let (f, a) = input.split_at(index);
    Ok((f, &a[DEFAULT_FRONT_MATTER_SEP.len()..]))
}

pub fn default_date_time(config: &Config) -> String {
    use chrono::{DateTime, Local};
    let local: DateTime<Local> = Local::now();
    let formatted = local.format(&config.dates.date_time_format).to_string();
    formatted
}

pub fn detect_date_time(input: &str, config: &Config) -> Result<(String, i64, NaiveDateTime)> {
    let parsed_date = NaiveDateTime::parse_from_str(&input, &config.dates.date_time_format)
        .or_else(|_| {
            NaiveDate::parse_from_str(&input, &config.dates.date_format)
                .and_then(|e| Ok(e.and_hms(10, 30, 30)))
        })
        .map_err(|e| TechouError::FrontMatter {
            issue: format!("{:?}: Invalid Date Format in Front Matter: {}", &input, &e),
        })?;
    Ok((
        parsed_date
            .format(&config.dates.output_date_time_format)
            .to_string(),
        parsed_date.timestamp(),
        parsed_date,
    ))
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
        let result =
            front_matter::detect_front_matter(contents, "testfile.md", &Default::default());
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
        assert_eq!(
            fm.meta.get("author"),
            Some(&"Benedikt Terhechte".to_string())
        );
    }

    #[test]
    fn test_rfc2822() {
        use crate::front_matter;
        let contents = r#"
[frontMatter]
title = "Hello World"
tags = ["first tag", "second tag"]
created = "2009-12-30"
description = "A run around the world"
published = true
---
this."#;
        let (fm, _) =
            front_matter::parse_front_matter(&contents, "yeah.md", &Default::default()).unwrap();
        assert!(fm.rfc2822().len() > 0);
    }

    #[test]
    fn test_default_front_matter() {
        use crate::front_matter;
        let mut content = front_matter::default_front_matter("we're default", "2009-12-30");
        content.push_str(front_matter::DEFAULT_FRONT_MATTER_SEP);
        content.push_str("Hello Jo");
        let (fm, _) =
            front_matter::parse_front_matter(&content, "yeah.md", &Default::default()).unwrap();
        assert_eq!(fm.title, "we're default");
        assert!(fm.rfc2822().len() > 0);
    }

    #[test]
    fn test_kewywords() {
        use crate::front_matter;
        let contents = r#"
[frontMatter]
title = "Hello World"
tags = ["first tag", "second tag"]
keywords = ["first keyword", "second keyword"]
created = "2009-12-30"
description = "A run around the world"
published = true
---
this."#;
        let (fm, _) =
            front_matter::parse_front_matter(&contents, "yeah.md", &Default::default()).unwrap();
        assert_eq!(fm.keywords.len(), 2);
        assert_eq!(fm.keywords[1], "second keyword");
    }
}
