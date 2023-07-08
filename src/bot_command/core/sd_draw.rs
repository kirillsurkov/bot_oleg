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
    pub settings: &'a crate::Settings,
}

#[derive(serde::Deserialize)]
struct SdResponse {
    images: Vec<String>,
}

#[async_trait]
impl<'a> super::Core<Args<'a>, anyhow::Result<Vec<u8>>> for SdDraw {
    async fn execute(args: Args<'a>) -> anyhow::Result<Vec<u8>> {
        let sd_timeout = args.settings.sd_timeout;
        let sd_timeout_list = &args.settings.sd_timeout_list;

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
            return Err(match args.settings.sd_timeout_message.as_ref() {
                Some(msg) => anyhow!(strfmt!(&msg, timeout).unwrap()),
                None => anyhow!("Stable diffusion timeout message is missing"),
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
            settings: &args.settings,
        })
        .await
        .context("no response from translation API")?;

        let SdResponse { images } = args
            .http_client
            .post(format!(
                "{}/sdapi/v1/txt2img",
                args.settings.sd_url,
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
