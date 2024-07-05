use super::r#macro::Macro;
use crate::db;
use crate::{InteractionMessage, Response};
use bson::doc;
use serenity::builder::CreateEmbed;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandError, CommandResult};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::{Message, UserId};
use serenity::model::user::User;
use serenity::prelude::Context;

#[command]
#[description = "affiche les macros de l'utilisateur"]
async fn show(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let embed = macro_embed(ctx, &msg.author).await?;
    let _: Message = msg
        .channel_id
        .send_message(&ctx.http, |m| m.set_embed(embed))
        .await?;
    Ok(())
}

async fn macro_embed(ctx: &Context, user: &User) -> Result<CreateEmbed, CommandError> {
    let macros = get_macros(ctx, user.id).await?;
    let pretty_macros = if macros.is_empty() {
        String::from("Vous n'avez aucune macro")
    } else {
        macros
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
            .join("\n")
    };
    let embed = CreateEmbed::default()
        .description(format!("Macros de {}", user.name)) // FIXME: use utils to get user name
        .field("Macros", pretty_macros, false)
        .color(serenity::utils::Colour::PURPLE)
        .clone();
    Ok(embed)
}

async fn get_macros(ctx: &Context, user_id: UserId) -> Result<Vec<Macro>, mongodb::error::Error> {
    let filter = doc! {"user_id": user_id.to_string()};
    db::get_objects::<Macro>(ctx, "macros", filter).await
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Response {
    let (content, embed) = match macro_embed(ctx, &command.user).await {
        Ok(embed) => (String::new(), Some(embed)),
        Err(e) => (
            format!("Une erreur s'est produite lors de l'accès aux macros : {e}"),
            None,
        ),
    };
    Response::Message(InteractionMessage::new(content, true, embed))
}
