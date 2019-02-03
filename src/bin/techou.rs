use std::env;
use std::path;
use std::sync::mpsc::channel;
use std::time::Duration;

use notify::Watcher;
use notify::DebouncedEvent::*;
use notify::RecursiveMode::*;

use clap::{Arg, App, SubCommand};

extern crate techou;

fn main() {
    let matches = App::new("techou")
        .version("0.0.1")
        .author("Benedikt Terhechte")
        .arg(Arg::with_name("project-dir").short("d").value_name("PROJECT-DIR").required(false))
        .arg(Arg::with_name("project-file").short("f").value_name("PROJECT-FILE").required(false))
        .arg(Arg::with_name("watch").short("w").long("watch").required(false))
        .arg(Arg::with_name("serve").short("s").long("serve").required(false))
        .subcommand(SubCommand::with_name("new")
            .about("Write a new post")
            .arg(Arg::with_name("filename")
                .value_name("FILENAME")
                .help("Optional filename. Otherwise techou will generate one")
                .required(false)))
        .subcommand(SubCommand::with_name("create")
            .about("Create new techou project (project.toml)")
            .arg(Arg::with_name("filename")
                .value_name("FILENAME")
                .help("Alternative name to project.toml ")
                .required(false)))
        .get_matches();
    let root_dir = matches.value_of("project-dir").unwrap_or(".");
    let project_file = matches.value_of("project-file").unwrap_or("");
    let should_watch = matches.is_present("watch");
    let should_serve = matches.is_present("serve");

    if let Some(matches) = matches.subcommand_matches("create") {
        if project_file.len() > 0 { panic!("You can't use --project-file / -f together with 'create'") }
        let new_project_file = matches.value_of("filename").unwrap_or("project.toml");
        let path = path::PathBuf::from(root_dir).join(new_project_file);
        if path.exists() {
            panic!("File {:?} already exists. Cowardly refusing to overwrite", &path);
        }
        techou::io_utils::spit(&path, techou::config::Config::exampleConfig());
        println!("New Config '{:?}' created.", &path);
        ::std::process::exit(0);
    }

    let config = match project_file.len() {
        0 => techou::config::Config::new(root_dir),
        _ => match techou::config::Config::file(project_file) {
            Ok(c) => c, Err(e) => panic!("Invalid Project File {:?}: {:?}", &project_file, &e)
        }
    };

    if let Some(matches) = matches.subcommand_matches("new") {
        let flags = &[
            ("title", "The title of this post", None),
            ("date", "The date/time for this post", Some("2016.1.1")),
            ("filename", "The filename for this post", Some("filename"))
        ];
        use std::io;
        #[derive(Default)]
        struct Options {
            filename: String,
            title: String,
            date: String,
            tags: Vec<String>
        }
        let mut options: Options = Default::default();
        for (key, title, default_value) in flags {
            println!("{}", &title);
            if let Some(default) = default_value {
                println!("(Default is `{}`)", match default {
                    &"filename" => options.filename,
                    _ => default.to_string()
                });
            }
            loop {
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(n) if n == 0 => {
                        println!("You have to enter a value");
                        continue;
                    },
                    Ok(n) => {
                        match key {
                            &"title" => {
                                // FIXME: there should be a config option with format syntax that
                                // allows the user to define how to generate post names
                                options.filename = input.to_lowercase()
                                    .replace(|c: char| !c.is_ascii_alphanumeric() && !c.is_ascii_whitespace(), "")
                                    .split_whitespace().collect::<Vec<&str>>().join("-");
                                options.filename.push_str(".md");
                                options.title = input;
                                break;
                            },
                            &"date" => {
                                match techou::front_matter::detect_date_time(&input, &config) {
                                    Ok(d) => options.date = d.0,
                                    Err(e) => println!("Invalid Date / Time Format. [Hint: {}]", &config.dates.date_format),
                                }
                                continue;
                            },
                            &"filename" => {
                                options.filename = input;
                                break;
                            },
                            _ => panic!("Invalid key {}", &key)
                        }
                    },
                    Err(error) =>  {
                        println!("error: {}", error);
                        continue;
                    }
                }
            }
        }
        // Finally we can write it
        let post_path = config.folders.posts_folder_path().join(&options.filename);
        if post_path.exists() {
            println!("Cowardly refusing to override existing post {:?}", &post_path);
            ::std::process::exit(0);
        }
        // FIXME: This should come from articles
        let content = format!(r#"[frontMatter]
title = "{}"
tags = []
created = "{}"
description = ""
published = false
---

# Hello World"#, &options.title, &options.date);
        techou::io_utils::spit(&post_path, &content);
        println!("Created new post {:?}", &post_path);
        ::std::process::exit(0);
    }

    match techou::executor::execute(false, &config) {
        Err(e) => println!("Error: {:?}", &e),
        _ => ()
    };

    let (reload_sender, reload_receiver) = channel();
    if should_watch {
        let innerConfig = config.clone();
        std::thread::spawn(move || {
            trigger_on_change(&[&innerConfig.folders.public_folder_path(), &innerConfig.folders.posts_folder_path()], |path| {
                match techou::executor::execute(true, &innerConfig) {
                    Err(e) => println!("Error: {:?}", &e),
                    _ => ()
                };
                &reload_sender.send(true);
                println!("Done");
            });
        });
    }

    if should_serve {
        techou::server::run_file_server(reload_receiver, &config);
    }
}

fn trigger_on_change<A: AsRef<path::Path>, F>(folders: &[A], closure: F)
    where
        F: Fn(&path::Path),
{
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    let mut watcher = match notify::watcher(tx, Duration::from_secs(1)) {
        Ok(w) => w,
        Err(e) => {
            panic!("Error while trying to watch the files:\n\n\t{:?}", e);
            ::std::process::exit(1)
        }
    };

    // Add the source directory to the watcher
    for folder in folders {
        if let Err(e) = watcher.watch(folder.as_ref(), Recursive) {
            panic!("Error while watching {:?}:\n    {:?}", &folder.as_ref(), e);
            ::std::process::exit(1);
        };
    }


    // Add the book.toml file to the watcher if it exists
    // FIXME: add support for our config file once there is one
    //let _ = watcher.watch(book.root.join("book.toml"), NonRecursive);

    println!("Listening for changes...");

    for event in rx.iter() {
        println!("Received filesystem event: {:?}", event);
        match event {
            Create(path) | Write(path) | Remove(path) | Rename(_, path) => {
                closure(&path);
            }
            _ => {}
        }
    }
}
