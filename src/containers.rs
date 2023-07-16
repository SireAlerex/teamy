use std::sync::Arc;

use serenity::{client::bridge::gateway::ShardManager, model::prelude::*, prelude::*};
use tokio::sync::Mutex;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct GuildGroup(pub Vec<GuildId>);

pub struct GuildIdContainer;

impl TypeMapKey for GuildIdContainer {
    type Value = Arc<GuildGroup>;
}

pub struct LogChanIdContainer;

impl TypeMapKey for LogChanIdContainer {
    type Value = Arc<ChannelId>;
}

pub struct DatabaseUri(pub String);

pub struct DatabaseUriContainer;

impl TypeMapKey for DatabaseUriContainer {
    type Value = Arc<DatabaseUri>;
}

pub struct TempChanContainer;

impl TypeMapKey for TempChanContainer {
    type Value = Arc<ChannelId>;
}
