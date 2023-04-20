use serenity::model::channel::Message;
use serenity::prelude::*;

pub async fn handle_reaction(msg: Message, _ctx: Context) {
    println!("found this : {}", msg.content);
}
