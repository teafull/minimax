use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("API error: {code} - {message}")]
    Api { code: i32, message: String },

    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Missing API key")]
    MissingApiKey,
}

pub type Result<T> = std::result::Result<T, Error>;
