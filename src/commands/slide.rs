use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
async fn slide(ctx: &Context, msg: &Message) -> CommandResult {
    msg.author.dm(&ctx.http, |m| m.content("Salut !")).await?;

    Ok(())
}
