#![feature(drain_filter)]

use server::Server;
use simple_logger::SimpleLogger;

pub mod client;
pub mod game;
pub mod world;

mod server;

fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();
    Server::new().run();
}
