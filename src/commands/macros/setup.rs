use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::{CommandOptionType, CommandType};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;

use super::{add, clear, del, edit, show};
use crate::{InteractionMessage, InteractionResponse};

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> InteractionResponse {
    match command.data.options[0].name.as_str() {
        "add" => add::run(ctx, command).await,
        "del" => del::run(ctx, command).await,
        "show" => show::run(ctx, command).await,
        "edit" => edit::run(ctx, command).await,
        "clear" => clear::run(ctx, command).await,
        _ => InteractionResponse::Message(InteractionMessage {
            content: "macro_unknown_subcommand".to_string(),
            ephemeral: true,
            embed: None,
        }),
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("macro")
        .description("macro_desc")
        .kind(CommandType::ChatInput)
        .create_option(|option| {
            option
                .name("add")
                .description("macro_add_desc")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|suboption| {
                    suboption
                        .name("nom")
                        .description("nom de la macro")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
                .create_sub_option(|suboption| {
                    suboption
                        .name("commande")
                        .description("commande de la macro")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
                .create_sub_option(|suboption| {
                    suboption
                        .name("arguments")
                        .description("arguments de la macro")
                        .kind(CommandOptionType::String)
                        .required(false)
                })
        })
        .create_option(|option| {
            option
                .name("edit")
                .description("macro_edit_desc")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("del")
                .description("macro_del_desc")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|suboption| {
                    suboption
                        .name("nom")
                        .description("nom de la macro")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("show")
                .description("macro_show_desc")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("clear")
                .description("macro_clear_desc")
                .kind(CommandOptionType::SubCommand)
        })
}
