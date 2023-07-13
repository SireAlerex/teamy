use serenity::framework::standard::macros::group;

pub mod dd;
pub mod model;
pub mod setup;
pub mod show;

use dd::DD_COMMAND;
use show::SHOW_COMMAND;

#[group]
#[prefix = "pdx"]
#[commands(dd, show)]
struct Pdx;
