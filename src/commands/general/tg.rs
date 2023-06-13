use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType, interaction::application_command::ApplicationCommandInteraction,
    },
    prelude::Context,
};

use crate::{db, utils};
use crate::{InteractionMessage, InteractionResponse};

async fn toggle_mute<
    T: core::fmt::Debug
        + serde::de::DeserializeOwned
        + serde::Serialize
        + std::marker::Unpin
        + std::marker::Send
        + std::marker::Sync,
>(
    ctx: &Context,
    collection: &str,
    object: T,
) -> Result<bool, mongodb::error::Error> {
    match db::is_object_in_coll(ctx, collection, &object).await? {
        true => {
            db::delete(ctx, collection, &object).await?;
            Ok(true)
        }
        false => {
            db::insert(ctx, collection, &object).await?;
            Ok(false)
        }
    }
}

fn check_muted_str(check: Result<bool, mongodb::error::Error>) -> String {
    match check {
        Ok(c) => {
            if c {
                String::from(":mute:")
            } else {
                String::from(":loud_sound:")
            }
        }
        Err(e) => format!(":x: (Erreur : {e}"),
    }
}

async fn mute_status(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let user_muted = db::is_object_in_coll(
        ctx,
        "mute_users",
        &db::User::builder(command.user.id.to_string()),
    )
    .await;
    let chan_muted = db::is_object_in_coll(
        ctx,
        "mute_chans",
        &db::Chan::builder(command.channel_id.to_string()),
    )
    .await;
    let guild_muted = if command.guild_id.is_some() {
        db::is_object_in_coll(
            ctx,
            "mute_guilds",
            &db::Guild::builder(command.guild_id.unwrap().to_string()),
        )
        .await
    } else {
        Ok(false)
    };
    format!(
        "Utilisateur muted : {}\nChan muted : {}\nServeur muted : {}",
        check_muted_str(user_muted),
        check_muted_str(chan_muted),
        check_muted_str(guild_muted)
    )
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> InteractionResponse {
    let res = match command.data.options[0].name.as_str() {
        "moi" => {
            match toggle_mute(
                ctx,
                "mute_users",
                db::User::builder(command.user.id.to_string()),
            )
            .await
            {
                Ok(b) => {
                    if b {
                        String::from("Le bot répondra à vos messages")
                    } else {
                        String::from("Le bot ne répondra plus à vos messages")
                    }
                }
                Err(e) => format!("Erreur : {e}"),
            }
        }
        "chan" => {
            if utils::admin_command(command) {
                match toggle_mute(
                    ctx,
                    "mute_chans",
                    db::Chan::builder(command.channel_id.to_string()),
                )
                .await
                {
                    Ok(b) => {
                        if b {
                            String::from("Le bot répondra aux messages de ce chan")
                        } else {
                            String::from("Le bot ne répondra plus aux messages de ce chan")
                        }
                    }
                    Err(e) => format!("Erreur : {e}"),
                }
            } else {
                String::from("Vous devez être admin pour utiliser cette commande")
            }
        }
        "serv" => {
            if command.guild_id.is_some() && utils::admin_command(command) {
                match toggle_mute(
                    ctx,
                    "mute_guilds",
                    db::Guild::builder(command.guild_id.unwrap().to_string()),
                )
                .await
                {
                    Ok(b) => {
                        if b {
                            String::from("Le bot répondra aux messages de ce serveur")
                        } else {
                            String::from("Le bot ne répondra plus aux messages de ce serveur")
                        }
                    }
                    Err(e) => format!("Erreur : {e}"),
                }
            } else {
                String::from("Vous devez être admin pour utiliser cette commande")
            }
        }
        "status" => mute_status(ctx, command).await,
        _ => String::from("unexpected subcommand"),
    };
    InteractionResponse::Message(InteractionMessage {
        content: res,
        ephemeral: true,
        embed: None,
    })
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("tg")
        .description("Toggle les réponses du bot")
        .create_option(|option| {
            option
                .name("moi")
                .description("Toggle les réponses du bot à vos messages")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("chan")
                .description("Toggle les réponses du bot aux messages du chan")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("serv")
                .description("Toggle les réponses du bot aux messages du serveur")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("status")
                .description("Affiche les informations sur les réponses du bot ici")
                .kind(CommandOptionType::SubCommand)
        })
}
