use bson::doc;
use serenity::{model::prelude::Message, prelude::Context, framework::standard::Args};
use tracing::{info, error};
use crate::{db, utils};
use crate::commands::general::roll;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Macro {
    _id: mongodb::bson::oid::ObjectId,
    user_id: String,
    name: String,
    command: String,
    args: Option<String>,
}

impl Macro {
    pub fn builder(user_id: String, name: String, command: String, args: Option<String>) -> Macro {
        Macro { _id: mongodb::bson::oid::ObjectId::new(), user_id, name, command, args }
    }
}

pub async fn handle_macro(ctx: &Context, msg: &Message) -> String {
    let name = msg.content[1..].split(' ').next().unwrap_or("");
    let filter = doc! {"user_id": msg.author.id.to_string(), "name": name};
    if let Ok(res) = db::find_filter::<Macro>(ctx, "macros", filter).await {
        match res {
            Some(macr) => {
                let x = macr.args.unwrap();
                match roll::roll(ctx, msg, Args::new(&x, &[])).await {
                    Ok(_) => info!("ok"),
                    Err(e) => error!("err : {e}"),
                }
            },
            None => utils::say_or_error(ctx, msg.channel_id, "La macro n'a pas été trouvée").await
        }
    } else {
        utils::say_or_error(ctx, msg.channel_id, "Problème avec la base de données").await
    }

    String::new()
}
