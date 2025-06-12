use thiserror::Error;

#[derive(Error, Debug)]
pub enum OEmbedError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("HTTP request failed: {0}")]
    HTTPError(#[from] reqwest::Error),
}
