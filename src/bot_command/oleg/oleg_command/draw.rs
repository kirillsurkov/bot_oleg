use super::OlegCommand;
use crate::bot_command::core::*;
use async_trait::async_trait;
use base64::Engine;
use openai::chat::*;
use std::sync::Arc;
use teloxide::{prelude::*, types::InputFile};
use tokio::sync::Mutex;

#[derive(serde::Deserialize)]
struct SdResponse {
    images: Vec<String>,
}

pub struct Draw;

pub struct Args<'a> {
    pub bot: &'a Bot,
    pub msg: &'a Message,
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

    async fn execute(args: Args<'a>) -> Option<Message> {
        match GoogleTranslate::execute(google_translate::Args {
            to_language: "en",
            text: args.description,
        })
        .await
        {
            Ok(text) => {
                match reqwest::Client::new()
                    .post(format!(
                        "{}/sdapi/v1/txt2img",
                        std::env::var("SD_URL").expect("Stable diffusion API URL is missing")
                    ))
                    .json(&serde_json::json!({
                        "steps": 20,
                        "sampler_name": "Euler a",
                        "width": 512,
                        "height": 768,
                        "hr_upscaler": "Latent",
                        "denoising_strength": 0.7,
                        "prompt": text,
                        "negative_prompt": "(worst quality, low quality:1.4), (zombie, sketch, interlocked fingers, comic)"
                    }))
                    .send()
                    .await
                {
                    Ok(res) => match res.json::<SdResponse>().await {
                        Ok(res) => {
                            if let Some(img) = res.images.first() {
                                match base64::engine::general_purpose::STANDARD.decode(img) {
                                    Ok(img) => {
                                        match args
                                            .bot
                                            .send_photo(args.msg.chat.id, InputFile::memory(img))
                                            .has_spoiler(args.nsfw)
                                            .reply_to_message_id(args.msg.id)
                                            .send()
                                            .await
                                        {
                                            Ok(msg) => {
                                                args.db.lock().await.add_caption(&msg, Some(&text));
                                                Some(msg)
                                            }
                                            Err(_) => None,
                                        }
                                    }
                                    Err(err) => args
                                        .bot
                                        .send_message(
                                            args.msg.chat.id,
                                            format!("Invalid base64 from SD API:\n{err}"),
                                        )
                                        .reply_to_message_id(args.msg.id)
                                        .send()
                                        .await
                                        .ok(),
                                }
                            } else {
                                args.bot
                                    .send_message(args.msg.chat.id, "Empty response from SD API")
                                    .reply_to_message_id(args.msg.id)
                                    .send()
                                    .await
                                    .ok()
                            }
                        }
                        Err(err) => args
                            .bot
                            .send_message(
                                args.msg.chat.id,
                                format!("Invalid response from SD API:\n{err}"),
                            )
                            .reply_to_message_id(args.msg.id)
                            .send()
                            .await
                            .ok(),
                    },
                    Err(err) => args
                        .bot
                        .send_message(args.msg.chat.id, format!("No response from SD API:\n{err}"))
                        .reply_to_message_id(args.msg.id)
                        .send()
                        .await
                        .ok(),
                }
            }
            Err(err) => args
                .bot
                .send_message(
                    args.msg.chat.id,
                    format!("No response from translation API:\n{err}"),
                )
                .reply_to_message_id(args.msg.id)
                .send()
                .await
                .ok(),
        }
    }
}
