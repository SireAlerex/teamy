mod commands;
pub mod consts;
mod loops;
mod message;
pub mod utils;

use anyhow::anyhow;
use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::interaction::Interaction;
use serenity::model::prelude::GuildId;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

use crate::commands::{bonjour::*, id::*, nerd::*, ping::*, roll::*, slide::*};

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<tokio::sync::Mutex<ShardManager>>;
}

struct GuildIdContainer;

impl TypeMapKey for GuildIdContainer {
    type Value = Arc<tokio::sync::Mutex<GuildId>>;
}

struct InteractionMessage {
    content: String,
    ephemeral: bool,
}

enum InteractionResponse {
    Message(InteractionMessage),
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
        match utils::first_letter(&msg.content) {
            '$' => (), // do nothing if command
            _ => message::handle_reaction(msg, ctx).await,
        }
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

        let data = ctx.data.read().await;
        let test_guild_id = match data.get::<GuildIdContainer>() {
            Some(id) => id,
            None => {
                error!("There was a problem getting the test guild id");
                return;
            }
        }
        .lock()
        .await;

        let commands = GuildId::set_application_commands(&test_guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::bonjour::register(command))
                .create_application_command(|command| commands::slide::register(command))
                .create_application_command(|command| commands::ping::register(command))
                .create_application_command(|command| commands::nerd::register_chat_input(command))
                .create_application_command(|command| commands::nerd::register_message(command))
                .create_application_command(|command| commands::id::register_user(command))
                .create_application_command(|command| commands::id::register_chat_input(command))
                .create_application_command(|command| commands::roll::register(command))
        })
        .await;

        info!("I have the following commands : {:#?}", commands);
        if commands.is_ok() {
            info!("Commands Ok !");
        } else {
            info!("Commands Error !");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let result: InteractionResponse = match command.data.kind {
                CommandType::ChatInput => match command.data.name.as_str() {
                    "bonjour" => InteractionResponse::Message(InteractionMessage {
                        content: commands::bonjour::run(),
                        ephemeral: false,
                    }),
                    "slide" => InteractionResponse::Message(InteractionMessage {
                        content: commands::slide::run(&ctx, &command).await,
                        ephemeral: true,
                    }),
                    "ping" => InteractionResponse::Message(InteractionMessage {
                        content: commands::ping::run(&ctx).await,
                        ephemeral: false,
                    }),
                    "nerd" => InteractionResponse::Message(InteractionMessage {
                        content: commands::nerd::run_chat_input(&command.data.options),
                        ephemeral: false,
                    }),
                    "id" => InteractionResponse::Message(InteractionMessage {
                        content: commands::id::run_chat_input(&command.data.options),
                        ephemeral: false,
                    }),
                    "roll" => InteractionResponse::Message(InteractionMessage {
                        content: commands::roll::run_chat_input(&command.data.options),
                        ephemeral: false,
                    }),
                    _ => InteractionResponse::Message(InteractionMessage {
                        content: format!("Unkown command ChatInput : {}", command.data.name),
                        ephemeral: true,
                    }),
                },
                CommandType::Message => match command.data.name.as_str() {
                    "nerd" => InteractionResponse::Message(InteractionMessage {
                        content: commands::nerd::run_message(&ctx, &command).await,
                        ephemeral: false,
                    }),
                    _ => InteractionResponse::Message(InteractionMessage {
                        content: format!("Unkown command Message : {}", command.data.name),
                        ephemeral: true,
                    }),
                },
                CommandType::User => match command.data.name.as_str() {
                    "id" => InteractionResponse::Message(InteractionMessage {
                        content: commands::id::run_user(&ctx, &command).await,
                        ephemeral: false,
                    }),
                    _ => InteractionResponse::Message(InteractionMessage {
                        content: format!("Unkown command User : {}", command.data.name),
                        ephemeral: true,
                    }),
                },
                _ => InteractionResponse::Message(InteractionMessage {
                    content: "Unkown data kind".to_owned(),
                    ephemeral: true,
                }),
            };

            match result {
                InteractionResponse::Message(interaction) => {
                    utils::interaction_response_message(
                        &ctx,
                        &command,
                        interaction.content,
                        interaction.ephemeral,
                    )
                    .await
                }
            }
        };
    }
}

#[group]
#[commands(bonjour, ping, slide, nerd, id, roll)]
struct General;

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
        .group(&GENERAL_GROUP);

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

    let test_guild_id = if let Some(id) = secret_store.get("TEST_GUILD_ID") {
        id
    } else {
        return Err(anyhow!("'TEST_GUILD_ID' was not found").into());
    };
    let test_guild_id = Arc::new(tokio::sync::Mutex::new(GuildId(
        test_guild_id.parse().expect("GUILD_ID should be u64"),
    )));

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<GuildIdContainer>(Arc::clone(&test_guild_id));
    }

    Ok(client.into())
}
