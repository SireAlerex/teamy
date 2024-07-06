use crate::{db, utils};
use rand::seq::SliceRandom;
use serenity::{
    model::{channel::Message, prelude::*},
    prelude::*,
};

pub static SALUTATIONS: [&str; 4] = ["Bonjour", "Salut", "Coucou", "Yo"];

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

fn substring_count(string: &str, targets: &[&str]) -> usize {
    targets
        .iter()
        .filter(|t| string.contains(&t.to_lowercase()))
        .count()
}

fn fullword_count(string: &str, targets: &[&str]) -> usize {
    let lowercase_targets: Vec<String> = targets.iter().map(|s| s.to_lowercase()).collect();
    let compare: Vec<&str> = lowercase_targets
        .iter()
        .map(std::string::String::as_str)
        .collect();
    string
        .split_whitespace()
        .filter(|word| compare.contains(word))
        .count()
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
    fullword_count(string, targets) > 0_usize
}

fn present_words(string: &str, targets: &[&str]) -> bool {
    substring_count(string, targets) > 0_usize
}

fn _capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or_else(String::new, |f| {
        format!("{}{}", f.to_uppercase().collect::<String>(), c.as_str())
    })
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
    (find_emoji(ctx, guild_id, name).await).map_or_else(
        || format!("<veuillez imaginer l'emoji \"{name}\">"),
        |emoji| emoji.to_string(),
    )
}

fn bot(message: &str) -> bool {
    present(message, &["bot", "robot", "teamy"])
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

pub async fn handle_reaction(
    ctx: &Context,
    msg: &Message,
) -> Result<Option<String>, HandleMessageError> {
    let user_message = msg.content.to_lowercase();

    // FIXME
    // if mute_checks(ctx, msg).await {
    //     return Ok(None);
    // }

    let user_nick = utils::get_user_name(msg.guild_id, ctx.http(), &msg.author).await;
    let bot = bot(&user_message);

    // emoji reactions
    // pirate
    if present_words(&user_message, &["belle bite"]) {
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
    if bot && present(&user_message, &SALUTATIONS) {
        return Ok(Some(format!("{} {} !", choose(&SALUTATIONS), user_nick)));
    }

    // societer
    if present(
        &user_message,
        &["société", "societe", "societer", "saucisse"],
    ) {
        return Ok(Some(emoji_or(ctx, msg.guild_id, "saucisse").await));
    }

    // sus
    if present(&user_message, &["sus", "sussy"]) {
        return Ok(Some(emoji_or(ctx, msg.guild_id, "afungus").await));
    }

    // civ bedge
    if present(&user_message, &["attend", "attends", "attendre"])
        && present(&user_message, &["civ"])
        && present(&user_message, &["Thomas"])
    {
        return Ok(Some(emoji_or(ctx, msg.guild_id, "bedge").await));
    }

    // cum
    if present(&user_message, &["cum", "cummies", "cummy"]) {
        return Ok(Some(":milk:".to_owned()));
    }

    // source
    if present_words(&user_message, &["source ?", "sources ?"]) {
        return Ok(Some(
            choose(&[
                "Ça m'est apparu dans un rêve",
                "Contexte ?",
                "Moi",
                "La Laitière",
                "Manuel Valls",
                "Mon cul",
                "Le ciel me l'a dit",
                "Trust me bro",
                "Do your own research",
                "J'ai appris ça sur Internet",
            ])
            .to_owned(),
        ));
    }

    // pas mal non
    if present_words(&user_message, &["pas mal non"]) {
        return Ok(Some("C'est français :flag_fr:".to_owned()));
    }

    // quoi
    if endwith(&user_message, &["quoi", "quoi ?"]) {
        return Ok(Some(choose(&["quoicoubeh", "feur"]).to_owned()));
    }

    // good bot
    if bot && present(&user_message, &["bon", "good", "gentil", "nice"]) {
        return Ok(Some(
            choose(&[
                ":smiley:",
                ":smile:",
                ":grin:",
                ":blush:",
                ":smiling_face_with_3_hearts:",
            ])
            .to_owned(),
        ));
    }

    // bad bot
    if bot && present(&user_message, &["bad", "mauvais", "méchant"]) {
        let reaction = choose(&[
            ":nerd:",
            ":pensive:",
            ":worried:",
            ":slight_frown:",
            ":frowning2:",
            ":cry:",
        ]);
        return match reaction {
            ":nerd:" => Ok(Some(utils::nerdify(&user_message))),
            _ => Ok(Some(reaction.to_owned())),
        };
    }

    // gay bot
    if bot && present(&user_message, &["gay"]) {
        return Ok(Some(choose(&[":hot_face:", ":shushing_face:"]).to_owned()));
    }

    // ou
    if bot && present(&user_message, &["ou"]) {
        return Ok(Some(ou(&user_message).unwrap_or("").to_owned()));
    }

    Ok(None)
}
