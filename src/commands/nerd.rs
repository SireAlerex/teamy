use crate::utils;
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

fn run(text: &str) -> String {
    format!("\"{} :nerd:\"", utils::nerdify(text))
}

pub fn run_chat_input(options: &[CommandDataOption]) -> String {
    let option = options
        .get(0)
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap();

    run(option)
}

pub async fn run_message(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    if let Some(target_id) = command.data.target_id {
        let message_id = target_id.to_message_id();
        if let Ok(message) = command.channel_id.message(&ctx.http, message_id).await {
            format!("{} -{}", run(&message.content), message.author.name)
        } else {
            format!(
                "Erreur pour trouver le message correspondant à {}",
                message_id
            )
        }
    } else {
        "Erreur pour accéder au MessageId de l'interaction".to_string()
    }
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
