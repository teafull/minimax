use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("API key is missing")]
    MissingApiKey,

    #[error("API error {code}: {message}")]
    Api { code: i32, message: String },

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
