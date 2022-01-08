use crate::config::Config;

use std::sync::mpsc::Receiver;

pub fn auto_reload_code(config: &Config) -> String {
    format!(
        r#"
<script language="Javascript">
    var wsUri = (window.location.protocol=='https:'&&'wss://'||'ws://')+window.location.host + '{}';
    let exampleSocket = new WebSocket(wsUri);
    exampleSocket.onopen = function (event) {{
      exampleSocket.send("Connect"); 
    }};
    exampleSocket.onmessage = function (event) {{
      //console.log(event.data);
      if (event.data == "reload") {{
        //window.setTimeout(document.location.reload, 250);
        document.location.reload();
      }}
    }}
</script>
    "#,
        &config.server.auto_reload_websocket_path
    )
}

pub fn run_file_server(_reload_receiver: Option<Receiver<bool>>, config: &Config) {
    let folder = config
        .folders
        .output_folder_path()
        .to_str()
        .expect("Expect output folder to serve")
        .to_string();

    println!(
        "Serving '{:?}' on {}",
        &folder, &config.server.server_address
    );
    use tiny_file_server::FileServer;
    let address = config.server.server_address.clone();
    FileServer::http(address.as_str())
        .expect("Server should be created")
        .run(&folder)
        .expect("Server should start");
}
