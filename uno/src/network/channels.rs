use naia_shared::{
    derive_channels, Channel, ChannelDirection, ChannelMode, ReliableSettings, TickBufferSettings,
};

#[derive_channels]
pub enum Channels {
    // Lobby,
    // Game,
    Uno,
}

pub const CHANNEL_CONFIG: &[Channel<Channels>] = &[
    /*
    Channel {
        index: Channels::Lobby,
        direction: ChannelDirection::Bidirectional,
        mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
    },
    Channel {
        index: Channels::Game,
        direction: ChannelDirection::Bidirectional,
        mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
    },
    */
    Channel {
        index: Channels::Uno,
        direction: ChannelDirection::Bidirectional,
        mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
    },
];
