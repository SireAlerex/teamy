use crate::commands::{Context, PoiseError};

#[poise::command(
    slash_command,
    prefix_command,
    category = "general",
    description_localized(
        "fr",
        "Donne des informations sur toutes les commandes du bot ou une précise en paramètre"
    )
)]
pub async fn help(ctx: Context<'_>, command: Option<String>) -> Result<(), PoiseError> {
    let configuration = poise::builtins::HelpConfiguration {
        show_context_menu_commands: true,
        show_subcommands: true,
        include_description: true,
        extra_text_at_bottom:
            "'$help <command>' pour avoir des informations sur une commande précise",
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), configuration).await?;
    Ok(())
}

// TODO: re-add pretty help with embed ?
// pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Response {
//     let data = ctx.data.read().await;
//     let command_groups_container = data.get::<CommandGroupsContainer>();

//     let Some(groups_container) =  command_groups_container else {
//         return Response::Message(InteractionMessage::ephemeral(
//             "Erreur pour accéder aux groupes de commandes",
//         ));
//     };
//     let groups_info = &groups_container.0;

//     let mut title: String = "uninitialised".to_owned();
//     let mut description: String = "uninitialised".to_owned();
//     let mut fields: Vec<(&'static str, String, bool)> = Vec::default();

//     // "/help" with a command name as argument
//     if let Some(raw_arg) = utils::command_option_str(&command.data.options, "commande") {
//         let arg = raw_arg.to_owned();
//         let groups = CommandGroups::new(groups_info.clone());

//         if let Some(search_group) = groups.find_group(&arg) {
//             if let Some(command) = search_group.find_command(&arg) {
//                 title = (*command.names.first().unwrap_or(&"no name")).to_string();
//                 description = command
//                     .desc
//                     .unwrap_or("Erreur : pas de description")
//                     .to_owned();
//                 // usage field
//                 if let Some(usage) = command.usage {
//                     fields.push(("Usage", format!("`{title} {usage}`"), true));
//                 }
//                 // examples field
//                 if !command.examples.is_empty() {
//                     fields.push((
//                         "Sample usage",
//                         command
//                             .examples
//                             .iter()
//                             .map(|s| format!("`{title} {s}`"))
//                             .collect::<Vec<String>>()
//                             .join("\n"),
//                         true,
//                     ));
//                 }
//                 // group field
//                 fields.push(("Group", search_group.name.to_owned(), true));
//                 // aliases
//                 if command.names.len() > 1 {
//                     fields.push((
//                         "Aliases",
//                         command
//                             .names
//                             .iter()
//                             .skip(1)
//                             .map(|s| format!("`{s}`"))
//                             .collect::<Vec<String>>()
//                             .join(","),
//                         true,
//                     ));
//                 }
//             }
//         } else {
//             title = arg.clone();
//             description = String::from("Erreur");
//             fields.push((
//                 "Erreur",
//                 "La commande n'a pas pu être trouvée".to_owned(),
//                 false,
//             ));
//         }
//     } else {
//         title = String::from("Help");
//         description = String::from("Pour obtenir plus d'informations à propos d'une commande, utilisez la commande en argument.");
//         for group in groups_info {
//             let name = group.name;
//             let mut command_names: Vec<String> = Vec::default();
//             for command in &group.commands {
//                 command_names.push(format!(
//                     "`{}`",
//                     command.names.first().unwrap_or(&"pas de nom de commandes")
//                 ));
//             }
//             let prefixes = group
//                 .prefixes
//                 .iter()
//                 .map(|p| format!("`{p}`"))
//                 .collect::<Vec<String>>()
//                 .join(", ");
//             let names = command_names.join("\n");
//             let value = format!("Préfixe(s) : {prefixes}\n{names}");
//             fields.push((name, value, true));
//         }
//     }

//     let embed = CreateEmbed::default()
//         .title(title)
//         .description(description)
//         .fields(fields)
//         .color(serenity::utils::Colour::PURPLE)
//         .clone();

//     Response::Message(InteractionMessage::new("", true, Some(embed)))
// }

// pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
//     command
//         .name("help")
//         .description("Donne des informations sur le bot")
//         .create_option(|option| {
//             option
//                 .name("commande")
//                 .description("commande dont on cherche les informations")
//                 .kind(CommandOptionType::String)
//                 .required(false)
//         })
// }
