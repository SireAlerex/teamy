use std::sync::Arc;
use std::time::Duration;

use poise::serenity_prelude;
use serenity::{
    all::{CacheHttp, GuildId},
    gateway::ConnectionStage,
    model::{
        id::ShardId,
        prelude::{ChannelId, CommandInteraction},
    },
    prelude::Context,
};
use tracing::error;

use crate::ShardManagerContainer;

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
        let manager = shard_manager;
        let runners = manager.runners.lock().await;
        let Some(runner) = runners.get(&ShardId(ctx.shard_id.0)) else {
            return Err("No shard found");
        };
        Ok(RunnerInfo {
            latency: runner.latency,
            connection: Some(runner.stage),
        })
    }
}

pub async fn get_user_name<'a>(
    maybe_guild_id: Option<GuildId>,
    cache_http: impl CacheHttp,
    user: &serenity_prelude::User,
) -> String {
    if let Some(guild_id) = maybe_guild_id {
        if let Some(nick) = user.nick_in(cache_http, guild_id).await {
            return nick;
        } else if let Some(global_name) = user.global_name.clone() {
            return global_name;
        }
    }
    user.name.clone()
}

// TODO: need a message to research from as a parameter ?
// pub async fn find_message(
//     ctx: &Context,
//     user: &User,
//     channel: &ChannelId,
//     text: String,
// ) -> Result<Message, serenity::Error> {
//     if let Some(m) = channel
//         // .messages(&ctx.http, |retriever| retriever.limit(10))
//         .messages(&ctx.http, GetMessages::new().after(m))
//         .await?
//         .iter()
//         .find(|m: &&Message| m.author == user.clone() && m.content == text)
//     {
//         Ok(m.clone())
//     } else {
//         Err(serenity::Error::Other("Aucun message correspondant"))
//     }
// }

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
        Some(s) => s.to_owned(),
        None => string_prefix.to_owned(),
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

pub fn admin_command(command: &CommandInteraction) -> bool {
    match command.member.as_ref() {
        Some(member) => match member.permissions {
            Some(perm) => perm.administrator(),
            None => false,
        },
        None => false,
    }
}

pub async fn say_or_error<T: Into<String>>(ctx: &Context, channel_id: ChannelId, content: T) {
    let content_string = content.into();
    if content_string.is_empty() {
        return;
    };
    if let Err(e) = channel_id.say(&ctx.http, content_string.clone()).await {
        error!("error sending message ({content_string}) in chan {channel_id} : {e}");
    }
}

pub async fn say_or_error2(ctx: &Context, channel_id: ChannelId, content: &str) {
    if content.is_empty() {
        return;
    };
    if let Err(e) = channel_id.say(&ctx.http, content).await {
        error!("error sending message ({content}) in chan {channel_id} : {e}");
    }
}

pub async fn get_temp_chan(ctx: &Context) -> Option<Arc<ChannelId>> {
    let data = ctx.data.read().await;
    let Some(temp_chan_mutex) = data.get::<crate::TempChanContainer>() else {
        error!("there was a problem getting the temp chan");
        return None;
    };
    let temp_chan_id = temp_chan_mutex;
    Some(Arc::clone(temp_chan_id))
}
