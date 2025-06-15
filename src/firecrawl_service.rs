use crate::errors::OEmbedError;
use crate::models::OEmbedResponse;
use log::debug;
use reqwest::Client;
use serde_json;
use std::env;
use url::Url;

#[derive(Clone)]
pub struct FirecrawlService {
    client: Client,
    api_token: String,
}

impl FirecrawlService {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let api_token = env::var("FIRECRAWL_API_TOKEN")
            .expect("FIRECRAWL_API_TOKEN must be set in environment variables");

        Self {
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::default())
                .build()
                .unwrap_or_else(|_| Client::new()),
            api_token,
        }
    }

    pub async fn firecrawl_extract(&self, url: &Url) -> Result<OEmbedResponse, OEmbedError> {
        let execution_id = self.extract(url).await?;

        let response = self.get_extract_result(&execution_id, url).await?;

        Ok(response)
    }

    async fn extract(&self, url: &Url) -> Result<String, OEmbedError> {
        let post_response = self
            .client
            .post("https://api.firecrawl.dev/v1/extract")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&serde_json::json!({
                "urls": [url.as_str()],
                "prompt": "Extract the article title, article image, description, provider url and provider name.",
                "schema": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "description": { "type": "string" },
                        "provider_url": { "type": "string" },
                        "provider_name": { "type": "string" },
                        "article_image": { "type": "string" }
                    },
                    "required": ["title", "article_image"]
                }
            }))
            .send()
            .await
            .map_err(|e| OEmbedError::InvalidUrl(format!("Failed to start extraction: {}", e)))?;

        let post_result: serde_json::Value = post_response.json().await.map_err(|e| {
            OEmbedError::InvalidUrl(format!("Failed to parse extraction response: {}", e))
        })?;

        let execution_id = post_result["id"]
            .as_str()
            .ok_or_else(|| OEmbedError::InvalidUrl("No execution ID in response".to_string()))?;

        Ok(execution_id.to_string())
    }
    async fn get_extract_result(
        &self,
        execution_id: &str,
        url: &Url,
    ) -> Result<OEmbedResponse, OEmbedError> {
        let mut attempts = 0;
        let max_attempts = 10;
        let delay = std::time::Duration::from_secs(2);

        loop {
            if attempts >= max_attempts {
                return Err(OEmbedError::InvalidUrl(
                    "Max polling attempts reached".to_string(),
                ));
            }

            let get_response = self
                .client
                .get(format!(
                    "https://api.firecrawl.dev/v1/extract/{}",
                    execution_id
                ))
                .header("Authorization", format!("Bearer {}", self.api_token))
                .send()
                .await
                .map_err(|e| {
                    OEmbedError::InvalidUrl(format!("Failed to get extraction status: {}", e))
                })?;

            let status_result: serde_json::Value = get_response.json().await.map_err(|e| {
                OEmbedError::InvalidUrl(format!("Failed to parse status response: {}", e))
            })?;

            let status = status_result["status"]
                .as_str()
                .ok_or_else(|| OEmbedError::InvalidUrl("No status in response".to_string()))?;

            match status {
                "completed" => {
                    let data = status_result["data"].as_object().ok_or_else(|| {
                        OEmbedError::InvalidUrl("No data in completed response".to_string())
                    })?;

                    debug!("Data: {:?}", data);

                    return Ok(OEmbedResponse {
                        response_type: "rich".to_string(),
                        version: "1.0".to_string(),
                        title: data["title"].as_str().map(String::from),
                        author_name: None,
                        author_url: None,
                        provider_name: data["provider_name"].as_str().map(String::from),
                        provider_url: data["provider_url"].as_str().map(String::from),
                        thumbnail_url: data["article_image"].as_str().map(String::from),
                        thumbnail_width: None,
                        thumbnail_height: None,
                        html: Some(format!(
                            r#"<div><h3>{}</h3><p>{}</p><a href="{}" target="_blank">View Original</a></div>"#,
                            data["title"].as_str().unwrap_or("Untitled"),
                            data["description"].as_str().unwrap_or(""),
                            url.as_str()
                        )),
                        width: None,
                        height: None,
                    });
                }
                "failed" | "cancelled" => {
                    return Err(OEmbedError::InvalidUrl(format!(
                        "Extraction {}: {}",
                        status, execution_id
                    )));
                }
                _ => {
                    tokio::time::sleep(delay).await;
                    attempts += 1;
                }
            }
        }
    }
}
