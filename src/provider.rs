use crate::errors::OEmbedError;
use crate::firecrawl_service::FirecrawlService;
use crate::models::{OEmbedRequest, OEmbedResponse, ProviderConfig};
use log::debug;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json;
use std::collections::HashMap;
use url::Url;

#[derive(Clone)]
pub struct Provider {
    providers: HashMap<String, ProviderConfig>,
    client: Client,
    firecrawl_service: FirecrawlService,
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

        providers.insert(
            "vimeo.com".to_string(),
            ProviderConfig {
                oembed_endpoint: Some(Url::parse("https://vimeo.com/api/oembed.json").unwrap()),
                url_patterns: vec!["vimeo.com/".to_string()],
            },
        );

        providers.insert(
            "tiktok.com".to_string(),
            ProviderConfig {
                oembed_endpoint: Some(Url::parse("https://www.tiktok.com/oembed").unwrap()),
                url_patterns: vec!["tiktok.com/".to_string()],
            },
        );

        providers.insert(
            "spotify.com".to_string(),
            ProviderConfig {
                oembed_endpoint: Some(Url::parse("https://open.spotify.com/oembed").unwrap()),
                url_patterns: vec![
                    "open.spotify.com/track/".to_string(),
                    "open.spotify.com/album/".to_string(),
                    "open.spotify.com/playlist/".to_string(),
                    "open.spotify.com/show/".to_string(),
                    "open.spotify.com/episode/".to_string(),
                ],
            },
        );

        providers.insert(
            "soundcloud.com".to_string(),
            ProviderConfig {
                oembed_endpoint: Some(Url::parse("https://soundcloud.com/oembed").unwrap()),
                url_patterns: vec!["soundcloud.com/".to_string()],
            },
        );

        providers.insert(
            "github.com".to_string(),
            ProviderConfig {
                oembed_endpoint: Some(Url::parse("https://github.com/api/oembed").unwrap()),
                url_patterns: vec!["github.com/".to_string(), "gist.github.com/".to_string()],
            },
        );

        providers.insert(
            "flickr.com".to_string(),
            ProviderConfig {
                oembed_endpoint: Some(
                    Url::parse("https://www.flickr.com/services/oembed").unwrap(),
                ),
                url_patterns: vec!["flickr.com/photos/".to_string(), "flic.kr/p/".to_string()],
            },
        );

        providers.insert(
            "medium.com".to_string(),
            ProviderConfig {
                oembed_endpoint: Some(Url::parse("https://medium.com/oembed").unwrap()),
                url_patterns: vec!["medium.com/".to_string()],
            },
        );

        Self {
            providers,
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::default())
                .build()
                .unwrap_or_else(|_| Client::new()),
            firecrawl_service: FirecrawlService::new(),
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
            let html_result = self.parse_html(&parsed_url).await?;

            // If we're missing essential fields, try firecrawl
            if html_result.thumbnail_url.is_none() || html_result.title.is_none() {
                match self.firecrawl_service.firecrawl_extract(&parsed_url).await {
                    Ok(response) => return Ok(Some(response)),
                    Err(_) => return Ok(Some(html_result)),
                }
            }

            return Ok(Some(html_result));
        }

        Ok(result)
    }

    pub async fn try_known_provider(
        &self,
        request: OEmbedRequest,
    ) -> Result<Option<OEmbedResponse>, OEmbedError> {
        let parsed_url =
            Url::parse(&request.url).map_err(|e| OEmbedError::InvalidUrl(e.to_string()))?;
        let host = parsed_url
            .host_str()
            .ok_or_else(|| OEmbedError::InvalidUrl("No host in URL".to_string()))?;

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
        let query_params = vec![("url", url.as_str()), ("format", "json")];

        let response = self
            .client
            .get(endpoint.as_str())
            .query(&query_params)
            .send()
            .await
            .ok()?;

        if response.status().is_success() {
            // Get the response text first for logging
            let response_text = response.text().await.ok()?;

            // Try to parse the JSON using reqwest's built-in functionality
            match serde_json::from_str::<OEmbedResponse>(&response_text) {
                Ok(json) => Some(json),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    async fn parse_html(&self, url: &Url) -> Result<OEmbedResponse, OEmbedError> {
        debug!("Parsing HTML for URL: {}", url);
        // Validate and parse the URL
        let host = url
            .host_str()
            .ok_or_else(|| OEmbedError::InvalidUrl("No host in URL".to_string()))
            .map_err(|e| OEmbedError::InvalidUrl(e.to_string()))?;

        debug!("Host: {}", host);

        // Fetch the URL's HTML with redirect following enabled and proper headers
        let response = self
            .client
            .get(url.as_str())
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Connection", "keep-alive")
            .send()
            .await
            .map_err(|e| OEmbedError::InvalidUrl(format!("Failed to fetch HTML: {}", e)))?;

        debug!("Response: {:?}", response.status());
        debug!("Response URL: {:?}", response.url());

        if !response.status().is_success() {
            return Err(OEmbedError::InvalidUrl(format!(
                "Failed to fetch HTML: HTTP {}",
                response.status()
            )));
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
