use crate::consts;
use rand::seq::SliceRandom;
use serenity::{model::{channel::Message, prelude::{Emoji, GuildId}}, prelude::Context};

fn full_word(string: &str, targets: &[&str]) -> i32 {
    let words = string.split_whitespace();
    let mut count = 0;
    for word in words {
        for target in targets {
            if word.eq_ignore_ascii_case(target) {
                count += 1;
            }
        }
    }
    count
}

fn present(string: &str, targets: &[&str]) -> bool {
    full_word(string, targets) > 0
}

fn _capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn choose<'a>(choices: &[&'a str]) -> &'a str {
    choices.choose(&mut rand::thread_rng()).unwrap_or(&"")
}

async fn find_emoji(ctx: &Context, guild_id: Option<GuildId>, name: &str) -> Option<Emoji> {
    if guild_id.is_none() {return None }
    let emojis = match guild_id.unwrap().emojis(&ctx.http).await {
        Ok(e) => e,
        Err(_) => return None,
    };
    match emojis.iter().find(|e| e.name == name) {
        Some(e) => Some(e.clone()),
        None => None,
    }
}

async fn emoji_or(ctx: &Context, guild_id: Option<GuildId>, name: &str) -> String {
    match find_emoji(ctx, guild_id, name).await {
        Some(emoji) => format!("{emoji}"),
        None => String::from(name),
    }
}

pub async fn handle_reaction(ctx: &Context, msg: &Message) -> String {
    let user_message = msg.content.to_lowercase();
    let user = msg.author.clone();
    let user_nick = match msg.is_private() {
        true => user.name,
        false => match user.nick_in(&ctx.http, msg.guild_id.unwrap()).await {
            Some(nick) => nick,
            None => user.name,
        },
    };

    // bonjour bot
    if present(&user_message, &consts::BOT) && present(&user_message, &consts::SALUTATIONS) {
        return format!("{} {} !", choose(&consts::SALUTATIONS), user_nick);
    }

    // societer
    if present(&user_message, &consts::SOCIETER) {
        return emoji_or(ctx, msg.guild_id, "saucisse").await;
    }

    // sus
    if present(&user_message, &consts::SUS) {
        return emoji_or(ctx, msg.guild_id, "afungus").await;
    }

    // civ bedge
    if present(&user_message, &consts::ATTENDRE)
    && present(&user_message, &["civ"])
        && present(&user_message, &["Thomas"])
    {
        return emoji_or(ctx, msg.guild_id, "Bedge").await;
    }

    // cum
    if present(&user_message, &consts::CUM) {
        return format!(":milk:");
    }

    String::from("")
}
