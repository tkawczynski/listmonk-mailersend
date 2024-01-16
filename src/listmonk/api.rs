use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::mailersend::api::EmailAddress;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Error, Debug)]
pub enum ListmonkApiError {
    #[error("Listmonk webhook failed: {0}")]
    WebhookError(String),

    #[error("Listmonk API error: {0}")]
    ApiError(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BounceType {
    #[serde(rename = "hard")]
    Hard,
    #[serde(rename = "soft")]
    Soft,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListmonkBounce {
    email: Option<String>,
    campaign_uuid: Option<String>,
    source: String,
    #[serde(rename = "type")]
    bounce_type: BounceType,
    meta: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryBlocklistRequest {
    query: String,
}

impl ListmonkBounce {
    pub fn new(email: &str, bounce_type: BounceType) -> Self {
        return ListmonkBounce {
            email: Some(email.to_string()),
            campaign_uuid: None,
            source: "mailersend".to_string(),
            bounce_type,
            meta: None,
        };
    }

    pub fn with_campaign_uuid(mut self, campaign_uuid: &str) -> Self {
        self.campaign_uuid = Some(campaign_uuid.to_string());
        return self;
    }

    pub fn with_meta(mut self, meta: &str) -> Self {
        self.meta = Some(meta.to_string());
        return self;
    }
}

#[derive(Clone)]
pub struct ListmonkAPI {
    http_client: Client,
    api_endpoint: String,
    api_username: String,
    api_password: String,
}

impl ListmonkAPI {
    pub fn new(api_endpoint: &str, api_username: &str, api_password: &str) -> Self {
        let http_client = Client::new();
        return ListmonkAPI {
            http_client,
            api_endpoint: api_endpoint.to_string(),
            api_username: api_username.to_string(),
            api_password: api_password.to_string(),
        };
    }

    pub async fn record_bounce(&self, record: ListmonkBounce) -> Result<()> {
        let request = self
            .http_client
            .post(&format!("{}/webhooks/bounce", self.api_endpoint))
            .basic_auth(&self.api_username, Some(&self.api_password))
            .json(&record);
        log::info!("Sending request: {:?}", request);
        let response = request.send().await?;
        let response_status = response.status();
        if !response_status.is_success() {
            log::error!("Listmonk API request failed: {:?}", response);
            let response_message = response.text().await?;
            return Err(ListmonkApiError::WebhookError(format!(
                "Listmonk API request failed: {} {}",
                response_status, response_message
            ))
            .into());
        }
        log::info!("Listmonk API request successful");
        Ok(())
    }

    pub async fn blocklist_by_email(&self, email: EmailAddress) -> Result<()> {
        let request = self
            .http_client
            .put(&format!(
                "{}/api/subscribers/query/blocklist",
                self.api_endpoint
            ))
            .basic_auth(&self.api_username, Some(&self.api_password))
            .json(&QueryBlocklistRequest {
                query: format!("subscribers.email LIKE '{}'", email.email()),
            });
        log::info!("Sending request: {:?}", request);
        let response = request.send().await?;
        let response_status = response.status();
        if !response_status.is_success() {
            log::error!("Listmonk API request failed: {:?}", response);
            let response_message = response.text().await?;
            return Err(ListmonkApiError::ApiError(format!(
                "Listmonk API request failed: {} {}",
                response_status, response_message
            ))
            .into());
        }
        log::info!("Listmonk API request successful");
        Ok(())
    }
}
