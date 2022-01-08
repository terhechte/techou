use crate::config::Config;
use crate::front_matter;
use crate::io_utils;
use crate::utils;

pub fn interactive(config: &Config) {
    let created_time = front_matter::default_date_time(&config);
    let flags = &[
        ("title", "The title of this post", None),
        (
            "date",
            "The date/time for this post",
            Some(created_time.as_str()),
        ),
        ("filename", "The filename for this post", Some("filename")),
    ];
    use std::io;
    #[derive(Default)]
    struct Options {
        filename: String,
        title: String,
        date: String,
    }
    let mut options: Options = Default::default();
    for (key, title, default_value) in flags {
        println!("# {}", &title);
        let default = default_value.map(|d| match d {
            "filename" => options.filename.clone(),
            _ => d.to_string(),
        });
        if let Some(ref default) = default {
            println!("  (Default is `{}`)", &default);
        }
        loop {
            let mut input = String::new();
            let res = io::stdin().read_line(&mut input);
            let mut trimmed = input.trim().to_string();
            match (&default, trimmed.len()) {
                (Some(ref d), n) if n == 0 => trimmed = d.clone(),
                _ => (),
            }
            match res {
                Ok(_) if trimmed.is_empty() => {
                    println!("You have to enter a value");
                    continue;
                }
                Ok(_) => {
                    match *key {
                        "title" => {
                            // FIXME: there should be a config option with format syntax that
                            // allows the user to define how to generate post names
                            options.filename = utils::slugify(&trimmed);
                            options.filename.push_str(".md");
                            options.title = trimmed;
                            break;
                        }
                        "date" => {
                            match front_matter::detect_date_time(&trimmed, &config) {
                                Ok(d) => {
                                    options.date = d.0;
                                    break;
                                }
                                Err(e) => println!(
                                    "Invalid Date / Time Format. [Hint: {}]\n{}",
                                    &config.dates.date_format, e
                                ),
                            }
                            continue;
                        }
                        "filename" => {
                            options.filename = trimmed;
                            break;
                        }
                        _ => panic!("Invalid key {}", &key),
                    }
                }
                Err(error) => {
                    println!("error: {}", error);
                    continue;
                }
            }
        }
    }
    // Finally we can write it
    let post_path = config.folders.posts_folder_path().join(&options.filename);
    if post_path.exists() {
        println!(
            "Cowardly refusing to override existing post {:?}",
            &post_path
        );
        ::std::process::exit(0);
    }
    let front_matter = front_matter::default_front_matter(&options.title, &options.date);
    let content = front_matter::join_front_matter_with_content(&front_matter, "\n# Hello World");
    io_utils::spit(&post_path, &content).expect("Could not write to path");
    println!("Created new post {:?}", &post_path);
    ::std::process::exit(0);
}
