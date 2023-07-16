mod bot;
pub mod command_info;
mod commands;
#[allow(clippy::impl_trait_in_params)]
pub mod db;
mod framework;
pub mod interaction;
mod loops;
mod message;
mod secrets;
pub mod utils;
pub mod web_scraper;

use anyhow::anyhow;
use serenity::client::bridge::gateway::ShardManager;
use serenity::http::Http;
use serenity::model::prelude::{ChannelId, GuildId};
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex;

use bot::Bot;
use command_info::CommandGroupsContainer;
use interaction::{InteractionMessage, InteractionResponse};

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct GuildGroup(Vec<GuildId>);

struct GuildIdContainer;

impl TypeMapKey for GuildIdContainer {
    type Value = Arc<GuildGroup>;
}

struct LogChanIdContainer;

impl TypeMapKey for LogChanIdContainer {
    type Value = Arc<ChannelId>;
}

struct DatabaseUri(String);

struct DatabaseUriContainer;

impl TypeMapKey for DatabaseUriContainer {
    type Value = Arc<DatabaseUri>;
}

struct TempChanContainer;

impl TypeMapKey for TempChanContainer {
    type Value = Arc<ChannelId>;
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in Secrets.toml
    let token = secrets::get(&secret_store, "DISCORD_TOKEN")?;
    let http = Http::new(&token);

    // Create framework for bot
    let framework = framework::get_framework(http).await?;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Bot {
            is_loop_running: AtomicBool::new(false),
        })
        .await
        .map_err(|e| anyhow!("Error creating client : {e}"))?;

    let guild_group = GuildGroup(secrets::parse_objects::<u64, GuildId>(
        &secret_store,
        "GUILD_ID",
    )?);
    let log_chan = ChannelId(secrets::parse(&secret_store, "LOG_CHAN_ID")?);
    let db_uri = DatabaseUri(secrets::get(&secret_store, "DATABASE_URI")?);
    let temp_chan = ChannelId(secrets::parse(&secret_store, "TEMP_CHAN")?);
    let groups = framework::get_command_groups();

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<GuildIdContainer>(Arc::new(guild_group));
        data.insert::<LogChanIdContainer>(Arc::new(log_chan));
        data.insert::<CommandGroupsContainer>(Arc::new(groups));
        data.insert::<DatabaseUriContainer>(Arc::new(db_uri));
        data.insert::<TempChanContainer>(Arc::new(temp_chan));
    }

    Ok(client.into())
}
