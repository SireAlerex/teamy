use crate::Data;

pub mod admin;
pub mod general;
// pub mod macros;
// pub mod pdx;

// TODO: better Error
pub type PoiseError = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, PoiseError>;
