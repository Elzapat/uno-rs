use bevy_ecs::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::network::protocol::Protocol"]
pub struct ChooseColor {
    _p: Property<()>,
}

impl ChooseColor {
    pub fn new() -> Self {
        ChooseColor::new_complete(())
    }
}
