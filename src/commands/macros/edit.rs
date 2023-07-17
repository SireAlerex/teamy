use super::r#macro::Macro;
use crate::commands::macros::r#macro;
use crate::{db, utils, InteractionMessage, Response};
use bson::doc;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
#[description = "modifie une macro"]
#[usage = "<nom de la macro> <nouveaux arguments>"]
#[example = "init d20+6"]
async fn edit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_id = msg.author.id.to_string();
    let name = args.single::<String>()?;
    let macro_args = args.single::<String>()?;
    edit_macro(ctx, user_id, name, macro_args).await?;

    utils::say_or_error(ctx, msg.channel_id, "La macro a bien été modifiée").await;
    Ok(())
}

async fn edit_macro(ctx: &Context, user_id: String, name: String, args: String) -> CommandResult {
    let query = doc! {"user_id": user_id, "name": name};
    let Some(original_macro) = db::find_filter::<Macro>(ctx, "macros", query).await? else {
        return Err(utils::command_error("Macro non trouvée"));
    };
    let modified_macro = original_macro.clone().edit(Some(&args));
    r#macro::test_macro(ctx, &modified_macro.command, &modified_macro.args).await?;
    let update = doc! {"$set": {"args": args}};
    db::update(ctx, "macros", &original_macro, &update).await?;
    Ok(())
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Response {
    let Some(subcommand) = &command.data.options.first() else {
        return Response::Message(InteractionMessage::ephemeral("Erreur : pas de sous-commandes"))
    };
    let user_id = command.user.id.to_string();
    let name = match utils::option_as_str(subcommand, "nom") {
        Some(s) => s.to_owned(),
        None => {
            return Response::Message(InteractionMessage::ephemeral(
                "erreur d'arguments : pas de 'nom'",
            ))
        }
    };
    let args = match utils::option_as_str(subcommand, "arguments") {
        Some(s) => s.to_owned(),
        None => {
            return Response::Message(InteractionMessage::ephemeral(
                "erreur d'arguments : pas de 'arguments'",
            ))
        }
    };
    let content = match edit_macro(ctx, user_id, name, args).await {
        Ok(_) => "La macro a bien été modifiée".to_owned(),
        Err(e) => format!("Une erreur s'est produite lors de la modification : {e}"),
    };
    Response::Message(InteractionMessage::ephemeral(content))
}
