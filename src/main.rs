use std::sync::Arc;
use tokio::sync::Mutex;

use teloxide::prelude::*;
use teloxide::types::Update;

mod bot_command;
use bot_command::{BotCommand, Command};

mod db;
use db::DB;

#[tokio::main]
async fn main() {
    dotenv::from_filename("./res/.env").unwrap();

    let bot = Bot::new(std::env::var("BOT_TOKEN").expect("Telegram bot api key is missing"));
    let db = Arc::new(Mutex::new(DB::new()));

    let handler = Update::filter_message()
        .branch(dptree::entry().filter_command::<BotCommand>().endpoint({
            let db = db.clone();
            move |bot: Bot, msg: Message, cmd: BotCommand| {
                let db = db.clone();
                async move {
                    match cmd {
                        BotCommand::Help => bot_command::Help::execute(bot, msg, ()).await,
                        BotCommand::Rm => bot_command::Rm::execute(bot, msg, ()).await,
                        BotCommand::Translate { to_language, text } => {
                            bot_command::Translate::execute(
                                bot,
                                msg,
                                bot_command::translate::Args { to_language, text },
                            )
                            .await
                        }
                        BotCommand::Oleg => {
                            tokio::spawn(bot_command::Oleg::execute(
                                bot,
                                msg,
                                bot_command::oleg::Args { db },
                            ));
                        }
                        _ => {
                            bot.send_message(msg.chat.id, "Not supported yet")
                                .reply_to_message_id(msg.id)
                                .await
                                .ok();
                        }
                    };

                    respond(())
                }
            }
        }))
        .branch(dptree::entry().endpoint({
            let db = db.clone();
            move |bot: Bot, msg: Message| {
                let db = db.clone();
                async move {
                    if let Some(reply) = msg.reply_to_message() {
                        let db_msg = db.lock().await.get_message(reply.chat.id.0, reply.id.0);
                        if let Some(db_msg) = db_msg {
                            match db_msg.cause.as_str() {
                                "oleg_a" => {
                                    bot_command::Oleg::execute(
                                        bot,
                                        msg,
                                        bot_command::oleg::Args { db },
                                    )
                                    .await;
                                    return respond(());
                                }
                                _ => {}
                            }
                        }
                    } else if let Some(caption) = msg.caption() {
                        if caption.starts_with("/oleg")
                            && (caption.len() == 5
                                || (caption.len() > 5
                                    && caption.chars().nth(5).unwrap().is_whitespace()))
                        {
                            bot_command::Oleg::execute(bot, msg, bot_command::oleg::Args { db })
                                .await;
                            return respond(());
                        }
                    }
                    db.lock().await.add_message("", &msg);
                    respond(())
                }
            }
        }));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
