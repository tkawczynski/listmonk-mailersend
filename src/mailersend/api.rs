use super::throttler::Throttler;
use actix_rt;
use actix_rt::task::JoinHandle;
use futures::future::join_all;
use lazy_static::lazy_static;
use log;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

lazy_static! {
    static ref RAW_EMAIL_REGEX: Regex =
        Regex::new(r"(?<mailbox>[^><\s@]+)@(?<domain>([^><\s@.,]+\.)+[^><\s@.,]{2,})").unwrap();
    static ref EMAIL_REGEX: Regex =
        Regex::new(r"(?<name>[^<]*)?<(?<mailbox>[^\s@]+)@(?<domain>([^\s@.,]+\.)+[^\s@.,]{2,})>",)
            .unwrap();
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailAddress {
    name: Option<String>,
    email: String,
}

impl EmailAddress {
    pub fn from_parts(name: Option<String>, email: &str) -> Self {
        let name = name.map_or(String::new(), |x| x.trim().to_string());
        return EmailAddress {
            name: if name.len() == 0 { None } else { Some(name) },
            email: email.to_string(),
        };
    }

    pub fn from_string(input: &str) -> Self {
        log::info!("Parsing email address: {}", input);
        let groups = EMAIL_REGEX.captures(input).unwrap_or_else(|| {
            RAW_EMAIL_REGEX
                .captures(input)
                .expect("Invalid email address")
        });
        let name = groups
            .name("name")
            .map_or(None, |x| Some(x.as_str().trim()));
        let mailbox = groups.name("mailbox").unwrap().as_str();
        let domain = groups.name("domain").unwrap().as_str();
        return EmailAddress {
            name: name.map(|x| x.to_string()),
            email: format!("{}@{}", mailbox, domain),
        };
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Email {
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub reply_to: Option<EmailAddress>,
    pub subject: String,
    pub text: Option<String>,
    pub html: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
struct ChunkResult {
    api_response_status: u16,
    api_response_message: String,
    emails_count: usize,
}

#[derive(Clone)]
pub struct MailerSendAPI {
    http_client: Client,
    api_endpoint: String,
    api_key: String,
    throttler: Arc<Mutex<Throttler>>,
}

impl MailerSendAPI {
    pub fn new(api_endpoint: &str, api_key: &str, req_per_min: u32) -> Self {
        MailerSendAPI {
            http_client: Client::new(),
            api_endpoint: api_endpoint.to_string(),
            api_key: api_key.to_string(),
            throttler: Arc::new(Mutex::new(Throttler::new(req_per_min))),
        }
    }

    pub async fn send_bulk(&self, emails: Vec<Email>, bulk_size: usize) -> Result<()> {
        log::info!("Sending {} emails in bulk", emails.len());
        let chunks: Vec<&[Email]> = emails.chunks(bulk_size as usize).collect();
        log::info!("Split emails list into {} chunks", chunks.len());
        let chunk_results = join_all(chunks.iter().map(|x| self.send_bulk_chunk(x.to_vec()))).await;
        log::info!("All MailerSend API requests finished");
        let mut errors = String::new();
        let mut had_errors = false;
        for result in chunk_results {
            match result.unwrap() {
                Ok(res) => {
                    log::info!("MailerSend API response: {:?}", res);
                    if res.api_response_status < 200 || res.api_response_status >= 300 {
                        had_errors = true;
                        errors.push_str(&format!(
                            "MailerSend API response: {} {}\n",
                            res.api_response_status, res.api_response_message
                        ));
                    }
                }
                Err(err) => {
                    had_errors = true;
                    errors.push_str(&format!("MailerSend API request failed: {}\n", err));
                }
            }
        }
        if had_errors {
            Err(errors.into())
        } else {
            Ok(())
        }
    }

    fn send_bulk_chunk(&self, emails_vec: Vec<Email>) -> JoinHandle<Result<ChunkResult>> {
        log::info!("Sending {} emails in chunk", emails_vec.len());
        let client = self.http_client.clone();
        let api_endpoint = format!("{}/bulk-email", self.api_endpoint);
        let api_key = self.api_key.clone();
        let throttler = self.throttler.clone();
        actix_rt::spawn(async move {
            log::info!("Throttling MailerSend API request");
            throttler
                .lock()
                .unwrap()
                .try_blocking(Duration::from_secs(120));
            log::info!("Sending MailerSend API request");
            let res = client
                .post(&api_endpoint)
                .json(&emails_vec)
                .header("Content-Type", "application/json")
                .header("X-Requested-With", "XMLHttpRequest")
                .bearer_auth(api_key)
                .send()
                .await;
            match res {
                Ok(res) => {
                    log::info!("MailerSend API response: {:?}", res);
                    Ok(ChunkResult {
                        api_response_message: res.status().to_string(),
                        emails_count: emails_vec.len(),
                        api_response_status: res.status().into(),
                    })
                }
                Err(err) => {
                    log::error!("MailerSend API request failed: {}", err);
                    Err(err.into())
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_address_from_string() {
        let email = EmailAddress::from_string("John Doe <john_doe@mail.com>");
        assert_eq!(email.name, Some("John Doe".to_string()));
        assert_eq!(email.email, "john_doe@mail.com".to_string());
    }

    #[test]
    fn test_email_address_from_string_no_name() {
        let email = EmailAddress::from_string("john_doe@mail.com");
        assert_eq!(email.name, None);
        assert_eq!(email.email, "john_doe@mail.com".to_string());
    }

    #[test]
    fn test_email_address_from_parts() {
        let email = EmailAddress::from_parts(Some("John Doe".to_string()), "john_doe@mail.com");
        assert_eq!(email.name, Some("John Doe".to_string()));
        assert_eq!(email.email, "john_doe@mail.com".to_string());
    }

    #[test]
    fn test_email_address_from_parts_no_name() {
        let email = EmailAddress::from_parts(None, "john_doe@mail.com");
        assert_eq!(email.name, None);
        assert_eq!(email.email, "john_doe@mail.com".to_string());
    }
}
