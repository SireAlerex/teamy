use crate::consts;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use serenity::builder::CreateApplicationCommand;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
async fn bonjour(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, run()).await?;

    Ok(())
}

pub fn run() -> String {
    format!(
        "{} !",
        consts::SALUTATIONS
            .iter()
            .choose(&mut thread_rng())
            .unwrap()
    )
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("bonjour")
        .description("Dis bonjour")
        .kind(CommandType::ChatInput)
}
