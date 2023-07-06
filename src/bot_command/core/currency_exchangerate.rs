use anyhow::Context;
use async_trait::async_trait;

use std::collections::HashMap;

pub struct CurrencyExchangeRate;

pub struct Args<'a> {
    pub base: &'a str,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub date: String,
    pub rates: HashMap<String, f64>,
}

const SUPPORTED_CURRENCIES: &[&str] = &["USD", "EUR", "UAH", "RUB", "CNY"];

fn is_supported_currency(currency: &str) -> bool {
    SUPPORTED_CURRENCIES.contains(&currency)
}

#[async_trait]
impl<'a> super::Core<Args<'a>, anyhow::Result<Response>> for CurrencyExchangeRate {
    async fn execute(args: Args<'a>) -> anyhow::Result<Response> {
        let url = reqwest::Url::parse_with_params(
            "https://api.exchangerate.host/latest",
            &[("base", args.base)],
        )
        .unwrap();
        let response = reqwest::Client::default()
            .get(url)
            .send()
            .await
            .context("Request failed")?;
        let mut response = response
            .json::<Response>()
            .await
            .context("can't parse response")?;
        response
            .rates
            .retain(|currency, _| is_supported_currency(currency));
        Ok(response)
    }
}
