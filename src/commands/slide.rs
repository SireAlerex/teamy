use serenity::builder::CreateApplicationCommand;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
#[description = "Slide dans tes dm"]
async fn slide(ctx: &Context, msg: &Message) -> CommandResult {
    msg.author.dm(&ctx.http, |m| m.content("Salut !")).await?;

    Ok(())
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    match command.user.dm(&ctx.http, |m| m.content("Salut !")).await {
        Ok(_) => "Un DM va être envoyé".to_owned(),
        Err(e) => format!("Une erreur c'est produite : {e}"),
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("slide")
        .description("Slide dans tes dm")
        .kind(CommandType::ChatInput)
}
