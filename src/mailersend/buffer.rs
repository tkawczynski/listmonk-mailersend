use std::sync::Arc;

use futures::lock::Mutex;

use super::api::Email;

#[derive(Clone)]
pub struct Buffer {
    emails: Arc<Mutex<Vec<Email>>>,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            emails: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn push_all(&self, emails_vec: Vec<Email>) {
        let mut emails = self.emails.lock().await;
        emails.extend(emails_vec);
    }

    pub async fn pop_all(&self) -> Vec<Email> {
        let mut emails = self.emails.lock().await;
        let mut result = Vec::new();
        std::mem::swap(&mut result, &mut emails);
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::mailersend::api::EmailAddress;

    use super::*;

    #[actix_rt::test]
    async fn test_buffer() {
        let buffer = Buffer::new();
        let emails = vec![
            Email {
                from: EmailAddress::from_parts(None, "testemail@email.com"),
                to: vec![EmailAddress::from_parts(None, "recipient@email.com")],
                reply_to: None,
                subject: "Test subject".to_string(),
                text: None,
                html: Some("<h1>Test</h1>".to_string()),
                tags: vec!["test".to_string()],
            },
            Email {
                from: EmailAddress::from_parts(None, "testemail@email.com"),
                to: vec![EmailAddress::from_parts(None, "recipient2@email.com")],
                reply_to: None,
                subject: "Test subject".to_string(),
                text: None,
                html: Some("<h1>Test</h1>".to_string()),
                tags: vec!["test".to_string()],
            },
        ];
        buffer.push_all(emails.clone()).await;
        let popped_emails = buffer.pop_all().await;
        assert_eq!(popped_emails.len(), 2);
    }
}
