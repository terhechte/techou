use std::path;
use std::sync::mpsc;
use std::time::Duration;

use notify::Watcher;
use notify::DebouncedEvent::*;
use notify::RecursiveMode::*;

use crate::config::Config;

pub fn reload<ActionFn>(paths: Vec<path::PathBuf>, config: &Config, action: ActionFn) -> mpsc::Receiver<bool>
    where ActionFn: Fn(&path::Path, &Config) + std::marker::Send + 'static {
    let (reload_sender, reload_receiver) = mpsc::channel();
    //let inner_paths = paths.clone();
    let inner_config = config.clone();
    std::thread::spawn(move || {
        trigger_on_change(paths, |path| {
            action(path, &inner_config);
            &reload_sender.send(true);
            println!("Done");
        });
    });
    reload_receiver
}

fn trigger_on_change<F>(folders: Vec<path::PathBuf>, closure: F)
    where
        F: Fn(&path::Path),
{
    // Create a channel to receive the events.
    let (tx, rx) = mpsc::channel();

    let mut watcher = match notify::watcher(tx, Duration::from_secs(1)) {
        Ok(w) => w,
        Err(e) => {
            panic!("Error while trying to watch the files:\n\n\t{:?}", e);
            ::std::process::exit(1)
        }
    };

    // Add the source directory to the watcher
    for folder in folders {
        if let Err(e) = watcher.watch(&folder, Recursive) {
            panic!("Error while watching {:?}:\n    {:?}", &folder, e);
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
