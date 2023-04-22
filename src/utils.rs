use std::sync::Arc;
use std::time::Duration;

use serenity::{
    client::bridge::gateway::ShardId,
    gateway::ConnectionStage,
    model::{
        prelude::{
            interaction::{
                application_command::ApplicationCommandInteraction, InteractionResponseType,
            },
            ChannelId, Message,
        },
        user::User,
    },
    prelude::Context,
};
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

pub async fn send_message(channel_id: ChannelId, ctx: &Context, text: &str) {
    if let Err(e) = channel_id.say(&ctx.http, text).await {
        error!("Error sending message: {:?}", e);
    }
}

pub async fn reply_message(msg: Message, ctx: &Context, text: &str) {
    if let Err(e) = msg.reply(&ctx.http, text).await {
        error!("Error sending message: {:?}", e);
    }
}

pub async fn send_dm(user: User, ctx: &Context, text: &str) {
    if let Err(e) = user.dm(&ctx.http, |m| m.content(text)).await {
        error!("Error sending message: {:?}", e);
    }
}

pub fn nerdify(text: &str) -> String {
    text.char_indices()
        .map(|(i, c)| {
            if i % 2 != 0 {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect()
}

pub async fn interaction_response_message(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    text: String,
) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(text))
        })
        .await
    {
        let error_message = format!("Erreur lors de la réponse à l'interaction : {why}");
        if let Err(e) = command.channel_id.say(&ctx.http, error_message).await {
            error!("Error sending error message ({why}) to channel because : {e}");
        }
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
