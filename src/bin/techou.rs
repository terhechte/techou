use std::env;
use std::path;
extern crate techou;

fn main() {
    techou::executor::execute(".").expect("Work?");
}