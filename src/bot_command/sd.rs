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
    pub http_client: reqwest::Client,
    pub translator: Arc<crate::Translator>,
    pub settings: Arc<crate::Settings>,
}

#[async_trait]
impl super::Command<Args> for Sd {
    async fn execute(bot: Bot, msg: Message, args: Args) {
        let answer = match SdDraw::execute(sd_draw::Args {
            instance: args.sd_draw.clone(),
            description: &args.description,
            msg: &msg,
            http_client: &args.http_client,
            translator: &args.translator,
            settings: &args.settings,
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
                bot.send_message(msg.chat.id, format!("{err:#}"))
                    .reply_to_message_id(msg.id)
                    .send()
                    .await
            }
        };

        let db = args.db.lock().await;
        db.add_message("sd_q", &msg);
        if let Ok(answer) = answer {
            db.add_message("oleg_a", &answer);
            if let Some(photo) = answer.photo().and_then(|p| p.last()) {
                db.add_caption(&photo.file.id, Some(&args.description));
            }
        }
    }
}
