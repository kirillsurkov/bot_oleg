use async_trait::async_trait;
use teloxide::prelude::*;

pub struct Rm;

type Args = ();

#[async_trait]
impl super::Command<Args> for Rm {
    async fn execute(bot: Bot, msg: Message, _args: Args) {
        match msg.reply_to_message() {
            Some(msg) => bot.delete_message(msg.chat.id, msg.id).await.ok(),
            None => bot.delete_message(msg.chat.id, msg.id).await.ok(),
        };
    }
}
