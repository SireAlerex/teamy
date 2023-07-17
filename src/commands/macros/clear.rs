use crate::{db, utils, InteractionMessage, Response};
use bson::doc;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

use super::r#macro::Macro;

#[command]
#[description = "supprime toutes les macros"]
async fn clear(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    del_macros(ctx, msg.author.id.to_string()).await?;

    utils::say_or_error(
        ctx,
        msg.channel_id,
        "Toutes vos macros ont bien été supprimées",
    )
    .await;
    Ok(())
}

async fn del_macros(ctx: &Context, user_id: String) -> Result<(), mongodb::error::Error> {
    let query = doc! { "user_id": user_id };
    db::delete_multiple_query::<Macro>(ctx, "macros", query).await
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Response {
    let content = match del_macros(ctx, command.user.id.to_string()).await {
        Ok(_) => "Toutes vos macros ont bien été supprimées".to_owned(),
        Err(e) => format!("Erreur lors de la suppression des macros : {e}"),
    };
    Response::Message(InteractionMessage::ephemeral(content))
}
