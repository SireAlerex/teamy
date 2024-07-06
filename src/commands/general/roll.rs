use anyhow::anyhow;
use itertools::Itertools;
use rand::Rng;
use std::cmp::Ordering;
use std::fmt::Display;
use std::str::FromStr;

use crate::commands::{Context as PoiseContext, PoiseError};
use poise::serenity_prelude;

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
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        if s.is_empty() {
            return Ok(Self::None);
        }

        let re = regex::Regex::new(r"(?P<dk>kh|kl|k|dh|dl|d)(?P<dk_val>\d+)")?;
        let Some(caps) = re.captures(s) else {
            return Err(anyhow!("erreur captures regex"));
        };

        let dk = if let Some(m) = caps.name("dk") {
            m.as_str()
        } else {
            return Ok(Self::None);
        };
        let dk_val = match caps.name("dk_val") {
            Some(m) => m.as_str(),
            None => return Err(anyhow!("erreur roll regex drop/keep : dk_val")),
        }
        .parse::<u64>()?;

        match dk {
            "d" | "dl" => Ok(Self::DL(dk_val)),
            "dh" => Ok(Self::DH(dk_val)),
            "k" | "kh" => Ok(Self::KH(dk_val)),
            "kl" => Ok(Self::KL(dk_val)),
            _ => Err(anyhow!("erreur roll regex drop/keep : dk")),
        }
    }
}

impl std::fmt::Display for DropKeep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let s = match self {
            Self::DL(x) => format!("d{x}"),
            Self::DH(x) => format!("dh{x}"),
            Self::KH(x) => format!("k{x}"),
            Self::KL(x) => format!("kl{x}"),
            Self::None => String::new(),
        };
        write!(f, "{s}")
    }
}

impl DropKeep {
    pub fn is_some(&self) -> bool {
        *self != Self::None
    }

    pub const fn get(&self) -> Option<u64> {
        match self {
            Self::DH(x) | Self::DL(x) | Self::KH(x) | Self::KL(x) => Some(*x),
            Self::None => None,
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn roll(&self) -> Result<RollResult, &str> {
        let mut rolls: Vec<u64> = Vec::new();
        let mut rng = rand::thread_rng();

        for _ in 0..self.number {
            rolls.push(rng.gen_range(1..=self.size));
        }

        let initial_roll = rolls_str(&rolls, self.modifier);

        let (s, new_rolls) = if self.dk.is_some() {
            let new_rolls = match self.dk {
                DropKeep::DL(x) => drop_low(rolls, x),
                DropKeep::KH(x) => drop_low(rolls, self.number - x),
                DropKeep::DH(x) => drop_high(rolls, x),
                DropKeep::KL(x) => drop_high(rolls, self.number - x),
                DropKeep::None => return Err("drop/keep is_some error"),
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

        let Ok(res) = sum(&new_rolls, self.modifier) else {
            return Err("modifieur conversion error");
        };
        let message = format!(
            "{self} {}",
            show_res(&s, res.to_string(), self.number, self.modifier)
        );
        Ok(RollResult {
            roll: *self,
            rolls: new_rolls,
            message,
        })
    }
}

impl Default for Roll {
    fn default() -> Self {
        Self {
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

        let s = format!("`[r {}d{}{show_mod}{}]`", self.number, self.size, self.dk);
        write!(f, "{s}")
    }
}

impl FromStr for Roll {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let re = regex::Regex::new(
            r"(?P<number>\d+)?d(?P<size>\d+)(?P<modifier>\+\d+|-\d+)?(?P<dk>kh\d+|kl\d+|k\d+|dh\d+|dl\d+|d\d+)?",
        )?;
        let Some(caps) = re.captures(s) else {
            return Err(anyhow!("erreur captures regex"));
        };

        let number = caps
            .name("number")
            .map_or("1", |n| n.as_str())
            .parse::<u64>()?;
        if !(1..=200).contains(&number) {
            return Err(anyhow!("le nombre de dés doit appartenir à [1; 200]",));
        }

        let size = match caps.name("size") {
            Some(m) => m.as_str().parse::<u64>()?,
            None => return Err(anyhow!("erreur pas de taille de dé")),
        };
        if size <= 1 {
            return Err(anyhow!(
                "la taille du dé doit être supérieure strictement à 1",
            ));
        }

        let modifier = caps
            .name("modifier")
            .map_or("0", |n| n.as_str())
            .parse::<i64>()?;

        let dk = DropKeep::from_str(caps.name("dk").map_or("", |n| n.as_str()))?;
        if let Some(x) = dk.get() {
            if x > number {
                return Err(anyhow!("valeur du drop/keep doit être <= nombre de dés",));
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
        Self {
            number: 1,
            size: 6,
            modifier: 0,
            dk: DropKeep::default(),
        }
    }
}

impl RollBuilder {
    pub fn new() -> Self {
        Self::default()
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
pub struct RollResult {
    roll: Roll,
    rolls: Vec<u64>,
    message: String,
}

impl RollResult {
    #[allow(dead_code)]
    pub fn sum(&self) -> Result<i64, std::num::TryFromIntError> {
        sum(&self.rolls, self.roll.modifier)
    }
}

impl Display for RollResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}

#[poise::command(
    slash_command,
    category = "general",
    description_localized("fr", "Lancer de dés")
)]
pub async fn roll(
    ctx: PoiseContext<'_>,
    #[description = "Taille des dés"]
    #[min = 2_u64]
    size: u64,
    #[description = "Nombre de dés"]
    #[min = 1_u64]
    #[max = 200_u64]
    number: Option<u64>,
    #[description = "Modificateur"] modifier: Option<i64>,
    #[description = "Valeurs possibles : (k, kh, kl, d, dh, dl) suivi d'un nombre"]
    drop_keep: Option<String>,
) -> Result<(), PoiseError> {
    ctx.say(roll_intern(size, number, modifier, drop_keep)?.message)
        .await?;
    Ok(())
}

fn roll_intern(
    size: u64,
    maybe_number: Option<u64>,
    maybe_modifer: Option<i64>,
    maybe_drop_keep: Option<String>,
) -> Result<RollResult, PoiseError> {
    let mut builder = RollBuilder::new();
    if let Some(number) = maybe_number {
        builder.number(number);
    }
    if let Some(modifier) = maybe_modifer {
        builder.modifier(modifier);
    }
    if let Some(s) = maybe_drop_keep {
        builder.drop_keep(DropKeep::from_str(&s)?);
    }
    let roll = builder.size(size).build();

    let res = roll.roll()?;
    Ok(res)
}

#[poise::command(
    prefix_command,
    aliases("r"),
    rename = "roll",
    category = "general",
    description_localized("fr", "Lancer de dés")
)]
pub async fn roll_prefix(ctx: PoiseContext<'_>, roll_str: String) -> Result<(), PoiseError> {
    roll_intern_str(ctx.serenity_context(), &ctx.channel_id(), roll_str).await
}

pub async fn roll_intern_str(
    ctx: &serenity_prelude::Context,
    channel_id: &serenity_prelude::ChannelId,
    roll_str: String,
) -> Result<(), PoiseError> {
    let roll = Roll::from_str(&roll_str)?;
    let res = roll.roll()?;
    let _ = channel_id.say(&ctx.http, res.message).await?;
    Ok(())
}

fn sum(rolls: &[u64], modifier: i64) -> Result<i64, std::num::TryFromIntError> {
    Ok(modifier + <u64 as TryInto<i64>>::try_into(rolls.iter().sum::<u64>())?)
}

fn show_res(roll: &str, res: String, number: u64, modifier: i64) -> String {
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

#[allow(clippy::cast_possible_truncation)]
fn drop_low(mut rolls: Vec<u64>, n: u64) -> Vec<u64> {
    let bad_elements = rolls
        .clone()
        .into_iter()
        .sorted()
        .rev()
        .skip(rolls.len() - (n as usize))
        .collect::<Vec<u64>>();
    for elem in bad_elements {
        if let Some(index) = rolls.clone().into_iter().position(|e| e == elem) {
            rolls.remove(index);
        }
    }
    rolls
}

#[allow(clippy::cast_possible_truncation)]
fn drop_high(mut rolls: Vec<u64>, n: u64) -> Vec<u64> {
    let bad_elements = rolls
        .clone()
        .into_iter()
        .sorted()
        .skip(rolls.len() - (n as usize))
        .collect::<Vec<u64>>();
    for elem in bad_elements {
        if let Some(index) = rolls.clone().into_iter().position(|e| e == elem) {
            rolls.remove(index);
        }
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_roll() {
        let d6 = Roll {
            number: 1,
            size: 6,
            modifier: 0,
            dk: DropKeep::None,
        };
        // default roll
        assert_eq!(Roll::new(), d6);

        // default rollbuilder
        let default_roll = RollBuilder::default().build();
        assert_eq!(default_roll, d6);

        let basic_roll_pos = RollBuilder::default().modifier(2).build();
        assert_eq!(
            basic_roll_pos,
            Roll {
                number: 1,
                size: 6,
                modifier: 2,
                dk: DropKeep::None
            }
        );

        let basic_roll_neg = RollBuilder::default().modifier(-2).build();
        assert_eq!(
            basic_roll_neg,
            Roll {
                number: 1,
                size: 6,
                modifier: -2,
                dk: DropKeep::None
            }
        );

        let dk_roll = RollBuilder::default()
            .number(4)
            .size(20)
            .drop_keep(DropKeep::KH(3))
            .build();
        assert_eq!(
            dk_roll,
            Roll {
                number: 4,
                size: 20,
                modifier: 0,
                dk: DropKeep::KH(3)
            }
        );

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
