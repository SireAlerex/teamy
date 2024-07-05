use crate::{
    commands::{Context, PoiseError},
    utils,
};
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command,
    prefix_command,
    category = "general",
    description_localized("fr", "Détermine si quelque chose est basé")
)]
pub async fn based(
    ctx: Context<'_>,
    #[description = "basé ou cringe ?"] texte: String,
) -> Result<(), PoiseError> {
    ctx.say(format!("\"{texte}\"\n{}", get_based())).await?;
    Ok(())
}

#[poise::command(
    context_menu_command = "Message basé ?",
    category = "general",
    description_localized("fr", "Détermine si un message est basé")
)]
pub async fn based_message(ctx: Context<'_>, message: serenity::Message) -> Result<(), PoiseError> {
    ctx.say(format!("\"{}\"\n{}", message.content, get_based()))
        .await?;
    Ok(())
}

#[poise::command(
    context_menu_command = "Basé",
    description_localized("fr", "Détermine si quelqu'un est basé")
)]
pub async fn based_user(ctx: Context<'_>, user: serenity::User) -> Result<(), PoiseError> {
    ctx.say(format!(
        "{} est {}",
        utils::get_user_name(ctx.guild_id(), ctx.http(), &user).await,
        get_based().to_lowercase()
    ))
    .await?;
    Ok(())
}

fn get_based<'a>() -> &'a str {
    if rand::random::<bool>() {
        "Basé"
    } else {
        "Cringe"
    }
}
