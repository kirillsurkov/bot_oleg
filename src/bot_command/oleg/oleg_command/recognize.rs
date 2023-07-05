use super::OlegCommand;
use crate::bot_command::core::*;
use async_trait::async_trait;
use openai::chat::*;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;

pub struct Recognize;

pub struct Args<'a> {
    pub bot: &'a Bot,
    pub msg: &'a Message,
    pub db: Arc<Mutex<crate::DB>>,
    pub file_id: &'a str,
}

#[async_trait]
impl<'a> OlegCommand<Args<'a>> for Recognize {
    fn desc() -> ChatCompletionFunctionDefinition {
        ChatCompletionFunctionDefinition {
            name: "recognize".to_owned(),
            description: Some("Recognize image".to_owned()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "file_id": {
                        "type": "string",
                        "description": "File id"
                    }
                },
                "required": ["file_id"],
            })),
        }
    }

    async fn execute(args: Args<'a>) -> (Option<Message>, Option<String>) {
        (
            None,
            Some(
                match SdWhat::execute(sd_what::Args {
                    db: args.db,
                    bot: args.bot.clone(),
                    file_id: Some(args.file_id.to_owned()),
                })
                .await
                {
                    Ok(caption) => caption,
                    Err(err) => err,
                },
            ),
        )
    }
}
