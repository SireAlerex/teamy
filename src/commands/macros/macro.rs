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
    let Some(strip_content) = msg.content.strip_prefix('!') else {
        return "Une macro doit contenir '!'".to_owned();
    };
    let name = strip_content.split(' ').next().unwrap_or("");
    let filter = doc! {"user_id": msg.author.id.to_string(), "name": name};
    if let Ok(res) = db::find_filter::<Macro>(ctx, "macros", filter).await {
        match res {
            Some(macr) => match macr.command.as_str() {
                "roll" => {
                    if let Some(args) = macr.args {
                        match roll::roll(ctx, msg, Args::new(&args, &[])).await {
                            Ok(_) => String::new(),
                            Err(e) => format!("Erreur lors de la macro : {e}"),
                        }
                    } else {
                        "la commande roll attends des arguments".to_owned()
                    }
                }
                _ => "La commande n'est pas prise en charge".to_owned(),
            },
            None => "La macro n'a pas été trouvée".to_owned(),
        }
    } else {
        "Problème avec la base de données".to_owned()
    }
}

pub async fn test_macro(
    ctx: &Context,
    command: &str,
    args: &Option<String>,
) -> Result<(), CommandError> {
    let Some(temp_chan) = utils::get_temp_chan(ctx).await else {
        return Err(utils::command_error("erreur lors du test de la macro"));
    };

    let msg = match command {
        "roll" => {
            roll::roll_intern(
                ctx,
                &temp_chan,
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
    let _msg = msg.delete(&ctx.http).await;
    Ok(())
}
