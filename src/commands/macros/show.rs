use crate::{InteractionMessage, InteractionResponse};
use serenity::builder::CreateApplicationCommand;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandResult, Args};
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
#[description = "macro_show_desc"]
async fn show(ctx: &Context, msg: &Message, args: Args) -> CommandResult {


    Ok(())
}

pub fn run() -> InteractionResponse {
    InteractionResponse::Message(InteractionMessage {
        content: "macro_show".to_string(),
        ephemeral: false,
        embed: None,
    })
}