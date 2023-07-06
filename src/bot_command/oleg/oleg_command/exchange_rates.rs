use super::OlegCommand;
use crate::bot_command::core::*;
use async_trait::async_trait;
use openai::chat::*;
use teloxide::prelude::*;

pub struct ExchangeRates;

pub struct Args<'a> {
    pub base: &'a str,
}

#[async_trait]
impl<'a> OlegCommand<Args<'a>> for ExchangeRates {
    fn desc() -> ChatCompletionFunctionDefinition {
        ChatCompletionFunctionDefinition {
            name: "exchange_rates".to_owned(),
            description: Some("Get exchange rates".to_owned()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "base": {
                        "type": "string",
                        "description": "Base currency in format e.g. USD, EUR"
                    },
                },
                "required": ["base"],
            })),
        }
    }

    async fn execute(args: Args<'a>) -> (Option<Message>, Option<String>) {
        (
            None,
            Some(
                match CurrencyExchangeRate::execute(currency_exchangerate::Args { base: args.base })
                    .await
                {
                    Ok(response) => serde_json::to_string(&response).unwrap(),
                    Err(err) => format!("Get exchange rates failed:\n{err:#}"),
                },
            ),
        )
    }
}
