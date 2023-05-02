use crate::commands::nerd;
use crate::consts;
use rand::seq::SliceRandom;
use serenity::{
    model::{
        channel::Message,
        prelude::{Emoji, GuildId},
    },
    prelude::Context,
};

fn full_word(string: &str, targets: &[&str]) -> i32 {
    targets
        .iter()
        .filter(|t| string.contains(&t.to_lowercase()))
        .count()
        .try_into()
        .unwrap()
}

fn endwith(string: &str, targets: &[&str]) -> bool {
    for target in targets {
        if string.ends_with(target) {
            return true;
        }
    }
    false
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
    guild_id?;
    let emojis = match guild_id.unwrap().emojis(&ctx.http).await {
        Ok(e) => e,
        Err(_) => return None,
    };
    emojis.iter().find(|e| e.name == name).cloned()
}

async fn emoji_or(ctx: &Context, guild_id: Option<GuildId>, name: &str) -> String {
    match find_emoji(ctx, guild_id, name).await {
        Some(emoji) => format!("{emoji}"),
        None => String::from(name),
    }
}

fn bot(message: &str) -> bool {
    present(message, &consts::BOT)
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
    if bot(&user_message) && present(&user_message, &consts::SALUTATIONS) {
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
        return ":milk:".to_owned();
    }

    // source
    if present(&user_message, &consts::SOURCE) {
        return "Ça m'est apparu dans un rêve".to_owned();
    }

    // pas mal non
    if present(&user_message, &["pas mal non"]) {
        return "C'est français :flag_fr:".to_owned();
    }

    // quoi
    if endwith(&user_message, &consts::QUOI) {
        return choose(&consts::QUOI_REPONSE).to_owned();
    }

    // good bot
    if bot(&user_message) && present(&user_message, &consts::GOOD) {
        return choose(&consts::GOOD_REACTION).to_owned();
    }

    // bad bot
    if bot(&user_message) && present(&user_message, &consts::BAD) {
        let reaction = choose(&consts::BAD_REACTION);
        match reaction {
            ":nerd:" => return nerd::run(&user_message),
            _ => return reaction.to_owned(),
        }
    }

    // gay bot
    if bot(&user_message) && present(&user_message, &["gay"]) {
        return choose(&consts::HOT).to_owned();
    }

    String::from("")
}
