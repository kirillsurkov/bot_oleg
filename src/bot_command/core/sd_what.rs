use anyhow::{anyhow, Context};
use async_trait::async_trait;
use base64::Engine;
use teloxide::{net::Download, prelude::*};
use tokio::sync::Mutex;

use std::sync::Arc;

pub struct SdWhat;

pub struct Args<'a> {
    pub db: Arc<Mutex<crate::DB>>,
    pub bot: Bot,
    pub file_id: Option<&'a str>,
    pub http_client: &'a reqwest::Client,
    pub settings: &'a crate::Settings,
}

#[derive(serde::Deserialize)]
struct Caption {
    caption: String,
}

fn request_body(encoded_image: &str) -> impl serde::Serialize + 'static {
    return Body {
        model: "clip",
        image: format!("data:image/png;base64,{encoded_image}"),
    };

    #[derive(serde::Serialize)]
    struct Body {
        model: &'static str,
        image: String,
    }
}

#[async_trait]
impl<'a> super::Core<Args<'a>, anyhow::Result<String>> for SdWhat {
    async fn execute(args: Args<'a>) -> anyhow::Result<String> {
        let file_id = args
            .file_id
            .ok_or_else(|| anyhow!("No photo to interrogate"))?;
        if let Ok(Some(caption)) = args.db.lock().await.get_caption(file_id) {
            return Ok(caption);
        }

        let file = args
            .bot
            .get_file(file_id)
            .await
            .context("getting file failed")?;
        let mut img = vec![];
        args.bot
            .download_file(&file.path, &mut img)
            .await
            .context("downloading image failed")?;

        let sd_url = &args.settings.sd_url;
        let encoded_image = base64::engine::general_purpose::STANDARD.encode(img);
        let Caption { caption } = args
            .http_client
            .post(format!("{sd_url}/sdapi/v1/interrogate"))
            .json(&request_body(&encoded_image))
            .send()
            .await
            .context("interrogation failed")?
            .json::<Caption>()
            .await
            .context("can't parse interrogate response")?;

        args.db.lock().await.add_caption(file_id, Some(&caption));
        Ok(caption)
    }
}
