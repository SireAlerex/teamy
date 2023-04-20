use anyhow::anyhow;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::Activity;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use tracing::info;
use rand::{seq::IteratorRandom, thread_rng};
use std::sync::Mutex;

mod message;
pub mod utils;

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        match utils::first_letter(&msg.content) {
            '$' => message::handle_command(msg, ctx).await,
            _ => (),
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    let manager = client.shard_manager.clone();
    let game_pool = Mutex::new(vec!["LoL avec les boys", "Deep Rock Galactic avec les boys",
    "Pathfinder avec les boys", "Minecraft avec les boys", "Civ6 avec les boys", "être raciste", "manger son caca",
    "[STRENG GEHEIM]"]);

    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let lock = manager.lock().await;
            let shard_runners = lock.runners.lock().await;
            let game = *game_pool.lock().unwrap().iter().choose(&mut thread_rng()).unwrap();

            for (id, runner) in shard_runners.iter() {
                runner.runner_tx.set_activity(Some(Activity::playing(game)));
                println!(
                    "Shard ID {} is {} with a latency of {:?}",
                    id, runner.stage, runner.latency
                );
            };
        }
    });

    Ok(client.into())
}
