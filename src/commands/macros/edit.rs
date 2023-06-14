use crate::{InteractionMessage, InteractionResponse, utils};
use serenity::builder::CreateApplicationCommand;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::Message;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;

#[command]
#[description = "macro_edit_desc"]
async fn edit(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    Ok(())
}

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> InteractionResponse {
    InteractionResponse::Message(InteractionMessage {
        content: String::from("macro_edit"),
        ephemeral: true,
        embed: None,
    })
}
