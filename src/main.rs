pub mod command;
mod commands;
pub mod interaction;
pub mod consts;
#[allow(clippy::impl_trait_in_params)]
pub mod db;
mod loops;
mod message;
pub mod utils;
pub mod web_scraper;

use anyhow::anyhow;
use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::macros::{group, help, hook};
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, HelpOptions,
};
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::application::command::Command;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::interaction::Interaction;
use serenity::model::prelude::{ChannelId, GuildId, UserId};
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

use interaction::{InteractionMessage, InteractionResponse};
use crate::command::{CommandGroupInfo, CommandGroups, CommandGroupsContainer, CommandInfo};
use crate::commands::general;
use crate::commands::general::{
    based::BASÉ_COMMAND, bonjour::BONJOUR_COMMAND, id::ID_COMMAND, nerd::NERD_COMMAND,
    ping::PING_COMMAND, roll::ROLL_COMMAND, slide::SLIDE_COMMAND,
};
use crate::commands::macros;
use crate::commands::macros::{
    add::ADD_COMMAND, clear::CLEAR_COMMAND, del::DEL_COMMAND, edit::EDIT_COMMAND,
    show::SHOW_COMMAND,
};
use crate::commands::pdx::dd::DD_COMMAND;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<tokio::sync::Mutex<ShardManager>>;
}

struct GuildGroup {
    guilds: Vec<GuildId>,
}

struct GuildIdContainer;

impl TypeMapKey for GuildIdContainer {
    type Value = Arc<tokio::sync::Mutex<GuildGroup>>;
}

struct LogChanIdContainer;

impl TypeMapKey for LogChanIdContainer {
    type Value = Arc<tokio::sync::Mutex<ChannelId>>;
}

struct DatabaseUri {
    db_uri: String,
}

struct DatabaseUriContainer;

impl TypeMapKey for DatabaseUriContainer {
    type Value = Arc<tokio::sync::Mutex<DatabaseUri>>;
}

struct TempChanContainer;

impl TypeMapKey for TempChanContainer {
    type Value = Arc<tokio::sync::Mutex<ChannelId>>;
}

struct Bot {
    is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        let content = match utils::first_letter(&msg.content) {
            Some('$') | None => String::new(), // do nothing if command or empty message
            Some('!') => commands::macros::r#macro::handle_macro(&ctx, &msg).await,
            _ => match message::handle_reaction(&ctx, &msg).await {
                Ok(s) => s,
                Err(e) => e.to_string(),
            },
        };
        utils::say_or_error(&ctx, msg.channel_id, &content).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let ctx_arc: Arc<Context> = Arc::new(ctx);
        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ctx1 = Arc::clone(&ctx_arc);

            tokio::spawn(async move {
                loop {
                    loops::status_loop(&Arc::clone(&ctx1));
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            });

            let ctx2 = Arc::clone(&ctx_arc);

            tokio::spawn(async move {
                loop {
                    loops::log_system_load(Arc::clone(&ctx2)).await;
                    tokio::time::sleep(Duration::from_secs(300)).await;
                }
            });
        }

        // clean global commands
        match Command::get_global_application_commands(&ctx_arc.http).await {
            Ok(commands) => {
                for command in commands {
                    if let Err(e) =
                        Command::delete_global_application_command(&ctx_arc.http, command.id).await
                    {
                        error!("error while deleting global applications command : {e}");
                    }
                }
            }
            Err(e) => error!("error while getting global application commands : {e}"),
        }

        let data = ctx_arc.data.read().await;
        let guild_group = if let Some(guild_group) = data.get::<GuildIdContainer>() {
            guild_group
        } else {
            error!("There was a problem getting the guild id");
            return;
        }
        .lock()
        .await;

        let mut results: Vec<(GuildId, Result<Vec<Command>, serenity::Error>)> = Vec::new();
        for guild in &guild_group.guilds {
            results.push((
                *guild,
                guild
                    .set_application_commands(&ctx_arc.http, |commands| {
                        commands
                            .create_application_command(|c| general::help::register(c))
                            .create_application_command(|c| general::bonjour::register(c))
                            .create_application_command(|c| general::slide::register(c))
                            .create_application_command(|c| general::ping::register(c))
                            .create_application_command(|c| general::nerd::register_chat_input(c))
                            .create_application_command(|c| general::nerd::register_message(c))
                            .create_application_command(|c| general::id::register_user(c))
                            .create_application_command(|c| general::id::register_chat_input(c))
                            .create_application_command(|c| general::roll::register(c))
                            .create_application_command(|c| general::based::register_chat_input(c))
                            .create_application_command(|c| general::based::register_message(c))
                            .create_application_command(|c| general::tg::register(c))
                            .create_application_command(|c| macros::setup::register(c))
                            .create_application_command(|c| macros::setup::register_message(c))
                    })
                    .await,
            ));
        }

        for res in results {
            match res.1 {
                Ok(_) => info!("Guild {} added commands without error", res.0),
                Err(e) => error!("Guild {} had an error adding commands : {e}", res.0),
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                let result: InteractionResponse = match command.data.kind {
                    CommandType::ChatInput => match command.data.name.as_str() {
                        "help" => general::help::run(&ctx, &command).await,
                        "bonjour" => general::bonjour::run(),
                        "slide" => general::slide::run(&ctx, &command).await,
                        "ping" => general::ping::run(&ctx).await,
                        "nerd" => general::nerd::run_chat_input(&command.data.options),
                        "id" => general::id::run_chat_input(&command.data.options),
                        "roll" => general::roll::run_chat_input(&command.data.options),
                        "basé" => general::based::run_chat_input(&command.data.options),
                        "tg" => general::tg::run(&ctx, &command).await,
                        "macro" => macros::setup::run(&ctx, &command).await,
                        _ => InteractionResponse::Message(InteractionMessage::ephemeral(format!("Unkown command ChatInput : {}", command.data.name))),
                    },
                    CommandType::Message => match command.data.name.as_str() {
                        "nerd" => general::nerd::run_message(&ctx, &command).await,
                        "basé" => general::based::run_message(&ctx, &command).await,
                        "macro add" => macros::add::run_message_form(&ctx, &command).await,
                        _ => InteractionResponse::Message(InteractionMessage::ephemeral(format!("Unkown command Message : {}", command.data.name))),
                    },
                    CommandType::User => match command.data.name.as_str() {
                        "id" => general::id::run_user(&ctx, &command).await,
                        _ => InteractionResponse::Message(InteractionMessage::ephemeral(format!("Unkown command User : {}", command.data.name))),
                    },
                    CommandType::Unknown => InteractionResponse::Message(InteractionMessage::ephemeral("Unkown data kind")),
                    _ => InteractionResponse::Message(InteractionMessage::ephemeral("wildcard data kind")),
                };

                match result {
                    InteractionResponse::Message(interaction_message) => {
                        interaction_message.send_from_command(&ctx, &command).await;
                    }
                    InteractionResponse::Modal => todo!(),
                    InteractionResponse::None => (),
                }
            }
            Interaction::ModalSubmit(modal) => {
                let res = match modal.data.custom_id.as_str() {
                    consts::MACRO_ADD_FORM_ID => macros::add::run_message(&ctx, &modal).await,
                    _ => InteractionResponse::Message(InteractionMessage::ephemeral("modal inconnu")),
                };
                if let InteractionResponse::Message(m) = res {
                    m.send_from_modal(&ctx, &modal).await;
                }
            }
            Interaction::Ping(_)
            | Interaction::Autocomplete(_)
            | Interaction::MessageComponent(_) => (),
        }
    }
}

#[group]
#[prefix = "macro"]
#[commands(add, edit, del, show, clear)]
struct Macro;

#[group]
#[commands(basé, bonjour, ping, slide, nerd, id, roll)]
struct General;

#[group]
#[prefix = "pdx"]
#[commands(dd)]
struct Pdx;

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

#[allow(clippy::expect_used, clippy::panic)]
#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let Some(token) = secret_store.get("DISCORD_TOKEN") else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };
    let http = Http::new(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {why:?}"),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("$"))
        .after(after)
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&MACRO_GROUP)
        .group(&PDX_GROUP);

    let static_groups = vec![&GENERAL_GROUP, &MACRO_GROUP, &PDX_GROUP];
    let groups: CommandGroups = CommandGroups {
        groups: {
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
            groups
        },
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Bot {
            is_loop_running: AtomicBool::new(false),
        })
        .await
        .expect("Error creating client");

    let guilds: Vec<GuildId> = if let Some(ids) = secret_store.get("GUILD_ID") {
        ids.split(',')
            .map(|id_str| GuildId(id_str.parse().expect("guild id should be u64")))
            .collect()
    } else {
        return Err(anyhow!("'GUILD_ID' was not found").into());
    };

    let guild_group = GuildGroup { guilds };

    let Some(log_chan_id) = secret_store.get("LOG_CHAN_ID") else {
        return Err(anyhow!("'LOG_CHAN_ID' was not found").into());
    };
    let log_chan = Arc::new(tokio::sync::Mutex::new(ChannelId(
        log_chan_id.parse().expect("LOG_CHAN_ID should be u64"),
    )));

    let Some(uri) = secret_store.get("DATABASE_URI") else {
        return Err(anyhow!("'DATABASE_URI' was not found").into());
    };
    let db_uri = Arc::new(tokio::sync::Mutex::new(DatabaseUri { db_uri: uri }));

    let Some(temp_chan_id) = secret_store.get("TEMP_CHAN") else {
        return Err(anyhow!("'TEMP_CHAN' was not found").into());
    };
    let temp_chan = ChannelId(temp_chan_id.parse().expect("TEMP_CHAN should be u64"));

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<GuildIdContainer>(Arc::new(tokio::sync::Mutex::new(guild_group)));
        data.insert::<LogChanIdContainer>(Arc::clone(&log_chan));
        data.insert::<CommandGroupsContainer>(Arc::new(tokio::sync::Mutex::new(groups)));
        data.insert::<DatabaseUriContainer>(Arc::clone(&db_uri));
        data.insert::<TempChanContainer>(Arc::new(tokio::sync::Mutex::new(temp_chan)));
    }

    Ok(client.into())
}
