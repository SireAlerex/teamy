use rand::Rng;
use serenity::builder::CreateApplicationCommand;
use serenity::framework::standard::{CommandResult, Args};
use serenity::framework::standard::macros::command;
use serenity::model::application::command::CommandOptionType;
use serenity::model::prelude::Message;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};
use serenity::prelude::Context;

#[command]
pub async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
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
