use crate::document::Document;
use sitemap;
use std::fs::File;
use std::path::Path;

pub struct SiteMap<'a> {
    url_writer: sitemap::writer::UrlSetWriter<File>,
    base_url: &'a str,
}

impl<'a> SiteMap<'a> {
    pub fn new<A: AsRef<Path>>(outfile: A, base_url: &'a str) -> SiteMap {
        let path = outfile.as_ref();
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .expect("expecting proper sitemap path");
        let writer = sitemap::writer::SiteMapWriter::new(file);
        let mut url_writer = writer.start_urlset().expect("Unable to write urlset");
        // Store the base URL (i.e. the index)
        let entry = sitemap::structs::UrlEntry::builder().loc(base_url);
        url_writer.url(entry).unwrap();
        SiteMap {
            url_writer,
            base_url,
        }
    }

    pub fn add_document(&mut self, document: &Document) {
        // FIXME: Add support for last-updated
        let entry = sitemap::structs::UrlEntry::builder()
            .loc(format!("{}{}", &self.base_url, &document.slug));
        self.url_writer.url(entry).unwrap();
    }

    pub fn finish(self) {
        self.url_writer.end().expect("Expect closing");
    }
}
