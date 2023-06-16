use crate::{utils, InteractionMessage, InteractionResponse};
use itertools::Itertools;
use rand::Rng;
use serenity::builder::CreateApplicationCommand;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandError, CommandResult};
use serenity::model::application::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};
use serenity::model::prelude::{ChannelId, Message};
use serenity::prelude::Context;
use std::cmp::Ordering;
use std::str::FromStr;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DropKeep {
    DL,
    DH,
    KL,
    KH,
    None,
}

impl FromStr for DropKeep {
    type Err = Box<dyn std::error::Error + Send + Sync>;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match s {
            "d" | "dl" => Ok(DropKeep::DL),
            "dh" => Ok(DropKeep::DH),
            "k" | "kh" => Ok(DropKeep::KH),
            "kl" => Ok(DropKeep::KL),
            _ => Ok(DropKeep::None),
        }
    }
}

impl std::fmt::Display for DropKeep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let s = match self {
            DropKeep::DL => "d",
            DropKeep::DH => "dh",
            DropKeep::KH => "k",
            DropKeep::KL => "kl",
            DropKeep::None => "",
        };
        write!(f, "{s}")
    }
}

impl DropKeep {
    pub fn is_some(self) -> bool {
        self != DropKeep::None
    }
}

#[derive(Debug)]
pub struct Roll {
    number: i64,
    size: i64,
    modifier: i64,
    dk: DropKeep,
    dk_val: Option<usize>,
}

impl Roll {
    pub fn builder(
        number: i64,
        size: i64,
        modifier: i64,
        dk: DropKeep,
        dk_val: Option<usize>,
    ) -> Roll {
        Roll {
            number,
            size,
            modifier,
            dk,
            dk_val,
        }
    }

    /// (number, size, modifier, dk, dk_val)
    pub fn values(&self) -> (i64, i64, i64, DropKeep, Option<usize>) {
        (self.number, self.size, self.modifier, self.dk, self.dk_val)
    }
}

#[command]
#[aliases("r")]
#[description = "Lancer de dés"]
#[usage = "<nombre de dés>d<taille des dés>+<modificateur><drop/keep><valeur du drop/keep>"]
#[example = "2d6+3"]
#[example = "3d4-1"]
#[example = "d8"]
#[example = "4d6k3 (équivalent à 4d6kh3)"]
#[example = "4d6d1 (équivalent à 4d6dl1)"]
#[example = "2d20dh1"]
#[example = "2d20kl1"]
pub async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let _ = roll_intern(ctx, msg.channel_id, args).await?;
    Ok(())
}

pub async fn roll_intern(
    ctx: &Context,
    channel_id: ChannelId,
    args: Args,
) -> Result<Message, CommandError> {
    let roll = regex_roll(args.message())?;
    let msg = channel_id.say(&ctx.http, run(roll)).await?;

    Ok(msg)
}

fn regex_roll(roll: impl ToString) -> Result<Roll, CommandError> {
    let re = regex::Regex::new(
        r"(?P<number>\d+)?d(?P<size>\d+)(?P<modifier>\+\d+|-\d+)?(?P<dk>kh|kl|k|dh|dl|d)?(?P<dk_val>\d+)?",
    )?;
    let s = &roll.to_string();
    let caps = match re.captures(s) {
        Some(caps) => caps,
        None => return Err(utils::command_error("erreur captures regex")),
    };

    let number = match caps.name("number") {
        Some(n) => n.as_str(),
        None => "1",
    }
    .parse::<i64>()?;
    if !(1..=100).contains(&number) {
        return Err(utils::command_error(
            "le nombre de dés doit appartenir à [1; 200]",
        ));
    }
    let size = match caps.name("size") {
        Some(m) => m.as_str().parse::<i64>()?,
        None => return Err(utils::command_error("erreur pas de taille de dé")),
    };
    if size <= 1 {
        return Err(utils::command_error(
            "la taille du dé doit être supérieure strictement à 1",
        ));
    }
    let modifier = match caps.name("modifier") {
        Some(n) => n.as_str(),
        None => "0",
    }
    .parse::<i64>()?;
    let dk: DropKeep = DropKeep::from_str(match caps.name("dk") {
        Some(m) => m.as_str(),
        None => "",
    })?;
    let dk_val = match caps.name("dk_val") {
        Some(m) => Some(m.as_str().parse::<usize>()?),
        None => None,
    };
    if dk.is_some() && dk_val.is_none() {
        return Err(utils::command_error("valeur attendu après le drop/keep"));
    }
    let roll = Roll::builder(number, size, modifier, dk, dk_val);
    Ok(roll)
}

pub fn run(roll: Roll) -> String {
    let (n, size, modifier, dk, dk_val) = roll.values();
    let start = start_message(&roll);
    let mut rolls: Vec<i64> = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..n {
        rolls.push(rng.gen_range(1..=size));
    }

    let initial_roll = rolls_str(&rolls, modifier);

    match dk {
        DropKeep::DL => {
            dl_str(rolls, n, modifier, dk_val.unwrap(), initial_roll, start)
        }
        DropKeep::KH => {
            let val = (n as usize).checked_sub(dk_val.unwrap());
            match val {
                Some(x) => dl_str(rolls, n, modifier, x, initial_roll, start),
                None => "la valeur de drop/keep doit être inférieure ou égale au nombre de dé".to_string()
            }
        }
        DropKeep::DH => {
            dh_str(rolls, n, modifier, dk_val.unwrap(), initial_roll, start)
        }
        DropKeep::KL => {
            let val = (n as usize).checked_sub(dk_val.unwrap());
            match val {
                Some(x) => dh_str(rolls, n, modifier, x, initial_roll, start),
                None => "la valeur de drop/keep doit être inférieure ou égale au nombre de dé".to_string()
            }
        }
        DropKeep::None => {
            let res = rolls.iter().sum::<i64>() + modifier;
            final_str(start, res, initial_roll, n, modifier)
        }
    }
}

fn dh_str(rolls: Vec<i64>, n: i64, modifier: i64, dk_val: usize, initial_roll: String, start: String) -> String {
    dk_str(drop_high(rolls, dk_val), n, modifier, initial_roll, start)
}

fn dl_str(rolls: Vec<i64>, n: i64, modifier: i64, dk_val: usize, initial_roll: String, start: String) -> String {
    dk_str(drop_low(rolls, dk_val), n, modifier, initial_roll, start)
}

fn dk_str(dk_rolls: Vec<i64>, n: i64, modifier: i64, initial_roll: String, start: String) -> String {
    let new_rolls = format!("({initial_roll}) -> {}", rolls_str(&dk_rolls, modifier));
    let res = sum(dk_rolls, modifier);
    final_str(start, res, new_rolls, n, modifier)
}

fn final_str(start: String, res: i64, rolls: String, n: i64, modifier: i64) -> String {
    if modifier == 0 && n == 1 {
        format!("{start} {res}")
    } else {
        format!("{start} {rolls} = {res}")
    }
}

fn sum(rolls: Vec<i64>, modifier: i64) -> i64 {
    rolls.iter().sum::<i64>() + modifier
}

fn rolls_str(rolls: &[i64], modifier: i64) -> String {
    add_modifier(
        rolls
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<String>>()
            .join(" + "),
        modifier,
    )
}

fn drop_low(mut rolls: Vec<i64>, n: usize) -> Vec<i64> {
    let bad_elements = rolls
        .clone()
        .into_iter()
        .sorted()
        .rev()
        .skip(rolls.len() - n)
        .collect::<Vec<i64>>();
    for elem in bad_elements {
        let index = rolls.clone().into_iter().position(|e| e == elem).unwrap();
        rolls.remove(index);
    }
    rolls
}

fn drop_high(mut rolls: Vec<i64>, n: usize) -> Vec<i64> {
    let bad_elements = rolls
        .clone()
        .into_iter()
        .sorted()
        .skip(rolls.len() - n)
        .collect::<Vec<i64>>();
    for elem in bad_elements {
        let index = rolls.clone().into_iter().position(|e| e == elem).unwrap();
        rolls.remove(index);
    }
    rolls
}

fn start_message(roll: &Roll) -> String {
    let (n, size, modifier, dk, dk_val) = roll.values();
    let show_mod = match modifier.cmp(&0) {
        Ordering::Greater => format!("+{modifier}"),
        Ordering::Less => modifier.to_string(),
        Ordering::Equal => String::new(),
    };
    let dk_val = if let Some(x) = dk_val {
        x.to_string()
    } else {
        String::new()
    };
    format!("`[r {n}d{size}{show_mod}{dk}{dk_val}]`")
}

fn add_modifier(s: String, modifier: i64) -> String {
    match modifier.cmp(&0) {
        Ordering::Greater => format!("{s} (+{modifier})"),
        Ordering::Less => format!("{s} ({modifier})"),
        Ordering::Equal => s,
    }
}

fn get_dk(s: impl ToString) -> Result<(DropKeep, Option<usize>), CommandError> {
    // default case
    if s.to_string().is_empty() {
        return Ok((DropKeep::None, None));
    }

    let re = regex::Regex::new(r"(?P<dk>kh|kl|k|dh|dl|d)(?P<dk_val>\d+)")?;
    let s = &s.to_string();
    let caps = match re.captures(s) {
        Some(caps) => caps,
        None => return Err(utils::command_error("erreur captures regex")),
    };
    let dk: DropKeep = DropKeep::from_str(match caps.name("dk") {
        Some(m) => m.as_str(),
        None => "",
    })?;
    let dk_val = match caps.name("dk_val") {
        Some(m) => Some(m.as_str().parse::<usize>()?),
        None => None,
    };
    if dk.is_some() && dk_val.is_none() {
        return Err(utils::command_error("valeur attendu après le drop/keep"));
    }

    Ok((dk, dk_val))
}

pub fn run_chat_input(options: &[CommandDataOption]) -> InteractionResponse {
    let mut n = 1;
    let mut size = 0;
    let mut modifier = 0;
    let mut init_dk = String::new();

    for arg in options {
        let value = arg.resolved.as_ref().unwrap();
        match arg.name.as_str() {
            "number" => {
                if let CommandDataOptionValue::Integer(x) = value {
                    n = *x;
                }
            }
            "size" => {
                if let CommandDataOptionValue::Integer(x) = value {
                    size = *x;
                }
            }
            "modifier" => {
                if let CommandDataOptionValue::Integer(x) = value {
                    modifier = *x;
                }
            }
            "drop_keep" => {
                if let CommandDataOptionValue::String(s) = value {
                    init_dk = s.to_string();
                }
            }
            _ => (),
        }
    }
    let (dk, dk_val) = get_dk(init_dk).unwrap_or((DropKeep::None, None));
    let roll = Roll::builder(n, size, modifier, dk, dk_val);

    InteractionResponse::Message(InteractionMessage {
        content: run(roll),
        ephemeral: false,
        embed: None,
    })
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("roll")
        .description("Lancer de dés")
        .create_option(|option| {
            option
                .name("size")
                .description("Taille des dés")
                .kind(CommandOptionType::Integer)
                .min_int_value(2)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("number")
                .description("Nombre de dés")
                .kind(CommandOptionType::Integer)
                .min_int_value(1)
                .max_int_value(200)
        })
        .create_option(|option| {
            option
                .name("modifier")
                .description("Modificateur")
                .kind(CommandOptionType::Integer)
        })
        .create_option(|option| {
            option
                .name("drop_keep")
                .description("Valeur possibles : (k, kh, kl, d, dh, dl) suivi d'un nombre")
                .kind(CommandOptionType::String)
        })
}
