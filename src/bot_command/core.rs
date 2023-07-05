use async_trait::async_trait;

pub mod google_translate;
pub use google_translate::GoogleTranslate;

pub mod sd_draw;
pub use sd_draw::SdDraw;

pub mod sd_what;
pub use sd_what::SdWhat;

pub mod bing_search;
pub use bing_search::BingSearch;

pub mod currency_exchangerate;
pub use currency_exchangerate::CurrencyExchangeRate;

#[async_trait]
pub trait Core<Args, Ret> {
    async fn execute(args: Args) -> Ret
    where
        Args: 'async_trait;
}
