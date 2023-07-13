use serenity::framework::standard::macros::group;

pub mod add;
pub mod clear;
pub mod del;
pub mod edit;
pub mod r#macro;
pub mod setup;
pub mod show;

use add::ADD_COMMAND;
use clear::CLEAR_COMMAND;
use del::DEL_COMMAND;
use edit::EDIT_COMMAND;
use show::SHOW_COMMAND;

#[group]
#[prefix = "macro"]
#[commands(add, edit, del, show, clear)]
struct Macro;
