use anyhow::{anyhow, Context};
use async_trait::async_trait;
use base64::Engine;
use teloxide::{net::Download, prelude::*};
use tokio::sync::Mutex;

use std::sync::Arc;

pub struct SdWhat;

pub struct Args {
    pub db: Arc<Mutex<crate::DB>>,
    pub bot: Bot,
    pub file_id: Option<String>,
}

#[derive(serde::Deserialize)]
struct Caption {
    caption: String,
}

#[async_trait]
impl super::Core<Args, anyhow::Result<String>> for SdWhat {
    async fn execute(args: Args) -> anyhow::Result<String> {
        let file_id = args
            .file_id
            .ok_or_else(|| anyhow!("No photo to interrogate"))?;
        if let Ok(Some(caption)) = args.db.lock().await.get_caption(&file_id) {
            return Ok(caption);
        }

        let file = args
            .bot
            .get_file(&file_id)
            .await
            .context("getting file failed")?;
        let mut img = vec![];
        args.bot
            .download_file(&file.path, &mut img)
            .await
            .context("downloading image failed")?;

        let sd_url = std::env::var("SD_URL").expect("Stable diffusion API URL is missing");
        let encoded_image = base64::engine::general_purpose::STANDARD.encode(img);
        let Caption { caption } = reqwest::Client::new()
            .post(format!("{sd_url}/sdapi/v1/interrogate"))
            .json(&serde_json::json!({
                "model": "clip",
                "image": format!("data:image/png;base64,{encoded_image}")
            }))
            .send()
            .await
            .context("interrogation failed")?
            .json::<Caption>()
            .await
            .context("can't parse interrogate response")?;

        args.db.lock().await.add_caption(&file_id, Some(&caption));
        Ok(caption)
    }
}
