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

pub async fn find_message(
    ctx: &Context,
    user: &User,
    channel: &ChannelId,
    text: String,
) -> Result<Message, serenity::Error> {
    if let Some(m) = channel
        .messages(&ctx.http, |retriever| retriever.limit(10))
        .await?
        .iter()
        .find(|m: &&Message| m.author == user.clone() && m.content == text)
    {
        Ok(m.clone())
    } else {
        Err(serenity::Error::Other("Aucun message correspondant"))
    }
}

pub fn first_letter(s: &str) -> char {
    s.chars().next().unwrap()
}

pub fn remove_suffix(s: &str) -> String {
    let mut c = s.chars();
    c.next();
    c.collect()
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
    ephemeral: bool,
) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(text).ephemeral(ephemeral))
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
