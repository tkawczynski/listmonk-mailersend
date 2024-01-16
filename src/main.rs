mod config;
mod listmonk;
mod mailersend;

use actix_web::{web, App, HttpServer};
use clap::Parser;
use config::Configuration;
use dotenv;
use listmonk::api::ListmonkAPI;
use mailersend::api::MailerSendAPI;
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv::dotenv().ok();
    let config = Configuration::parse();
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
