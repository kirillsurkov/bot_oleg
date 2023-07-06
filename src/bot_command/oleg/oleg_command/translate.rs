use super::OlegCommand;
use crate::bot_command::core::*;
use async_trait::async_trait;
use openai::chat::*;
use teloxide::prelude::*;

pub struct Translate;

pub struct Args<'a> {
    pub bot: &'a Bot,
    pub msg: &'a Message,
    pub to_language: &'a str,
    pub text: &'a str,
}

#[async_trait]
impl<'a> OlegCommand<Args<'a>> for Translate {
    fn desc() -> ChatCompletionFunctionDefinition {
        ChatCompletionFunctionDefinition {
            name: "translate".to_owned(),
            description: Some("Translate text".to_owned()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "to_language": {
                        "type": "string",
                        "description": "BCP-47 language code"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text to translate"
                    }
                },
                "required": ["text", "to_language"],
            })),
        }
    }

    async fn execute(args: Args<'a>) -> (Option<Message>, Option<String>) {
        (
            None,
            Some(
                match GoogleTranslate::execute(google_translate::Args {
                    to_language: args.to_language,
                    text: args.text,
                })
                .await
                {
                    Ok(text) => text,
                    Err(err) => format!("Failed to translate:\n{err:#}"),
                },
            ),
        )
    }
}
