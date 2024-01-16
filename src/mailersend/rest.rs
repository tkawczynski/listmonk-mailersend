use crate::{
    listmonk::api::{BounceType, ListmonkAPI, ListmonkBounce},
    mailersend::api::EmailAddress,
};

use actix_web::{web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct RecipientData {
    object: String,
    id: String,
    email: String,
    created_at: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EmailData {
    object: String,
    id: String,
    created_at: String,
    from: String,
    subject: String,
    status: String,
    tags: Option<Vec<String>>,
    recipient: RecipientData,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct WebhookData {
    object: String,
    id: String,
    #[serde(rename(deserialize = "type"))]
    data_type: String,
    created_at: String,
    email: EmailData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WebhookRequest {
    #[serde(rename(deserialize = "type"))]
    request_type: String,
    domain_id: String,
    created_at: String,
    webhook_id: String,
    url: String,
    data: WebhookData,
}

pub async fn webhook_handler(
    listmonk_api: web::Data<ListmonkAPI>,
    payload: web::Json<WebhookRequest>,
) -> Result<impl Responder> {
    log::info!("Received webhook request: {:?}", payload);
    match payload.request_type.as_str() {
        "activity.soft_bounced" | "activity.hard_bounced" => {
            handle_bounce(listmonk_api, payload).await
        }
        "activity.spam_complaint" => handle_spam_complaint(listmonk_api, payload).await,
        _ => {
            log::info!("Ignoring webhook request");
            Ok(HttpResponse::Ok().body("OK")).into()
        }
    }
}

async fn handle_spam_complaint(
    listmonk_api: web::Data<ListmonkAPI>,
    payload: web::Json<WebhookRequest>,
) -> Result<HttpResponse> {
    log::info!("Received webhook request: {:?}", payload);
    let recipient_email = &payload.data.email.recipient.email;
    match listmonk_api
        .blocklist_by_email(EmailAddress::from_string(&recipient_email).expect("Invalid email"))
        .await
    {
        Ok(_) => {
            log::info!("Successfully blacklisted recipient");
            Ok(HttpResponse::Ok().body("OK"))
        }
        Err(e) => {
            log::error!("Failed to blacklist recipient: {}", e);
            Ok(HttpResponse::InternalServerError().body("Internal Server Error"))
        }
    }
}

async fn handle_bounce(
    listmonk_api: web::Data<ListmonkAPI>,
    payload: web::Json<WebhookRequest>,
) -> Result<HttpResponse> {
    log::info!("Received webhook request: {:?}", payload);
    let recipient_email = &payload.data.email.recipient.email;
    let bounce_type = if payload.request_type == "activity.soft_bounced" {
        BounceType::Soft
    } else {
        BounceType::Hard
    };
    let capaign_uuid_tag = payload
        .data
        .email
        .tags
        .as_ref()
        .and_then(|tags| tags.iter().find(|tag| tag.starts_with("campaign:")))
        .map(|tag| tag.replace("campaign:", ""));
    let meta = &payload.data.email.id;
    let mut listmonk_bounce = ListmonkBounce::new(recipient_email, bounce_type).with_meta(meta);
    if let Some(campaign_uuid) = capaign_uuid_tag {
        listmonk_bounce = listmonk_bounce.with_campaign_uuid(&campaign_uuid);
    }
    match listmonk_api.record_bounce(listmonk_bounce).await {
        Ok(_) => {
            log::info!("Successfully recorded bounce event");
            Ok(HttpResponse::Ok().body("OK"))
        }
        Err(e) => {
            log::error!("Failed to record bounce: {}", e);
            Ok(HttpResponse::InternalServerError().body("Internal Server Error"))
        }
    }
}
