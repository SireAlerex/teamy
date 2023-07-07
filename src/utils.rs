use std::sync::Arc;
use std::time::Duration;

use serenity::{
    client::bridge::gateway::ShardId,
    framework::standard::CommandError,
    gateway::ConnectionStage,
    json::Value,
    model::{
        prelude::{
            component::InputTextStyle,
            interaction::{
                application_command::{ApplicationCommandInteraction, CommandDataOption},
                modal::ModalSubmitInteraction,
                InteractionResponseType,
            },
            ChannelId, Message,
        },
        user::User,
    },
    prelude::Context,
};
use tracing::error;

use crate::{InteractionMessage, ShardManagerContainer};

pub struct RunnerInfo {
    pub latency: Option<Duration>,
    pub connection: Option<ConnectionStage>,
}

impl RunnerInfo {
    pub async fn info<'a>(ctx: Arc<Context>) -> Result<Self, &'a str> {
        let data = ctx.data.read().await;
        let Some(shard_manager) = data.get::<ShardManagerContainer>() else {
            return Err("There was a problem getting the shard manager");
        };
        let manager = shard_manager.lock().await;
        let runners = manager.runners.lock().await;
        let Some(runner) = runners.get(&ShardId(ctx.shard_id)) else {
            return Err("No shard found");
        };
        Ok(RunnerInfo {
            latency: runner.latency,
            connection: Some(runner.stage),
        })
    }
}

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

pub fn first_letter(s: &str) -> Option<char> {
    s.chars().next()
}

pub fn remove_suffix(s: &str) -> String {
    let mut c = s.chars();
    c.next();
    c.collect()
}

pub fn strip_prefix_suffix(initial_string: &str, c: char) -> String {
    let string_prefix = match initial_string.strip_prefix(c) {
        Some(s) => s,
        None => initial_string,
    };
    match string_prefix.strip_suffix(c) {
        Some(s) => s.to_string(),
        None => string_prefix.to_string(),
    }
}

pub fn nerdify(text: &str) -> String {
    text.char_indices()
        .map(|(i, c)| {
            if i % 2 == 0 {
                c.to_ascii_lowercase()
            } else {
                c.to_ascii_uppercase()
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

pub async fn say_or_error<T: Into<String>>(ctx: &Context, channel_id: ChannelId, content: T) {
    let content = content.into();
    if content.is_empty() {
        return;
    };
    if let Err(e) = channel_id.say(&ctx.http, content.clone()).await {
        error!(
            "error sending message ({content}) in chan {} : {e}",
            channel_id
        );
    }
}

pub fn command_error<T: Into<String>>(message: T) -> CommandError {
    Box::<dyn std::error::Error + Send + Sync>::from(message.into())
}

pub fn mongodb_error_message(message: &str) -> Option<String> {
    let Ok(re) = regex::Regex::new(r"error:(.*),") else {
        return None
    };
    re.captures(message).map(|capture| capture[1].to_string())
}

pub async fn get_temp_chan(ctx: &Context) -> Option<ChannelId> {
    let data = ctx.data.read().await;
    let Some(temp_chan) = data.get::<crate::TempChanContainer>() else {
        error!("there was a problem getting the temp chan");
        return None;
    };
    let temp_chan = temp_chan.lock().await;
    Some(*temp_chan)
}

pub fn get_option<'a>(data: &'a CommandDataOption, name: &str) -> Option<&'a Value> {
    data.options
        .iter()
        .find(|o| o.name == *name)?
        .value
        .as_ref()
}

pub async fn interaction_response_message(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    interaction_message: InteractionMessage,
) {
    let (text, ephemeral, embed) = (
        interaction_message.content,
        interaction_message.ephemeral,
        interaction_message.embed,
    );
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

pub async fn interaction_response_message_from_modal(
    ctx: &Context,
    modal: &ModalSubmitInteraction,
    interaction_message: InteractionMessage,
) {
    let (text, ephemeral, embed) = (
        interaction_message.content,
        interaction_message.ephemeral,
        interaction_message.embed,
    );
    if let Err(why) = modal
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
        if let Err(e) = modal.channel_id.say(&ctx.http, error_message).await {
            error!("Error sending error message ({why}) to channel because : {e}");
        }
    }
}

pub async fn interaction_response_modal(ctx: &Context, command: &ApplicationCommandInteraction) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::Modal)
                .interaction_response_data(|modal| {
                    modal
                        .title("Formulaire")
                        .components(|input| {
                            input.create_action_row(|f| {
                                f.create_input_text(|t| {
                                    t.label("label").custom_id(2).style(InputTextStyle::Short)
                                })
                            })
                        })
                        .custom_id(1)
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
