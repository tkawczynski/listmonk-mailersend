use crate::{
    listmonk::api::{BounceType, ListmonkAPI, ListmonkBounce},
    Options,
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
    if payload.request_type != "activity.soft_bounced"
        && payload.request_type != "activity.hard_bounced"
    {
        return Ok(HttpResponse::Ok().body("OK"));
    }
    let recipient_email = &payload.data.email.recipient.email;
    let bounce_type = if payload.request_type == "activity.soft_bounced" {
        BounceType::Soft
    } else {
        BounceType::Hard
    };
    // let campaign_uuid = &payload.data.email.tags[0];
    let meta = &payload.data.email.id;
    let listmonk_bounce = ListmonkBounce::new(recipient_email, bounce_type)
        // .with_campaign_uuid(campaign_uuid)
        .with_meta(meta);
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
