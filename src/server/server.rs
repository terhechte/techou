use crossbeam::channel;
use rouille;
use rouille::websocket;
use rouille::*;

use crate::config::Config;

use super::state::ServerState;
use super::websocket_helper::websocket_handler;
use super::{BrowserAction, BrowserResult};

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

pub fn run_file_server(
    reload_receiver: Option<super::ReloadReceiver<BrowserResult>>,
    config: &Config,
) {
    let folder = config
        .folders
        .output_folder_path()
        .to_str()
        .expect("Expect output folder to serve")
        .to_string();

    println!(
        "Serving '{:?} on http://{}'",
        &folder, &config.server.server_address
    );

    let state = Arc::new(ServerState {
        receiver: reload_receiver.map(|e| Mutex::new(e)),
        websocket_payload: auto_reload_code(),
        serve_dir: PathBuf::from(folder),
    });

    rouille::start_server(&config.server.server_address, move |request| {
        router!(request,
                (GET) (/ws) => {
                    websocket_handler(&request, Arc::clone(&state))
                },
                _ => {
                    let path: PathBuf = match request.url().as_str() {
                        "/" => state.serve_dir.join("index.html"),
                        p => state.serve_dir.join(&p[1..].to_owned())
                    };
                    let is_html = path.extension().map(|e|e.to_str().unwrap_or("")).unwrap_or("")
                        .ends_with("html");
                    match is_html {
                        true => {
                            modified_file_contents(&path, &state.websocket_payload)
                                .map(rouille::Response::html)
                                .unwrap_or(rouille::Response::empty_404())
                        },
                        false => {
                            let file = match std::fs::File::open(&path) {
                                Err(_) => return rouille::Response::empty_404(),
                                Ok(f) => f
                            };
                            let filetype = terrible_extension_to_mimetype(&path);
                            rouille::Response::from_file(filetype, file)
                        }
                    }.with_no_cache()
                }
        )
    });
}

fn auto_reload_code() -> String {
    format!(
        r#"
<script language="Javascript">
    var wsUri = (window.location.protocol=='https:'&&'wss://'||'ws://')+window.location.host + '/ws';
    let socket = new WebSocket(wsUri, "echo");
    socket.onopen = function (event) {{
      console.log("Websocket connect");
      socket.send("Connect");
    }};
    socket.onmessage = function (event) {{
      console.log("Websocket receive:", event.data);
      if (event.data == "reload") {{
        socket.send("success");
        document.location.reload();
      }}
    }}
</script>
    "#
    )
}

fn modified_file_contents(path: &Path, payload: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path).map(|string| {
        let mut mutable_string = string;
        mutable_string.push_str(payload);
        mutable_string
    })
}

fn terrible_extension_to_mimetype(filename: &Path) -> &'static str {
    let default = "application/octet-stream";
    extension_to_mime(
        filename
            .extension()
            .map(|e| e.to_str().unwrap_or(default))
            .unwrap_or(default),
    )
}
