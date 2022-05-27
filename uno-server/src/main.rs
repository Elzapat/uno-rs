#![feature(drain_filter)]

use log::error;
use server::Server;
use simple_logger::SimpleLogger;

pub mod client;
pub mod game;

mod server;

fn main() {
    SimpleLogger::new().init().unwrap();

    if let Err(e) = Server::new().run() {
        error!("{}", e);
    }
}
