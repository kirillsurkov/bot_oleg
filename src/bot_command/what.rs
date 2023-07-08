use super::core::*;
use async_trait::async_trait;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;

pub struct What;

pub struct Args<'a> {
    pub db: Arc<Mutex<crate::DB>>,
    pub http_client: &'a reqwest::Client,
    pub settings: &'a crate::Settings,
}

#[async_trait]
impl<'a> super::Command<Args<'a>> for What {
    async fn execute(bot: Bot, msg: Message, args: Args<'a>) {
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
            file_id: msg.photo().and_then(|p| p.last().map(|p| &p.file.id[..])),
            http_client: args.http_client,
            settings: args.settings,
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
