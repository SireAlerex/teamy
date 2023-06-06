use crate::{InteractionMessage, InteractionResponse};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};
use serenity::model::user::User;
use serenity::utils::ArgumentConvert;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandType, interaction::application_command::ApplicationCommandInteraction,
        Message,
    },
    prelude::Context,
};

#[command]
#[description = "Affiche l'id d'un utilisateur"]
#[usage = "<nom OU nom#tag OU mention>"]
#[example = "boup"]
#[example = "boup#1234"]
#[example = "@boup"]
pub async fn id(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let user_input = args.message();
    let text = match User::convert(ctx, msg.guild_id, Some(msg.channel_id), user_input).await {
        Ok(user) => format!("L'id de {} est {}", user.clone().tag(), user.id),
        Err(e) => format!("L'utilisateur n'a pas pu être trouvé : {e}"),
    };
    let _ = msg.channel_id.say(&ctx.http, text).await?;
    Ok(())
}

pub async fn run_user(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> InteractionResponse {
    let result = match command.data.target_id {
        Some(target_id) => match target_id.to_user_id().to_user(&ctx.http).await {
            Ok(user) => format!("L'id de {} est {}", user.tag(), target_id),
            Err(e) => format!("Erreur avec l'id {} : {}", target_id, e),
        },
        None => String::from("Pas de TargetId dans l'interaction"),
    };
    InteractionResponse::Message(InteractionMessage {
        content: result,
        ephemeral: false,
        embed: None,
    })
}

pub fn run_chat_input(options: &[CommandDataOption]) -> InteractionResponse {
    let option = options.get(0).unwrap().resolved.as_ref().unwrap();
    let result = match option {
        CommandDataOptionValue::User(user, _) => format!("L'id de {} est {}", user.tag(), user.id),
        _ => String::from("L'utilisateur n'a pas pu être trouvé"),
    };
    InteractionResponse::Message(InteractionMessage {
        content: result,
        ephemeral: false,
        embed: None,
    })
}

pub fn register_user(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("id").kind(CommandType::User)
}

pub fn register_chat_input(
    command: &mut CreateApplicationCommand,
) -> &mut CreateApplicationCommand {
    command
        .name("id")
        .description("Affiche l'id d'un utilisateur")
        .create_option(|option| {
            option
                .name("utilisateur")
                .description("Utilisateur dont on cherche l'id")
                .kind(CommandOptionType::User)
                .required(true)
        })
        .kind(CommandType::ChatInput)
}
