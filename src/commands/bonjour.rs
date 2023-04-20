use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
async fn bonjour(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Bonjour !").await?;

    Ok(())
}
