use super::core::*;
use async_trait::async_trait;
use teloxide::prelude::*;

pub struct Find;

pub struct Args<'a> {
    pub query: String,
    pub http_client: &'a reqwest::Client,
}

#[async_trait]
impl<'a> super::Command<Args<'a>> for Find {
    async fn execute(bot: Bot, msg: Message, args: Args<'a>) {
        let Args { query, http_client } = &args;
        bot.send_message(
            msg.chat.id,
            match BingSearch::execute(bing_search::Args { query, http_client }).await {
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
