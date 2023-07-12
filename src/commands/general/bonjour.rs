use crate::consts;
use crate::{InteractionMessage, InteractionResponse};
use rand::seq::IteratorRandom;
use rand::thread_rng;
use serenity::builder::CreateApplicationCommand;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
#[description = "Dis bonjour"]
async fn bonjour(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, salutation()).await?;

    Ok(())
}

fn salutation() -> String {
    format!(
        "{} !",
        consts::SALUTATIONS
            .iter()
            .choose(&mut thread_rng())
            .unwrap()
    )
}

pub fn run() -> InteractionResponse {
    InteractionResponse::Message(InteractionMessage::with_content(salutation()))
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("bonjour")
        .description("Dis bonjour")
        .kind(CommandType::ChatInput)
}
