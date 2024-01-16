use std::collections::HashMap;

use crate::config::Configuration;
use crate::mailersend::api::{Email, EmailAddress, MailerSendAPI};
use actix_web::{web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Recipient {
    uuid: String,
    email: String,
    name: Option<String>,
    status: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Campaign {
    uuid: String,
    name: String,
    from_email: String,
    headers: Vec<HashMap<String, String>>,
    tags: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MessengerRequest {
    subject: String,
    body: String,
    content_type: String,
    recipients: Vec<Recipient>,
    campaign: Campaign,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MessengerResponse {
    status: String,
    message: Option<String>,
    data: Option<String>,
}

pub async fn messenger_handler(
    app_config: web::Data<Configuration>,
    mailersend_api: web::Data<MailerSendAPI>,
    messenger_req: web::Json<MessengerRequest>,
) -> Result<impl Responder> {
    log::info!("Received messenger request: {:?}", messenger_req);
    let emails = messenger_req
        .recipients
        .iter()
        .map(|recipient| Email {
            from: EmailAddress::from_string(&messenger_req.campaign.from_email),
            to: vec![EmailAddress::from_parts(
                recipient.name.clone(),
                &recipient.email,
            )],
            reply_to: None,
            subject: messenger_req.subject.clone(),
            text: None,
            html: Some(messenger_req.body.clone()),
            tags: messenger_req.campaign.tags.clone(),
        })
        .collect();
    match mailersend_api
        .send_bulk(emails, app_config.api_email_bulk_size)
        .await
    {
        Ok(_) => {
            log::info!("Successfully sent messenger request");
            Ok(HttpResponse::Ok().json(MessengerResponse {
                status: String::from("success"),
                message: None,
                data: None,
            }))
        }

        Err(e) => {
            log::error!("Failed to send messenger request: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(MessengerResponse {
                status: String::from("error"),
                message: Some(String::from("Failed to send messenger request")),
                data: None,
            }))
        }
    }
}
