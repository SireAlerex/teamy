use crate::{InteractionMessage, InteractionResponse};
use serenity::builder::CreateApplicationCommand;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
#[description = "macro_clear_desc"]
async fn clear(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    Ok(())
}

pub fn run() -> InteractionResponse {
    InteractionResponse::Message(InteractionMessage {
        content: "macro_clear".to_string(),
        ephemeral: false,
        embed: None,
    })
}
