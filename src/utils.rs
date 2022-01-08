use lazy_static::*;
use regex::Regex;
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

pub struct DebugTimer {
    main: std::time::Instant,
    sub: std::time::Instant,
    level: i32,
    enable: bool,
}

impl DebugTimer {
    pub fn begin(level: i32, config: &crate::config::Config) -> DebugTimer {
        // the default min level is 1 so that we always get at least one `>`
        DebugTimer {
            main: std::time::Instant::now(),
            sub: std::time::Instant::now(),
            level: level + 1,
            enable: config.project.debug_instrumentation,
        }
    }

    pub fn sub_step(&mut self, name: &str) {
        if !self.enable {
            return;
        }
        for _x in 0..=self.level {
            print!(">");
        }
        let next = std::time::Instant::now();
        println!(" {}: {:?}", name, next - self.sub);
        self.sub = next;
    }

    pub fn end(self) {
        if !self.enable {
            return;
        }
        for _x in 0..=self.level {
            print!(">");
        }
        let next = std::time::Instant::now();
        println!(" Finish {:?}:", next - self.main);
    }
}
