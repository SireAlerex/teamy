use crate::commands::{Context, PoiseError};

#[poise::command(
    slash_command,
    category = "admin",
    hide_in_help,
    owners_only,
    description_localized("fr", "Register boutons pour les commandes")
)]
pub async fn register(ctx: Context<'_>) -> Result<(), PoiseError> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}
