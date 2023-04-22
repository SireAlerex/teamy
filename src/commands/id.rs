use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
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
pub async fn id(ctx: &Context, msg: &Message) -> CommandResult {
    let mut user_input = msg.content.split_whitespace();
    if let Ok(user) = User::convert(
        ctx,
        msg.guild_id,
        Some(msg.channel_id),
        user_input.nth(1).unwrap(),
    )
    .await
    {
        let text = format!("L'id de {} est {}", user.clone().tag(), user.id);
        let _ = msg.channel_id.say(&ctx.http, text).await?;
    } else {
        let _ = msg
            .channel_id
            .say(&ctx.http, "L'utilisateur n'a pas pu être trouvé")
            .await?;
    }
    Ok(())
}

pub async fn run_user(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    if let Some(target_id) = command.data.target_id {
        if let Ok(user) = target_id.to_user_id().to_user(&ctx.http).await {
            format!("L'id de {} est {}", user.tag(), target_id)
        } else {
            format!("L'id de l'utilisateur est {}", target_id)
        }
    } else {
        format!("Pas de TargetId dans l'interaction")
    }
}

pub fn run_chat_input(options: &[CommandDataOption]) -> String {
    let option = options.get(0).unwrap().resolved.as_ref().unwrap();
    if let CommandDataOptionValue::User(user, _member) = option {
        format!("L'id de {} est {}", user.tag(), user.id)
    } else {
        "L'utilisateur n'a pas pu être trouvé".to_string()
    }
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
