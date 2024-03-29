use std::sync::Arc;

use crate::utils;
use crate::{InteractionMessage, Response};
use serenity::builder::CreateApplicationCommand;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
#[description = "Donne la latence du bot"]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, latency(ctx).await).await?;

    Ok(())
}

async fn latency(ctx: &Context) -> String {
    let res = match utils::RunnerInfo::info(Arc::new(ctx.clone())).await {
        Ok(runnner) => match runnner.latency {
            Some(ping) => format!("``{ping:#?}``"),
            None => "(il y a un problème pour accéder à la latence du bot, veuillez réessayer dans 1min)".to_owned()
        },
        Err(e) => format!("(une erreur s'est produite : {e}")
    };
    format!("Pong ! {res}")
}

pub async fn run(ctx: &Context) -> Response {
    Response::Message(InteractionMessage::with_content(latency(ctx).await))
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("ping")
        .description("Donne la latence du bot")
        .kind(CommandType::ChatInput)
}
