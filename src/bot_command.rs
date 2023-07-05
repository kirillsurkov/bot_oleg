use async_trait::async_trait;
use teloxide::{
    prelude::*,
    utils::command::{BotCommands, ParseError},
};

pub mod core;

pub mod help;
pub use help::Help;

pub mod rm;
pub use rm::Rm;

pub mod oleg;
pub use oleg::Oleg;

pub mod translate;
pub use translate::Translate;

pub mod sd;
pub use sd::Sd;

pub mod what;
pub use what::What;

fn parse_translate(input: String) -> Result<(String, String), ParseError> {
    if let Some(cmd) = input.split_once(" ") {
        Ok((cmd.0.to_owned(), cmd.1.to_owned()))
    } else {
        Err(ParseError::TooFewArguments {
            expected: 2,
            found: 1,
            message: "".to_owned(),
        })
    }
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum BotCommand {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Remove message sent by bot")]
    Rm,
    #[command(description = "Run stable diffusion generation")]
    Sd { description: String },
    #[command(description = "Get photo description")]
    What,
    #[command(description = "Translate text", parse_with = parse_translate)]
    Translate { to_language: String, text: String },
    #[command(description = "Ask a question")]
    Oleg,
}

#[async_trait]
pub trait Command<Args> {
    async fn execute(bot: Bot, msg: Message, args: Args)
    where
        Args: 'async_trait;
}
