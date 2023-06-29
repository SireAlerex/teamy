use super::pdx::{PdxFollow, PdxGame, PdxLinks};
use crate::{db, utils, web_scraper};
use bson::doc;
use serenity::builder::CreateEmbed;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::sync::Arc;
use tokio::task::JoinHandle;

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
    let results: Arc<Mutex<Vec<(PdxGame, Result<Option<String>, web_scraper::ScraperError>)>>> =
        Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let mut handles: Vec<JoinHandle<()>> = vec![];
    // for each (game, latest_link) return (game, option<link>) which is some if a more recent one exist or none else
    for (game, link) in pdx.all_latest() {
        let results = Arc::clone(&results);
        let t = tokio::spawn(async move {
            let web = web_scraper::pdx_scraper(&link).await;
            let mut r = results.as_ref().lock().await;
            r.push((game, web));
        });
        handles.push(t);
    }

    for handle in handles {
        handle.await.unwrap();
    }
    let res = results.lock().await.to_vec();
    if res.iter().any(|r| r.1.is_err()) {
        // can unwrap since we found an error
        let first_err = res
            .iter()
            .find(|r| r.1.is_err())
            .unwrap()
            .1
            .clone()
            .unwrap_err();
        Err(utils::command_error(format!(
            "link threaded error : {first_err}"
        )))
    } else {
        let res: Vec<(PdxGame, Option<String>)> =
            res.into_iter().map(|r| (r.0, r.1.unwrap())).collect();
        Ok(res)
    }
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
        for (game, link) in new_links.iter().filter(|(_, o)| o.is_some()) {
            // only options are some so unwrap is safe
            new_pdx.update(game, link.clone().unwrap())?;
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
        .to_owned();

    let _: Message = msg
        .channel_id
        .send_message(&ctx.http, |m| m.set_embed(embed))
        .await?;

    Ok(())
}

async fn get_follows(ctx: &Context, user_id: String) -> Result<PdxFollow, CommandError> {
    let filter = doc! { "user_id": user_id.clone() };
    match db::find_filter::<PdxFollow>(ctx, "pdx_follows", filter).await? {
        Some(pdx) => Ok(pdx),
        None => {
            let pdx = PdxFollow::new(user_id);
            _ = db::insert(ctx, "pdx_follows", &pdx).await?;
            Ok(pdx)
        }
    }
}
