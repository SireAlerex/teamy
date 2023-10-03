use super::model::{PdxGame, PdxLinks};
use crate::interaction::Response;
use crate::{db, utils, web_scraper};
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::*;
use serenity::prelude::*;

use super::show;

#[command]
#[description = "Mets à jour les derniers DD et les affiche"]
pub async fn dd(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let links = run_intern(ctx).await?;
    show::show_intern(ctx, &msg.author, msg.channel_id, &links).await
}

async fn run_intern(ctx: &Context) -> Result<PdxLinks, CommandError> {
    let pdx = PdxLinks::db_links(ctx).await?;
    let results = check_links(&pdx).await?;
    update_links(ctx, pdx, results).await
}

// TODO: optimize ?
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
    // TODO: replace with find ?
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

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Response {
    show::show_interaction(ctx, command, run_intern(ctx).await).await
}
