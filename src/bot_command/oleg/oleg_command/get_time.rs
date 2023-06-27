use super::OlegCommand;
use async_trait::async_trait;
use chrono::Local;
use openai::chat::*;
use teloxide::prelude::*;

pub struct GetTime;

pub struct Args<'a> {
    pub bot: &'a Bot,
    pub msg: &'a Message,
}

#[async_trait]
impl<'a> OlegCommand<Args<'a>> for GetTime {
    fn desc() -> ChatCompletionFunctionDefinition {
        ChatCompletionFunctionDefinition {
            name: "get_time".to_owned(),
            description: Some("Get current time".to_owned()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {},
                "required": [],
            })),
        }
    }

    async fn execute(args: Args<'a>) -> Option<Message> {
        args.bot
            .send_message(
                args.msg.chat.id,
                Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            )
            .reply_to_message_id(args.msg.id)
            .send()
            .await
            .ok()
    }
}
