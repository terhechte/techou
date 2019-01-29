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
        match &self.scope {
            Some(ref s) => {
                //s.split(",").collect(),
                for selector in s.split(",") {
                    // some themes include a -(ignore). we ignore the ignore
                    if selector.contains("(") { continue }
                    // spaces define hierachies. We need to add dots
                    let replaced: Vec<String> = selector.trim().split(" ").map(|s| format!(".{}", &s)).collect();
                    results.push(self.css_entry(&root, &replaced.join(" ")));
                }
            },
            None => {
                results.push(self.css_entry(&root, "*"));
            }
        };
        results
    }

    fn css_entry(&self, root: &str, selector: &str) -> String {
        // Only use a dot for the non-global selector
        let mut result = format!("{} {} {{\n", &root, &selector);
        if let Some(ref fg) = self.foreground {
            result.push_str(&format!("\tcolor: {};\n", &fg));
        } if let Some(ref bg) = self.background {
            result.push_str(&format!("\tbackground-color: {};\n", &bg));
        }
        let translated = match self.font_style.as_ref().map(|s|s.as_str()) {
            Some("bold") => "font-weight: bold",
            Some("italic") => "font-style: italic",
            Some("underline") => "font-decoration: underline",
            _ => ""
        };
        if translated.len() > 0 {
            result.push_str(&format!("\t{};\n", &translated));
        }
        result.push_str("}\n");
        result
    }
}



fn main() {
    use plist;
    use clap::{Arg, App};
    use plist::Value;

    let matches = App::new("tm2css")
        .version("0.0.1")
        .author("Benedikt Terhechte")
        .arg(Arg::with_name("tm-theme").short("s").value_name("TM-THEME").required(true))
        .arg(Arg::with_name("root-selector").short("f").value_name("ROOT-SELECTOR").required(false))
        .arg(Arg::with_name("ignore-background").short("i").required(false))
        .get_matches();
    let path = matches.value_of("tm-theme").unwrap();
    let container_name = matches.value_of("root-selector").unwrap_or("pre > code");
    let ignore_background: bool = matches.is_present("ignore-background");

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
        if !ignore_background {
            if let Some(Value::String(background)) = settings.get("background") {
                current.background = Some(background.to_string());
            }
        }
        if let Some(Value::String(font_style)) = settings.get("fontStyle") {
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
