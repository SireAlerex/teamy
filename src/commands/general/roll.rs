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
use serenity::model::prelude::*;
use serenity::prelude::Context;
use std::cmp::Ordering;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub enum DropKeep {
    DL,
    DH,
    KL,
    KH,
    #[default]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Roll {
    number: i64,
    size: i64,
    modifier: i64,
    dk: DropKeep,
    dk_val: Option<i64>,
}

impl Roll {
    pub fn new() -> Roll {
        Roll::default()
    }

    pub fn roll(&self) -> RollResult {
        let mut rolls: Vec<i64> = Vec::new();
        let mut rng = rand::thread_rng();

        for _ in 0..self.number {
            rolls.push(rng.gen_range(1..=self.size));
        }

        let initial_roll = rolls_str(&rolls, self.modifier);

        let (s, rolls) = if self.dk.is_some() {
            // dk_val is forced to be some if dk is some
            let dk_val = self.dk_val.unwrap();
            let new_rolls = match self.dk {
                DropKeep::DL => drop_low(rolls, dk_val),
                DropKeep::KH => drop_low(rolls, self.number - dk_val),
                DropKeep::DH => drop_high(rolls, dk_val),
                DropKeep::KL => drop_high(rolls, self.number - dk_val),
                _ => panic!("drop/keep is_some error"),
            };
            (
                format!(
                    "({initial_roll}) -> {}",
                    rolls_str(&new_rolls, self.modifier)
                ),
                new_rolls,
            )
        } else {
            (initial_roll, rolls)
        };

        let res = sum(&rolls, self.modifier).to_string();
        let message = format!("{self} {}", show_res(s, res, self.number, self.modifier));
        RollResult {
            roll: self,
            rolls,
            message,
        }
    }
}

impl Default for Roll {
    fn default() -> Self {
        Roll {
            number: 1,
            size: 6,
            modifier: 0,
            dk: DropKeep::default(),
            dk_val: None,
        }
    }
}

impl std::fmt::Display for Roll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let show_mod = match self.modifier.cmp(&0) {
            Ordering::Greater => format!("+{}", self.modifier),
            Ordering::Less => self.modifier.to_string(),
            Ordering::Equal => String::new(),
        };
        let dk_val = if let Some(x) = self.dk_val {
            x.to_string()
        } else {
            String::new()
        };
        let s = format!(
            "`[r {}d{}{show_mod}{}{dk_val}]`",
            self.number, self.size, self.dk
        );
        write!(f, "{s}")
    }
}

impl FromStr for Roll {
    type Err = Box<dyn std::error::Error + Send + Sync>;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let re = regex::Regex::new(
            r"(?P<number>\d+)?d(?P<size>\d+)(?P<modifier>\+\d+|-\d+)?(?P<dk>kh|kl|k|dh|dl|d)?(?P<dk_val>\d+)?",
        )?;
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
            Some(m) => Some(m.as_str().parse::<i64>()?),
            None => None,
        };
        if dk.is_some() && dk_val.is_none() {
            return Err(utils::command_error("valeur attendu après le drop/keep"));
        }
        if dk_val.is_some() && dk_val.unwrap() > number {
            return Err(utils::command_error(
                "la valeur du drop/keep doit être inférieure ou égale au nombre de dés",
            ));
        }

        let roll = RollBuilder::new()
            .number(number)
            .size(size)
            .modifier(modifier)
            .drop_keep(dk)
            .drop_keep_value(dk_val)
            .build();
        Ok(roll)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct RollBuilder {
    number: i64,
    size: i64,
    modifier: i64,
    dk: DropKeep,
    dk_val: Option<i64>,
}

impl Default for RollBuilder {
    fn default() -> Self {
        RollBuilder {
            number: 1,
            size: 6,
            modifier: 0,
            dk: DropKeep::default(),
            dk_val: None,
        }
    }
}

impl RollBuilder {
    pub fn new() -> RollBuilder {
        RollBuilder::default()
    }

    pub fn number(&mut self, number: i64) -> &mut Self {
        self.number = number;
        self
    }

    pub fn size(&mut self, size: i64) -> &mut Self {
        self.size = size;
        self
    }

    pub fn modifier(&mut self, modifier: i64) -> &mut Self {
        self.modifier = modifier;
        self
    }

    pub fn drop_keep(&mut self, dk: DropKeep) -> &mut Self {
        self.dk = dk;
        self
    }

    pub fn drop_keep_value(&mut self, dk_val: Option<i64>) -> &mut Self {
        self.dk_val = dk_val;
        self
    }

    pub fn build(&mut self) -> Roll {
        Roll {
            number: self.number,
            size: self.size,
            modifier: self.modifier,
            dk: self.dk,
            dk_val: self.dk_val,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct RollResult<'a> {
    roll: &'a Roll,
    rolls: Vec<i64>,
    message: String,
}

impl RollResult<'_> {
    pub fn sum(&self) -> i64 {
        sum(&self.rolls, self.roll.modifier)
    }
}

impl Display for RollResult<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
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
    let msg = channel_id
        .say(&ctx.http, Roll::from_str(args.message())?.roll())
        .await?;

    Ok(msg)
}

fn sum(rolls: &[i64], modifier: i64) -> i64 {
    rolls.iter().sum::<i64>() + modifier
}

fn show_res(roll: String, res: String, number: i64, modifier: i64) -> String {
    if number == 1 && modifier == 0 {
        res
    } else {
        format!("{roll} = {res}")
    }
}

fn rolls_str(rolls: &[i64], modifier: i64) -> String {
    add_modifier(
        rolls
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>()
            .join(" + "),
        modifier,
    )
}

fn drop_low(mut rolls: Vec<i64>, n: i64) -> Vec<i64> {
    let bad_elements = rolls
        .clone()
        .into_iter()
        .sorted()
        .rev()
        .skip(rolls.len() - (n as usize))
        .collect::<Vec<i64>>();
    for elem in bad_elements {
        // each element is necessary inside iterator so unwrap is safe
        let index = rolls.clone().into_iter().position(|e| e == elem).unwrap();
        rolls.remove(index);
    }
    rolls
}

fn drop_high(mut rolls: Vec<i64>, n: i64) -> Vec<i64> {
    let bad_elements = rolls
        .clone()
        .into_iter()
        .sorted()
        .skip(rolls.len() - (n as usize))
        .collect::<Vec<i64>>();
    for elem in bad_elements {
        // each element is necessary inside iterator so unwrap is safe
        let index = rolls.clone().into_iter().position(|e| e == elem).unwrap();
        rolls.remove(index);
    }
    rolls
}

fn add_modifier(s: String, modifier: i64) -> String {
    match modifier.cmp(&0) {
        Ordering::Greater => format!("{s} (+{modifier})"),
        Ordering::Less => format!("{s} ({modifier})"),
        Ordering::Equal => s,
    }
}

fn get_dk(s: impl ToString) -> Result<(DropKeep, Option<i64>), CommandError> {
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
        Some(m) => Some(m.as_str().parse::<i64>()?),
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
    let r = RollBuilder::new()
        .number(n)
        .size(size)
        .modifier(modifier)
        .drop_keep(dk)
        .drop_keep_value(dk_val)
        .build();

    InteractionResponse::Message(InteractionMessage {
        content: r.roll().to_string(),
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
