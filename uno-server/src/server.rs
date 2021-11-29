use log::{ error, info };
use std::{
    net::{
        TcpListener, TcpStream,
    },
    thread,
    sync::atomic::{ AtomicUsize, Ordering },
    collections::HashMap,
};
use crate::{
    server_result::{ ServerResult, ServerError },
    client::Client,
    game::Game,
};
use uno::packet::{
    Packet, Command, PacketError,
    read_socket, write_socket,
};

const MAX_LOBBY_PLAYERS: usize = 10;
const MAX_LOBBIES: usize = 10;

fn new_lobby_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    COUNTER.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
        Some(value % (MAX_LOBBIES * 2) + 1)
    }).unwrap()

}

pub struct Server {
    listener: TcpListener,
    clients: Vec<Client>,
    // lobbies: Vec<Lobby>,
    lobbies: HashMap<usize, usize>,
    game_threads: Vec<thread::JoinHandle<()>>,
}

impl Server {
    fn new() -> ServerResult<Server> {
        Ok(Server {
            listener: TcpListener::bind("0.0.0.0:2905")?,
            clients: Vec::new(),
            lobbies: HashMap::new(),
            game_threads: Vec::new(),
        })
    }

    pub fn run() -> ServerResult<()> {
        let mut server = Server::new()?;
        server.listener.set_nonblocking(true)?;

        loop {
            server.new_connections()?;
            for client in server.clients.iter_mut() {
                client.incoming_packets = match read_socket(&mut client.socket) {
                    Ok(packets) => { info!("{:?}", packets); packets },
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                    Err(ref e) if let Some(&PacketError::ZeroSizePacket) = e.get_ref() => {
                        continue 
                    },
                    Err(e) => return Err(ServerError::IoError(e)),
                };
            }
            server.execute_commands()?;
        }
    }

    fn new_connections(&mut self) -> ServerResult<()> {
        match self.listener.accept() {
            Ok((socket, ip)) => {
                socket.set_nonblocking(true)?;
                self.clients.push(Client::new(socket));
                info!("New connection: {}", ip);
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {},
            Err(e) => error!("{}", e),
        };

        Ok(())
    }

    fn execute_commands(&mut self) -> ServerResult<()> {
        for client in self.clients.iter_mut() {
            for packet in client.incoming_packets.drain(..) {
                match packet.command {
                    Command::CreateLobby => {
                        if self.lobbies.len() < MAX_LOBBIES {
                            let new_lobby_id = new_lobby_id();
                            write_socket(&mut client.socket, Command::JoinLobby, new_lobby_id as u8)?;
                            self.lobbies.insert(new_lobby_id, 0);
                        } else {
                            write_socket(&mut client.socket, Command::Error, "Maximum lobby amount".as_bytes())?;
                        }
                    },
                    Command::JoinLobby => {
                        if let Some(lobby_id) = packet.args.get(0) {
                            if self.lobbies.contains_key(&(*lobby_id as usize)) {
                                *self.lobbies.get_mut(&(*lobby_id as usize)).unwrap() += 1;
                                write_socket(&mut client.socket, Command::JoinLobby, *lobby_id)?;
                            } else {
                                write_socket(&mut client.socket, Command::Error, "Lobby doesn't exist".as_bytes())?;
                            }
                        }
                    },
                    Command::LeaveLobby => {
                        if let Some(lobby_id) = packet.args.get(0) {
                            write_socket(&mut client.socket, Command::LeaveLobby, vec![])?;
                            if let Some(n) = self.lobbies.get_mut(&(*lobby_id as usize)) {
                                if *n > 0 {
                                    *n -= 1;
                                }
                            }
                        }
                    }
                    Command::LobbiesInfo => {
                        let mut infos = vec![];

                        for (id, n_players) in &self.lobbies {
                            infos.push(*id as u8);
                            infos.push(*n_players as u8);
                        }

                        write_socket(&mut client.socket, Command::LobbiesInfo, infos)?;
                    }
                    _ => {},
                }
            }
        }

        Ok(())
    }
}
