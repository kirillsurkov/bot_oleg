use async_trait::async_trait;

pub struct BingSearch;

pub struct Args<'a> {
    pub query: &'a str,
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
impl<'a> super::Core<Args<'a>, Result<String, String>> for BingSearch {
    async fn execute(args: Args<'a>) -> Result<String, String> {
        let key = std::env::var("BING_API_KEY").expect("Bing API key is missing");
        match reqwest::Client::default()
            .get(
                reqwest::Url::parse_with_params(
                    "https://api.bing.microsoft.com/v7.0/search",
                    &[("q", args.query), ("textFormat", "HTML")],
                )
                .unwrap(),
            )
            .header("Ocp-Apim-Subscription-Key", key)
            .send()
            .await
        {
            Ok(response) => match response.json::<Response>().await {
                Ok(response) => Ok(serde_json::to_string(&response.web_pages.value[0..3]).unwrap()),
                Err(err) => Err(format!("Can't parse response:\n{err}")),
            },
            Err(err) => Err(format!("Request failed:\n{err}")),
        }
    }
}
