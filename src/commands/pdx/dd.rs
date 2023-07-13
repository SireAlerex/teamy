use super::model::{PdxFollow, PdxGame, PdxLinks};
use crate::{db, utils, web_scraper};
use bson::doc;
use serenity::builder::CreateEmbed;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[description = "Affiche les derniers Dev Diaries de Paradox"]
pub async fn dd(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let pdx = pdx_links(ctx).await?;
    let results = check_links(&pdx).await?;
    let final_pdx = update_links(ctx, pdx, results).await?;
    show(ctx, msg, &final_pdx).await?;

    Ok(())
}

async fn pdx_links(ctx: &Context) -> Result<PdxLinks, CommandError> {
    db::find_filter(ctx, "pdx_links", None)
        .await?
        .ok_or(utils::command_error("no pdx link db"))
}

async fn check_links(pdx: &PdxLinks) -> Result<Vec<(PdxGame, Option<String>)>, CommandError> {
    let mut results: Vec<(PdxGame, Result<Option<String>, web_scraper::ScraperError>)> = Vec::new();
    let mut handles = Vec::new();
    let client = reqwest::Client::default();
    // for each (game, latest_link) return (game, option<link>) which is some if a more recent one exist or none else
    for (game, link) in pdx.all_latest() {
        let client_loop = client.clone();
        handles.push(tokio::spawn(async move {
            let web = web_scraper::pdx_scraper(link, &client_loop).await;
            (game, web)
        }));
    }

    for handle in handles {
        results.push(handle.await?);
    }
    for (_, res) in &results {
        if let Err(e) = res {
            return Err(utils::command_error(format!("link threaded error : {e}")));
        }
    }
    Ok(results
        .into_iter()
        .map(|(game, res)| (game, res.unwrap_or(None)))
        .collect())
}

async fn update_links(
    ctx: &Context,
    pdx: PdxLinks,
    new_links: Vec<(PdxGame, Option<String>)>,
) -> Result<PdxLinks, CommandError> {
    if new_links.iter().all(|(_, o)| o.is_none()) {
        Ok(pdx)
    } else {
        let mut new_pdx = pdx.clone();
        for (game, link) in new_links {
            new_pdx.update(game, link)?;
        }
        db::delete(ctx, "pdx_links", &pdx).await?;
        let _: bson::Bson = db::insert(ctx, "pdx_links", &new_pdx).await?;
        Ok(new_pdx)
    }
}

async fn show(ctx: &Context, msg: &Message, links: &PdxLinks) -> CommandResult {
    let pdx = get_follows(ctx, msg.author.id.to_string()).await?;
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
    // create emebd and send it
    let embed = CreateEmbed::default()
        .title("Paradox Dev Diaries")
        .description("Liens des derniers DD")
        .fields(fields)
        .color(serenity::utils::Color::PURPLE)
        .clone();

    let _: Message = msg
        .channel_id
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
