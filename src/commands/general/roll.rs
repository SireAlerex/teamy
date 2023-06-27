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
    DL(u64),
    DH(u64),
    KL(u64),
    KH(u64),
    #[default]
    None,
}

impl FromStr for DropKeep {
    type Err = Box<dyn std::error::Error + Send + Sync>;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        if s.is_empty() {
            return Ok(DropKeep::None);
        }

        let re = regex::Regex::new(
            r"(?P<dk>kh|kl|k|dh|dl|d)(?P<dk_val>\d+)",
        )?;
        let caps = match re.captures(s) {
            Some(caps) => caps,
            None => return Err(utils::command_error("erreur captures regex")),
        };
        
        let dk = if let Some(m) = caps.name("dk") {m.as_str()} else {return Ok(DropKeep::None);};
        let dk_val = match caps.name("dk_val") {
            Some(m) => m.as_str(),
            None => return Err(utils::command_error("erreur roll regex drop/keep : dk_val")),
        }.parse::<u64>()?;

        match dk {
            "d" | "dl" => Ok(DropKeep::DL(dk_val)),
            "dh" => Ok(DropKeep::DH(dk_val)),
            "k" | "kh" => Ok(DropKeep::KH(dk_val)),
            "kl" => Ok(DropKeep::KL(dk_val)),
            _ => return Err(utils::command_error("erreur roll regex drop/keep : dk")),
        }
    }
}

impl std::fmt::Display for DropKeep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let s = match self {
            DropKeep::DL(x) => format!("d{x}"),            
            DropKeep::DH(x) => format!("dh{x}"),
            DropKeep::KH(x) => format!("k{x}"),            
            DropKeep::KL(x) => format!("kl{x}"),
            DropKeep::None => String::new(),
        };
        write!(f, "{s}")
    }
}

impl DropKeep {
    pub fn is_some(&self) -> bool {
        *self != DropKeep::None
    }

    pub fn get(&self) -> Option<u64> {
        match self {
            DropKeep::DH(x) | DropKeep::DL(x) | DropKeep::KH(x) | DropKeep::KL(x) => Some(*x),
            DropKeep::None => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Roll {
    number: u64,
    size: u64,
    modifier: i64,
    dk: DropKeep,
}

impl Roll {
    #[allow(dead_code)]
    pub fn new() -> Roll {
        Roll::default()
    }

    pub fn roll(&self) -> RollResult {
        let mut rolls: Vec<u64> = Vec::new();
        let mut rng = rand::thread_rng();

        for _ in 0..self.number {
            rolls.push(rng.gen_range(1..=self.size));
        }

        let initial_roll = rolls_str(&rolls, self.modifier);

        let (s, rolls) = if self.dk.is_some() {
            let new_rolls = match self.dk {
                DropKeep::DL(x) => drop_low(rolls, x),
                DropKeep::KH(x) => drop_low(rolls, self.number - x),
                DropKeep::DH(x) => drop_high(rolls, x),
                DropKeep::KL(x) => drop_high(rolls, self.number - x),
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

        let s = format!(
            "`[r {}d{}{show_mod}{}]`",
            self.number, self.size, self.dk
        );
        write!(f, "{s}")
    }
}

impl FromStr for Roll {
    type Err = Box<dyn std::error::Error + Send + Sync>;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let re = regex::Regex::new(
            r"(?P<number>\d+)?d(?P<size>\d+)(?P<modifier>\+\d+|-\d+)?(?P<dk>kh\d+|kl\d+|k\d+|dh\d+|dl\d+|d\d+)?",
        )?;
        let caps = match re.captures(s) {
            Some(caps) => caps,
            None => return Err(utils::command_error("erreur captures regex")),
        };

        let number = match caps.name("number") {
            Some(n) => n.as_str(),
            None => "1",
        }
        .parse::<u64>()?;
        if !(1..=200).contains(&number) {
            return Err(utils::command_error(
                "le nombre de dés doit appartenir à [1; 200]",
            ));
        }
        let size = match caps.name("size") {
            Some(m) => m.as_str().parse::<u64>()?,
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

        if let Some(x) = dk.get() {
            if x > number {
                return Err(utils::command_error("valeur du drop/keep doit être <= nombre de dés"));
            }
        }

        let roll = RollBuilder::new()
            .number(number)
            .size(size)
            .modifier(modifier)
            .drop_keep(dk)
            .build();
        Ok(roll)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct RollBuilder {
    number: u64,
    size: u64,
    modifier: i64,
    dk: DropKeep,
}

impl Default for RollBuilder {
    fn default() -> Self {
        RollBuilder {
            number: 1,
            size: 6,
            modifier: 0,
            dk: DropKeep::default(),
        }
    }
}

impl RollBuilder {
    pub fn new() -> RollBuilder {
        RollBuilder::default()
    }

    pub fn number(&mut self, number: u64) -> &mut Self {
        self.number = number;
        self
    }

    pub fn size(&mut self, size: u64) -> &mut Self {
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

    pub fn build(&mut self) -> Roll {
        Roll {
            number: self.number,
            size: self.size,
            modifier: self.modifier,
            dk: self.dk,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct RollResult<'a> {
    roll: &'a Roll,
    rolls: Vec<u64>,
    message: String,
}

impl RollResult<'_> {
    #[allow(dead_code)]
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

fn sum(rolls: &[u64], modifier: i64) -> i64 {
    modifier + <u64 as TryInto<i64>>::try_into(rolls.iter().sum::<u64>()).unwrap()
}

fn show_res(roll: String, res: String, number: u64, modifier: i64) -> String {
    if number == 1 && modifier == 0 {
        res
    } else {
        format!("{roll} = {res}")
    }
}

fn rolls_str(rolls: &[u64], modifier: i64) -> String {
    add_modifier(
        rolls
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>()
            .join(" + "),
        modifier,
    )
}

fn drop_low(mut rolls: Vec<u64>, n: u64) -> Vec<u64> {
    let bad_elements = rolls
        .clone()
        .into_iter()
        .sorted()
        .rev()
        .skip(rolls.len() - (n as usize))
        .collect::<Vec<u64>>();
    for elem in bad_elements {
        // each element is necessary inside iterator so unwrap is safe
        let index = rolls.clone().into_iter().position(|e| e == elem).unwrap();
        rolls.remove(index);
    }
    rolls
}

fn drop_high(mut rolls: Vec<u64>, n: u64) -> Vec<u64> {
    let bad_elements = rolls
        .clone()
        .into_iter()
        .sorted()
        .skip(rolls.len() - (n as usize))
        .collect::<Vec<u64>>();
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
    let dk = if let Ok(d) = DropKeep::from_str(&init_dk) {
        d
    } else {
        return InteractionResponse::Message(InteractionMessage { content: "erreur dk".to_string(), ephemeral: true, embed: None });
    };

    let r = RollBuilder::new()
        .number(n.try_into().unwrap())
        .size(size.try_into().unwrap())
        .modifier(modifier)
        .drop_keep(dk)
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_roll() {
        let d6 = Roll {number: 1, size: 6, modifier: 0, dk: DropKeep::None};
        // default roll
        assert_eq!(Roll::new(), d6);

        // default rollbuilder
        let default_roll = RollBuilder::default().build();
        assert_eq!(default_roll, d6);

        let basic_roll_pos = RollBuilder::default().modifier(2).build();
        assert_eq!(basic_roll_pos, Roll {number: 1, size: 6, modifier: 2, dk: DropKeep::None});

        let basic_roll_neg = RollBuilder::default().modifier(-2).build();
        assert_eq!(basic_roll_neg, Roll {number: 1, size: 6, modifier: -2, dk: DropKeep::None});

        let dk_roll = RollBuilder::default().number(4).size(20).drop_keep(DropKeep::KH(3)).build();
        assert_eq!(dk_roll, Roll {number: 4, size: 20, modifier: 0, dk: DropKeep::KH(3)});

        assert!(Roll::from_str("d6").is_ok());
        assert!(Roll::from_str("d4+2").is_ok());
        assert!(Roll::from_str("d8-3").is_ok());
        assert!(Roll::from_str("2d10").is_ok());
        assert!(Roll::from_str("3d12+4").is_ok());
        assert!(Roll::from_str("105d20-1").is_ok());
        assert!(Roll::from_str("4d6k3").is_ok());
        assert!(Roll::from_str("2d20d1").is_ok());

        assert!(Roll::from_str("0d6").is_err());
        assert!(Roll::from_str("1d0").is_err());
        assert!(Roll::from_str("201d6").is_err());
    }
}
