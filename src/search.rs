use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use elasticlunr::Index;
use lazy_static::*;
use pulldown_cmark::*;
use serde_derive::*;
use serde_json;

use crate::book::Book;
use crate::config::Config;
use crate::document::Document;
use crate::error::*;
use crate::utils;

// This was lifted & adapted from mdbook / searcher.rs

fn clean_html(html: &str) -> String {
    lazy_static! {
        static ref AMMONIA: ammonia::Builder<'static> = {
            let mut clean_content = HashSet::new();
            clean_content.insert("script");
            clean_content.insert("style");
            let mut builder = ammonia::Builder::new();
            builder
                .tags(HashSet::new())
                .tag_attributes(HashMap::new())
                .generic_attributes(HashSet::new())
                .link_rel(None)
                .allowed_classes(HashMap::new())
                .clean_content_tags(clean_content);
            builder
        };
    }
    AMMONIA.clean(html).to_string()
}

pub struct Searcher<'a> {
    index: Index,
    doc_urls: Vec<String>,
    config: &'a Config,
}

impl<'a> Searcher<'a> {
    pub fn new(config: &'a Config) -> Searcher {
        let index = Index::new(&["title", "body", "breadcrumbs"]);
        let doc_urls = Vec::new();
        Searcher {
            index,
            doc_urls,
            config,
        }
    }

    /// Index one document
    pub fn index_document(&mut self, document: &Document) -> Result<()> {
        // Don't index documents that opt out of indexing
        if !document.info.indexed {
            return Ok(());
        }
        self.render_item(
            &document.info.title,
            &document.slug,
            &document.info.description_html,
            &document.raw_content,
        )
    }

    /// Index all chapters of a book and the book itself
    pub fn index_book(&mut self, book: &Book) -> Result<()> {
        // Don't index documents that opt out of indexing
        if !book.info.indexed {
            return Ok(());
        }

        if !&book.info.description.is_empty() {
            self.render_item(
                &book.info.title,
                &book.slug,
                &book.info.description_html,
                "",
            )?;
        }
        for chapter in &book.chapters {
            // Don't index documents that opt out of indexing
            if !chapter.document.info.indexed {
                continue;
            }
            self.render_item(
                &chapter.document.info.title,
                &chapter.document.slug,
                &chapter.document.info.description_html,
                &chapter.document.raw_content,
            )?;
        }
        Ok(())
    }

    /// This returns the `js` contents of the search index file
    pub fn finalize(self) -> Result<String> {
        let index = self.write_to_json()?;
        println!("Writing search index ✓");
        if index.len() > 10_000_000 {
            println!("searchindex.json is very large ({} bytes)", index.len());
        }
        Ok(format!("window.search = {};", index))
    }

    /// Renders markdown into flat unformatted text and adds it to the search index.
    fn render_item(
        &mut self,
        title: &str,
        slug: &str,
        description_html: &str,
        contents: &str,
    ) -> Result<()> {
        // As said below, we have to have one parse method and not do it 5 times.
        // this is terrible for performance
        let first_index = match contents.find("---\n") {
            Some(o) => o,
            None => return Ok(()),
        };

        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        opts.insert(Options::ENABLE_FOOTNOTES);
        // Cut out the front matter. This should happen in one go that parses everything out of the markdown
        // that we need.
        let p = Parser::new_ext(&contents[(first_index + 4)..], opts);

        // FIXME: Instead of parsing each document 10 times we should do it once in a seperate place that does everything

        let mut in_header = false;
        let max_section_depth = self.config.search.heading_split_level as i32;
        let mut section_id = None;
        let mut heading = String::new();
        let mut body = String::new();
        let mut breadcrumbs = vec![title.to_string()];
        let mut footnote_numbers = HashMap::new();
        let mut heading_counter = 1;

        // add the description, too
        self.add_doc(&slug, &None, &[&title, &clean_html(&description_html)]);

        for event in p {
            match event {
                Event::Start(Tag::Header(i)) if i <= max_section_depth => {
                    if !heading.is_empty() {
                        // Section finished, the next header is following now
                        // Write the data to the index, and clear it for the next section
                        self.add_doc(
                            &slug,
                            &section_id,
                            &[&heading, &body, &breadcrumbs.join(" » ")],
                        );
                        section_id = None;
                        heading.clear();
                        body.clear();
                        breadcrumbs.pop();
                    }

                    heading_counter += 1;

                    in_header = true;
                }
                Event::End(Tag::Header(i)) if i <= max_section_depth => {
                    in_header = false;
                    //section_id = Some(utils::id_from_content(&heading));
                    section_id = Some(format!("head-{}", &heading_counter));

                    breadcrumbs.push(heading.clone());
                }
                Event::Start(Tag::FootnoteDefinition(name)) => {
                    let number = footnote_numbers.len() + 1;
                    footnote_numbers.entry(name).or_insert(number);
                }
                Event::Start(_) | Event::End(_) | Event::SoftBreak | Event::HardBreak => {
                    // Insert spaces where HTML output would usually seperate text
                    // to ensure words don't get merged together
                    if in_header {
                        heading.push(' ');
                    } else {
                        body.push(' ');
                    }
                }
                Event::Text(text) => {
                    if in_header {
                        heading.push_str(&text);
                    } else {
                        body.push_str(&text);
                    }
                }
                Event::Html(html) | Event::InlineHtml(html) => {
                    body.push_str(&clean_html(&html));
                }
                Event::FootnoteReference(name) => {
                    let len = footnote_numbers.len() + 1;
                    let number = footnote_numbers.entry(name).or_insert(len);
                    body.push_str(&format!(" [{}] ", number));
                }
            }
        }

        if !heading.is_empty() {
            // Make sure the last section is added to the index
            self.add_doc(
                &slug,
                &section_id,
                &[&heading, &body, &breadcrumbs.join(" » ")],
            );
        }

        Ok(())
    }

    /// Uses the given arguments to construct a search document, then inserts it to the given index.
    fn add_doc(&mut self, anchor_base: &str, section_id: &Option<String>, items: &[&str]) {
        let url = if let Some(ref id) = *section_id {
            Cow::Owned(format!("{}#{}", anchor_base, id))
        } else {
            Cow::Borrowed(anchor_base)
        };
        let url = utils::collapse_whitespace(url.trim());
        let doc_ref = self.doc_urls.len().to_string();
        self.doc_urls.push(url.into());

        let items = items.iter().map(|&x| utils::collapse_whitespace(x.trim()));
        self.index.add_doc(&doc_ref, items);
    }

    fn write_to_json(self) -> Result<String> {
        use elasticlunr::config::{SearchBool, SearchOptions, SearchOptionsField};
        use std::collections::BTreeMap;

        #[derive(Serialize)]
        struct ResultsOptions {
            limit_results: u32,
            teaser_word_count: u32,
        }

        #[derive(Serialize)]
        struct SearchindexJson {
            /// The options used for displaying search results
            results_options: ResultsOptions,
            /// The searchoptions for elasticlunr.js
            search_options: SearchOptions,
            /// Used to lookup a document's URL from an integer document ref.
            doc_urls: Vec<String>,
            /// The index for elasticlunr.js
            index: elasticlunr::Index,
        }

        let mut fields = BTreeMap::new();
        let mut opt = SearchOptionsField::default();
        opt.boost = Some(self.config.search.boost_title);
        fields.insert("title".into(), opt);
        opt.boost = Some(self.config.search.boost_paragraph);
        fields.insert("body".into(), opt);
        opt.boost = Some(self.config.search.boost_hierarchy);
        fields.insert("breadcrumbs".into(), opt);

        let search_options = SearchOptions {
            bool: if self.config.search.use_boolean_and {
                SearchBool::And
            } else {
                SearchBool::Or
            },
            expand: self.config.search.expand,
            fields,
        };

        let results_options = ResultsOptions {
            limit_results: self.config.search.limit_results,
            teaser_word_count: self.config.search.teaser_word_count,
        };

        let json_contents = SearchindexJson {
            results_options,
            search_options,
            doc_urls: self.doc_urls,
            index: self.index,
        };

        // By converting to serde_json::Value as an intermediary, we use a
        // BTreeMap internally and can force a stable ordering of map keys.
        let json_contents =
            serde_json::to_value(&json_contents).ctx("Writing JSON Search Index")?;
        let json_contents =
            serde_json::to_string(&json_contents).ctx("Writing JSON Search Index")?;

        Ok(json_contents)
    }
}
