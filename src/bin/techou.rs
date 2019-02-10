use clap::{Arg, App, SubCommand};
use std::env;
use std::path;

extern crate techou;

fn main() {
    let matches = App::new("techou")
        .version("0.0.1")
        .author("Benedikt Terhechte")
        .arg(Arg::with_name("project-dir").short("d").value_name("PROJECT-DIR").required(false))
        .arg(Arg::with_name("project-file").short("f").value_name("PROJECT-FILE").required(false))
        .arg(Arg::with_name("watch").short("w").long("watch").required(false))
        .arg(Arg::with_name("serve").short("s").long("serve").required(false))
        .subcommand(SubCommand::with_name("new")
            .about("Write a new post")
            .arg(Arg::with_name("filename")
                .value_name("FILENAME")
                .help("Optional filename. Otherwise techou will generate one")
                .required(false)))
        .subcommand(SubCommand::with_name("create")
            .about("Create new techou project (project.toml)")
            .arg(Arg::with_name("filename")
                .value_name("FILENAME")
                .help("Alternative name to project.toml ")
                .required(false)))
        .get_matches();
    let root_dir = matches.value_of("project-dir").unwrap_or(".");
    let project_file = matches.value_of("project-file").unwrap_or("");
    let should_watch = matches.is_present("watch");
    let should_serve = matches.is_present("serve");

    if let Some(matches) = matches.subcommand_matches("create") {
        if project_file.len() > 0 { panic!("You can't use --project-file / -f together with 'create'") }
        let new_project_file = matches.value_of("filename").unwrap_or("project.toml");
        let path = path::PathBuf::from(root_dir).join(new_project_file);
        if path.exists() {
            panic!("File {:?} already exists. Cowardly refusing to overwrite", &path);
        }
        techou::io_utils::spit(&path, techou::config::Config::exampleConfig());
        println!("New Config '{:?}' created.", &path);
        ::std::process::exit(0);
    }

    let mut config = match project_file.len() {
        0 => techou::config::Config::new(root_dir),
        _ => match techou::config::Config::file(project_file) {
            Ok(c) => c, Err(e) => panic!("Invalid Project File {:?}: {:?}", &project_file, &e)
        }
    };

    // If the server is on, the user is debugging, and we perform the auto reload
    config.server.auto_reload_browser_via_websocket_on_change = should_serve;

    if let Some(matches) = matches.subcommand_matches("new") {
        techou::new_post::interactive(&config);
    }

    let load_fn = |path: &path::Path, config: &techou::config::Config| {
        match techou::executor::execute(false, &config) {
            Err(e) => println!("Error: {:?}", &e),
            _ => ()
        };
    };

    // Do the first call
    load_fn(&path::PathBuf::from(root_dir), &config);

    let mut reload_receiver: Option<::std::sync::mpsc::Receiver<bool>> = None;
    if should_watch {
        let mut paths = vec![config.folders.public_folder_path(), config.folders.posts_folder_path()];
        if project_file.len() > 0 {
            paths.push(path::PathBuf::from(&project_file));
        }
        reload_receiver = Some(techou::reload::reload(paths, &config, load_fn));
    }

    if should_serve {
        techou::server::run_file_server(reload_receiver, &config);
    }
}

