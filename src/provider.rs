use crate::errors::OEmbedError;
use crate::models::{OEmbedRequest, OEmbedResponse, ProviderConfig};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use url::Url;

#[derive(Clone)]
pub struct Provider {
    providers: HashMap<String, ProviderConfig>,
    client: Client,
}

impl Provider {
    pub fn new() -> Self {
        let mut providers = HashMap::new();

        providers.insert(
            "youtube.com".to_string(),
            ProviderConfig {
                oembed_endpoint: Some(Url::parse("https://www.youtube.com/oembed").unwrap()),
                url_patterns: vec!["youtube.com/watch?v=".to_string(), "youtu.be/".to_string()],
            },
        );
        providers.insert(
            "x.com".to_string(),
            ProviderConfig {
                oembed_endpoint: Some(Url::parse("https://publish.twitter.com/oembed").unwrap()),
                url_patterns: vec!["x.com/".to_string(), "twitter.com/".to_string()],
            },
        );

        Self {
            providers,
            client: Client::new(),
        }
    }

    pub async fn get_oembed(
        &self,
        request: OEmbedRequest,
    ) -> Result<Option<OEmbedResponse>, OEmbedError> {
        let parsed_url =
            Url::parse(&request.url).map_err(|e| OEmbedError::InvalidUrl(e.to_string()))?;

        let result = self.try_known_provider(request).await?;

        if result.is_none() {
            return self.parse_html(&parsed_url).await.map(Some);
        }

        Ok(result)
    }

    pub async fn try_known_provider(
        &self,
        request: OEmbedRequest,
    ) -> Result<Option<OEmbedResponse>, OEmbedError> {
        let parsed_url =
            Url::parse(&request.url).map_err(|e| OEmbedError::InvalidUrl(e.to_string()))?;
        println!("parsed_url: {:?}", parsed_url);
        let host = parsed_url
            .host_str()
            .ok_or_else(|| OEmbedError::InvalidUrl("No host in URL".to_string()))?;

        println!("host: {:?}", host);

        for (domain, provider) in &self.providers {
            if host.contains(domain)
                || provider
                    .url_patterns
                    .iter()
                    .any(|pattern| request.url.as_str().contains(pattern))
            {
                if let Some(endpoint) = &provider.oembed_endpoint {
                    return Ok(self.fetch_oembed(endpoint, &parsed_url).await);
                }
            }
        }

        // Err(OEmbedError::UnsupportedProvider(host.to_string()))
        Ok(None)
    }

    fn extract_meta_content(document: &Html, selectors: &[&str]) -> Option<String> {
        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    if selector_str.starts_with("title") {
                        return Some(
                            element
                                .text()
                                .collect::<Vec<_>>()
                                .join("")
                                .trim()
                                .to_string(),
                        );
                    } else if let Some(content) = element.value().attr("content") {
                        return Some(content.to_string());
                    }
                }
            }
        }
        None
    }

    async fn fetch_oembed(&self, endpoint: &Url, url: &Url) -> Option<OEmbedResponse> {
        println!("fetch_oembed: {:?} {:?}", endpoint.as_str(), url.as_str());
        let query_params = vec![("url", url.as_str()), ("format", "json")];

        let response = self
            .client
            .get(endpoint.as_str())
            .query(&query_params)
            .send()
            .await
            .ok()?;

        println!("response: {:?}", response.status());

        if response.status().is_success() {
            response.json::<OEmbedResponse>().await.ok()
        } else {
            None
        }
    }

    async fn parse_html(&self, url: &Url) -> Result<OEmbedResponse, OEmbedError> {
        // Validate and parse the URL
        let host = url
            .host_str()
            .ok_or_else(|| OEmbedError::InvalidUrl("No host in URL".to_string()))
            .map_err(|e| OEmbedError::InvalidUrl(e.to_string()))?;

        // Featch the URL's HTML
        let response = self
            .client
            .get(url.as_str())
            .send()
            .await
            .map_err(|e| OEmbedError::InvalidUrl(e.to_string()))?;

        if !response.status().is_success() {
            return Err(OEmbedError::InvalidUrl("No host in URL".to_string()));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| OEmbedError::InvalidUrl(e.to_string()))?;

        // Parse HTML
        let document = Html::parse_document(&response_text);

        let title = Self::extract_meta_content(
            &document,
            &[
                "meta[property='og:title']",
                "meta[name='twitter:title']",
                "title",
            ],
        );

        let description = Self::extract_meta_content(
            &document,
            &[
                "meta[property='og:description']",
                "meta[name='twitter:description']",
                "meta[name='description']",
            ],
        );

        let thumbnail = Self::extract_meta_content(
            &document,
            &["meta[property='og:image']", "meta[name='twitter:image']"],
        );

        let site_name = Self::extract_meta_content(
            &document,
            &["meta[property='og:site_name']", "meta[name='twitter:site']"],
        );

        Ok(OEmbedResponse {
            response_type: "rich".to_string(),
            version: "1.0".to_string(),
            title: title.clone(),
            author_name: None,
            author_url: None,
            provider_name: site_name,
            provider_url: Some(format!("https://{}", host)),
            thumbnail_url: thumbnail,
            thumbnail_width: None,
            thumbnail_height: None,
            html: Some(format!(
                r#"<div><h3>{}</h3><p>{}</p><a href="{}" target="_blank">View Original</a></div>"#,
                title.as_deref().unwrap_or("Untitled"),
                description.as_deref().unwrap_or(""),
                url.as_str()
            )),
            width: None,
            height: None,
        })
    }
}
