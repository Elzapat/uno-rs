use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct ThisPlayer {
    pub entity: Property<u64>,
}

impl ThisPlayer {
    pub fn new(entity: Entity) -> Self {
        ThisPlayer::new_complete(entity.to_bits())
    }
}
