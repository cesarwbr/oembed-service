use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct OEmbedResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub version: String,
    pub title: Option<String>,
    pub author_name: Option<String>,
    pub author_url: Option<String>,
    pub provider_name: Option<String>,
    pub provider_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub thumbnail_width: Option<u32>,
    pub thumbnail_height: Option<u32>,
    pub html: Option<String>,
    #[serde(deserialize_with = "deserialize_width_height")]
    pub width: Option<u32>,
    #[serde(deserialize_with = "deserialize_width_height")]
    pub height: Option<u32>,
}

fn deserialize_width_height<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(u32),
    }

    match Option::<StringOrInt>::deserialize(deserializer)? {
        Some(StringOrInt::String(val)) => {
            // Remove the % sign if present and parse as u32
            let clean_val = val.trim_end_matches('%');
            clean_val.parse::<u32>().map(Some).map_err(Error::custom)
        }
        Some(StringOrInt::Int(val)) => Ok(Some(val)),
        None => Ok(None),
    }
}

#[derive(Deserialize)]
pub struct OEmbedRequest {
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct ProviderConfig {
    pub oembed_endpoint: Option<Url>,
    pub url_patterns: Vec<String>,
}
