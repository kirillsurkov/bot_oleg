use super::core::*;
use async_trait::async_trait;
use teloxide::prelude::*;

pub struct Translate;

pub struct Args<'a> {
    pub text: String,
    pub to_language: String,
    pub settings: &'a crate::Settings,
}

#[async_trait]
impl<'a> super::Command<Args<'a>> for Translate {
    async fn execute(bot: Bot, msg: Message, args: Args<'a>) {
        bot.send_message(
            msg.chat.id,
            match GoogleTranslate::execute(google_translate::Args {
                to_language: &args.to_language,
                text: &args.text,
                settings: &args.settings,
            })
            .await
            {
                Ok(text) => text,
                Err(err) => format!("Failed to translate:\n{err:#}"),
            },
        )
        .reply_to_message_id(msg.id)
        .send()
        .await
        .ok();
    }
}
