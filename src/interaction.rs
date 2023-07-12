use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::interaction::modal::ModalSubmitInteraction;
use serenity::prelude::*;
use serenity::builder::{CreateEmbed, CreateInteractionResponse};

use crate::utils;

pub enum InteractionResponse {
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
        Self { content: content.into(), ephemeral, embed }
    }

    pub fn ephemeral<T: Into<String>>(content: T) -> Self {
        Self { content: content.into(), ephemeral: true, embed: None }
    }

    pub fn with_content<T: Into<String>>(content: T) -> Self {
        Self { content: content.into(), ephemeral: false, embed: None }
    }

    pub async fn send_from_command(
        self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) {
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| self.channel_message_with_source(response))
            .await
        {
            let error_message = format!("Erreur lors de la réponse à l'interaction : {why}");
            utils::say_or_error(ctx, command.channel_id, error_message).await;
        }
    }

    pub async fn send_from_modal(
        self,
        ctx: &Context,
        modal: &ModalSubmitInteraction
    ) {
        if let Err(why) = modal
            .create_interaction_response(&ctx.http, |response| self.channel_message_with_source(response))
            .await
        {
            let error_message = format!("Erreur lors de la réponse à l'interaction : {why}");
            utils::say_or_error(ctx, modal.channel_id, error_message).await;
        }
    }

    fn channel_message_with_source<'a, 'b>(&'b self, response: &'a mut CreateInteractionResponse<'b>) -> &'a mut CreateInteractionResponse {
        let content = self.content.clone();
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(move |message| {
                if let Some(e) = self.embed.clone() {
                    message.content(content).ephemeral(self.ephemeral).add_embed(e)
                } else {
                    message.content(content).ephemeral(self.ephemeral)
                }
            })
    }
}