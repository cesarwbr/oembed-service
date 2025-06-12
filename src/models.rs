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
    pub width: Option<u32>,
    pub height: Option<u32>,
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
