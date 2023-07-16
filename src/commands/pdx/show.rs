use mongodb::bson::doc;
use serenity::builder::CreateEmbed;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandError, CommandResult};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::*;
use serenity::model::user::User;
use serenity::prelude::*;

use super::model::{PdxFollow, PdxGame, PdxLinks};
use crate::db;
use crate::interaction::{InteractionMessage, Response};
use crate::utils;

#[command]
#[description = "Affiche les derniers DD"]
async fn show(ctx: &Context, msg: &Message) -> CommandResult {
    let links = PdxLinks::db_links(ctx).await?;
    show_intern(ctx, &msg.author, msg.channel_id, &links).await
}

pub async fn show_intern(
    ctx: &Context,
    user: &User,
    channel_id: ChannelId,
    links: &PdxLinks,
) -> CommandResult {
    let embed = embed(ctx, user, links).await?;
    show_embed(ctx, embed, channel_id).await?;

    Ok(())
}

pub async fn embed(
    ctx: &Context,
    user: &User,
    links: &PdxLinks,
) -> Result<CreateEmbed, CommandError> {
    let pdx = get_follows(ctx, user.id.to_string()).await?;
    let mut fields: Vec<(String, String, bool)> = Vec::new();
    for game in PdxGame::iterator() {
        if pdx.follows().any(|(g, sub)| (g == game) && sub) {
            let (title, value) = links
                .game_links(game)
                .ok_or(utils::command_error(format!("no links for {game}")))?
                .embed_value();
            fields.push((title, value, true));
        }
    }

    Ok(CreateEmbed::default()
        .title("Paradox Dev Diaries")
        .description("Liens des derniers DD")
        .fields(fields)
        .color(serenity::utils::Color::PURPLE)
        .clone())
}

pub async fn show_embed(ctx: &Context, embed: CreateEmbed, channel_id: ChannelId) -> CommandResult {
    let _: Message = channel_id
        .send_message(&ctx.http, |m| m.set_embed(embed))
        .await?;

    Ok(())
}

async fn get_follows(ctx: &Context, user_id: String) -> Result<PdxFollow, CommandError> {
    let filter = doc! { "user_id": user_id.clone() };
    if let Some(pdx) = db::find_filter::<PdxFollow>(ctx, "pdx_follows", filter).await? {
        Ok(pdx)
    } else {
        let pdx = PdxFollow::new(user_id);
        _ = db::insert(ctx, "pdx_follows", &pdx).await?;
        Ok(pdx)
    }
}

pub async fn show_interaction(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    links_res: Result<PdxLinks, CommandError>,
) -> Response {
    match links_res {
        Ok(links) => {
            let (content, embed) = match embed(ctx, &command.user, &links).await {
                Ok(e) => (String::new(), Some(e)),
                Err(err) => (err.to_string(), None),
            };

            Response::Message(InteractionMessage::new(content, true, embed))
        }
        Err(e) => Response::Message(InteractionMessage::with_content(format!(
            "erreur pour accéder aux liens: {e}"
        ))),
    }
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Response {
    show_interaction(ctx, command, PdxLinks::db_links(ctx).await).await
}
