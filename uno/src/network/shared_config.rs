use crate::network::channels::{Channels, CHANNEL_CONFIG};
use naia_shared::{LinkConditionerConfig, SharedConfig, SocketConfig};
use std::time::Duration;

pub fn shared_config() -> SharedConfig<Channels> {
    let tick_interval = Some(Duration::from_millis(100));
    let link_condition = Some(LinkConditionerConfig::good_condition());

    SharedConfig::new(
        SocketConfig::new(link_condition, None),
        CHANNEL_CONFIG,
        tick_interval,
        None,
    )
}
