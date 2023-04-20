use std::sync::Arc;
use std::time::Duration;

use serenity::{model::prelude::Message, prelude::Context, client::bridge::gateway::ShardId, gateway::ConnectionStage};
use tracing::error;

use crate::ShardManagerContainer;

pub fn first_letter(s: &str) -> char {
    s.chars().next().unwrap()
}

pub fn remove_suffix(s: &str) -> String {
    let mut c = s.chars();
    c.next();
    c.collect()
}

pub async fn send_message(msg: Message, ctx: Context, text: &str) {
    if let Err(e) = msg.channel_id.say(&ctx.http, text).await {
        error!("Error sending message: {:?}", e);
    }
}

pub async fn send_dm(msg: Message, ctx: Context, text: &str) {
    if let Err(e) = msg.author.dm(&ctx.http, |m| m.content(text)).await {
        error!("Error sending message: {:?}", e);
    }
}

pub async fn runner_connection(ctx: Arc<Context>) -> Option<ConnectionStage> {
    let data = ctx.data.read().await;
    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            error!("There was a problem getting the shard manager");
            return None;
        }
    };
    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            error!("No shard found");
            return None;
        }
    };
    Some(runner.stage)
}

pub async fn runner_latency(ctx: Arc<Context>) -> Option<Duration> {
    let data = ctx.data.read().await;
    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            error!("There was a problem getting the shard manager");
            return None;
        }
    };
    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            error!("No shard found");
            return None;
        }
    };
    runner.latency
}
