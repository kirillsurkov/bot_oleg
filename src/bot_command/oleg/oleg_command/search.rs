use super::OlegCommand;
use crate::bot_command::core::*;
use async_trait::async_trait;
use openai::chat::*;
use teloxide::prelude::*;

pub struct Search;

pub struct Args<'a> {
    pub query: &'a str,
}

#[async_trait]
impl<'a> OlegCommand<Args<'a>> for Search {
    fn desc() -> ChatCompletionFunctionDefinition {
        ChatCompletionFunctionDefinition {
            name: "search".to_owned(),
            description: Some("Web search".to_owned()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Web search query"
                    },
                },
                "required": ["query"],
            })),
        }
    }

    async fn execute(args: Args<'a>) -> (Option<Message>, Option<String>) {
        (
            None,
            Some(
                match BingSearch::execute(bing_search::Args { query: args.query }).await {
                    Ok(text) => text,
                    Err(err) => format!("Web search failed:\n{err:#}"),
                },
            ),
        )
    }
}
