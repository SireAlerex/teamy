use crate::commands::general::nerd;
use crate::consts;
use crate::db;
use rand::seq::SliceRandom;
use serenity::{
    model::{channel::Message, prelude::*},
    prelude::*,
};

#[derive(Debug)]
pub enum HandleMessageError {
    #[allow(dead_code)]
    General(String),
    Serenity(SerenityError),
    ReactionConversion(ReactionConversionError),
}

impl std::fmt::Display for HandleMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::General(s) => write!(f, "{s}"),
            Self::Serenity(e) => write!(f, "{e}"),
            Self::ReactionConversion(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for HandleMessageError {}

impl From<SerenityError> for HandleMessageError {
    fn from(value: SerenityError) -> Self {
        Self::Serenity(value)
    }
}

impl From<ReactionConversionError> for HandleMessageError {
    fn from(value: ReactionConversionError) -> Self {
        Self::ReactionConversion(value)
    }
}

fn full_word(string: &str, targets: &[&str]) -> i32 {
    targets
        .iter()
        .filter(|t| string.contains(&t.to_lowercase()))
        .count()
        .try_into()
        .unwrap_or(i32::MAX)
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
    full_word(string, targets) > 0_i32
}

fn _capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => format!("{}{}", f.to_uppercase().collect::<String>(), c.as_str()),
    }
}

fn choose<'a>(choices: &[&'a str]) -> &'a str {
    choices.choose(&mut rand::thread_rng()).unwrap_or(&"")
}

async fn find_emoji(ctx: &Context, guild: Option<GuildId>, name: &str) -> Option<Emoji> {
    if let Some(guild_id) = guild {
        let Ok(emojis) = guild_id.emojis(&ctx.http).await else {
            return None;
        };
        emojis.iter().find(|e| e.name == name).cloned()
    } else {
        None
    }
}

async fn emoji_or(ctx: &Context, guild_id: Option<GuildId>, name: &str) -> String {
    if let Some(emoji) = find_emoji(ctx, guild_id, name).await {
        emoji.to_string()
    } else {
        format!("<veuillez imaginer l'emoji \"{name}\">")
    }
}

fn bot(message: &str) -> bool {
    present(message, &consts::BOT)
}

fn ou(message: &str) -> Option<&str> {
    let mut options = message.split(" ou ");
    let Ok(re) = regex::Regex::new(r"bot|robot|teamy") else {
        return None;
    };
    let a = re.split(options.next()?).last()?;
    let b = re.split(options.next()?).next()?;
    Some(choose(&[a, b]))
}

// true if is mute and shouldn't react
async fn mute_checks(ctx: &Context, msg: &Message) -> bool {
    (if let Some(guild_id) = msg.guild_id {
        db::is_object_in_coll(
            ctx,
            "mute_guilds",
            &db::Guild::builder(guild_id.to_string()),
        )
        .await
        .unwrap_or(false)
    } else {
        false
    }) || db::is_object_in_coll(
        ctx,
        "mute_chans",
        &db::Chan::builder(msg.channel_id.to_string()),
    )
    .await
    .unwrap_or(false)
        || db::is_object_in_coll(
            ctx,
            "mute_users",
            &db::User::builder(msg.author.id.to_string()),
        )
        .await
        .unwrap_or(false)
}

pub async fn handle_reaction(ctx: &Context, msg: &Message) -> Result<String, HandleMessageError> {
    let user_message = msg.content.to_lowercase();
    let user = msg.author.clone();

    if mute_checks(ctx, msg).await {
        return Ok(String::new());
    }

    let user_nick = if let Some(guild_id) = msg.guild_id {
        match user.nick_in(&ctx.http, guild_id).await {
            Some(nick) => nick,
            None => user.name,
        }
    } else {
        user.name
    };

    // emoji reactions
    // pirate
    if present(&user_message, &["belle bite"]) {
        let pirate = ReactionType::try_from("🏴‍☠️")?;
        let crossed_swords = ReactionType::try_from("⚔️")?;
        let _: Reaction = msg.react(&ctx.http, pirate).await?;
        let _: Reaction = msg.react(&ctx.http, crossed_swords).await?;
    }

    // bengala
    if present(&user_message, &["bengala"]) {
        let _: Reaction = msg.react(&ctx.http, '🍆').await?;
    }

    // string reactions
    // bonjour bot
    if bot(&user_message) && present(&user_message, &consts::SALUTATIONS) {
        return Ok(format!("{} {} !", choose(&consts::SALUTATIONS), user_nick));
    }

    // societer
    if present(&user_message, &consts::SOCIETER) {
        return Ok(emoji_or(ctx, msg.guild_id, "saucisse").await);
    }

    // sus
    if present(&user_message, &consts::SUS) {
        return Ok(emoji_or(ctx, msg.guild_id, "afungus").await);
    }

    // civ bedge
    if present(&user_message, &consts::ATTENDRE)
        && present(&user_message, &["civ"])
        && present(&user_message, &["Thomas"])
    {
        return Ok(emoji_or(ctx, msg.guild_id, "nerd").await);
    }

    // cum
    if present(&user_message, &consts::CUM) {
        return Ok(":milk:".to_owned());
    }

    // source
    if present(&user_message, &consts::SOURCE) {
        return Ok("Ça m'est apparu dans un rêve".to_owned());
    }

    // pas mal non
    if present(&user_message, &["pas mal non"]) {
        return Ok("C'est français :flag_fr:".to_owned());
    }

    // quoi
    if endwith(&user_message, &consts::QUOI) {
        return Ok(choose(&consts::QUOI_REPONSE).to_owned());
    }

    // good bot
    if bot(&user_message) && present(&user_message, &consts::GOOD) {
        return Ok(choose(&consts::GOOD_REACTION).to_owned());
    }

    // bad bot
    if bot(&user_message) && present(&user_message, &consts::BAD) {
        let reaction = choose(&consts::BAD_REACTION);
        match reaction {
            ":nerd:" => return Ok(nerd::run(&user_message)),
            _ => return Ok(reaction.to_owned()),
        }
    }

    // gay bot
    if bot(&user_message) && present(&user_message, &["gay"]) {
        return Ok(choose(&consts::HOT).to_owned());
    }

    // ou
    if bot(&user_message) && present(&user_message, &["ou"]) {
        return Ok(ou(&user_message).unwrap_or("").to_owned());
    }

    Ok(String::new())
}
