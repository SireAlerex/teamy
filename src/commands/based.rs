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
    let _ = msg.channel_id.say(&ctx.http, based()).await?;
    Ok(())
}

fn based<'a>() -> &'a str {
    match rand::random::<bool>() {
        true => "Basé",
        false => "Cringe",
    }
}

pub async fn run_message(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    if let Some(target_id) = command.data.target_id {
        let message_id = target_id.to_message_id();
        if let Ok(message) = command.channel_id.message(&ctx.http, message_id).await {
            format!("\"{}\"\n{}", message.content, based())
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

pub fn run_chat_input(options: &[CommandDataOption]) -> String {
    format!(
        "\"{}\"\n{}",
        options[0].value.as_ref().unwrap().as_str().unwrap(),
        based()
    )
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
