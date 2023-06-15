use super::r#macro::{test_macro, Macro};
use crate::{db, utils};
use crate::{InteractionMessage, InteractionResponse};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
#[description = "crée une macro"]
#[usage = "<nom de la macro> <commande> <arguments>"]
#[example = "init roll d20+4"]
#[example = "d6 roll d6"]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_id = msg.author.id;
    let name = args.single::<String>()?;
    let command = args.single::<String>()?;
    let args = if args.len() == 3 {
        Some(args.single::<String>()?)
    } else {
        None
    };
    add_macro(ctx, user_id.to_string(), name, command, args).await?;

    utils::say_or_error(ctx, msg.channel_id, "La macro a bien été ajoutée").await;
    Ok(())
}

async fn add_macro(
    ctx: &Context,
    user_id: String,
    name: String,
    command: String,
    args: Option<String>,
) -> CommandResult {
    test_macro(ctx, &command, &args).await?;
    db::insert(ctx, "macros", &Macro::builder(user_id, name, command, args)).await?;
    Ok(())
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> InteractionResponse {
    let subcommand = &command.data.options[0];
    let name = utils::get_option(subcommand, "nom")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let command_name = utils::get_option(subcommand, "commande")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let args = if let Some(value) = utils::get_option(subcommand, "arguments") {
        Some(value.as_str().unwrap().to_string())
    } else {
        None
    };
    let content = match add_macro(ctx, command.user.id.to_string(), name, command_name, args).await
    {
        Ok(_) => "La macro a bien été ajoutée".to_string(),
        Err(e) => format!("Erreur lors de l'ajout de macro : {e}"),
    };
    InteractionResponse::Message(InteractionMessage {
        content,
        ephemeral: true,
        embed: None,
    })
}
