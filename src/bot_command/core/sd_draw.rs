use anyhow::{anyhow, Context};
use async_trait::async_trait;
use base64::Engine;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use super::*;

#[derive(Default)]
pub struct SdDraw {
    timeouts: HashMap<i64, std::time::Instant>,
}

pub struct Args<'a> {
    pub instance: Arc<Mutex<SdDraw>>,
    pub description: &'a str,
    pub msg: &'a teloxide::types::Message,
    pub http_client: &'a reqwest::Client,
}

#[derive(serde::Deserialize)]
struct SdResponse {
    images: Vec<String>,
}

#[async_trait]
impl<'a> super::Core<Args<'a>, anyhow::Result<Vec<u8>>> for SdDraw {
    async fn execute(args: Args<'a>) -> anyhow::Result<Vec<u8>> {
        let sd_timeout = std::env::var("SD_TIMEOUT")
            .expect("Stable diffusion timeout is missing")
            .parse::<u64>()
            .expect("Can't parse stable diffusion timeout as u64");

        let sd_timeout_list = std::env::var("SD_TIMEOUT_LIST")
            .expect("Stable diffusion timeout list is missing")
            .split(',')
            .map(|id| {
                id.parse::<i64>()
                    .expect("ID in stable diffusion timeout list can't be parsed")
            })
            .collect::<Vec<_>>();

        if args
            .instance
            .lock()
            .await
            .timeouts
            .get(&args.msg.chat.id.0)
            .map_or(false, |time| {
                sd_timeout_list.contains(&args.msg.chat.id.0)
                    && time.elapsed().as_secs() < sd_timeout
            })
        {
            use strfmt::*;
            let chat_timeout = args.instance.lock().await.timeouts[&args.msg.chat.id.0]
                .elapsed()
                .as_secs();
            let timeout = sd_timeout - chat_timeout;
            return Err(match std::env::var("SD_TIMEOUT_MESSAGE") {
                Ok(msg) => anyhow!(strfmt!(&msg, timeout).unwrap()),
                Err(_) => anyhow!("Stable diffusion timeout message is missing"),
            });
        }

        args.instance
            .lock()
            .await
            .timeouts
            .insert(args.msg.chat.id.0, std::time::Instant::now());

        let translated_prompt = GoogleTranslate::execute(google_translate::Args {
            to_language: "en",
            text: args.description,
        })
        .await
        .context("no response from translation API")?;

        let SdResponse { images } = args
            .http_client
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
                "prompt": format!("(masterpiece, best quality), {}", translated_prompt),
                "negative_prompt": "EasyNegativeV2"
            }))
            .send()
            .await
            .context("no response from SD API")?
            .json::<SdResponse>()
            .await
            .context("invalid response from SD API")?;

        let image = images
            .first()
            .ok_or_else(|| anyhow!("Empty response from SD API"))?;
        let res = base64::engine::general_purpose::STANDARD
            .decode(image)
            .context("Invalid base64 from SD API")?;
        Ok(res)
    }
}
