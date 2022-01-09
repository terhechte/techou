use techou;

#[test]
fn test_generate_site() {
    let config =
        techou::config::Config::from_file("site/project.toml").expect("a valid config");
    let cache = techou::build_cache::BuildCache::new("/tmp/cache.techou");
    techou::executor::execute(true, &config, &cache, None).expect("a working build");

    let o = config.folders.output_folder_path();
    assert!(o.exists());

    // Some initial validations
    let contents = techou::io_utils::slurp("site/html/index.html").expect("An index");
    assert!(contents.contains("searchbar-outer"));
    assert!(contents.contains("The Book is Plom"));
}
