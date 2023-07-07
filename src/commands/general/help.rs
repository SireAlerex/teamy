use crate::command::{CommandGroups, CommandGroupsContainer};
use crate::{utils, InteractionMessage, InteractionResponse};
use serenity::model::application::command::CommandOptionType;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> InteractionResponse {
    let data = ctx.data.read().await;
    let groups_container = data.get::<CommandGroupsContainer>();

    if groups_container.is_none() {
        return InteractionResponse::Message(InteractionMessage {
            content: "Erreur pour accéder aux groupes de commandes".to_owned(),
            ephemeral: true,
            embed: None,
        });
    }
    let groups_container = groups_container.unwrap().lock().await;

    let groups = &groups_container.groups;

    let title: String;
    let description: String;
    let mut fields: Vec<(&'static str, String, bool)> = Vec::default();
    let arg = command.data.options.first();
    if arg.is_some() {
        let arg =
            utils::strip_prefix_suffix(arg.unwrap().value.as_ref().unwrap().as_str().unwrap(), '"');

        let x = CommandGroups {
            groups: groups.clone(),
        };
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
                ));
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
                ));
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
            let mut command_names: Vec<String> = Vec::default();
            for command in &group.commands {
                command_names.push(format!("`{}`", command.names.first().unwrap()));
            }
            let prefixes = group
                .prefixes
                .iter()
                .map(|p| format!("`{p}`"))
                .collect::<Vec<String>>()
                .join(", ");
            let command_names = command_names.join("\n");
            let value = format!("Préfixe(s) : {prefixes}\n{command_names}");
            fields.push((name, value, true));
        }
    }

    let embed = CreateEmbed::default()
        .title(title)
        .description(description)
        .fields(fields)
        .color(serenity::utils::Colour::PURPLE)
        .clone();

    InteractionResponse::Message(InteractionMessage {
        content: String::default(),
        ephemeral: true,
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
