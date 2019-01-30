use crate::config::Config;

use actix::prelude::*;
use actix_web::{
    fs, http, middleware, server, ws, App, Error, HttpRequest, HttpResponse
};

use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use actix_broker::{BrokerSubscribe, BrokerIssue};

pub fn auto_reload_code(config: &Config) -> String {
    format!(r#"
<script language="Javascript">
    var exampleSocket = new WebSocket("ws://localhost:{}/{}");
    exampleSocket.onopen = function (event) {{
      exampleSocket.send("Connect"); 
    }};
    exampleSocket.onmessage = function (event) {{
      console.log(event.data);
      document.location.reload();
    }}
</script>
    "#, &config.server_port, &config.auto_reload_websocket_path)
}

#[derive(Clone, Debug, Message)]
struct ReloadMessage(String);

struct ReloadReceiverActor {
    receiver: Arc<Mutex<Receiver<bool>>>
}

impl ReloadReceiverActor {
    fn new(receiver: Arc<Mutex<Receiver<bool>>>) -> Self {
        ReloadReceiverActor {
            receiver
        }
    }
}

impl Actor for ReloadReceiverActor {
    type Context = ws::WebsocketContext<Self, AppState>;
    fn started(&mut self, ctx: &mut Self::Context) {
        /*for event in self.receiver.iter() {
            println!("Received do reload event");
        }*/
    }
}

struct ReloadWebSocketActor;

impl Actor for ReloadWebSocketActor {
    type Context = ws::WebsocketContext<Self, AppState>;
    fn started(&mut self, ctx: &mut Self::Context) {
        //self.subscribe_sync::<ReloadMessage>(ctx);
        println!("started actor");
        let wrapped_receiver = &ctx.state().state.clone();
        let r = wrapped_receiver.lock().unwrap();
        for event in r.iter() {
            println!("event: {:?}", &event);
            &ctx.text("hello javascript");
        }
    }
}

impl Handler<ReloadMessage> for ReloadWebSocketActor {
    type Result = ();
    fn handle(&mut self,  msg: ReloadMessage, ctx: &mut Self::Context) {
        // this is where we send a message
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for ReloadWebSocketActor {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        println!("Receveid message: {:?}", &msg);
        ctx.pong("hellow");
    }
}

impl ReloadWebSocketActor {
    fn new() -> Self {
        Self { }
    }

    /*fn start_receiving(&self, ctx: &mut <Self as Actor>::Context) {
        // Maybe this needs to be in a thread?
        //println!("received");
        self.subscribe_sync
        //println!("r: {:?}", self.receiver);
        /*for event in self.receiver.iter() {
            println!("Received do reload event");
        }*/
    }*/
}

struct AppState {
    state: Arc<Mutex<Receiver<bool>>>
}

pub fn run_file_server(reload_receiver: Receiver<bool>, config: &Config) {
    let folder = config.output_folder_path().to_str()
        .expect("Expect output folder to serve").to_string();
    println!("Serving '{}' on {}", &folder, &config.server_port);
    let receiver = Arc::new(Mutex::new(reload_receiver));
    //let receive_actor = ReloadReceiverActor::new(receiver);
    server::new(move || {
        App::with_state(AppState { state: receiver.clone() })
            //new()
            .resource("/ws/", |r| r.method(http::Method::GET).f(|req| ws::start(req, ReloadWebSocketActor::new())))
            .handler(
                "/",
                fs::StaticFiles::new(&folder)
                    .unwrap()
                    .show_files_listing())
            .finish()
    })
        .bind("127.0.0.1:8001")
        .expect("Can not bind to port 8001")
        .run();
}
