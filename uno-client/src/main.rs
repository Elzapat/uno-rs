#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
mod game;
mod menu;
mod network;
pub mod utils;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_slinet::{
    client::{ClientPlugin, ConnectionEstablishEvent, PacketReceiveEvent},
    packet_length_serializer::LittleEndian,
    protocols::tcp::TcpProtocol,
    serializers::bincode::{BincodeSerializer, DefaultOptions},
    ClientConfig,
};
use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use tungstenite::WebSocket;
use uno::packet::{read_socket, Packet};
use utils::drag_and_drop::*;

struct NetworkConfig;

impl ClientConfig for NetworkConfig {
    type ClientPacket = NetworkPacket;
    type ServerPacket = NetworkPacket;
    type Protocol = TcpProtocol;
    type Serializer = BincodeSerializer<DefaultOptions>;
    type LengthSerializer = LittleEndian<u32>;
}

#[derive(Serialize, Deserialize, Debug)]
struct NetworkPacket(Vec<u8>);

// States
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameState {
    Lobbies,
    Game,
    EndLobby,
}

// Components
#[derive(Component)]
pub struct SpriteSize {
    width: f32,
    height: f32,
}
#[derive(Debug, Component)]
pub struct Server {
    socket: WebSocket<TcpStream>,
}

// Resources
pub struct Settings {
    username: String,
    enable_animations: bool,
}
#[derive(Deref, DerefMut)]
pub struct IncomingPackets(Vec<Packet>);

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Uno!".to_owned(),
            // width: 1920.0,
            // height: 1080.0,
            // vsync: false,
            ..WindowDescriptor::default()
        })
        .insert_resource(IncomingPackets(Vec::new()))
        .add_state(GameState::Lobbies)
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(ClientPlugin::<NetworkConfig>::connect("127.0.0.1:2905"))
        .add_plugin(utils::cursor_state::CursorStatePlugin)
        .add_plugin(utils::drag_and_drop::DragAndDropPlugin)
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .add_startup_system(setup)
        .add_system(utils::errors::display_error)
        .add_system(read_server_socket)
        // .add_system(animate_sprite_system)
        .insert_resource(Settings {
            username: String::from(""),
            enable_animations: true,
        })
        .run();
}

pub fn read_server_socket(
    mut server_query: Query<&mut Server>,
    mut incoming_packets: ResMut<IncomingPackets>,
) {
    if let Ok(mut server) = server_query.get_single_mut() {
        if let Ok(packet) = read_socket(&mut server.socket) {
            incoming_packets.push(packet);
        }
    }
}

/*
fn animate_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index as usize + 1) % texture_atlas.textures.len();
        }
    }
}
*/

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
