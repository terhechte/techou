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
        .get_matches();
    let root_dir = matches.value_of("project-dir").unwrap_or(".");
    let project_file = matches.value_of("project-file").unwrap_or("");
    let should_watch = matches.is_present("watch");

    let config = match project_file.len() {
        0 => techou::config::Config::new(root_dir),
        _ => match techou::config::Config::file(project_file) {
            Ok(c) => c, Err(e) => panic!("Invalid Project File {:?}: {:?}", &project_file, &e)
        }
    };

    match techou::executor::execute(&config) {
        Err(e) => println!("Error: {:?}", &e),
        _ => ()
    };

    if should_watch {
        trigger_on_change(&[&config.public_folder_path(), &config.posts_folder_path()], |path| {
            match techou::executor::execute(&config) {
                Err(e) => println!("Error: {:?}", &e),
                _ => ()
            };
            println!("Done");
        });
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