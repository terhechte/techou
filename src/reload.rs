use notify::DebouncedEvent::*;
use notify::RecursiveMode::*;
use notify::Watcher;

use crate::config::Config;

use crate::server::BrowserAction;
use crate::server::BrowserResult;
use std::path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

pub fn reload<ActionFn>(
    paths: Vec<path::PathBuf>,
    config: &Config,
    action: ActionFn,
) -> crossbeam::channel::Receiver<BrowserResult>
where
    ActionFn: Fn(&path::Path, &Config) + std::marker::Send + std::marker::Sync + 'static,
{
    let (reload_sender, reload_receiver) = crossbeam::channel::unbounded();
    let inner_config = config.clone();
    let cloned_sender = reload_sender.clone();

    if !paths.is_empty() {
        println!("Observing changes in the following files/folders:");
        for path in paths.iter() {
            println!("  {}", path.display());
        }
    }

    std::thread::spawn(move || {
        let cloned_sender = cloned_sender.clone();
        trigger_on_change(paths, move |path| {
            action(path, &inner_config);
            cloned_sender
                .send(BrowserResult::Ok(BrowserAction::Reload))
                .unwrap();
        });
    });
    reload_receiver
}

fn trigger_on_change<F>(folders: Vec<path::PathBuf>, closure: F)
where
    F: std::marker::Send + 'static,
    F: Fn(&path::Path),
{
    // Create a channel to receive the events.
    let (tx, rx) = mpsc::channel();

    let mut watcher = match notify::watcher(tx, Duration::from_secs(1)) {
        Ok(w) => w,
        Err(e) => {
            panic!("Error while trying to watch the files:\n\n\t{:?}", e);
        }
    };

    // Add the source directory to the watcher
    for folder in folders {
        if let Err(e) = watcher.watch(&folder, Recursive) {
            panic!("Error while watching {:?}:\n    {:?}", &folder, e);
        };
    }

    // Add the book.toml file to the watcher if it exists
    // FIXME: add support for our config file once there is one
    //let _ = watcher.watch(book.root.join("book.toml"), NonRecursive);

    println!("Listening for changes...");

    let (delay_tx, delay_rx): (mpsc::Sender<path::PathBuf>, mpsc::Receiver<path::PathBuf>) =
        mpsc::channel();
    std::thread::spawn(move || {
        let mut last: Option<PathBuf> = None;
        loop {
            // if we have an event, we wait another brief time. Only if there're no new
            // events, do we forward
            if let Ok(n) = delay_rx.recv_timeout(std::time::Duration::from_millis(400)) {
                println!("      File change {:?}", &n);
                last = Some(n);
                continue;
            }
            if let Some(n) = last.take() {
                println!("Trigger Reload: {:?}", &n);
                closure(&n);
            }
        }
    });

    for event in rx.iter() {
        println!("Received filesystem event: {:?}", event);
        match event {
            Create(path) | Write(path) | Remove(path) | Rename(_, path) => {
                // ignore changes to hidden files. this includes
                // emacs or vim temporary buffers and more
                // FIXME: Use and_then
                if let Some(fname_os) = path.as_path().file_name() {
                    if let Some(fname) = fname_os.to_str() {
                        if fname.get(0..1) == Some(".") {
                            println!("Ignoring hidden file change: {:?}", &path);
                            continue;
                        }
                    }
                }
                delay_tx.send(path.clone()).unwrap();
            }
            _ => {}
        }
    }
}
