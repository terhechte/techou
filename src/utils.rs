use regex::Regex;
use lazy_static::*;
use std::borrow::Cow;

pub fn hash_string(input: &str, length: usize) -> String {
    use sha2::Digest;
    use sha2::Sha256;
    let hash = format!("{:x}", Sha256::digest(input.as_bytes()));
    // 10 seems to be a good prefix for distinctness
    let (short_hash, _) = hash.split_at(length);
    short_hash.to_string()
}

pub fn slugify(input: &str) -> String {
    input
        .to_lowercase()
        .replace(
            |c: char| !c.is_ascii_alphanumeric() && !c.is_ascii_whitespace(),
            "",
        )
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
}

pub fn collapse_whitespace<'a>(text: &'a str) -> Cow<'a, str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\s\s+").unwrap();
    }
    RE.replace_all(text, " ")
}
