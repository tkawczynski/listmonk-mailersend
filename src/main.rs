mod listmonk;
mod mailersend;

use actix_web::{web, App, HttpServer};
use clap::Parser;
use dotenv;
use listmonk::api::ListmonkAPI;
use log;
use mailersend::api::MailerSendAPI;
use std::io;

#[derive(Parser, Debug, Clone)]
struct Options {
    #[arg(long, short = 'H', env, default_value_t = String::from("127.0.0.1"), help="Service bind IP address")]
    host: String,

    #[arg(
        long,
        short = 'p',
        env,
        default_value_t = 9000,
        help = "Service bind port"
    )]
    port: u16,

    #[arg(long, short = 'l', env, default_value_t = log::Level::Info, help="Log level")]
    log_level: log::Level,

    #[arg(long, short = 'e', env, help = "MailSender API endpoint", default_value_t = String::from("https://api.mailersend.com/v1"))]
    mailersend_api_endpoint: String,

    #[arg(long, short = 't', env, help = "MailSender API token")]
    mailersend_api_token: String,

    #[arg(long, short = 'm', env, help = "Listmonk API endpoint", default_value_t = String::from("http://localhost:9001"))]
    listmonk_api_endpoint: String,

    #[arg(long, short = 'u', env, help = "Listmonk API username")]
    listmonk_api_username: String,

    #[arg(long, short = 'w', env, help = "Listmonk API password")]
    listmonk_api_password: String,

    #[arg(
        long,
        short = 'b',
        env,
        help = "MailSender API batch size",
        default_value_t = 500
    )]
    api_email_bulk_size: usize,

    #[arg(
        long,
        short = 'r',
        env,
        help = "MailSender API requests per minute",
        default_value_t = 10
    )]
    api_bulk_req_per_min: u32,

    #[arg(long, short = 's', env, help = "MailSender Webhooks signing secret")]
    signing_secret: Option<String>,
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv::dotenv().ok();
    let config = Options::parse();
    simple_logger::init_with_level(config.log_level).unwrap();

    let mailersend_api = MailerSendAPI::new(
        &config.mailersend_api_endpoint,
        &config.mailersend_api_token,
        config.api_bulk_req_per_min,
    );
    let listmonk_api = ListmonkAPI::new(
        &config.listmonk_api_endpoint,
        &config.listmonk_api_username,
        &config.listmonk_api_password,
    );
    let host = config.host.clone();
    let port = config.port;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(mailersend_api.clone()))
            .app_data(web::Data::new(listmonk_api.clone()))
            .app_data(web::Data::new(config.clone()))
            .route(
                "/api/messenger",
                web::post().to(listmonk::rest::messenger_handler),
            )
            .route(
                "/webhooks/service/mailersend",
                web::post().to(mailersend::rest::webhook_handler),
            )
    })
    .bind((host, port))?
    .run()
    .await
    .unwrap();
    Ok(())
}
