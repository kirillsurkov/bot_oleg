use super::OlegCommand;
use crate::bot_command::core::*;
use async_trait::async_trait;
use openai::chat::*;
use std::sync::Arc;
use teloxide::{prelude::*, types::InputFile};
use tokio::sync::Mutex;

pub struct Draw;

pub struct Args<'a> {
    pub bot: &'a Bot,
    pub msg: &'a Message,
    pub sd_draw: Arc<Mutex<SdDraw>>,
    pub db: Arc<Mutex<crate::DB>>,
    pub description: &'a str,
    pub nsfw: bool,
}

#[async_trait]
impl<'a> OlegCommand<Args<'a>> for Draw {
    fn desc() -> ChatCompletionFunctionDefinition {
        ChatCompletionFunctionDefinition {
            name: "draw".to_owned(),
            description: Some("Draw image".to_owned()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "description": {
                        "type": "string",
                        "description": "Description of an image"
                    },
                    "nsfw": {
                        "type": "boolean",
                        "nsfw": "Is description NSFW or not"
                    },
                },
                "required": ["description", "nsfw"],
            })),
        }
    }

    async fn execute(args: Args<'a>) -> (Option<Message>, Option<String>) {
        match SdDraw::execute(sd_draw::Args {
            instance: args.sd_draw.clone(),
            description: args.description,
            msg: args.msg,
        })
        .await
        {
            Ok(img) => {
                let answer = args
                    .bot
                    .send_photo(args.msg.chat.id, InputFile::memory(img))
                    .reply_to_message_id(args.msg.id)
                    .has_spoiler(args.nsfw)
                    .send()
                    .await
                    .ok();
                if let Some(photo) = answer
                    .as_ref()
                    .and_then(|a| a.photo())
                    .and_then(|p| p.last())
                {
                    args.db
                        .lock()
                        .await
                        .add_caption(&photo.file.id, Some(args.description));
                }
                (answer, None)
            }
            Err(err) => (None, Some(format!("{err:#}"))),
        }
    }
}
