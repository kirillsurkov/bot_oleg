use google_translate3::{hyper, hyper_rustls, oauth2};
use std::sync::Arc;
use tokio::sync::Mutex;

use teloxide::prelude::*;
use teloxide::types::Update;

mod bot_command;
use bot_command::{BotCommand, Command};

mod db;
use db::DB;

pub mod fmt;

mod settings;
use settings::Settings;

type Translator =
    google_translate3::Translate<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;

#[tokio::main]
async fn main() {
    dotenv::from_filename("./res/.env").unwrap();

    let settings = Arc::new(envy::from_env::<Settings>().unwrap());
    let bot = Bot::new(&settings.bot_token);
    let db = Arc::new(Mutex::new(DB::new()));
    let sd_draw = Arc::new(Mutex::new(bot_command::core::SdDraw::default()));
    let http_client = reqwest::Client::new();
    let google_account = &settings.google_service_account_json;
    let service_account_key = oauth2::read_service_account_key(format!("./res/{google_account}",))
        .await
        .unwrap();
    let auth = oauth2::ServiceAccountAuthenticator::builder(service_account_key)
        .build()
        .await
        .unwrap();
    let hyper_client = hyper::Client::builder().build(
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build(),
    );
    let translator: Arc<Translator> = Arc::new(google_translate3::Translate::new(
        hyper_client,
        auth,
    ));

    let handler = Update::filter_message()
        .branch(dptree::entry().filter_command::<BotCommand>().endpoint({
            let db = db.clone();
            let sd_draw = sd_draw.clone();
            let http_client = http_client.clone();
            let translator = translator.clone();
            let settings = settings.clone();
            move |bot: Bot, msg: Message, cmd: BotCommand| {
                let db = db.clone();
                let sd_draw = sd_draw.clone();
                let http_client = http_client.clone();
                let translator = translator.clone();
                let settings = settings.clone();
                async move {
                    match cmd {
                        BotCommand::Help => bot_command::Help::execute(bot, msg, ()).await,
                        BotCommand::Rm => bot_command::Rm::execute(bot, msg, ()).await,
                        BotCommand::Translate { to_language, text } => {
                            bot_command::Translate::execute(
                                bot,
                                msg,
                                bot_command::translate::Args {
                                    to_language,
                                    text,
                                    translator: &translator,
                                    settings: &settings,
                                },
                            )
                            .await
                        }
                        BotCommand::Oleg => {
                            tokio::spawn(bot_command::Oleg::execute(
                                bot,
                                msg,
                                bot_command::oleg::Args {
                                    sd_draw: sd_draw.clone(),
                                    db,
                                    http_client,
                                    translator,
                                    settings,
                                },
                            ));
                        }
                        BotCommand::Sd { description } => {
                            tokio::spawn(bot_command::Sd::execute(
                                bot,
                                msg,
                                bot_command::sd::Args {
                                    sd_draw: sd_draw.clone(),
                                    db,
                                    description,
                                    http_client,
                                    translator,
                                    settings,
                                },
                            ));
                        }
                        BotCommand::What => {
                            bot_command::What::execute(
                                bot,
                                msg,
                                bot_command::what::Args {
                                    db,
                                    http_client: &http_client,
                                    settings: &settings,
                                },
                            )
                            .await;
                        }
                        BotCommand::Find { query } => {
                            bot_command::Find::execute(
                                bot,
                                msg,
                                bot_command::find::Args {
                                    query,
                                    http_client: &http_client,
                                    settings: &settings,
                                },
                            )
                            .await
                        }
                    };

                    respond(())
                }
            }
        }))
        .branch(dptree::entry().endpoint({
            let db = db.clone();
            let sd_draw = sd_draw.clone();
            let http_client = http_client.clone();
            let translator = translator.clone();
            let settings = settings.clone();
            move |bot: Bot, msg: Message| {
                let db = db.clone();
                let sd_draw = sd_draw.clone();
                let http_client = http_client.clone();
                let translator = translator.clone();
                let settings = settings.clone();
                async move {
                    if let Some(reply) = msg.reply_to_message() {
                        if msg
                            .text()
                            .or(msg.caption())
                            .map_or(true, |t| !t.starts_with("/q"))
                        {
                            let db_msg = db.lock().await.get_message(reply.chat.id.0, reply.id.0);
                            if let Some(db_msg) = db_msg {
                                match db_msg.cause.as_str() {
                                    "oleg_a" => {
                                        bot_command::Oleg::execute(
                                            bot,
                                            msg,
                                            bot_command::oleg::Args {
                                                sd_draw: sd_draw.clone(),
                                                db,
                                                http_client,
                                                translator,
                                                settings,
                                            },
                                        )
                                        .await;
                                        return respond(());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else if let Some(caption) = msg.caption() {
                        if caption.starts_with("/oleg")
                            && (caption.len() == 5
                                || (caption.len() > 5
                                    && caption.chars().nth(5).unwrap().is_whitespace()))
                        {
                            bot_command::Oleg::execute(
                                bot,
                                msg,
                                bot_command::oleg::Args {
                                    sd_draw: sd_draw.clone(),
                                    db,
                                    http_client,
                                    translator,
                                    settings,
                                },
                            )
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
