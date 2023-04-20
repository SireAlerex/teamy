use serenity::{model::prelude::Message, prelude::Context};
use tracing::error;

pub fn first_letter(s: &String) -> char {
    s.chars().next().unwrap()
}

pub fn remove_suffix(s: &String) -> String {
    let mut c = s.chars();
    c.next();
    c.collect()
}

pub async fn send_message(msg: Message, ctx: Context, text: &str) {
    if let Err(e) = msg.channel_id.say(&ctx.http, text).await {
        error!("Error sending message: {:?}", e);
    }
}

pub async fn send_dm(msg: Message, ctx: Context, text: &str) {
    if let Err(e) = msg.author.dm(&ctx.http, |m| m.content(text)).await {
        error!("Error sending message: {:?}", e);
    }
}
