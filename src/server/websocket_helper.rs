use rouille;
use rouille::websocket;

use super::state::*;

use std::thread;

pub fn websocket_handler(
    request: &rouille::Request,
    receiver: super::ReloadReceiver<BrowserResult>,
) -> rouille::Response {
    println!("New websocket connection from {}", &request.remote_addr());
    let (response, websocket) = websocket::start(&request, Some("echo")).unwrap();
    let cloned_receiver = receiver.clone();
    thread::spawn(move || {
        let ws = websocket.recv().unwrap();
        websocket_handling_thread(ws, cloned_receiver);
    });

    response
}

fn websocket_handling_thread(
    mut websocket: websocket::Websocket,
    receiver: super::ReloadReceiver<BrowserResult>,
) {
    // Empty the socket welcome message
    if let Some(msg) = websocket.next() {
        println!("Websocket Connect: {:?}", &msg);
    }
    let mut counter = 0;
    loop {
        let msg = match receiver.try_recv() {
            Ok(msg) => msg,
            Err(_) => {
                thread::sleep(std::time::Duration::from_millis(50));
                // If a socket disconnects in a state where our `receiver` does not
                // receive any updates anymore, the `loop` will go on.
                // Therefore, after each `try_recv` iteration on the receiver, we also
                // make sure that the socket still works.
                // It is sufficient to do this every 2 seconds though

                if counter > 10 {
                    counter = 0;
                    match websocket.send_text("ping") {
                        Ok(_) => continue,
                        Err(_) => {
                            println!("Lost old websocket");
                            break;
                        }
                    }
                } else {
                    counter += 1;
                    continue;
                }
            }
        };
        let result = match msg {
            Ok(BrowserAction::Reload) => dbg!(websocket.send_text("reload")),
            Err(Some(_errors)) => websocket.send_text("errors"),
            _ => {
                println!("Error: {:?}", &msg);
                continue;
            }
        };
        if result.is_err() {
            println!("Websocket closed: {:?}", result.err().unwrap());
            break;
        }
        if let Some(msg) = websocket.next() {
            println!("Reply: {:?}", &msg);
            // If the browser reloaded, it will open a new websocket, so we close this one
            break;
        }
    }
}
