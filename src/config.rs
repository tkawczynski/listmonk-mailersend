use clap::Parser;
use log;

#[derive(Parser, Debug, Clone)]
pub struct Configuration {
    #[arg(long, short = 'H', env, default_value_t = String::from("127.0.0.1"), help="Service bind IP address")]
    pub host: String,

    #[arg(
        long,
        short = 'p',
        env,
        default_value_t = 9000,
        help = "Service bind port"
    )]
    pub port: u16,

    #[arg(long, short = 'c', env, help = "Outgoing cron schedule", default_value_t = String::from("0 */1 * * *"))]
    pub outgoing_cron: String,

    #[arg(long, short = 'l', env, default_value_t = log::Level::Info, help="Log level")]
    pub log_level: log::Level,

    #[arg(long, short = 'e', env, help = "MailSender API endpoint", default_value_t = String::from("https://api.mailersend.com/v1"))]
    pub mailersend_api_endpoint: String,

    #[arg(long, short = 't', env, help = "MailSender API token")]
    pub mailersend_api_token: String,

    #[arg(long, short = 'm', env, help = "Listmonk API endpoint", default_value_t = String::from("http://localhost:9001"))]
    pub listmonk_api_endpoint: String,

    #[arg(long, short = 'u', env, help = "Listmonk API username")]
    pub listmonk_api_username: String,

    #[arg(long, short = 'w', env, help = "Listmonk API password")]
    pub listmonk_api_password: String,

    #[arg(
        long,
        short = 'b',
        env,
        help = "MailSender API batch size",
        default_value_t = 500
    )]
    pub api_email_bulk_size: usize,

    #[arg(
        long,
        short = 'r',
        env,
        help = "MailSender API requests per minute",
        default_value_t = 10
    )]
    pub api_bulk_req_per_min: u32,

    #[arg(long, short = 's', env, help = "MailSender Webhooks signing secret")]
    pub signing_secret: Option<String>,
}
