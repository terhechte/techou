use std::path;
use techou;

#[test]
fn test_config_generation() {
    let contents = include_str!("config.toml");
    let folder = path::PathBuf::new();
    let config = techou::config::Config::from_toml(&contents, folder).unwrap();
    assert!(config.project.keywords.contains(&"rust".to_string()));
    assert_eq!(
        config.rss.unwrap().description,
        Some("Various Tech Tidbits".to_owned())
    );
    assert_eq!(config.server.server_address, "134.122.89.213:80");
}
