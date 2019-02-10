use crate::config::Config;

use actix::prelude::*;
use actix_web::{
    fs, http, middleware, server, ws, App, Error, HttpRequest, HttpResponse
};
use std::time::{Instant, Duration};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};


pub fn auto_reload_code(config: &Config) -> String {
    format!(r#"
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
    "#, &config.server.auto_reload_websocket_path)
}

#[derive(Message)]
enum ReloadMessage {
    Reload
}

struct ReloadWebSocketActor {
    last_handle: Option<SpawnHandle>
}

impl Actor for ReloadWebSocketActor {
    type Context = ws::WebsocketContext<Self, AppState>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl ReloadWebSocketActor {
    fn new() -> Self {
      ReloadWebSocketActor {
          last_handle: None
      }
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        {
            if &ctx.state().state.is_none() == &true {
                return;
            }
        }

        // if the receiver is nil, don't do anything
        // check every 100ms
        ctx.run_interval(Duration::from_millis(100), |act, ctx| {
            // we will only get here if we do have a state
            let wrapped_receiver = &ctx.state().state.clone();
            let r = match wrapped_receiver {
                Some(x) => x,
                None => return
            };
            let x = r.lock().unwrap();
            let mut iter = x.try_iter();
            // Consume the iterator while also getting the last value. if we have
            // one value, it is true, if we have more values, the last is also true
            // This way, we only reload once even if multiple `reload`s did make it into the iterator
            if iter.last() == Some(true) {
                // If we have multiple updates, only the last one is valid
                if let Some(handle) = act.last_handle.take() {
                    let c = ctx.cancel_future(handle);
                    println!("cancelled future: {}", c);
                }
                /*let handle = ctx.run_later(Duration::from_millis(250), |act, ctx| {
                    println!("told to reload");
                    ctx.text("reload");
                });*/
                let handle = ctx.notify_later(ReloadMessage::Reload, Duration::from_millis(1000));
                act.last_handle.replace(handle);
            }
        });
    }
}

impl Handler<ReloadMessage> for ReloadWebSocketActor {
    type Result = ();
    fn handle(&mut self, msg: ReloadMessage, ctx: &mut Self::Context) {
        match msg {
            ReloadMessage::Reload => {
                println!("told to reload");
                ctx.text("reload");
            }
        }
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for ReloadWebSocketActor {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
    }
}

struct AppState {
    state: Option<Arc<Mutex<Receiver<bool>>>>
}

pub fn run_file_server(reload_receiver: Option<Receiver<bool>>, config: &Config) {
    let sys = actix::System::new("techou");

    let folder = config.folders.output_folder_path().to_str()
        .expect("Expect output folder to serve").to_string();
    let ws_path = config.server.auto_reload_websocket_path.clone();

    println!("Serving '{:?}' on {}", &folder, &config.server.server_address);
    let receiver: Option<Arc<Mutex<Receiver<bool>>>> = match reload_receiver {
        Some(o) => Some(Arc::new(Mutex::new(o))),
        None => None
    };

    server::new(move || {
        App::with_state(AppState { state: receiver.clone() })
            .resource(&ws_path, |r| r.method(http::Method::GET).f(|req| {
                ws::start(req, ReloadWebSocketActor::new())
            }))
            .handler(
                "/",
                fs::StaticFiles::new(&folder)
                    .unwrap()
                    .show_files_listing())
            .finish()
    })
        .bind(&config.server.server_address)
        .expect(&format!("Can not bind to {}", &config.server.server_address))
        .start();

    sys.run();
}
