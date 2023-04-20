use crate::utils;
use serenity::model::channel::Message;
use serenity::prelude::*;

pub async fn handle_command(msg: Message, ctx: Context) {
    let command = utils::remove_suffix(&msg.content);
    match command.as_str() {
        "bonjour" => utils::send_message(msg, ctx, "Bonjour !").await,
        "slide" => utils::send_dm(msg, ctx, "Salut !").await,
        _ => utils::send_message(msg, ctx, format!("Commande inconnue : {}", &command).as_str()).await,
    }
}
