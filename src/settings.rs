use serde::{de, Deserialize, Deserializer};

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
    #[serde(deserialize_with = "deserialize_formatter")]
    pub sd_timeout_message: Option<crate::fmt::Formatter>,
    pub sd_url: String,
}

fn deserialize_sd_timeout_list<'de, D>(de: D) -> Result<Vec<SdId>, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(de)?
        .split(',')
        .map(std::str::FromStr::from_str)
        .collect::<Result<_, _>>()
        .map_err(de::Error::custom)
}

fn deserialize_formatter<'de, D>(de: D) -> Result<Option<crate::fmt::Formatter>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(de)?
        .map(|raw| raw.parse().map_err(de::Error::custom))
        .transpose()
}
