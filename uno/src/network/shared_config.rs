use crate::network::channels::{Channels, CHANNEL_CONFIG};
use naia_shared::{SharedConfig, SocketConfig};

pub fn shared_config() -> SharedConfig<Channels> {
    SharedConfig::new(SocketConfig::new(None, None), CHANNEL_CONFIG, None, None)
}
