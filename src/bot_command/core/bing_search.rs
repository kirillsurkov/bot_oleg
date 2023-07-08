use anyhow::Context;
use async_trait::async_trait;
pub struct BingSearch;
pub struct Args<'a> {
    pub query: &'a str,
    pub http_client: &'a reqwest::Client,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct WebPage {
    url: String,
    snippet: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct WebPages {
    value: Vec<WebPage>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    web_pages: WebPages,
}

#[async_trait]
impl<'a> super::Core<Args<'a>, anyhow::Result<String>> for BingSearch {
    async fn execute(args: Args<'a>) -> anyhow::Result<String> {
        let key = std::env::var("BING_API_KEY").expect("Bing API key is missing");
        let url = reqwest::Url::parse_with_params(
            "https://api.bing.microsoft.com/v7.0/search",
            &[("q", args.query), ("textFormat", "HTML")],
        )
        .unwrap();
        let response = args.http_client
            .get(url)
            .header("Ocp-Apim-Subscription-Key", key)
            .send()
            .await
            .context("Request failed")?;
        let response = response
            .json::<Response>()
            .await
            .context("Can't parse response")?;
        Ok(serde_json::to_string(&response.web_pages.value[0..3]).unwrap())
    }
}
