use crate::command::{CommandGroupInfo, CommandGroups, CommandGroupsContainer};
use crate::{utils, InteractionMessage, InteractionResponse};
use serenity::model::application::command::CommandOptionType;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};
use std::sync::Arc;
use tracing::{error, info};

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> InteractionResponse {
    let data = ctx.data.read().await;
    let groups_container: Option<&Arc<tokio::sync::Mutex<CommandGroups>>> =
        data.get::<CommandGroupsContainer>();

    if groups_container.is_none() {
        return InteractionResponse::Message(InteractionMessage {
            content: "Erreur pour accéder aux groupes de commandes".to_owned(),
            ephemeral: true,
            embed: None,
        });
    }
    let groups_container: tokio::sync::MutexGuard<CommandGroups> =
        groups_container.unwrap().lock().await;

    let groups: &Vec<CommandGroupInfo> = &groups_container.groups;

    let title: String;
    let description: String;
    let mut fields: Vec<(&'static str, String, bool)> = Vec::default();
    let arg = command.data.options.first();
    if arg.is_some() {
        let arg = utils::strip_prefix_suffix(arg.unwrap().value.as_ref().unwrap().to_string(), '"');

        let x = CommandGroups {
            groups: groups.to_vec(),
        };
        match x.find_command(&arg) {
            Some(command) => info!("commande trouve : {:?}", command),
            None => error!("commande non trouvée pour {}", arg),
        }
        if let Some(search_group) = x.find_group(&arg) {
            let command = search_group.find_command(&arg).unwrap();
            title = command.names[0].to_owned();
            description = command
                .desc
                .unwrap_or("Erreur : pas de description")
                .to_string();
            // usage field
            if let Some(usage) = command.usage {
                fields.push(("Usage", format!("`{title} {usage}`"), true));
            }
            // examples field
            if !command.examples.is_empty() {
                fields.push((
                    "Sample usage",
                    command
                        .examples
                        .iter()
                        .map(|s| format!("`{title} {s}`"))
                        .collect::<Vec<String>>()
                        .join("\n"),
                    true,
                ))
            }
            // group field
            fields.push(("Group", search_group.name.to_owned(), true));
            // aliases
            if command.names.len() > 1 {
                fields.push((
                    "Aliases",
                    command.names[1..]
                        .iter()
                        .map(|s| format!("`{s}`"))
                        .collect::<Vec<String>>()
                        .join(","),
                    true,
                ))
            }
        } else {
            title = arg.clone();
            description = String::from("Erreur");
            fields.push((
                "Erreur",
                "La commande n'a pas pu être trouvée".to_string(),
                false,
            ));
        }
    } else {
        title = String::from("Help");
        description = String::from("Pour obtenir plus d'informations à propos d'une commande, utilisez la commande en argument.");
        for group in groups {
            let name = group.name;
            let mut commandes_names: Vec<&'static str> = Vec::default();
            for command in &group.commands {
                commandes_names.push(command.names.first().unwrap());
            }
            let value = commandes_names.join("\n");
            fields.push((name, value, false));
        }
    }

    let embed = CreateEmbed::default()
        .title(title)
        .description(description)
        .fields(fields)
        .color(serenity::utils::Colour::PURPLE)
        .to_owned();

    InteractionResponse::Message(InteractionMessage {
        content: String::default(),
        ephemeral: false,
        embed: Some(embed),
    })
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("help")
        .description("Donne des informations sur le bot")
        .create_option(|option| {
            option
                .name("commande")
                .description("commande dont on cherche les informations")
                .kind(CommandOptionType::String)
                .required(false)
        })
}
