use super::core::*;
use async_trait::async_trait;
use std::sync::Arc;
use teloxide::{prelude::*, types::InputFile};
use tokio::sync::Mutex;

pub struct Sd;

pub struct Args {
    pub sd_draw: Arc<Mutex<super::core::SdDraw>>,
    pub db: Arc<Mutex<crate::DB>>,
    pub description: String,
}

#[async_trait]
impl super::Command<Args> for Sd {
    async fn execute(bot: Bot, msg: Message, args: Args) {
        match SdDraw::execute(sd_draw::Args {
            instance: args.sd_draw.clone(),
            description: &args.description,
            msg: &msg,
        })
        .await
        {
            Ok(img) => {
                bot.send_photo(msg.chat.id, InputFile::memory(img))
                    .reply_to_message_id(msg.id)
                    .has_spoiler(true)
                    .send()
                    .await
            }
            Err(err) => {
                bot.send_message(msg.chat.id, err)
                    .reply_to_message_id(msg.id)
                    .send()
                    .await
            }
        }
        .ok();
    }
}
