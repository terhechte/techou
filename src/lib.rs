mod article;
mod list;
mod front_matter;
mod io_utils;
mod utils;
mod error;
mod template;
mod parse_event_handlers;
mod feeds;

pub mod server;
pub mod config;
pub mod executor;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
