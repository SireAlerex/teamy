use crate::{InteractionMessage, InteractionResponse, db};
use bson::doc;
use serenity::builder::CreateApplicationCommand;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandResult, Args};
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

use super::r#macro::Macro;

#[command]
#[description = "macro_del_desc"]
async fn del(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_id = msg.author.id;
    let name = args.single::<String>()?;
    del_macro(ctx, user_id.to_string(), name).await?;

    Ok(())
}

async fn del_macro(ctx: &Context, user_id: String, name: String) -> Result<(), mongodb::error::Error> {
    let query = doc! { "user_id": user_id, "name": name };
    db::delete_filter::<Macro>(ctx, "macros", query).await
}

pub fn run() -> InteractionResponse {
    InteractionResponse::Message(InteractionMessage {
        content: "macro_del".to_string(),
        ephemeral: false,
        embed: None,
    })
}
