use crate::commands::{Context, PoiseError};
use crate::message;
use rand::seq::IteratorRandom;
use rand::thread_rng;

#[poise::command(
    slash_command,
    prefix_command,
    category = "general",
    description_localized("fr", "Dis bonjour")
)]
pub async fn hello(ctx: Context<'_>) -> Result<(), PoiseError> {
    ctx.say(salutation()).await?;
    Ok(())
}

fn salutation() -> String {
    format!(
        "{} !",
        message::SALUTATIONS
            .iter()
            .choose(&mut thread_rng())
            .unwrap_or(&"Bonjour !")
    )
}
