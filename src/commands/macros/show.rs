use super::r#macro::Macro;
use crate::db;
use crate::{InteractionMessage, InteractionResponse};
use bson::doc;
use serenity::builder::CreateEmbed;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandError, CommandResult};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::{Message, UserId};
use serenity::model::user::User;
use serenity::prelude::Context;
use tracing::info;

#[command]
#[description = "macro_show_desc"]
async fn show(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let embed = macro_embed(ctx, &msg.author).await?;
    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| m.set_embed(embed))
        .await?;
    Ok(())
}

async fn macro_embed(ctx: &Context, user: &User) -> Result<CreateEmbed, CommandError> {
    let macros = get_macros(ctx, user.id).await?;
    info!("macro:{:#?} empty?:{}", macros, macros.is_empty());
    let pretty_macros = if !macros.is_empty() { macros
        .iter()
        .map(|macr| {
            format!(
                "{} : {} {}",
                macr.name,
                macr.command,
                macr.args.clone().unwrap_or(String::new())
            )
        })
        .collect::<Vec<String>>()
        .join("\n") }
    else {
        String::from("Vous n'avez aucune macro")
    };
    let embed = CreateEmbed::default()
        .description(format!("Macros de {}", user.name))
        .field("Macros", pretty_macros, false)
        .color(serenity::utils::Colour::PURPLE)
        .to_owned();
    Ok(embed)
}

async fn get_macros(ctx: &Context, user_id: UserId) -> Result<Vec<Macro>, mongodb::error::Error> {
    let filter = doc! {"user_id": user_id.to_string()};
    db::get_objects::<Macro>(ctx, "macros", filter).await
}

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> InteractionResponse {
    let (content, embed) = match macro_embed(ctx, &command.user).await {
        Ok(embed) => (String::new(), Some(embed)),
        Err(e) => (
            format!("Une erreur s'est produite lors de l'accès aux macros : {e}"),
            None,
        ),
    };
    InteractionResponse::Message(InteractionMessage {
        content,
        ephemeral: true,
        embed,
    })
}
