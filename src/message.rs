use serenity::prelude::*;
use serenity::model::channel::Message;
use tracing::error;

pub async fn handle_command(msg: Message, ctx: Context) {
    let command = msg.content;
    match command.as_str()   {
        "bonjour" => if let Err(e) = msg.channel_id.say(&ctx.http, "Bonjour !").await {
            error!("Error sending message: {:?}", e);
        },
        _ => if let Err(e) = msg.channel_id.say(&ctx.http, "Commande inconnue").await {
            error!("Error sending message: {:?}", e);
        },
    }
}