use std::collections::HashMap;

use async_trait::async_trait;

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

#[async_trait]
impl<'a> super::Core<Args<'a>, Result<Response, String>> for CurrencyExchangeRate {
    async fn execute(args: Args<'a>) -> Result<Response, String> {
        match reqwest::Client::default()
            .get(
                reqwest::Url::parse_with_params(
                    "https://api.exchangerate.host/latest",
                    &[("base", args.base)],
                )
                .unwrap(),
            )
            .send()
            .await
        {
            Ok(response) => match response.json::<Response>().await {
                Ok(mut response) => {
                    response.rates = response
                        .rates
                        .iter()
                        .filter_map(|(k, v)| {
                            if ["USD", "EUR", "UAH", "RUB", "CNY"].contains(&k.as_str()) {
                                Some((k.clone(), *v))
                            } else {
                                None
                            }
                        })
                        .collect();
                    Ok(response)
                }
                Err(err) => Err(format!("Can't parse response:\n{err}")),
            },
            Err(err) => Err(format!("Request failed:\n{err}")),
        }
    }
}
