use crate::{
    commands::{Context, PoiseError},
    utils,
};
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command,
    prefix_command,
    category = "general",
    description_localized("fr", "Nerdifie un texte")
)]
pub async fn nerd(
    ctx: Context<'_>,
    #[description = "Texte à transformer"] texte: String,
) -> Result<(), PoiseError> {
    ctx.say(nerdify_command(&texte)).await?;
    Ok(())
}

#[poise::command(
    context_menu_command = "Nerdifie",
    category = "general",
    description_localized("fr", "Nerdifie un message")
)]
pub async fn nerd_message(ctx: Context<'_>, message: serenity::Message) -> Result<(), PoiseError> {
    ctx.say(format!(
        "{} -{}",
        nerdify_command(&message.content),
        utils::get_user_name(ctx.guild_id(), ctx.http(), &message.author).await
    ))
    .await?;
    Ok(())
}

fn nerdify_command(text: &str) -> String {
    format!("\"{} :nerd:\"", utils::nerdify(text))
}
