use serenity::builder::CreateApplicationCommand;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandResult, Args};
use serenity::model::prelude::command::{CommandType, CommandOptionType};
use serenity::model::prelude::Message;
use serenity::prelude::Context;
use tracing::info;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("macro")
        .description("macro_desc")
        .kind(CommandType::ChatInput)
        .create_option(|option| {
            option
                .name("macro_add")
                .description("macro_add_desc")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("macro_edit")
                .description("macro_edit_desc")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("macro_del")
                .description("macro_del_desc")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("macro_show")
                .description("macro_show_desc")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("macro_clear")
                .description("macro_clear_desc")
                .kind(CommandOptionType::SubCommand)
        })
}