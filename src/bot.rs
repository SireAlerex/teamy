use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use poise::{serenity_prelude, Framework};
use serenity::model::application::Command;
use serenity::{async_trait, prelude::*};
use tracing::{error, info};

use crate::commands::{general::roll, PoiseError};
use crate::message::handle_reaction;
use crate::{loops, Data, GuildIdContainer};

pub struct Bot {
    pub is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: serenity_prelude::Ready) {
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
        match Command::get_global_commands(&ctx_arc.http).await {
            Ok(commands) => {
                for command in commands {
                    if let Err(e) = Command::delete_global_command(&ctx_arc.http, command.id).await
                    {
                        error!("error while deleting global applications command : {e}");
                    }
                }
            }
            Err(e) => error!("error while getting global application commands : {e}"),
        }
    }
}

pub async fn register_guild(
    ctx: &serenity_prelude::Context,
    framework: &Framework<Data, Box<dyn Error + Send + Sync>>,
) {
    // getting guilds id to add commands to
    let data = ctx.data.read().await;
    let Some(guild_group) = data.get::<GuildIdContainer>() else {
        error!("There was a problem getting the guild id");
        return;
    };

    // gettings bot commands
    let commands = &framework.options().commands;
    let create_commands = poise::builtins::create_application_commands(commands);

    let mut results: Vec<(
        serenity_prelude::GuildId,
        Result<Vec<Command>, serenity::Error>,
    )> = Vec::new();
    for guild in &guild_group.0 {
        let x = guild.set_commands(&ctx.http, create_commands.clone()).await;
        results.push((*guild, x));
    }

    for res in results {
        match res.1 {
            Ok(_) => info!("Guild {} added commands without error", res.0),
            Err(e) => error!("Guild {} had an error adding commands : {e}", res.0),
        }
    }
}

pub fn apply_desc_from(commands: &mut [poise::Command<Data, PoiseError>], locale: &str) {
    for command in commands {
        if let Some(desc) = command.description_localizations.get(locale) {
            if command.description.is_none() {
                command.description = Some(desc.to_string());
            }
        }
    }
}

pub async fn event_handler(
    ctx: &serenity_prelude::Context,
    event: &serenity_prelude::FullEvent,
    _: poise::FrameworkContext<'_, Data, PoiseError>,
    _: &Data,
) -> Result<(), PoiseError> {
    match event {
        serenity_prelude::FullEvent::Ready { data_about_bot, .. } => {
            info!("Logged in as {}", data_about_bot.user.name);
        }
        serenity_prelude::FullEvent::Message { new_message } => {
            if msg_check(new_message) {
                let res = match handle_reaction(ctx, new_message).await {
                    Ok(s) => s,
                    Err(e) => {
                        // not important, just log and return
                        error!("message checked err: {e}");
                        return Ok(());
                    }
                };
                if let Some(content) = res {
                    let _ = new_message.channel_id.say(&ctx.http, content).await?;
                }
            } else if new_message.content.starts_with("$roll") {
                let rest = new_message.content[5..].to_string();
                if let Err(e) = roll::roll_intern_str(ctx, &new_message.channel_id, rest).await {
                    error!("message $roll err: {e}");
                    return Err(e);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn msg_check(msg: &serenity_prelude::Message) -> bool {
    !msg.content.is_empty() && !msg.content.starts_with('$') && !msg.author.bot
}
