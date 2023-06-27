use super::OlegCommand;
use async_trait::async_trait;
use openai::chat::*;
use teloxide::prelude::*;

pub struct Ban;

pub struct Args<'a> {
    pub bot: &'a Bot,
    pub msg: &'a Message,
}

#[async_trait]
impl<'a> OlegCommand<Args<'a>> for Ban {
    fn desc() -> ChatCompletionFunctionDefinition {
        ChatCompletionFunctionDefinition {
            name: "ban".to_owned(),
            description: Some("Restrict user for bad behaviour".to_owned()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {},
                "required": [],
            })),
        }
    }

    async fn execute(args: Args<'a>) -> Option<Message> {
        args.bot
            .send_message(args.msg.chat.id, "/ban")
            .reply_to_message_id(args.msg.id)
            .send()
            .await
            .ok()
    }
}
