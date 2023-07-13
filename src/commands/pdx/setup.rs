use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::{CommandOptionType, CommandType},
        interaction::application_command::ApplicationCommandInteraction,
    },
    prelude::*,
};

use super::{dd, show};
use crate::interaction::{InteractionMessage, InteractionResponse};

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> InteractionResponse {
    if let Some(first) = command.data.options.first() {
        match first.name.as_str() {
            "dd" => dd::run(ctx, command).await,
            "show" => show::run(ctx, command).await,
            _ => InteractionResponse::Message(InteractionMessage::with_content(
                "pdx run unknown name",
            )),
        }
    } else {
        InteractionResponse::Message(InteractionMessage::ephemeral("pdx run no option"))
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("pdx")
        .description("pdx_desc")
        .kind(CommandType::ChatInput)
        .create_option(|option| {
            option
                .name("dd")
                .description("Mets à jour les derniers DD et les affiche")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("show")
                .description("Affiche les derniers DD")
                .kind(CommandOptionType::SubCommand)
        })
}
