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

pub mod recognize;
pub use recognize::Recognize;

pub mod search;
pub use search::Search;

pub mod exchange_rates;
pub use exchange_rates::ExchangeRates;

#[async_trait]
pub trait OlegCommand<Args> {
    fn desc() -> ChatCompletionFunctionDefinition;
    async fn execute(args: Args) -> (Option<Message>, Option<String>)
    where
        Args: 'async_trait;
}
