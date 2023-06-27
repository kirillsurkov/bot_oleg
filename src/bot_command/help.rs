use async_trait::async_trait;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

pub struct Help;

type Args = ();

#[async_trait]
impl super::Command<Args> for Help {
    async fn execute(bot: Bot, msg: Message, _args: Args) {
        bot.send_message(msg.chat.id, super::BotCommand::descriptions().to_string())
            .reply_to_message_id(msg.id)
            .await
            .ok();
    }
}
