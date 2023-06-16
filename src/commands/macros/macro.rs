use crate::commands::general::roll;
use crate::{db, utils};
use bson::doc;
use serenity::framework::standard::CommandError;
use serenity::{framework::standard::Args, model::prelude::Message, prelude::Context};

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Macro {
    _id: mongodb::bson::oid::ObjectId,
    user_id: String,
    pub name: String,
    pub command: String,
    pub args: Option<String>,
}

impl Macro {
    pub fn builder(user_id: String, name: String, command: String, args: Option<String>) -> Macro {
        Macro {
            _id: mongodb::bson::oid::ObjectId::new(),
            user_id,
            name,
            command,
            args,
        }
    }

    pub fn edit(mut self, args: Option<&String>) -> Self {
        self.args = args.cloned();
        self
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct TempMacro {
    _id: mongodb::bson::oid::ObjectId,
    pub user_id: String,
    pub command: String,
    pub args: Option<String>,
}

impl TempMacro {
    pub fn builder(user_id: String, command: String, args: Option<String>) -> TempMacro {
        TempMacro {
            _id: mongodb::bson::oid::ObjectId::new(),
            user_id,
            command,
            args,
        }
    }
}

pub async fn handle_macro(ctx: &Context, msg: &Message) -> String {
    let name = msg.content[1..].split(' ').next().unwrap_or("");
    let filter = doc! {"user_id": msg.author.id.to_string(), "name": name};
    if let Ok(res) = db::find_filter::<Macro>(ctx, "macros", filter).await {
        match res {
            Some(macr) => match macr.command.as_str() {
                "roll" => {
                    let x = macr.args.unwrap();
                    match roll::roll(ctx, msg, Args::new(&x, &[])).await {
                        Ok(_) => String::new(),
                        Err(e) => format!("Erreur lors de la macro : {e}"),
                    }
                }
                _ => "La commande n'est pas prise en charge".to_string(),
            },
            None => "La macro n'a pas été trouvée".to_string(),
        }
    } else {
        "Problème avec la base de données".to_string()
    }
}

pub async fn test_macro(
    ctx: &Context,
    command: &str,
    args: &Option<String>,
) -> Result<(), CommandError> {
    let temp_chan = if let Some(id) = utils::get_temp_chan(ctx).await {
        id
    } else {
        return Err(utils::command_error("erreur lors du test de la macro"));
    };
    let msg = match command {
        "roll" => {
            roll::roll_intern(
                ctx,
                temp_chan,
                Args::new(&args.clone().unwrap_or(String::new()), &[]),
            )
            .await?
        }
        _ => {
            return Err(utils::command_error(
                "La commande ne peut pas être utilisée comme macro",
            ))
        }
    };
    // error of deletion is unrelated to macro test so ignore it
    let _ = msg.delete(&ctx.http).await;
    Ok(())
}
