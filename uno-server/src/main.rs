use simple_logger::SimpleLogger;
use log::error;
use server::Server;

pub mod client;
pub mod game;

mod server;

fn main() {
    SimpleLogger::new().init().unwrap();

    if let Err(e) = Server::run() {
        error!("{}", e);
    }
}
