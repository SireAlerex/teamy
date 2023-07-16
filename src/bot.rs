use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serenity::model::application::command::Command;
use serenity::model::application::interaction::Interaction;
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::*;
use serenity::{async_trait, prelude::*};
use tracing::{error, info};

use crate::interaction::{InteractionMessage, InteractionResponse};
use crate::{commands, interaction, loops, GuildIdContainer};
use commands::{general, macros, pdx};

pub struct Bot {
    pub is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Bot {
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
