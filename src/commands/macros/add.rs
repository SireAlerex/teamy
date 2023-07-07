use super::r#macro::{test_macro, Macro, TempMacro};
use crate::{consts, db, utils};
use crate::{InteractionMessage, InteractionResponse};
use bson::doc;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandError, CommandResult};
use serenity::model::prelude::component::{ActionRowComponent, InputTextStyle};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::interaction::modal::ModalSubmitInteraction;
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
#[description = "crée une macro"]
#[usage = "<nom de la macro> <commande> <arguments>"]
#[example = "init roll d20+4"]
#[example = "d6 roll d6"]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_id = msg.author.id;
    let name = args.single::<String>()?;
    let command = args.single::<String>()?;
    let args = if args.len() == 3 {
        Some(args.single::<String>()?)
    } else {
        None
    };
    add_macro(ctx, user_id.to_string(), name, command, args).await?;

    utils::say_or_error(ctx, msg.channel_id, "La macro a bien été ajoutée").await;
    Ok(())
}

async fn add_macro(
    ctx: &Context,
    user_id: String,
    name: String,
    command: String,
    args: Option<String>,
) -> CommandResult {
    test_macro(ctx, &command, &args).await?;
    db::insert(ctx, "macros", &Macro::builder(user_id, name, command, args)).await?;
    Ok(())
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> InteractionResponse {
    let subcommand = &command.data.options[0];
    let name = utils::get_option(subcommand, "nom")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let command_name = utils::get_option(subcommand, "commande")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let args =
        utils::get_option(subcommand, "arguments").map(|value| value.as_str().unwrap().to_string());
    let content = match add_macro(ctx, command.user.id.to_string(), name, command_name, args).await
    {
        Ok(_) => "La macro a bien été ajoutée".to_string(),
        Err(e) => format!("Erreur lors de l'ajout de macro : {e}"),
    };
    InteractionResponse::Message(InteractionMessage {
        content,
        ephemeral: true,
        embed: None,
    })
}

async fn add_temp_macro(
    ctx: &Context,
    msg: &Message,
    command: &ApplicationCommandInteraction,
) -> Result<(), CommandError> {
    let user_id = command.user.id.to_string();
    // delete temporary macros of user
    temp_cleanup(ctx, user_id.clone()).await?;

    match utils::first_letter(&msg.content) {
        // macro call
        Some('!') => {
            // find macro in db
            let filter =
                doc! {"user_id": msg.author.id.to_string(), "name": msg.content.strip_prefix('!')};
            let macr = db::find_filter::<Macro>(ctx, "macros", filter).await?;
            if let Some(macr) = macr {
                let temp_macro = TempMacro::builder(user_id, macr.command, macr.args);
                let _ = db::insert(ctx, "temp_macros", &temp_macro).await?;
                Ok(())
            } else {
                Err(utils::command_error("macro non trouvée"))
            }
        }
        // empty message
        None => Err(utils::command_error("bad message")),
        // command message
        _ => {
            if msg.content.starts_with("`[r") {
                // roll macro
                let args = roll_args(&msg.content)?;
                let temp_macro = TempMacro::builder(user_id, "roll".to_string(), Some(args));
                let _ = db::insert(ctx, "temp_macros", &temp_macro).await?;
                Ok(())
            } else {
                // non roll macros will need to be dealt with here
                Ok(())
            }
        }
    }
}

fn roll_args(s: &str) -> Result<String, CommandError> {
    let re = regex::Regex::new(r"\[r (?P<args>.*)\]")?;
    let Some(caps) = re.captures(s) else {
        return Err(utils::command_error("erreur captures regex"));
    };
    match caps.name("args") {
        Some(m) => Ok(m.as_str().to_string()),
        None => Err(utils::command_error("pas d'arguments")),
    }
}

pub async fn run_message_form(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> InteractionResponse {
    // create temporary macro from message and send modal to get macro name
    let content = if let Some(msg) = &command.data.resolved.messages.values().next() {
        match add_temp_macro(ctx, msg, command).await {
            Ok(_) => {
                modal(ctx, command).await;
                return InteractionResponse::None;
            }
            Err(e) => format!("erreur lors de la préparation de l'ajout de macro : {e}"),
        }
    } else {
        "pas de message".to_string()
    };
    InteractionResponse::Message(InteractionMessage {
        content,
        ephemeral: true,
        embed: None,
    })
}

async fn temp_cleanup(ctx: &Context, user_id: String) -> Result<(), mongodb::error::Error> {
    let query = doc! {"user_id": user_id};
    db::delete_multiple_query::<TempMacro>(ctx, "temp_macros", query).await
}

async fn complete_macro(
    ctx: &Context,
    user_id: String,
    name: String,
) -> Result<(), mongodb::error::Error> {
    // get temp macro and completes it
    let filter = doc! { "user_id": user_id.clone() };
    match db::find_filter::<TempMacro>(ctx, "temp_macros", filter).await? {
        Some(temp) => {
            let macr = Macro::builder(temp.user_id, name, temp.command, temp.args);
            temp_cleanup(ctx, user_id).await?;
            let _ = db::insert::<Macro>(ctx, "macros", &macr).await?;
            Ok(())
        }
        None => Err(db::mongodb_error("macro temporaire non trouvée")),
    }
}

pub async fn run_message(ctx: &Context, modal: &ModalSubmitInteraction) -> InteractionResponse {
    // take name from modal and completes macro
    let component = &modal.data.components.first().unwrap().components.first();
    let content = if let Some(ActionRowComponent::InputText(input)) = component {
        if input.custom_id == consts::MACRO_ADD_FORM_NAME {
            match complete_macro(ctx, modal.user.id.to_string(), input.value.clone()).await {
                Ok(_) => "La macro a bien été ajouté".to_string(),
                Err(e) => format!("erreur lors de la complétion de la macro : {e}"),
            }
        } else {
            "erreur modal id".to_string()
        }
    } else {
        "erreur modal component".to_string()
    };
    InteractionResponse::Message(InteractionMessage {
        content,
        ephemeral: true,
        embed: None,
    })
}

pub async fn modal(ctx: &Context, command: &ApplicationCommandInteraction) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::Modal)
                .interaction_response_data(|modal| {
                    modal
                        .title("Ajout de macro")
                        .components(|input| {
                            input.create_action_row(|f| {
                                f.create_input_text(|t| {
                                    t.label("Nom de la macro")
                                        .custom_id(consts::MACRO_ADD_FORM_NAME)
                                        .style(InputTextStyle::Short)
                                })
                            })
                        })
                        .custom_id(consts::MACRO_ADD_FORM_ID)
                })
        })
        .await
    {
        let error_message = format!("Erreur lors de la réponse à l'interaction : {why}");
        utils::say_or_error(ctx, command.channel_id, error_message).await;
    }
}
