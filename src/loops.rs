use chrono::prelude::*;
use rand::{seq::IteratorRandom, thread_rng};
use serenity::all::ActivityData;
use serenity::all::CreateEmbed;
use serenity::all::CreateMessage;
use serenity::prelude::Context;
use std::sync::Arc;
use tracing::error;

use crate::utils;
use crate::LogChanIdContainer;

pub async fn log_system_load(ctx: Arc<Context>) {
    let time = Local::now().to_rfc2822();
    let cpu_load = match sys_info::loadavg() {
        Ok(load_avg) => format!("{:.2}%", load_avg.one * 10.0_f64),
        Err(e) => format!("error while getting load average : {e}"),
    };
    let mem_use = match sys_info::mem_info() {
        Ok(mem_info) => format!(
            "{:.1}% ({:.2} MB Free out of {:.2} MB)",
            (((mem_info.total as f64 / 1024.0_f64) - (mem_info.free as f64 / 1024.0_f64))
                / (mem_info.total as f64 / 1024.0_f64))
                * 100.0_f64,
            mem_info.free as f64 / 1024.0_f64,
            mem_info.total as f64 / 1024.0_f64
        ),
        Err(e) => format!("error while getting memory info : {e}"),
    };
    let latency = match utils::RunnerInfo::info(Arc::clone(&ctx)).await {
        Ok(runner) => runner.latency,
        Err(err) => {
            error!("There was a problem getting runner info of shard: {err}");
            return;
        }
    };

    let data = ctx.data.read().await;
    let Some(log_chan_id) = data.get::<LogChanIdContainer>() else {
        error!("There was a problem getting the log chan id");
        return;
    };

    let message = log_chan_id
        .send_message(
            &ctx,
            CreateMessage::new().embed(
                CreateEmbed::new()
                    .title("System Resource Load")
                    .field("Time", time, false)
                    .field("CPU Load Average", cpu_load, false)
                    .field("Memory Usage", mem_use, false)
                    .field("Latency", format!("{latency:?}"), false),
            ),
        )
        .await;
    if let Err(why) = message {
        error!("Error sending message: {why:?}");
    };
}

pub fn status_loop(ctx: &Arc<Context>) {
    let game = [
        "LoL avec les boys",
        "Deep Rock Galactic avec les boys",
        "Pathfinder avec les boys",
        "Minecraft avec les boys",
        "Civ6 avec les boys",
        "être raciste",
        "manger son caca",
        "[STRENG GEHEIM]",
    ]
    .iter()
    .choose(&mut thread_rng())
    .unwrap_or(&"no game :(");
    ctx.shard.set_activity(Some(ActivityData::playing(*game)));
}
