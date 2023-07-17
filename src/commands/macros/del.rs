use crate::utils;
use crate::{db, InteractionMessage, Response};
use bson::doc;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

use super::r#macro::Macro;

#[command]
#[description = "supprime une macro"]
#[usage = "<nom de la macro>"]
#[example = "init"]
async fn del(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_id = msg.author.id;
    let name = args.single::<String>()?;
    del_macro(ctx, user_id.to_string(), name).await?;

    utils::say_or_error(ctx, msg.channel_id, "La macro a bien été supprimée").await;
    Ok(())
}

async fn del_macro(
    ctx: &Context,
    user_id: String,
    name: String,
) -> Result<(), mongodb::error::Error> {
    let query = doc! { "user_id": user_id, "name": name };
    db::delete_query::<Macro>(ctx, "macros", query).await
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Response {
    let Some(subcommand) = &command.data.options.first() else {
        return Response::Message(InteractionMessage::ephemeral("Erreur : pas de sous-commandes"));
    };
    let name = match utils::option_as_str(subcommand, "nom") {
        Some(s) => s.to_owned(),
        None => {
            return Response::Message(InteractionMessage::ephemeral(
                "erreur d'arguments : pas de 'nom'",
            ))
        }
    };
    let content = match del_macro(ctx, command.user.id.to_string(), name).await {
        Ok(_) => "La macro a bien été supprimée".to_owned(),
        Err(e) => format!("Erreur lors de la suppression de macro : {e}"),
    };
    Response::Message(InteractionMessage::ephemeral(content))
}
