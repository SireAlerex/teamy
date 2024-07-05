use crate::commands::{Context as PoiseContext, PoiseError};

#[poise::command(
    slash_command,
    prefix_command,
    category = "general",
    description_localized("fr", "Donne la latence du bot")
)]
pub async fn ping(ctx: PoiseContext<'_>) -> Result<(), PoiseError> {
    let ping = ctx.ping().await;
    let res = if !ping.is_zero() {
        format!("``{:#?}``", ping)
    } else {
        "(il y a un problème pour accéder à la latence du bot, veuillez réessayer dans 1min)"
            .to_string()
    };

    ctx.say(format!("Pong! {res}")).await?;
    Ok(())
}
