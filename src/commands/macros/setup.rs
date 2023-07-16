use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::{CommandOptionType, CommandType};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;

use super::{add, clear, del, edit, show};
use crate::{InteractionMessage, Response};

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Response {
    match command.data.options[0].name.as_str() {
        "add" => add::run(ctx, command).await,
        "del" => del::run(ctx, command).await,
        "show" => show::run(ctx, command).await,
        "edit" => edit::run(ctx, command).await,
        "clear" => clear::run(ctx, command).await,
        _ => Response::Message(InteractionMessage::ephemeral("macro_unknown_subcommand")),
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
                .description("crée une macro")
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
                .description("modifie une macro")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|suboption| {
                    suboption
                        .name("nom")
                        .description("nom de la macro à modifier")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
                .create_sub_option(|suboption| {
                    suboption
                        .name("arguments")
                        .description("nouveaux arguments de la macro")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("del")
                .description("supprime une macro")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|suboption| {
                    suboption
                        .name("nom")
                        .description("nom de la macro à supprimer")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("show")
                .description("affiche les macros de l'utilisateur")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("clear")
                .description("supprime toutes les macros")
                .kind(CommandOptionType::SubCommand)
        })
}

pub fn register_message(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("macro add").kind(CommandType::Message)
}
