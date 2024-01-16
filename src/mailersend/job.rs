use actix_jobs::Job;

use super::{api::MailerSendAPI, buffer::Buffer};

pub struct OutgoingEmailsJob {
    cron: String,
    mailersend_api: MailerSendAPI,
    emails_buffer: Buffer,
    bulk_size: usize,
}

impl OutgoingEmailsJob {
    pub fn new(
        cron: &str,
        mailersend_api: MailerSendAPI,
        emails_buffer: Buffer,
        bulk_size: usize,
    ) -> Self {
        OutgoingEmailsJob {
            cron: cron.to_string(),
            mailersend_api,
            emails_buffer,
            bulk_size,
        }
    }
}

impl Job for OutgoingEmailsJob {
    fn cron(&self) -> &str {
        &self.cron
    }

    fn run(&mut self) {
        let emails_buffer = self.emails_buffer.clone();
        let bulk_size = self.bulk_size;
        let mailersend_api = self.mailersend_api.clone();
        actix_rt::spawn(async move {
            let emails = emails_buffer.pop_all().await;
            if emails.is_empty() {
                return;
            }
            log::info!("Sending {} cached emails", emails.len());
            match mailersend_api.send_bulk(emails, bulk_size).await {
                Ok(_) => log::info!("Successfully sent cached emails"),
                Err(err) => log::error!("Failed to cached emails due to error: {}", err),
            }
        });
    }
}
