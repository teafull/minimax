use crate::error::{Error, Result};
use crate::types::{Model, ModelList};

pub struct Client {
    api_key: String,
    base_url: String,
}

impl Client {
    pub fn new(api_key: &str) -> Result<Self> {
        Self::with_base_url(api_key, "https://api.minimaxi.com/v1")
    }

    pub fn with_base_url(api_key: &str, base_url: &str) -> Result<Self> {
        if api_key.is_empty() {
            return Err(Error::MissingApiKey);
        }
        Ok(Self {
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
        })
    }

    pub fn chat(&self) -> crate::chat::ChatBuilder<'_> {
        crate::chat::ChatBuilder::new(self)
    }

    pub fn models(&self) -> Models<'_> {
        Models::new(self)
    }

    pub fn request<T>(&self, request: reqwest::blocking::RequestBuilder) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = request
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()?;

        if !response.status().is_success() {
            let code = response.status().as_u16() as i32;
            let message = response.text().unwrap_or_default();
            return Err(Error::Api { code, message });
        }

        response.json().map_err(|e| Error::InvalidResponse(e.to_string()))
    }

    pub(crate) fn send_streaming_request(&self, request: reqwest::blocking::RequestBuilder) -> Result<String> {
        let response = request
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()?;

        if !response.status().is_success() {
            let code = response.status().as_u16() as i32;
            let message = response.text().unwrap_or_default();
            return Err(Error::Api { code, message });
        }

        response.text().map_err(|e| Error::InvalidResponse(e.to_string()))
    }

    pub fn http_client(&self) -> reqwest::blocking::Client {
        reqwest::blocking::Client::builder()
            .build()
            .expect("Failed to create HTTP client")
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

pub struct Models<'a> {
    client: &'a Client,
}

impl<'a> Models<'a> {
    fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub fn list(&self) -> Result<ModelList> {
        let url = format!("{}/models", self.client.base_url());
        let request = self.client.http_client().get(&url);
        self.client.request(request)
    }

    pub fn get(&self, model_id: &str) -> Result<Model> {
        let url = format!("{}/models/{}", self.client.base_url(), model_id);
        let request = self.client.http_client().get(&url);
        self.client.request(request)
    }
}
