use async_trait::async_trait;
use once_cell::sync::Lazy;
use openai::chat::*;
use teloxide::prelude::*;
use tokio::sync::Mutex;

use std::sync::Arc;

mod oleg_command;
use oleg_command::OlegCommand;

pub struct Oleg;

pub struct Args {
    pub sd_draw: Arc<Mutex<super::core::SdDraw>>,
    pub db: Arc<Mutex<crate::DB>>,
    pub http_client: reqwest::Client,
    pub translator: Arc<crate::Translator>,
    pub settings: Arc<crate::Settings>,
}

static FUNCTION_DESCS: Lazy<[ChatCompletionFunctionDefinition; 6]> = Lazy::new(|| {
    [
        oleg_command::GetTime::desc(),
        oleg_command::Translate::desc(),
        oleg_command::Draw::desc(),
        oleg_command::Recognize::desc(),
        oleg_command::Search::desc(),
        oleg_command::ExchangeRates::desc(),
    ]
});

async fn get_answer(bot: &Bot, msg: &Message, args: &Args) -> Result<Option<Message>, String> {
    let mut messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(args.settings.oleg_prompt.clone()),
        name: None,
        function_call: None,
    }];

    messages.extend(
        args.db
            .lock()
            .await
            .unwind_thread(msg, args.settings.oleg_memory_size, |text| {
                if let Some(command) = text.strip_prefix(crate::BOT_COMMAND_PREFIX) {
                    !command.trim().is_empty()
                } else {
                    !text.is_empty()
                }
            })
            .iter()
            .map(|m| {
                let role = match m.cause.as_str() {
                    "oleg_a" => ChatCompletionMessageRole::Assistant,
                    "oleg_f" => ChatCompletionMessageRole::Function,
                    _ => ChatCompletionMessageRole::User,
                };
                ChatCompletionMessage {
                    role,
                    content: match role {
                        ChatCompletionMessageRole::User => m.text.as_ref().map(|text| {
                            format!(
                                "{}: {}",
                                m.sender.clone().unwrap(),
                                text.strip_prefix(crate::BOT_COMMAND_PREFIX).map_or(&text[..], |s| s.trim()),
                            )
                        }),
                        ChatCompletionMessageRole::Function => {
                            m.function_res.as_ref().map(|f| f.res.clone())
                        }
                        _ => m.text.clone(),
                    },
                    name: m
                        .function_req
                        .as_ref()
                        .map(|f| f.name.clone())
                        .or(m.function_res.as_ref().map(|f| f.name.clone())),
                    function_call: m.function_req.as_ref().map(|f| ChatCompletionFunctionCall {
                        name: f.name.clone(),
                        arguments: f.args.clone(),
                    }),
                }
            }),
    );

    println!("{messages:#?}");

    let functions = &*FUNCTION_DESCS;
    let completion = ChatCompletion::builder("gpt-3.5-turbo-0613", messages.clone())
        .functions(functions.clone())
        .create()
        .await;

    println!("{completion:?}");
    let completion = completion.map_err(|err| format!("Completion error:\n{err}"))?;

    let completion = completion.choices[0].message.clone();
    if let Some(text) = completion.content.as_ref() {
        return Ok(bot
            .send_message(
                msg.chat.id,
                text.split_once("###ID###").map_or(&text[..], |s| s.0),
            )
            .reply_to_message_id(msg.id)
            .send()
            .await
            .ok());
    }

    let Some(function) = completion.function_call else {
        return Err("Empty response".to_owned());
    };

    args.db
        .lock()
        .await
        .add_function(msg, &function.name, Some(&function.arguments), None);
    let (answer, function_response) = match function.name.as_str() {
        "get_time" => {
            let args: serde_json::Value =
                serde_json::from_str(&function.arguments).unwrap_or_default();
            oleg_command::GetTime::execute(oleg_command::get_time::Args {
                offset_m: args["offset_minutes"].as_i64().unwrap_or_default() as i32,
                offset_h: args["offset_hours"].as_i64().unwrap_or_default() as i32,
            })
            .await
        }
        "translate" => {
            let func_args: serde_json::Value =
                serde_json::from_str(&function.arguments).unwrap_or_default();
            oleg_command::Translate::execute(oleg_command::translate::Args {
                bot,
                msg,
                to_language: func_args["to_language"].as_str().unwrap_or_default(),
                text: func_args["text"].as_str().unwrap_or_default(),
                translator: &args.translator,
                settings: &args.settings,
            })
            .await
        }
        "draw" => {
            let cmd_args: serde_json::Value =
                serde_json::from_str(&function.arguments).unwrap_or_default();
            oleg_command::Draw::execute(oleg_command::draw::Args {
                bot,
                msg,
                sd_draw: args.sd_draw.clone(),
                db: args.db.clone(),
                description: cmd_args["description"].as_str().unwrap_or_default(),
                nsfw: cmd_args["nsfw"].as_bool().unwrap_or_default(),
                http_client: &args.http_client,
                translator: &args.translator,
                settings: &args.settings,
            })
            .await
        }
        "ban" => oleg_command::Ban::execute(oleg_command::ban::Args { bot, msg }).await,
        "recognize" => {
            let cmd_args: serde_json::Value =
                serde_json::from_str(&function.arguments).unwrap_or_default();
            oleg_command::Recognize::execute(oleg_command::recognize::Args {
                bot,
                msg,
                db: args.db.clone(),
                file_id: cmd_args["file_id"].as_str().unwrap_or_default(),
                http_client: &args.http_client,
                settings: &args.settings,
            })
            .await
        }
        "search" => {
            let cmd_args: serde_json::Value =
                serde_json::from_str(&function.arguments).unwrap_or_default();
            oleg_command::Search::execute(oleg_command::search::Args {
                query: cmd_args["query"].as_str().unwrap_or_default(),
                http_client: &args.http_client,
                settings: &args.settings,
            })
            .await
        }
        "exchange_rates" => {
            let cmd_args: serde_json::Value =
                serde_json::from_str(&function.arguments).unwrap_or_default();
            oleg_command::ExchangeRates::execute(oleg_command::exchange_rates::Args {
                base: cmd_args["base"].as_str().unwrap_or_default(),
                http_client: &args.http_client,
            })
            .await
        }
        _ => (None, None),
    };

    if let Some(function_response) = function_response {
        args.db
            .lock()
            .await
            .add_function(msg, &function.name, None, Some(&function_response));
        Ok(None)
    } else {
        Ok(answer)
    }
}

#[async_trait]
impl super::Command<Args> for Oleg {
    async fn execute(bot: Bot, msg: Message, args: Args) {
        openai::set_key(args.settings.openai_token.clone());

        let mut max_iter = 5;

        let answer = loop {
            if max_iter <= 0 {
                break None;
            }
            max_iter -= 1;

            match get_answer(&bot, &msg, &args).await {
                Ok(answer) => match answer {
                    Some(answer) => break Some(answer),
                    None => continue,
                },
                Err(err) => {
                    break bot
                        .send_message(msg.chat.id, err)
                        .reply_to_message_id(msg.id)
                        .send()
                        .await
                        .ok()
                }
            }
        };

        let db = args.db.lock().await;
        db.add_message("oleg_q", &msg);
        if let Some(answer) = answer {
            db.add_message("oleg_a", &answer);
        }
    }
}
