use crate::utils;
use crate::{InteractionMessage, InteractionResponse};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::prelude::Message;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandType, interaction::application_command::ApplicationCommandInteraction,
    },
    prelude::Context,
};

#[command]
#[description = "Nerdifie un texte"]
#[usage = "<texte à transformer>"]
pub async fn nerd(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.say(&ctx.http, run(args.message())).await?;
    Ok(())
}

pub fn run(text: &str) -> String {
    format!("\"{} :nerd:\"", utils::nerdify(text))
}

pub fn run_chat_input(options: &[CommandDataOption]) -> InteractionResponse {
    let content =
        utils::command_option_str(options, "texte").map_or("erreur: pas de texte?".to_owned(), run);

    InteractionResponse::Message(InteractionMessage::with_content(content))
}

pub async fn run_message(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> InteractionResponse {
    let content = match command.data.target_id {
        Some(target_id) => {
            let message_id = target_id.to_message_id();
            match command.channel_id.message(&ctx.http, message_id).await {
                Ok(message) => format!("{} -{}", run(&message.content), message.author.name),
                Err(e) => format!("Erreur pour trouver le message ({message_id}) : {e}"),
            }
        }
        None => String::from("Erreur pour accéder au MessageId de l'interaction"),
    };
    InteractionResponse::Message(InteractionMessage::with_content(content))
}

pub fn register_message(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("nerd").kind(CommandType::Message)
}

pub fn register_chat_input(
    command: &mut CreateApplicationCommand,
) -> &mut CreateApplicationCommand {
    command
        .name("nerd")
        .description("Nerdifie un texte")
        .create_option(|option| {
            option
                .name("texte")
                .description("Texte à transformer")
                .kind(CommandOptionType::String)
                .required(true)
        })
        .kind(CommandType::ChatInput)
}
