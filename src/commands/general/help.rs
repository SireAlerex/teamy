use crate::command_info::{CommandGroups, CommandGroupsContainer};
use crate::{utils, InteractionMessage, InteractionResponse};
use serenity::model::application::command::CommandOptionType;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> InteractionResponse {
    let data = ctx.data.read().await;
    let command_groups_container = data.get::<CommandGroupsContainer>();

    let Some(groups_container) =  command_groups_container else {
        return InteractionResponse::Message(InteractionMessage::ephemeral(
            "Erreur pour accéder aux groupes de commandes",
        ));
    };
    let groups_info = &groups_container.groups;

    let mut title: String = "uninitialised".to_owned();
    let mut description: String = "uninitialised".to_owned();
    let mut fields: Vec<(&'static str, String, bool)> = Vec::default();

    // "/help" with a command name as argument
    if let Some(raw_arg) = utils::command_option_str(&command.data.options, "commande") {
        let arg = raw_arg.to_owned();
        let groups = CommandGroups::new(groups_info.clone());

        if let Some(search_group) = groups.find_group(&arg) {
            if let Some(command) = search_group.find_command(&arg) {
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
        for group in groups_info {
            let name = group.name;
            let mut command_names: Vec<String> = Vec::default();
            for command in &group.commands {
                command_names.push(format!(
                    "`{}`",
                    command.names.first().unwrap_or(&"pas de nom de commandes")
                ));
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

    InteractionResponse::Message(InteractionMessage::new("", true, Some(embed)))
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
