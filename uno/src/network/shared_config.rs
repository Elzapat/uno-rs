use crate::network::channels::{Channels, CHANNEL_CONFIG};
use naia_shared::{SharedConfig, SocketConfig};
use std::time::Duration;

pub fn shared_config() -> SharedConfig<Channels> {
    let tick_interval = Some(Duration::from_millis(10));

    SharedConfig::new(
        SocketConfig::new(None, None),
        CHANNEL_CONFIG,
        tick_interval,
        None,
    )
}
