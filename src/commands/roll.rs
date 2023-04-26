use rand::Rng;
use serenity::builder::CreateApplicationCommand;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::application::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
pub async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let d_split: Vec<&str> = args.message().split('d').collect();
    let size;
    let n;
    let mut modifier = 0;
    if d_split.len() == 2 {
        n = d_split[0].parse::<i64>().unwrap_or(1);
        if !(1..=200).contains(&n) {
            return Err("1 <= #dices <= 200".into());
        }
        let (char, pos) = if !d_split[1].contains("-") {
            ('+', true)
        } else {
            ('-', false)
        };
        let mod_split: Vec<&str> = d_split[1].split(char).collect();
        size = mod_split[0].parse::<i64>()?;
        if size < 1 {
            return Err("dice size >= 1".into());
        }
        if mod_split.len() == 2 {
            modifier = if pos {
                mod_split[1].parse::<i64>()?
            } else {
                -mod_split[1].parse::<i64>()?
            };
        }
    } else {
        return Err("bad syntax, must be '$roll <x>d<n>+<y>'".into());
    }
    msg.channel_id
        .say(&ctx.http, run(size, n, modifier))
        .await?;
    Ok(())
}

pub fn run(size: i64, n: i64, modifier: i64) -> String {
    let mut rng = rand::thread_rng();
    let mut res: String = String::new();
    let mut sum = modifier;

    for _ in 0..n {
        let roll = rng.gen_range(1..=size);
        sum += roll;
        res = format!("{}{}{}", res, roll.to_string(), " + ");
    }
    res = res[..res.len() - 2].to_string(); //remove last "+ "

    if modifier > 0 {
        res = format!("{}(+{}) ", res, modifier);
    } else if modifier < 0 {
        res = format!("{}({}) ", res, modifier);
    }

    if modifier != 0 || n > 1 {
        res = format!("{}= {}", res, sum);
    }
    res
}

pub fn run_chat_input(_options: &[CommandDataOption]) -> String {
    let mut n = 1;
    let mut size = 0;
    let mut modifier = 0;

    for arg in _options {
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
            _ => (),
        }
    }

    run(size, n, modifier)
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("roll")
        .description("roll desc")
        .create_option(|option| {
            option
                .name("size")
                .description("size desc")
                .kind(CommandOptionType::Integer)
                .min_int_value(2)
                .max_int_value(1000)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("number")
                .description("number desc")
                .kind(CommandOptionType::Integer)
                .min_int_value(1)
                .max_int_value(100)
        })
        .create_option(|option| {
            option
                .name("modifier")
                .description("modifier desc")
                .kind(CommandOptionType::Integer)
        })
}
