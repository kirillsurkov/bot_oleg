use super::core::*;
use async_trait::async_trait;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;

pub struct What;

pub struct Args {
    pub db: Arc<Mutex<crate::DB>>,
}

#[async_trait]
impl super::Command<Args> for What {
    async fn execute(bot: Bot, msg: Message, args: Args) {
        let msg = msg
            .photo()
            .and(Some(&msg))
            .or(msg
                .reply_to_message()
                .map(|r| r.photo())
                .and(msg.reply_to_message()))
            .unwrap_or(&msg);
        match SdWhat::execute(sd_what::Args {
            db: args.db,
            bot: bot.clone(),
            file_id: msg
                .photo()
                .and_then(|p| p.last().map(|p| p.file.id.clone())),
        })
        .await
        {
            Ok(caption) => bot.send_message(msg.chat.id, caption),
            Err(err) => bot.send_message(msg.chat.id, format!("{err:#}")),
        }
        .reply_to_message_id(msg.id)
        .send()
        .await
        .ok();
    }
}
