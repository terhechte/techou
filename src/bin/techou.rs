use clap::{App, Arg, SubCommand};

extern crate techou;

use std::path;
use std::path::PathBuf;

fn main() {
    let matches = App::new("techou")
        .version("0.0.1")
        .author("Benedikt Terhechte")
        .arg(
            Arg::with_name("project-dir")
                .short("d")
                .value_name("PROJECT-DIR")
                .required(false),
        )
        .arg(
            Arg::with_name("project-file")
                .short("f")
                .value_name("PROJECT-FILE")
                .required(false),
        )
        .arg(
            Arg::with_name("watch")
                .short("w")
                .long("watch")
                .required(false),
        )
        .arg(
            Arg::with_name("serve")
                .short("s")
                .long("serve")
                .required(false),
        )
        .subcommand(
            SubCommand::with_name("new").about("Write a new post").arg(
                Arg::with_name("filename")
                    .value_name("FILENAME")
                    .help("Optional filename. Otherwise techou will generate one")
                    .required(false),
            ),
        )
        .subcommand(
            SubCommand::with_name("create")
                .about("Create new techou project (project.toml)")
                .arg(
                    Arg::with_name("filename")
                        .value_name("FILENAME")
                        .help("Alternative name to project.toml ")
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("scaffold-book")
                .about("Generate the folder structure and files for a book scaffolding")
                .arg(
                    Arg::with_name("filename")
                        .value_name("FILENAME")
                        .help("Path to the `summary.toml` to use")
                        .required(true),
                ),
        )
        .get_matches();
    let root_dir = matches.value_of("project-dir").unwrap_or(".");
    let project_file = matches.value_of("project-file").unwrap_or("");
    let should_watch = matches.is_present("watch");
    let should_serve = matches.is_present("serve");

    if let Some(matches) = matches.subcommand_matches("create") {
        if !project_file.is_empty() {
            panic!("You can't use --project-file / -f together with 'create'")
        }
        let new_project_file = matches.value_of("filename").unwrap_or("project.toml");
        let path = path::PathBuf::from(root_dir).join(new_project_file);
        if path.exists() {
            panic!(
                "File {:?} already exists. Cowardly refusing to overwrite",
                &path
            );
        }
        techou::io_utils::spit(&path, techou::config::Config::example_config()).expect("Expect to write config");
        println!("New Config '{:?}' created.", &path);
        ::std::process::exit(0);
    }

    let mut config = match project_file.len() {
        0 => techou::config::Config::new(root_dir),
        _ => match techou::config::Config::file(project_file) {
            Ok(c) => c,
            Err(e) => panic!("Invalid Project File {:?}: {:?}", &project_file, &e),
        },
    };

    // If we have the task to scaffold a book structure, we do that
    if let Some(matches) = matches.subcommand_matches("scaffold-book") {
        let scaffold_file = matches.value_of("filename").expect("Expecting path summary.toml scaffold file");
        let path = std::path::PathBuf::from(&scaffold_file);
        let folder = path.parent().expect("Expecting parent folder for scaffold file");
        // we change the book folder to nothing, as in this case we just use the current folder

        let contents = match techou::io_utils::slurp(&path) {
            Ok(s) => s, Err(e) => panic!("Error Reading {:?}: {:?}", &path, &e),
        };
        let chapters = techou::book::parse_chapter(&contents, &folder, "");
        let path_buf = folder.to_path_buf();
        match techou::io_utils::generate_book_folders(&config, &chapters, &path_buf) {
            Ok(_) => (),
            Err(e) => panic!("Could not generate folders: {}", &e)
        };
        println!("Done.");
        ::std::process::exit(0);
    }

    // If the server is on, the user is debugging, and we perform the auto reload
    config.server.auto_reload_browser_via_websocket_on_change = should_serve;

    if let Some(_matches) = matches.subcommand_matches("new") {
        techou::new_post::interactive(&config);
    }

    let cache = techou::build_cache::BuildCache::new();
    let load_fn = move |_path: &path::Path, config: &techou::config::Config| {
        let cache_clone = cache.clone();
        match techou::executor::execute(false, &config, &cache_clone) {
            Err(e) => println!("Error: {:?}", &e),
            _ => (),
        };
    };

    // Do the first call
    load_fn(&path::PathBuf::from(root_dir), &config);

    let reload_receiver = if should_watch {
        let mut paths = vec![
            config.folders.public_folder_path(),
            config.folders.posts_folder_path(),
            config.folders.pages_folder_path(),
        ];

        if !config.folders.books.is_empty() {
            let book_root = PathBuf::from(&config.folders.books_folder);
            config.folders.books.iter().for_each(|book| {
                if let Some(folder) = PathBuf::from(&book).parent() {
                    if config.folders.books_folder.is_empty() {
                        paths.push(folder.to_path_buf());
                    } else {
                        paths.push(book_root.join(&folder));
                    }
                }
            })
        }

        if !project_file.is_empty() {
            paths.push(path::PathBuf::from(&project_file));
        }
        Some(techou::reload::reload(paths, &config, load_fn))
    } else { None };

    if should_serve {
        techou::server::run_file_server(reload_receiver, &config);
    }
}
