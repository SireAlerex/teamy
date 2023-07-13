use serenity::framework::standard::macros::group;

pub mod based;
pub mod bonjour;
pub mod help;
pub mod id;
pub mod nerd;
pub mod ping;
pub mod roll;
pub mod slide;
pub mod tg;

use based::BASÉ_COMMAND;
use bonjour::BONJOUR_COMMAND;
use id::ID_COMMAND;
use nerd::NERD_COMMAND;
use ping::PING_COMMAND;
use roll::ROLL_COMMAND;
use slide::SLIDE_COMMAND;

#[group]
#[commands(basé, bonjour, ping, slide, nerd, id, roll)]
struct General;
