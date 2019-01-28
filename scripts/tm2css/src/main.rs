struct StyleEntry {
    scope: Option<String>,
    foreground: Option<String>,
    background: Option<String>,
    font_style: Option<String>,
}
impl Default for StyleEntry {
    fn default() -> Self {
        StyleEntry {
            scope: None,
            foreground: None,
            background: None,
            font_style: None
        }
    }
}

impl StyleEntry {
    fn css_entries(&self, root: &str) -> Vec<String> {
        let mut results: Vec<String> = Vec::new();
        let selectors: Vec<&str> = match &self.scope {
            Some(ref s) => s.split(".").collect(),
            None => vec!["*"]
        };
        for selector in selectors {
            results.push(self.css_entry(&root, &selector));
        }
        results
    }

    fn css_entry(&self, root: &str, selector: &str) -> String {
        let mut result = format!("{} > .{} {{\n", &root, &selector);
        if let Some(ref fg) = self.foreground {
            result.push_str(&format!("\tcolor: {};\n", &fg));
        }
        if let Some(ref bg) = self.background {
            result.push_str(&format!("\tbackground-color: {};\n", &bg));
        }
        result.push_str(&format!("\t{}\n", match self.font_style.as_ref().map(|s|s.as_str()) {
            Some("bold") => "font-weight: bold;",
            Some("italic") => "font-style: italic;",
            Some("underline") => "font-decoration: underline;",
            _ => ""
        }));
        result.push_str("}\n");
        result
    }
}



fn main() {
    use plist;
    use std::io;
    use std::io::prelude::*;
    use std::fs::File;
    use clap::{Arg, App, SubCommand};
    use plist::Value;

    let matches = App::new("tm2css")
        .version("0.0.1")
        .author("Benedikt Terhechte")
        .arg(Arg::with_name("tm-theme").short("s").value_name("TM-THEME").required(true))
        .arg(Arg::with_name("root-selector").short("f").value_name("ROOT-SELECTOR").required(false))
        .get_matches();
    let path = matches.value_of("tm-theme").unwrap();
    let container_name = matches.value_of("root-selector").unwrap_or("pre > code");

    let value = Value::from_file(&path).unwrap();
    let dict = match value {
        Value::Dictionary(dict) => dict,
        _ => panic!("Wrong type")
    };
    if let Some(Some(author)) = dict.get("author").map(|s|s.as_string()) {
        println!("/* Author: {} */", &author);
    }
    if let Some(Some(name)) = dict.get("name").map(|s|s.as_string()) {
        println!("/* Name: {} */", &name);
    }

    let theme_settings = match dict.get("settings") {
        Some(Value::Array(array)) => array,
        _ => panic!("Wrong Array Type")
    };
    let mut entries: Vec<StyleEntry> = Vec::new();
    for item in theme_settings {
        let item_dict = match item {
            Value::Dictionary(d) => d,
            _ => continue
        };
        let settings = match item_dict.get("settings") {
            Some(Value::Dictionary(s)) => s,
            _ => continue
        };
        let mut current: StyleEntry = Default::default();
        if let Some(Value::String(scope)) = item_dict.get("scope") {
            current.scope = Some(scope.to_string());
        }
        if let Some(Value::String(foreground)) = settings.get("foreground") {
            current.foreground = Some(foreground.to_string());
        }
        if let Some(Value::String(background)) = item_dict.get("background") {
            current.background = Some(background.to_string());
        }
        if let Some(Value::String(font_style)) = item_dict.get("fontStyle") {
            current.font_style = Some(font_style.to_string());
        }

        entries.push(current);
    }
    for e in entries {
        for s in e.css_entries(&container_name) {
            println!("{}", &s);
        }
    }
}
