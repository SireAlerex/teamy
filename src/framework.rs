use std::collections::HashSet;

use anyhow::anyhow;
use serenity::framework::standard::macros::{help, hook};
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, HelpOptions,
};
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::prelude::*;
use serenity::prelude::*;
use shuttle_runtime::Error;
use tracing::{error, info};

use crate::command_info::{CommandGroupInfo, CommandGroups, CommandInfo};
use crate::commands::general::GENERAL_GROUP;
use crate::commands::macros::MACRO_GROUP;
use crate::commands::pdx::PDX_GROUP;
use crate::utils;

#[help]
#[individual_command_tip = "Pour obtenir plus d'informations à propos d'une commande, utilisez la commande en argument."]
#[command_not_found_text = "Commande non trouvée : '{}'."]
#[max_levenshtein_distance(3)]
#[lacking_permissions = "Hide"]
async fn my_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _: Message =
        help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await?;
    Ok(())
}

#[hook]
async fn after(ctx: &Context, msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => info!("Processed command '{}'", command_name),
        Err(why) => {
            error!(
                "Command '{}' returned error {:?} (message was '{}')",
                command_name, why, msg.content
            );
            utils::say_or_error(
                ctx,
                msg.channel_id,
                format!("Erreur lors de la commande : {why}"),
            )
            .await;
        }
    }
}

pub async fn get_framework(http: Http) -> Result<StandardFramework, Error> {
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => return Err(anyhow!("Could not access application info: {why:?}").into()),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("$"))
        .after(after)
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&MACRO_GROUP)
        .group(&PDX_GROUP);

    Ok(framework)
}

pub fn get_command_groups() -> CommandGroups {
    let static_groups = vec![&GENERAL_GROUP, &MACRO_GROUP, &PDX_GROUP];
    let mut groups: Vec<CommandGroupInfo> = Vec::default();
    for group in &static_groups {
        let mut commands: Vec<CommandInfo> = Vec::default();
        for command in group.options.commands {
            let x = CommandInfo {
                names: command.options.names,
                desc: command.options.desc,
                usage: command.options.usage,
                examples: command.options.examples,
            };
            commands.push(x);
        }
        groups.push(CommandGroupInfo {
            name: group.name,
            commands,
            prefixes: group.options.prefixes,
        });
    }
    CommandGroups(groups)
}
