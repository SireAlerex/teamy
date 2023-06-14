use std::sync::Arc;
use std::time::Duration;

use serenity::{
    builder::CreateEmbed,
    client::bridge::gateway::ShardId,
    framework::standard::CommandError,
    gateway::ConnectionStage,
    json::Value,
    model::{
        prelude::{
            interaction::{
                application_command::{ApplicationCommandInteraction, CommandDataOption},
                InteractionResponseType,
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

pub fn strip_prefix_suffix(s: String, c: char) -> String {
    s.strip_prefix(c)
        .unwrap()
        .strip_suffix(c)
        .unwrap()
        .to_owned()
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

pub fn admin_command(command: &ApplicationCommandInteraction) -> bool {
    match command.member.as_ref() {
        Some(member) => match member.permissions {
            Some(perm) => perm.administrator(),
            None => false,
        },
        None => false,
    }
}

pub async fn say_or_error(ctx: &Context, channel_id: ChannelId, content: &str) {
    if content.is_empty() {
        return;
    };
    if let Err(e) = channel_id.say(&ctx.http, content).await {
        error!(
            "error sending message ({content}) in chan {} : {e}",
            channel_id
        );
    }
}

pub fn command_error(message: impl ToString) -> CommandError {
    Box::<dyn std::error::Error + Send + Sync>::from(message.to_string())
}

pub async fn get_temp_chan(ctx: &Context) -> Option<ChannelId> {
    let data = ctx.data.read().await;
    let temp_chan = match data.get::<crate::TempChanContainer>() {
        Some(chan) => chan,
        None => {
            error!("there was a problem getting the temp chan");
            return None;
        }
    };
    let temp_chan = temp_chan.lock().await;
    Some(temp_chan.clone())
}

pub fn get_option<'a>(data: &'a CommandDataOption, name: impl ToString) -> Option<&'a Value> {
    data.options
        .iter()
        .find(|o| o.name == name.to_string())?
        .value
        .as_ref()
}

pub async fn interaction_response_message(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    text: String,
    ephemeral: bool,
    embed: Option<CreateEmbed>,
) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    if let Some(e) = embed {
                        message.content(text).ephemeral(ephemeral).add_embed(e)
                    } else {
                        message.content(text).ephemeral(ephemeral)
                    }
                })
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
