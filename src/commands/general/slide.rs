use crate::commands::{Context as PoiseContext, PoiseError};
use serenity::all::CreateMessage;

#[poise::command(
    slash_command,
    prefix_command,
    category = "general",
    description_localized("fr", "Slide dans tes dm"),
    ephemeral
)]
pub async fn slide(ctx: PoiseContext<'_>) -> Result<(), PoiseError> {
    if ctx.prefix() == "/" {
        let _ = ctx.say("Un DM va être envoyé").await?;
    }
    let _ = ctx
        .author()
        .dm(ctx.http(), CreateMessage::new().content("Salut !"))
        .await?;
    Ok(())
}
