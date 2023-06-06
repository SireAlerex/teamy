use chrono::prelude::*;
use rand::{seq::IteratorRandom, thread_rng};
use serenity::model::prelude::Activity;
use serenity::prelude::Context;
use std::sync::Arc;
use tracing::error;

use crate::utils;
use crate::{consts, LogChanIdContainer};

pub async fn log_system_load(ctx: Arc<Context>) {
    let time = Local::now().to_rfc2822();
    let cpu_load = sys_info::loadavg().unwrap();
    let mem_use = sys_info::mem_info().unwrap();
    let latency = utils::runner_latency(Arc::clone(&ctx)).await;

    let data = ctx.data.read().await;
    let log_chan_id = match data.get::<LogChanIdContainer>() {
        Some(id) => id,
        None => {
            error!("There was a problem getting the log chan id");
            return;
        }
    }
    .lock()
    .await;

    let message = log_chan_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("System Resource Load")
                    .field("Time", time, false)
                    .field(
                        "CPU Load Average",
                        format!("{:.2}%", cpu_load.one * 10.0),
                        false,
                    )
                    .field(
                        "Memory Usage",
                        format!(
                            "{:.1}% ({:.2} MB Free out of {:.2} MB)",
                            (((mem_use.total as f32 / 1024.0) - (mem_use.free as f32 / 1024.0))
                                / (mem_use.total as f32 / 1024.0))
                                * 100.0,
                            mem_use.free as f32 / 1024.0,
                            mem_use.total as f32 / 1024.0
                        ),
                        false,
                    )
                    .field("Latency", format!("{:?}", latency), false)
            })
        })
        .await;
    if let Err(why) = message {
        error!("Error sending message: {:?}", why);
    };
}

pub async fn status_loop(ctx: Arc<Context>) {
    let game = *consts::GAME_POOL.iter().choose(&mut thread_rng()).unwrap();
    ctx.shard.set_activity(Some(Activity::playing(game)));
}
