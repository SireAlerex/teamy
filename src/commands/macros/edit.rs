use super::r#macro::Macro;
use crate::commands::macros::r#macro;
use crate::{db, utils, InteractionMessage, InteractionResponse};
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
    let args = args.single::<String>()?;
    edit_macro(ctx, user_id, name, args).await?;

    utils::say_or_error(ctx, msg.channel_id, "La macro a bien été modifiée").await;
    Ok(())
}

async fn edit_macro(ctx: &Context, user_id: String, name: String, args: String) -> CommandResult {
    let query = doc! {"user_id": user_id, "name": name};
    let original_macro = if let Some(m) = db::find_filter::<Macro>(ctx, "macros", query).await? {
        m
    } else {
        return Err(utils::command_error("Macro non trouvée"));
    };
    let modified_macro = original_macro.clone().edit(Some(&args));
    r#macro::test_macro(ctx, &modified_macro.command, &modified_macro.args).await?;
    let update = doc! {"$set": {"args": args}};
    db::update(ctx, "macros", &original_macro, &update).await?;
    Ok(())
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> InteractionResponse {
    let subcommand = &command.data.options[0];
    let user_id = command.user.id.to_string();
    let name = utils::get_option(subcommand, "nom")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let args = utils::get_option(subcommand, "arguments")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let content = match edit_macro(ctx, user_id, name, args).await {
        Ok(_) => "La macro a bien été modifiée".to_string(),
        Err(e) => format!("Une erreur s'est produite lors de la modification : {e}"),
    };
    InteractionResponse::Message(InteractionMessage {
        content,
        ephemeral: true,
        embed: None,
    })
}
