use crate::utils;
use crate::{InteractionMessage, Response};
use serenity::{
    builder::CreateApplicationCommand,
    framework::standard::{macros::command, CommandResult},
    model::prelude::{
        command::{CommandOptionType, CommandType},
        interaction::application_command::{ApplicationCommandInteraction, CommandDataOption},
        Message,
    },
    prelude::Context,
};

#[command]
#[description = "Détermine si quelque chose est basé"]
#[usage = "<texte>"]
pub async fn basé(ctx: &Context, msg: &Message) -> CommandResult {
    let _: Message = msg.channel_id.say(&ctx.http, based()).await?;
    Ok(())
}

fn based<'a>() -> &'a str {
    if rand::random::<bool>() {
        "Basé"
    } else {
        "Cringe"
    }
}

pub async fn run_message(ctx: &Context, command: &ApplicationCommandInteraction) -> Response {
    let content = match command.data.target_id {
        Some(target_id) => {
            let message_id = target_id.to_message_id();
            match command.channel_id.message(&ctx.http, message_id).await {
                Ok(message) => format!("\"{}\"\n{}", message.content, based()),
                Err(e) => format!("Erreur pour trouver le message ({message_id}) : {e}"),
            }
        }
        None => String::from("Erreur pour accéder au MessageId de l'interaction"),
    };
    Response::Message(InteractionMessage::with_content(content))
}

pub fn run_chat_input(options: &[CommandDataOption]) -> Response {
    let content = format!(
        "\"{}\"\n{}",
        utils::command_option_str(options, "texte").unwrap_or("<erreur>"),
        based()
    );
    Response::Message(InteractionMessage::with_content(content))
}

pub fn register_message(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("basé").kind(CommandType::Message)
}

pub fn register_chat_input(
    command: &mut CreateApplicationCommand,
) -> &mut CreateApplicationCommand {
    command
        .name("basé")
        .description("Détermine si quelque chose est basé")
        .create_option(|option| {
            option
                .name("texte")
                .description("basé ou cringe ?")
                .kind(CommandOptionType::String)
                .required(true)
        })
        .kind(CommandType::ChatInput)
}
