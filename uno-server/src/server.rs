use log::{ error, info };
use std::{
    net::{
        TcpListener, TcpStream,
    },
    thread,
    sync::atomic::{ AtomicUsize, Ordering },
};
use crate::{
    server_result::ServerResult,
    client::Client,
    game::Game,
};
use uno::packet::{ Packet, Command };

struct Lobby {
    id: usize,
    number_players: usize,
}

impl PartialEq for Lobby {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Lobby {
    const MAX_PLAYERS: usize = 10;
    const MAX_LOBBIES: usize = 10;

    fn new() -> Self {
        Self {
            id: Lobby::get_lobby_id(),
            number_players: 0,
        }
    }

    fn with_id(id: usize) -> Self {
        Self {
            id,
            number_players: 0,
        }
    }

    fn get_lobby_id() -> usize {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        COUNTER.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            Some(value % (Lobby::MAX_LOBBIES * 2) + 1)
        }).unwrap()
    }
}

pub struct Server {
    listener: TcpListener,
    clients: Vec<Client>,
    lobbies: Vec<Lobby>,
    game_threads: Vec<thread::JoinHandle<()>>,
}

impl Server {
    fn new() -> ServerResult<Server> {
        Ok(Server {
            listener: TcpListener::bind("0.0.0.0:2905")?,
            clients: Vec::new(),
            lobbies: Vec::new(),
            game_threads: Vec::new(),
        })
    }

    pub fn run() -> ServerResult<()> {
        let mut server = Server::new()?;

        loop {
            server.new_connections()?;
            for client in server.clients.iter_mut() {
                client.read()?;
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
            Err(e) => error!("{}", e),
        };

        Ok(())
    }

    fn execute_commands(&mut self) -> ServerResult<()> {
        for client in self.clients.iter_mut() {
            for packet in client.incoming_packets.clone() {
                match packet.command {
                    Command::CreateLobby => {
                        if self.lobbies.len() < Lobby::MAX_LOBBIES {
                            let new_lobby = Lobby::new();
                            client.send(Command::JoinLobby, new_lobby.id as u8)?;
                            self.lobbies.push(new_lobby);
                        } else {
                            client.send(Command::Error, "Maximum lobby amount".as_bytes())?;
                        }
                    },
                    Command::JoinLobby => {
                        if let Some(lobby_id) = packet.args.get(0) {
                            if self.lobbies.contains(&Lobby::with_id(*lobby_id as usize)) {
                                client.send(Command::JoinLobby, *lobby_id)?;
                            } else {
                                client.send(Command::Error, "Lobby doesn't exist".as_bytes())?;
                            }
                        }
                    },
                    Command::LobbiesInfo => {
                        let mut infos = vec![];

                        for lobby in &self.lobbies {
                            infos.push(lobby.id as u8);
                            infos.push(lobby.number_players as u8);
                        }

                        client.send(Command::LobbiesInfo, infos)?;
                    }
                    _ => {},
                }
            }

            client.incoming_packets.drain(..);
        }

        Ok(())
    }
}
