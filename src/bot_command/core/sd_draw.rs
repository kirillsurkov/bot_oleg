use super::*;
use async_trait::async_trait;
use base64::Engine;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct SdDraw {
    timeouts: HashMap<i64, std::time::Instant>,
}

pub struct Args<'a> {
    pub instance: Arc<Mutex<SdDraw>>,
    pub description: &'a str,
    pub msg: &'a teloxide::types::Message,
}

#[derive(serde::Deserialize)]
struct SdResponse {
    images: Vec<String>,
}

#[async_trait]
impl<'a> super::Core<Args<'a>, Result<Vec<u8>, String>> for SdDraw {
    async fn execute(args: Args<'a>) -> Result<Vec<u8>, String> {
        if args
            .instance
            .lock()
            .await
            .timeouts
            .get(&args.msg.chat.id.0)
            .map(|time| {
                time.elapsed().as_secs()
                    >= std::env::var("SD_TIMEOUT")
                        .expect("Stable diffusion timeout is missing")
                        .parse::<u64>()
                        .expect("Can't parse stable diffusion timeout as u64")
            })
            .unwrap_or(true)
        {
            args.instance
                .lock()
                .await
                .timeouts
                .insert(args.msg.chat.id.0, std::time::Instant::now());
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
                            "steps": 25,
                            "cfg_scale": 10.0,
                            "sampler_name": "Euler a",
                            "width": 512,
                            "height": 1024,
                            "prompt": format!("(masterpiece, best quality), {}", text),
                            "negative_prompt": "EasyNegativeV2"
                        }))
                        .send()
                        .await
                    {
                        Ok(res) => match res.json::<SdResponse>().await {
                            Ok(res) => {
                                if let Some(img) = res.images.first() {
                                    match base64::engine::general_purpose::STANDARD.decode(img) {
                                        Ok(img) => Ok(img),
                                        Err(err) => {
                                            Err(format!("Invalid base64 from SD API:\n{err}"))
                                        }
                                    }
                                } else {
                                    Err("Empty response from SD API".to_owned())
                                }
                            }
                            Err(err) => Err(format!("Invalid response from SD API:\n{err}")),
                        },
                        Err(err) => Err(format!("No response from SD API:\n{err}")),
                    }
                }
                Err(err) => Err(format!("No response from translation API:\n{err}")),
            }
        } else {
            Err(match std::env::var("SD_TIMEOUT_MESSAGE") {
                Ok(msg) => msg,
                Err(_) => "Stable diffusion timeout message is missing".to_owned(),
            })
        }
    }
}
