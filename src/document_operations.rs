use crate::document::{Document, DocumentLink};
use crate::list::*;

use std::collections::BTreeMap;

pub fn posts_by_date<'a>(posts: &'a [Document]) -> Vec<Year<'a>> {
    let mut date_map: BTreeMap<i32, BTreeMap<u32, Vec<&'a Document>>> = BTreeMap::new();
    for post in posts {
        let year = date_map
            .entry(post.info.date_info.year)
            .or_insert_with(BTreeMap::new);
        let month = year.entry(post.info.date_info.month).or_insert(Vec::new());
        month.push(post);
    }
    date_map
        .into_iter()
        .rev()
        .map(|(year, entries)| {
            Year::from((year, entries.into_iter().rev().map(Month::from).collect()))
        })
        .collect()
}

pub fn posts_by_array<'a, RetrieveFn, D: AsRef<Document>>(
    posts: &'a [D],
    retriever: RetrieveFn,
) -> Vec<Category<'a>>
where
    RetrieveFn: Fn(&Document) -> &[String],
{
    let mut tag_map: BTreeMap<&'a str, Vec<&'a Document>> = BTreeMap::new();
    for post in posts {
        let post = post.as_ref();
        let strings = retriever(&post);
        for tag in strings {
            let tags = tag_map.entry(&tag).or_insert_with(Vec::new);
            tags.push(post);
        }
    }
    tag_map
        .into_iter()
        .rev()
        .map(|(tag, entries)| Category {
            name: tag,
            count: entries.len() as u32,
            posts: entries,
        })
        .collect()
}

pub fn make_similarity(for_documents: &mut Vec<Document>, amount: usize) {
    // FIXME: Do I really need to do it this complicated? Can't have &mut and & (obviously)
    // so iterating over items while also calculating something for all items is impossible
    // This is surprisingly tricky. We currently expect that the order does not change.
    let mut sims: Vec<Option<Vec<(u32, DocumentLink)>>> = for_documents
        .iter()
        .map(|document| Some(documents_by_similarity(document, &for_documents, amount)))
        .collect();
    for (index, d) in sims.iter_mut().enumerate() {
        for_documents[index].similar_documents = d.take().unwrap();
    }
}

pub fn make_document_siblings(for_documents: &mut Vec<Document>) {
    let mut previous: Option<DocumentLink> = None;
    let mut iter = for_documents.iter_mut().peekable();
    while let Some(doc) = iter.next() {
        doc.previous_document = previous.take();
        doc.next_document = iter.peek().map(|d| d.link());
        previous = Some(doc.link());
    }
}

pub fn documents_by_similarity<'a, D: AsRef<Document>>(
    to_document: &'a Document,
    in_documents: &'a [D],
    nr: usize,
) -> Vec<(u32, DocumentLink)> {
    // sort by similarity index
    // rudimentary implementation. I can think of tons of better ways but we're trying
    // to finish this thing.
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use strsim::normalized_damerau_levenshtein;
    let mut items = HashSet::new();
    let max_tags: usize = 10; // if we have more than 10 matches, we have a winner
    for tag in &to_document.info.tags {
        items.insert(tag);
    }
    let mut sorted: Vec<(u32, DocumentLink)> = in_documents
        .iter()
        .filter_map(|item| {
            let item = item.as_ref();
            if item.identifier == to_document.identifier {
                return None;
            }
            let titlen = normalized_damerau_levenshtein(&item.info.title, &to_document.info.title);
            let descn = normalized_damerau_levenshtein(
                &item.info.description,
                &to_document.info.description,
            );
            let tag_intersection = HashSet::from_iter(item.info.tags.iter())
                .intersection(&items)
                .count();
            let tagn = ::std::cmp::max(max_tags, tag_intersection) as f64 / max_tags as f64;
            // current dist is:
            // 35% title, 35% desc, 20% tags
            let similarity = (((titlen * 0.35) + (descn * 0.35) + (tagn * 0.2)) * 100.0) as u32;
            Some((similarity, item.link()))
        })
        .collect();
    sorted.sort_by_key(|k| k.0);
    sorted.into_iter().rev().take(nr).collect()
}

#[cfg(test)]
mod tests {
    use crate::document::Document;
    use crate::front_matter;

    #[test]
    fn test_relevance_simple() {
        use crate::document_operations::documents_by_similarity;
        let t1 = self::make_doc("1", "hey jude", "hey yude", &["a", "b", "c"]);
        let t2 = self::make_doc("2", "hei jude", "hey yude", &["a", "b", "c"]);
        let t3 = self::make_doc("3", "judehei ", "hey yude", &["a", "b", "c"]);
        let docs = [t2, t3];
        let d = documents_by_similarity(&t1, &docs, 2);
        assert_eq!(d[0].1.identifier, "2");
        assert_eq!(d[1].1.identifier, "3");
    }

    #[test]
    fn test_relevance_hard() {
        use crate::document_operations::documents_by_similarity;
        let t1 = self::make_doc(
            "1",
            "How to use pattern matching",
            "pattern matching can be used to simplify code",
            &["rust", "match"],
        );
        let t2 = self::make_doc(
            "2",
            "Rust Best Practices",
            "these are some best practices, like pattern matching, to help you improve",
            &["practice", "tips", "pattern"],
        );
        let t3 = self::make_doc(
            "3",
            "Compiler issues",
            "How to figure out what the borrow checker wants",
            &["borrow checker", "help"],
        );
        let docs = [t2, t3];
        let d = documents_by_similarity(&t1, &docs, 2);
        assert_eq!(d[0].1.identifier, "2");
        assert_eq!(d[1].1.identifier, "3");
    }

    fn make_doc(
        iden: &'static str,
        title: &'static str,
        desc: &'static str,
        tags: &'static [&'static str],
    ) -> Document {
        let tags: Vec<String> = tags.into_iter().map(|t| format!(r#""{}""#, t)).collect();
        let tgs = tags.join(",");
        let contents = format!(
            r#"
[frontMatter]
title = "{}"
tags = [{}]
created = "2009-12-30"
description = "{}"
published = true
---
this."#,
            title, tgs, desc
        );
        let (fm, _) =
            front_matter::parse_front_matter(&contents, "yeah.md", &Default::default()).unwrap();
        Document {
            identifier: iden.to_owned(),
            filename: "$".to_owned(),
            info: fm,
            slug: "yah".to_string(),
            content: "".to_string(),
            raw_content: "".to_string(),
            sections: Vec::new(),
            similar_documents: Vec::new(),
            previous_document: None,
            next_document: None,
            updated: true,
        }
    }
}
