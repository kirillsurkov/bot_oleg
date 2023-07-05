use async_trait::async_trait;
use base64::Engine;
use std::sync::Arc;
use teloxide::{net::Download, prelude::*};
use tokio::sync::Mutex;

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
impl super::Core<Args, Result<String, String>> for SdWhat {
    async fn execute(args: Args) -> Result<String, String> {
        if let Some(file_id) = args.file_id {
            let caption = args
                .db
                .lock()
                .await
                .get_caption(&file_id)
                .ok()
                .flatten();
            match caption {
                Some(caption) => Ok(caption),
                None => match args.bot.get_file(&file_id).await {
                    Ok(file) => {
                        let mut img = vec![];
                        match args.bot.download_file(&file.path, &mut img).await {
                                Ok(_) => match reqwest::Client::new()
                                    .post(format!(
                                        "{}/sdapi/v1/interrogate",
                                        std::env::var("SD_URL")
                                            .expect("Stable diffusion API URL is missing"),
                                    ))
                                    .json(&serde_json::json!({
                                        "model": "clip",
                                        "image": format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(img))
                                    }))
                                    .send()
                                    .await
                                {
                                    Ok(res) => match res.json::<Caption>().await {
                                        Ok(caption) => {
                                            args.db
                                                .lock()
                                                .await
                                                .add_caption(&file_id, Some(&caption.caption));
                                            Ok(caption.caption)
                                        }
                                        Err(err) => Err(format!("Can't parse interrogate response:\n{err}"))
                                    },
                                    Err(err) => Err(format!("Interrogate failed:\n{err}"))
                                },
                                Err(err) => Err(format!("Download image failed:\n{err}"))
                            }
                    }
                    Err(err) => Err(format!("Get file failed:\n{err}")),
                },
            }
        } else {
            Err("No photo to interrogate".to_owned())
        }
    }
}
