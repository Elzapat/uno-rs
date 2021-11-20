use std::net::{ TcpListener, TcpStream };
use std::thread;
use std::io::{ Read, Write };
use uno::packet::Packet;
use crate::server_result::ServerResult;
use crate::client::Client;
use crate::game::Game;
use log::{ error, info };

pub struct Server {
    listener: TcpListener,
    clients: Vec<Client>,
    game_threads: Vec<thread::JoinHandle<()>>,
}

impl Server {
    fn new() -> ServerResult<Server> {
        Ok(Server {
            listener: TcpListener::bind("0.0.0.0:2905")?,
            clients: Vec::new(),
            game_threads: Vec::new(),
        })
    }

    pub fn run() -> ServerResult<()> {
        let mut server = Server::new()?;

        loop {
            server.new_connections()?;
            server.read_sockets()?;
        }
    }

    fn new_connections(&mut self) -> ServerResult<()> {
        match self.listener.accept() {
            Ok((socket, ip)) => {
                socket.set_nonblocking(true)?;
                self.clients.push(Client::new(socket));
                info!("New connection: {}", ip);
            },
            Err(e) => error!("{}", e),
        };

        Ok(())
    }

    fn read_sockets(&mut self) -> ServerResult<()> {
        let mut buffer = [0; 32];
        let mut packets = Vec::new();

        for client in self.clients.iter_mut() {
            let size = client.socket.read(&mut buffer)?;

            for i in 0..size {
                if uno::packet::DELIMITER == i as u8 {
                    packets.push((&client.socket, client.current_packet.clone()));
                    client.current_packet.drain(..);
                } else {
                    client.current_packet.push(buffer[i]);
                }
            }
        }

        for (socket, packet) in packets {
            let packet = Packet::parse_packet(packet)?;
        }

        Ok(())
    }
}
