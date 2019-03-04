use notify::DebouncedEvent::*;
use notify::RecursiveMode::*;
use notify::Watcher;

use crate::config::Config;

use std::path;
use std::sync::mpsc;
use std::time::Duration;

pub fn reload<ActionFn>(
    paths: Vec<path::PathBuf>,
    config: &Config,
    action: ActionFn,
) -> mpsc::Receiver<bool>
where
    ActionFn: Fn(&path::Path, &Config) + std::marker::Send + std::marker::Sync + 'static,
{
    let (reload_sender, reload_receiver) = mpsc::channel();
    let inner_config = config.clone();
    let cloned_sender = reload_sender.clone();
    std::thread::spawn(move || {
        let cloned_sender = cloned_sender.clone();
        trigger_on_change(paths, move |path| {
            action(path, &inner_config);
            cloned_sender.send(true);
            println!("Done");
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

    let (delay_tx, delay_rx): (mpsc::Sender<path::PathBuf>, mpsc::Receiver<path::PathBuf>) = mpsc::channel();
    let delayed_control = std::thread::spawn(move || {
        while let Ok(msg) = delay_rx.recv() {
            closure(&msg);
        }
    });

    let mut last_receiver: Option<mpsc::Sender<bool>> = None;
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
                if let Some(existing) = last_receiver {
                    existing.send(true);
                    last_receiver = None;
                }
                let (tx, rx) = mpsc::channel();
                last_receiver = Some(tx);
                let delay_clone = delay_tx.clone();
                let inner_path = path.clone();
                std::thread::spawn(move || {
                    let delay = std::time::Duration::from_millis(500);
                    std::thread::sleep(delay);

                    // If something was send, leave early
                    if let Ok(true) = rx.try_recv() {
                        return;
                    }
                    // Otherwise, execute the closure
                    delay_clone.send(inner_path);
                });
            }
            _ => {}
        }
    }
}
