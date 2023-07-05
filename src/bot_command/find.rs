use super::core::*;
use async_trait::async_trait;
use teloxide::prelude::*;

pub struct Find;

pub struct Args {
    pub query: String,
}

#[async_trait]
impl super::Command<Args> for Find {
    async fn execute(bot: Bot, msg: Message, args: Args) {
        bot.send_message(
            msg.chat.id,
            match BingSearch::execute(bing_search::Args { query: &args.query }).await {
                Ok(text) => text,
                Err(err) => format!("Failed to translate:\n{err}"),
            },
        )
        .reply_to_message_id(msg.id)
        .send()
        .await
        .ok();
    }
}
