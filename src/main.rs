pub mod command_info;
mod commands;
#[allow(clippy::impl_trait_in_params)]
pub mod db;
pub mod interaction;
mod loops;
mod message;
mod secrets;
pub mod utils;
pub mod web_scraper;

use anyhow::anyhow;
use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::macros::{help, hook};
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
use tokio::sync::Mutex;
use tracing::{error, info};

use command_info::{CommandGroupInfo, CommandGroups, CommandGroupsContainer, CommandInfo};
use commands::general;
use commands::general::GENERAL_GROUP;
use commands::macros;
use commands::macros::MACRO_GROUP;
use commands::pdx;
use commands::pdx::PDX_GROUP;
use interaction::{InteractionMessage, InteractionResponse};

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct GuildGroup(Vec<GuildId>);

struct GuildIdContainer;

impl TypeMapKey for GuildIdContainer {
    type Value = Arc<GuildGroup>;
}

struct LogChanIdContainer;

impl TypeMapKey for LogChanIdContainer {
    type Value = Arc<ChannelId>;
}

struct DatabaseUri(String);

struct DatabaseUriContainer;

impl TypeMapKey for DatabaseUriContainer {
    type Value = Arc<DatabaseUri>;
}

struct TempChanContainer;

impl TypeMapKey for TempChanContainer {
    type Value = Arc<ChannelId>;
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
        let Some(guild_group) = data.get::<GuildIdContainer>() else {
            error!("There was a problem getting the guild id");
            return;
        };

        let mut results: Vec<(GuildId, Result<Vec<Command>, serenity::Error>)> = Vec::new();
        for guild in &guild_group.0 {
            results.push((
                *guild,
                guild
                    .set_application_commands(&ctx_arc.http, |commands| {
                        commands
                            .create_application_command(general::help::register)
                            .create_application_command(general::bonjour::register)
                            .create_application_command(general::slide::register)
                            .create_application_command(general::ping::register)
                            .create_application_command(general::nerd::register_chat_input)
                            .create_application_command(general::nerd::register_message)
                            .create_application_command(general::id::register_user)
                            .create_application_command(general::id::register_chat_input)
                            .create_application_command(general::roll::register)
                            .create_application_command(general::based::register_chat_input)
                            .create_application_command(general::based::register_message)
                            .create_application_command(general::tg::register)
                            .create_application_command(macros::setup::register)
                            .create_application_command(macros::setup::register_message)
                            .create_application_command(pdx::setup::register)
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
                let result: InteractionResponse =
                    match command.data.kind {
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
                            "pdx" => pdx::setup::run(&ctx, &command).await,
                            _ => InteractionResponse::Message(InteractionMessage::ephemeral(
                                format!("Unkown command ChatInput : {}", command.data.name),
                            )),
                        },
                        CommandType::Message => match command.data.name.as_str() {
                            "nerd" => general::nerd::run_message(&ctx, &command).await,
                            "basé" => general::based::run_message(&ctx, &command).await,
                            "macro add" => macros::add::run_message_form(&ctx, &command).await,
                            _ => InteractionResponse::Message(InteractionMessage::ephemeral(
                                format!("Unkown command Message : {}", command.data.name),
                            )),
                        },
                        CommandType::User => match command.data.name.as_str() {
                            "id" => general::id::run_user(&ctx, &command).await,
                            _ => InteractionResponse::Message(InteractionMessage::ephemeral(
                                format!("Unkown command User : {}", command.data.name),
                            )),
                        },
                        CommandType::Unknown => InteractionResponse::Message(
                            InteractionMessage::ephemeral("Unkown data kind"),
                        ),
                        _ => InteractionResponse::Message(InteractionMessage::ephemeral(
                            "wildcard data kind",
                        )),
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
                    interaction::MACRO_ADD_FORM_ID => macros::add::run_message(&ctx, &modal).await,
                    _ => {
                        InteractionResponse::Message(InteractionMessage::ephemeral("modal inconnu"))
                    }
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

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in Secrets.toml
    let token = secrets::get(&secret_store, "DISCORD_TOKEN")?;
    let http = Http::new(&token);

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

    let guild_group = GuildGroup(secrets::parse_objects::<u64, GuildId>(
        &secret_store,
        "GUILD_ID",
    )?);
    let log_chan = ChannelId(secrets::parse(&secret_store, "LOG_CHAN_ID")?);
    let db_uri = DatabaseUri(secrets::get(&secret_store, "DATABASE_URI")?);
    let temp_chan = ChannelId(secrets::parse(&secret_store, "TEMP_CHAN")?);

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<GuildIdContainer>(Arc::new(guild_group));
        data.insert::<LogChanIdContainer>(Arc::new(log_chan));
        data.insert::<CommandGroupsContainer>(Arc::new(groups));
        data.insert::<DatabaseUriContainer>(Arc::new(db_uri));
        data.insert::<TempChanContainer>(Arc::new(temp_chan));
    }

    Ok(client.into())
}
