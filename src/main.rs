pub mod command;
mod commands;
pub mod consts;
pub mod db;
mod loops;
mod message;
pub mod utils;

use anyhow::anyhow;
use serenity::async_trait;
use serenity::builder::CreateEmbed;
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

use crate::command::{CommandGroupInfo, CommandGroups, CommandGroupsContainer, CommandInfo};
use crate::commands::general;
use crate::commands::general::{based::*, bonjour::*, id::*, nerd::*, ping::*, roll::*, slide::*};
use crate::commands::macros;
use crate::commands::macros::{add::*, clear::*, del::*, edit::*, show::*};

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

pub enum InteractionResponse {
    Message(InteractionMessage),
}

pub struct InteractionMessage {
    content: String,
    ephemeral: bool,
    embed: Option<CreateEmbed>,
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
            '$' => String::new(), // do nothing if command
            '!' => commands::macros::r#macro::handle_macro(&ctx, &msg).await,
            _ => message::handle_reaction(&ctx, &msg).await,
        };
        utils::say_or_error(&ctx, msg.channel_id, &content).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let ctx: Arc<Context> = Arc::new(ctx);
        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ctx1 = Arc::clone(&ctx);

            tokio::spawn(async move {
                loop {
                    loops::status_loop(Arc::clone(&ctx1)).await;
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            });

            let ctx2 = Arc::clone(&ctx);

            tokio::spawn(async move {
                loop {
                    loops::log_system_load(Arc::clone(&ctx2)).await;
                    tokio::time::sleep(Duration::from_secs(300)).await;
                }
            });
        }

        // clean global commands
        for command in Command::get_global_application_commands(&ctx.http)
            .await
            .unwrap()
        {
            let _ = Command::delete_global_application_command(&ctx.http, command.id).await;
        }

        let data = ctx.data.read().await;
        let guild_group = match data.get::<GuildIdContainer>() {
            Some(guild_group) => guild_group,
            None => {
                error!("There was a problem getting the guild id");
                return;
            }
        }
        .lock()
        .await;

        let mut results: Vec<(GuildId, Result<Vec<Command>, serenity::Error>)> = Vec::new();
        for guild in &guild_group.guilds {
            results.push((
                *guild,
                guild
                    .set_application_commands(&ctx.http, |commands| {
                        commands
                            .create_application_command(|command| general::help::register(command))
                            .create_application_command(|command| {
                                general::bonjour::register(command)
                            })
                            .create_application_command(|command| general::slide::register(command))
                            .create_application_command(|command| general::ping::register(command))
                            .create_application_command(|command| {
                                general::nerd::register_chat_input(command)
                            })
                            .create_application_command(|command| {
                                general::nerd::register_message(command)
                            })
                            .create_application_command(|command| {
                                general::id::register_user(command)
                            })
                            .create_application_command(|command| {
                                general::id::register_chat_input(command)
                            })
                            .create_application_command(|command| general::roll::register(command))
                            .create_application_command(|command| {
                                general::based::register_chat_input(command)
                            })
                            .create_application_command(|command| {
                                general::based::register_message(command)
                            })
                            .create_application_command(|command| general::tg::register(command))
                            .create_application_command(|command| macros::setup::register(command))
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
        if let Interaction::ApplicationCommand(command) = interaction {
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
                    _ => InteractionResponse::Message(InteractionMessage {
                        content: format!("Unkown command ChatInput : {}", command.data.name),
                        ephemeral: true,
                        embed: None,
                    }),
                },
                CommandType::Message => match command.data.name.as_str() {
                    "nerd" => general::nerd::run_message(&ctx, &command).await,
                    "basé" => general::based::run_message(&ctx, &command).await,
                    _ => InteractionResponse::Message(InteractionMessage {
                        content: format!("Unkown command Message : {}", command.data.name),
                        ephemeral: true,
                        embed: None,
                    }),
                },
                CommandType::User => match command.data.name.as_str() {
                    "id" => general::id::run_user(&ctx, &command).await,
                    _ => InteractionResponse::Message(InteractionMessage {
                        content: format!("Unkown command User : {}", command.data.name),
                        ephemeral: true,
                        embed: None,
                    }),
                },
                _ => InteractionResponse::Message(InteractionMessage {
                    content: "Unkown data kind".to_owned(),
                    ephemeral: true,
                    embed: None,
                }),
            };

            match result {
                InteractionResponse::Message(interaction) => {
                    utils::interaction_response_message(
                        &ctx,
                        &command,
                        interaction.content,
                        interaction.ephemeral,
                        interaction.embed,
                    )
                    .await
                }
            }
        };
    }
}

#[group]
#[prefix = "macro"]
#[commands(add, edit, del, show, clear)]
struct Macro;

#[group]
#[commands(basé, bonjour, ping, slide, nerd, id, roll)]
struct General;

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
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;
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
            .await
        }
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };
    let http = Http::new(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("$"))
        .after(after)
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&MACRO_GROUP);

    let static_groups = vec![&GENERAL_GROUP, &MACRO_GROUP];
    let mut groups: Vec<CommandGroupInfo> = Vec::default();
    for group in static_groups {
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
    let groups: CommandGroups = CommandGroups { groups };

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

    let log_chan_id = if let Some(id) = secret_store.get("LOG_CHAN_ID") {
        id
    } else {
        return Err(anyhow!("'LOG_CHAN_ID' was not found").into());
    };
    let log_chan_id = Arc::new(tokio::sync::Mutex::new(ChannelId(
        log_chan_id.parse().expect("LOG_CHAN_ID should be u64"),
    )));

    let db_uri = if let Some(uri) = secret_store.get("DATABASE_URI") {
        uri
    } else {
        return Err(anyhow!("'DATABASE_URI' was not found").into());
    };
    let db_uri = Arc::new(tokio::sync::Mutex::new(DatabaseUri { db_uri }));

    let temp_chan = if let Some(channel_id) = secret_store.get("TEMP_CHAN") {
        channel_id
    } else {
        return Err(anyhow!("'TEMP_CHAN' was not found").into());
    };
    let temp_chan = ChannelId(temp_chan.parse().expect("TEMP_CHAN should be u64"));

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<GuildIdContainer>(Arc::new(tokio::sync::Mutex::new(guild_group)));
        data.insert::<LogChanIdContainer>(Arc::clone(&log_chan_id));
        data.insert::<CommandGroupsContainer>(Arc::new(tokio::sync::Mutex::new(groups)));
        data.insert::<DatabaseUriContainer>(Arc::clone(&db_uri));
        data.insert::<TempChanContainer>(Arc::new(tokio::sync::Mutex::new(temp_chan)));
    }

    Ok(client.into())
}
