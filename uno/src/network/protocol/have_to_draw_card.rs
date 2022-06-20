use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct HaveToDrawCard {
    _p: Property<()>,
}

impl HaveToDrawCard {
    pub fn new() -> Self {
        HaveToDrawCard::new_complete(())
    }
}
