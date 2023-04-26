use std::sync::Arc;

use crate::utils;
use serenity::builder::CreateApplicationCommand;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
#[description = "Donne la latence du bot"]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, run(ctx).await).await?;

    Ok(())
}

pub async fn run(ctx: &Context) -> String {
    match utils::runner_latency(Arc::new(ctx.clone())).await {
        Some(l) => format!("Pong ! ``{:#?}``", l),
        None => "Pong ! (il y a un problème pour accéder à la latence du bot, veuillez réessayer dans 1min)".to_string()
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("ping")
        .description("Donne la latence du bot")
        .kind(CommandType::ChatInput)
}
