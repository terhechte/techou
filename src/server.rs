use actix_web::{server, App, fs};

use crate::config::Config;

pub fn run_file_server(config: &Config) {
    let folder = config.output_folder_path().to_str()
        .expect("Expect output folder to serve").to_string();
    println!("Serving '{}' on 8001", &folder);
    server::new(move || {
        App::new()
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