use serde::Deserialize;

pub type SdId = i64;

#[derive(Deserialize)]
pub struct Settings {
    pub bot_token: String,
    pub oleg_prompt: String,
    pub oleg_memory_size: usize,
    pub openai_token: String,
    pub bing_api_key: String,
    pub google_service_account_json: String,
    pub sd_timeout: u64,
    #[serde(deserialize_with = "deserialize_sd_timeout_list")]
    pub sd_timeout_list: Vec<SdId>,
    pub sd_timeout_message: Option<String>,
    pub sd_url: String,
}

fn deserialize_sd_timeout_list<'de, D>(de: D) -> Result<Vec<SdId>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    String::deserialize(de)?
        .split(',')
        .map(std::str::FromStr::from_str)
        .collect::<Result<_, _>>()
        .map_err(serde::de::Error::custom)
}
