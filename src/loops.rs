use chrono::prelude::*;
use rand::{seq::IteratorRandom, thread_rng};
use serenity::model::prelude::Activity;
use serenity::{model::prelude::ChannelId, prelude::Context};
use std::sync::Arc;
use tracing::error;

use crate::consts;
use crate::utils;

pub async fn log_system_load(ctx: Arc<Context>) {
    let time = Local::now().to_rfc2822();
    let cpu_load = sys_info::loadavg().unwrap();
    let mem_use = sys_info::mem_info().unwrap();
    let latency = utils::runner_latency(Arc::clone(&ctx)).await;

    let message = ChannelId(1098593646569340968)
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
