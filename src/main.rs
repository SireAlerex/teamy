mod bot;
// pub mod command_info;
mod commands;
mod containers;
#[allow(clippy::impl_trait_in_params)]
pub mod db;
// mod framework;
// TODO: decide what to do with module after macros re-implemented
// pub mod interaction;
mod loops;
mod message;
mod secrets;
pub mod utils;
// pub mod web_scraper;

use anyhow::anyhow;
use commands::{
    admin::register::register,
    general::{
        based::{based, based_message, based_user},
        hello::hello,
        help::help,
        id::{id, id_user},
        nerd::{nerd, nerd_message},
        ping::ping,
        roll::{roll, roll_prefix},
        slide::slide,
    },
};
use serenity::model::prelude::{ChannelId, GuildId};
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use std::env;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use bot::Bot;
use containers::{
    DatabaseUri, DatabaseUriContainer, GuildGroup, GuildIdContainer, LogChanIdContainer,
    ShardManagerContainer, TempChanContainer,
};

#[derive(Debug)]
pub struct Data {} // User data, which is stored and accessible in all command invocations

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    env::set_var("RUST_BACKTRACE", "1");
    // Get the discord token set in Secrets.toml
    let token = secrets::get(&secret_store, "DISCORD_TOKEN")?;

    // Commands
    let mut commands = vec![
        hello(),
        based(),
        based_message(),
        based_user(),
        help(),
        id(),
        id_user(),
        nerd(),
        nerd_message(),
        ping(),
        roll(),
        roll_prefix(),
        slide(),
        register(),
    ];
    bot::apply_desc_from(&mut commands, "fr");

    // Create framework for bot
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("$".into()),
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(3600),
                ))),
                case_insensitive_commands: true,
                ..Default::default()
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(bot::event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                bot::register_guild(ctx, framework).await;
                Ok(Data {})
            })
        })
        .build();

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
    let log_chan = ChannelId::new(secrets::parse(&secret_store, "LOG_CHAN_ID")?);
    let db_uri = DatabaseUri(secrets::get(&secret_store, "DATABASE_URI")?);
    let temp_chan = ChannelId::new(secrets::parse(&secret_store, "TEMP_CHAN")?);

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<GuildIdContainer>(Arc::new(guild_group));
        data.insert::<LogChanIdContainer>(Arc::new(log_chan));
        data.insert::<DatabaseUriContainer>(Arc::new(db_uri));
        data.insert::<TempChanContainer>(Arc::new(temp_chan));
    }

    Ok(client.into())
}
