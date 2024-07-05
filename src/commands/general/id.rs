use crate::{
    commands::{Context, PoiseError},
    utils,
};
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command,
    prefix_command,
    category = "general",
    description_localized("fr", "Affiche l'id d'un utilisateur")
)]
pub async fn id(
    ctx: Context<'_>,
    #[description = "utilisateur"] user: serenity::User,
) -> Result<(), PoiseError> {
    ctx.say(get_id(ctx, user).await).await?;
    Ok(())
}

#[poise::command(
    context_menu_command = "ID Discord",
    description_localized("fr", "Affiche l'id d'un utilisateur")
)]
pub async fn id_user(ctx: Context<'_>, user: serenity::User) -> Result<(), PoiseError> {
    ctx.say(get_id(ctx, user).await).await?;
    Ok(())
}

async fn get_id(ctx: Context<'_>, user: serenity::User) -> String {
    format!(
        "L'id de {} est {}",
        utils::get_user_name(ctx.guild_id(), ctx.http(), &user).await,
        user.id
    )
}
