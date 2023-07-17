use serenity::builder::{CreateEmbed, CreateInteractionResponse};
use serenity::model::application::interaction::Interaction as SerenityInteraction;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::interaction::modal::ModalSubmitInteraction;
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::prelude::*;
use SerenityInteraction::{ApplicationCommand, ModalSubmit, Ping, Autocomplete, MessageComponent};

use crate::utils;

// interaction modal custom ids
pub const MACRO_ADD_FORM_ID: &str = "macro_add_form";
pub const MACRO_ADD_FORM_NAME: &str = "macro_add_name";

pub struct Interaction {
    response: Response,
    serenity_interaction: SerenityInteraction,
}

impl Interaction {
    pub fn new(response: Response, serenity_interaction: SerenityInteraction) -> Self {
        Self {
            response,
            serenity_interaction,
        }
    }

    pub async fn send(self, ctx: &Context) {
        if let Response::Message(msg) = self.response {
            match self.serenity_interaction {
                ApplicationCommand(command) => msg.send_from_command(ctx, &command).await,
                ModalSubmit(modal) => msg.send_from_modal(ctx, &modal).await,
                Ping(_) | MessageComponent(_) | Autocomplete(_) => (),
            }
        }
    }
}

pub enum Response {
    Message(InteractionMessage),
    Modal,
    None,
}

pub struct InteractionMessage {
    content: String,
    ephemeral: bool,
    embed: Option<CreateEmbed>,
}

impl InteractionMessage {
    pub fn new<T: Into<String>>(content: T, ephemeral: bool, embed: Option<CreateEmbed>) -> Self {
        Self {
            content: content.into(),
            ephemeral,
            embed,
        }
    }

    pub fn ephemeral<T: Into<String>>(content: T) -> Self {
        Self {
            content: content.into(),
            ephemeral: true,
            embed: None,
        }
    }

    pub fn with_content<T: Into<String>>(content: T) -> Self {
        Self {
            content: content.into(),
            ephemeral: false,
            embed: None,
        }
    }

    pub async fn send_from_command(self, ctx: &Context, command: &ApplicationCommandInteraction) {
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                self.channel_message_with_source(response)
            })
            .await
        {
            let error_message = format!("Erreur lors de la réponse à l'interaction : {why}");
            utils::say_or_error(ctx, command.channel_id, error_message).await;
        }
    }

    pub async fn send_from_modal(self, ctx: &Context, modal: &ModalSubmitInteraction) {
        if let Err(why) = modal
            .create_interaction_response(&ctx.http, |response| {
                self.channel_message_with_source(response)
            })
            .await
        {
            let error_message = format!("Erreur lors de la réponse à l'interaction : {why}");
            utils::say_or_error(ctx, modal.channel_id, error_message).await;
        }
    }

    fn channel_message_with_source<'a, 'b>(
        self,
        response: &'a mut CreateInteractionResponse<'b>,
    ) -> &'a mut CreateInteractionResponse<'b> {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(move |message| {
                let m = message.content(self.content).ephemeral(self.ephemeral);
                if let Some(e) = self.embed {
                    m.add_embed(e)
                } else {
                    m
                }
            })
    }
}
