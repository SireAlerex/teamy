use crate::{InteractionMessage, InteractionResponse};
use serenity::builder::CreateApplicationCommand;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandResult, Args};
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::{Message, UserId};
use serenity::prelude::Context;
use super::r#macro::Macro;
use crate::db;
use tracing::info;

#[command]
#[description = "macro_add_desc"]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_id = msg.author.id;
    let name = args.single::<String>()?;
    let command = args.single::<String>()?;
    let args = if args.len() == 3 { Some(args.single:: <String>()? ) } else { None };
    add_macro(ctx, user_id.to_string(), name, command, args).await?;

    Ok(())
}

async fn add_macro(ctx: &Context, user_id: String, name: String, command: String, args: Option<String>) -> Result<(), mongodb::error::Error> {
    //let args = if args.is_empty() { None } else { Some(args) };
    let macr = Macro::builder(user_id, name, command, args);
    db::insert(ctx, "macros", &macr).await
}

pub fn run() -> InteractionResponse {
    InteractionResponse::Message(InteractionMessage {
        content: "macro_add".to_string(),
        ephemeral: false,
        embed: None,
    })
}
