use std::collections::HashMap;

use crate::mailersend::api::{Email, EmailAddress};
use crate::mailersend::buffer::Buffer;
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
    tags: Option<Vec<String>>,
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
    email_buffer: web::Data<Buffer>,
    messenger_req: web::Json<MessengerRequest>,
) -> Result<impl Responder> {
    log::info!("Received messenger request: {:?}", messenger_req);
    let mut tags = messenger_req.campaign.tags.clone().unwrap_or_default();
    tags.push(format!("campaign:{}", messenger_req.campaign.uuid));
    let from_address =
        EmailAddress::from_string(&messenger_req.campaign.from_email).expect("Invalid from email");
    let emails = messenger_req
        .recipients
        .iter()
        .filter(|recipient| {
            if recipient.status == "enabled" {
                return true;
            }
            log::info!(
                "Recipient {} is not enabled, skipping",
                recipient.email.clone()
            );
            false
        })
        .map(|recipient| Email {
            from: from_address.clone(),
            to: vec![EmailAddress::from_parts(
                recipient.name.clone(),
                &recipient.email,
            )],
            reply_to: None,
            subject: messenger_req.subject.clone(),
            text: None,
            html: Some(messenger_req.body.clone()),
            tags: tags.clone(),
        })
        .collect();
    email_buffer.push_all(emails).await;
    Ok(HttpResponse::Ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mailersend::api::EmailAddress;

    #[actix_rt::test]
    async fn test_messenger_handler() {
        let email_buffer = web::Data::new(Buffer::new());
        let messenger_req = web::Json(MessengerRequest {
            subject: "Test subject".to_string(),
            body: "<h1>Test</h1>".to_string(),
            content_type: "text/html".to_string(),
            recipients: vec![
                Recipient {
                    uuid: "123".to_string(),
                    email: "test@email.com".to_string(),
                    name: None,
                    status: "enabled".to_string(),
                },
                Recipient {
                    uuid: "456".to_string(),
                    email: "test2@email.com".to_string(),
                    name: Some("Test recipient".to_string()),
                    status: "enabled".to_string(),
                },
                Recipient {
                    uuid: "156".to_string(),
                    email: "test3@email.com".to_string(),
                    name: Some("Test recipient".to_string()),
                    status: "blocklisted".to_string(),
                },
            ],
            campaign: Campaign {
                uuid: "789".to_string(),
                name: "Test campaign".to_string(),
                from_email: "from@email.com".to_string(),
                headers: vec![],
                tags: None,
            },
        });
        messenger_handler(email_buffer.clone(), messenger_req)
            .await
            .unwrap();
        let emails = email_buffer.pop_all().await;
        assert_eq!(emails.len(), 2);
        assert_eq!(
            emails[0].from,
            EmailAddress::from_string("from@email.com").expect("Invalid from email")
        );

        assert_eq!(emails[0].to.len(), 1);
        assert_eq!(
            emails[0].to[0],
            EmailAddress::from_parts(None, "test@email.com")
        );
        assert_eq!(emails[0].reply_to, None);
        assert_eq!(emails[0].subject, "Test subject".to_string());
        assert_eq!(emails[0].text, None);
        assert_eq!(emails[0].html, Some("<h1>Test</h1>".to_string()));
        assert_eq!(emails[0].tags.len(), 1);
        assert_eq!(emails[0].tags[0], "campaign:789".to_string());
    }
}
