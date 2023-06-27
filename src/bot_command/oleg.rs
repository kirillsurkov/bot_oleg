use async_trait::async_trait;
use openai::chat::*;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;

mod oleg_command;
use oleg_command::OlegCommand;

pub struct Oleg;

pub struct Args {
    pub db: Arc<Mutex<crate::DB>>,
}

#[async_trait]
impl super::Command<Args> for Oleg {
    async fn execute(bot: Bot, msg: Message, args: Args) {
        openai::set_key(std::env::var("OPENAI_TOKEN").expect("OpenAI api key is missing"));

        let mut messages = vec![ChatCompletionMessage {
            role: ChatCompletionMessageRole::System,
            content: Some(std::env::var("OLEG_PROMPT").expect("Oleg prompt is missing")),
            name: None,
            function_call: None,
        }];
        messages.extend(
            args.db
                .lock()
                .await
                .unwind_thread(
                    &msg,
                    std::env::var("OLEG_MEMORY_SIZE")
                        .expect("Oleg memory size is missing")
                        .parse::<usize>()
                        .expect("Can't parse Oleg memory size as usize"),
                    |text| {
                        if text.starts_with("/oleg") {
                            text[5..].trim().len() > 0
                        } else {
                            text.len() > 0
                        }
                    },
                )
                .iter()
                .map(|m| {
                    let role = match m.cause.as_str() {
                        "oleg_a" => ChatCompletionMessageRole::Assistant,
                        _ => ChatCompletionMessageRole::User,
                    };
                    ChatCompletionMessage {
                        role,
                        content: Some(match role {
                            ChatCompletionMessageRole::User => {
                                format!(
                                    "{}: {}",
                                    m.sender.clone(),
                                    if m.text.starts_with("/oleg") {
                                        m.text[5..].trim()
                                    } else {
                                        &m.text
                                    }
                                )
                            }
                            _ => m.text.clone(),
                        }),
                        name: None,
                        function_call: None,
                    }
                }),
        );

        println!("{:#?}", messages);

        let completion = ChatCompletion::builder("gpt-3.5-turbo-0613", messages.clone())
            .functions([
                oleg_command::GetTime::desc(),
                oleg_command::Translate::desc(),
                oleg_command::Draw::desc(),
                oleg_command::Ban::desc(),
            ])
            .create()
            .await;

        println!("{:?}", completion);

        let answer = {
            match completion {
                Ok(completion) => {
                    let completion = completion.choices.first().unwrap().message.clone();
                    if let Some(text) = completion.content {
                        bot.send_message(
                            msg.chat.id,
                            if text.starts_with("Олег:") {
                                &text["Олег:".len()..]
                            } else if text.starts_with("Oleg:") {
                                &text["Oleg:".len()..]
                            } else {
                                &text[..]
                            }
                            .to_owned(),
                        )
                        .reply_to_message_id(msg.id)
                        .send()
                        .await
                        .ok()
                    } else if let Some(function) = completion.function_call {
                        match function.name.as_str() {
                            "get_time" => {
                                oleg_command::GetTime::execute(oleg_command::get_time::Args {
                                    bot: &bot,
                                    msg: &msg,
                                })
                                .await
                            }
                            "translate" => {
                                let args: serde_json::Value =
                                    serde_json::from_str(&function.arguments).unwrap_or_default();
                                oleg_command::Translate::execute(oleg_command::translate::Args {
                                    bot: &bot,
                                    msg: &msg,
                                    to_language: args["to_language"].as_str().unwrap_or_default(),
                                    text: args["text"].as_str().unwrap_or_default(),
                                })
                                .await
                            }
                            "draw" => {
                                let args: serde_json::Value =
                                    serde_json::from_str(&function.arguments).unwrap_or_default();
                                oleg_command::Draw::execute(oleg_command::draw::Args {
                                    bot: &bot,
                                    msg: &msg,
                                    description: args["description"].as_str().unwrap_or_default(),
                                    nsfw: args["nsfw"].as_bool().unwrap_or_default(),
                                })
                                .await
                            }
                            "ban" => {
                                oleg_command::Ban::execute(oleg_command::ban::Args {
                                    bot: &bot,
                                    msg: &msg,
                                })
                                .await
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                Err(err) => bot
                    .send_message(msg.chat.id, format!("Completion error:\n{err}"))
                    .reply_to_message_id(msg.id)
                    .send()
                    .await
                    .ok(),
            }
        };

        let db = args.db.lock().await;
        db.add_message("oleg_q", &msg);
        if let Some(answer) = answer {
            db.add_message("oleg_a", &answer);
        }
    }
}
