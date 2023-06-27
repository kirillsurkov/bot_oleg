use async_trait::async_trait;
use openai::chat::*;
use teloxide::prelude::*;

pub mod get_time;
pub use get_time::GetTime;

pub mod translate;
pub use translate::Translate;

pub mod draw;
pub use draw::Draw;

pub mod ban;
pub use ban::Ban;

#[async_trait]
pub trait OlegCommand<Args> {
    fn desc() -> ChatCompletionFunctionDefinition;
    async fn execute(args: Args) -> Option<Message>
    where
        Args: 'async_trait;
}